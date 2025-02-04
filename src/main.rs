use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cli::args::Args;
use cli::canvas_size::CanvasSize;
use cli::color::Color;
use cli::style::Style;
use clap::Parser;
use image::codecs::gif::GifDecoder;
use image::codecs::png::PngDecoder;
use image::codecs::webp::WebPDecoder;
use image::io::Reader as ImageReader;
use image::error::ImageResult;
use image::{AnimationDecoder, DynamicImage, Frame, GenericImage, ImageDecoder, RgbaImage, Pixel};
use image::imageops;
use image_to_ansi::image_to_ansi_into;

pub mod image_to_ansi;
pub mod cli;

fn interruptable_sleep(duration: Duration) -> bool {
    #[cfg(target_family = "unix")]
    {
        let req = libc::timespec {
            tv_sec:  duration.as_secs() as libc::time_t,
            tv_nsec: duration.subsec_nanos() as i64,
        };
        let ret = unsafe { libc::nanosleep(&req, std::ptr::null_mut()) };
        return ret == 0;
    }

    #[cfg(not(target_family = "unix"))]
    {
        std::thread::sleep(duration);
        return true;
    }
}

#[inline]
fn fill_color(image: &mut RgbaImage, color: Color) {
    match color {
        Color::Transparent => {
            image.fill(0);
        }
        Color::Solid(rgb) => {
            let rgba = rgb.to_rgba();
            for pixel in image.pixels_mut() {
                *pixel = rgba;
            }
        }
    }
}

fn main() -> ImageResult<()> {
    use std::io::Write;

    let args = Args::parse();

    let alpha_threshold = args.alpha_threshold;
    let style = args.style;
    let canvas_size = args.canvas_size;
    let run_anim = Arc::new(AtomicBool::new(true));
    let path = args.path;
    let filter = args.filter.into();
    let background_color = args.background_color;
    let endl = args.line_end.as_str();
    let inline = args.inline;

    {
        let run_anim = run_anim.clone();
        let _ = ctrlc::set_handler(move || {
            run_anim.store(false, Ordering::Relaxed);
        });
    }

    let reader = ImageReader::open(path)?.with_guessed_format()?;

    let mut lock = std::io::stdout().lock();
    // CSI ?  7 l     No Auto-Wrap Mode (DECAWM), VT100.
    // CSI ? 25 l     Hide cursor (DECTCEM), VT220
    print!("\x1B[?25l\x1B[?7l");

    let mut linebuf = String::new();

    enum DecodedImage {
        Animated(u32, u32, Vec<Frame>),
        Still(RgbaImage)
    }

    impl DecodedImage {
        #[inline]
        fn size(&self) -> (u32, u32) {
            match self {
                DecodedImage::Animated(width, height, _) => (*width, *height),
                DecodedImage::Still(img) => (img.width(), img.height()),
            }
        }
    }

    let anim = match reader.format() {
        Some(image::ImageFormat::Gif) => {
            let decoder = GifDecoder::new(reader.into_inner())?;
            let (width, height) = decoder.dimensions();
            let frames = decoder.into_frames().collect_frames()?;

            if frames.len() == 1 {
                DecodedImage::Still(frames.into_iter().next().unwrap().into_buffer())
            } else {
                DecodedImage::Animated(width, height, frames)
            }
        },
        Some(image::ImageFormat::WebP) => {
            let decoder = WebPDecoder::new(reader.into_inner())?;
            if decoder.has_animation() {
                let (width, height) = decoder.dimensions();
                let frames = decoder.into_frames().collect_frames()?;

                if frames.len() == 1 {
                    DecodedImage::Still(frames.into_iter().next().unwrap().into_buffer())
                } else {
                    DecodedImage::Animated(width, height, frames)
                }
            } else {
                DecodedImage::Still(DynamicImage::from_decoder(decoder)?.to_rgba8())
            }
        },
        Some(image::ImageFormat::Png) => {
            let decoder = PngDecoder::new(reader.into_inner())?;
            if decoder.is_apng()? {
                let (width, height) = decoder.dimensions();
                let frames = decoder.apng()?.into_frames().collect_frames()?;

                if frames.len() == 1 {
                    DecodedImage::Still(frames.into_iter().next().unwrap().into_buffer())
                } else {
                    DecodedImage::Animated(width, height, frames)
                }
            } else {
                DecodedImage::Still(DynamicImage::from_decoder(decoder)?.to_rgba8())
            }
        },
        _ => DecodedImage::Still(reader.decode()?.to_rgba8())
    };

    let mut term_canvas = match canvas_size {
        CanvasSize::Exact(width, height) => Some(RgbaImage::new(width, height)),
        CanvasSize::Window =>
            term_size::dimensions().map(|(width, height)|
                match background_color {
                    Color::Transparent => RgbaImage::new(width as u32, height as u32 * 2),
                    Color::Solid(rgb) => RgbaImage::from_pixel(width as u32, height as u32 * 2, rgb.to_rgba()),
                }),
        CanvasSize::Image =>
            match style {
                Style::Position(x, y, size) => {
                    let (image_width, image_height) = anim.size();
                    let (w, h) = size.to_size(image_width, image_height);
                    let x = x.unwrap_or(0);
                    let y = y.unwrap_or(0);

                    let w = (w as i64 + x as i64).max(0);
                    let h = (h as i64 + y as i64).max(0);
                    let w = if w > u32::MAX as i64 { u32::MAX } else { w as u32 };
                    let h = if h > u32::MAX as i64 { u32::MAX } else { h as u32 };

                    if let Color::Solid(rgb) = background_color {
                        Some(RgbaImage::from_pixel(w, h, rgb.to_rgba()))
                    } else {
                        Some(RgbaImage::new(w, h))
                    }
                },
                _ =>
                    if let Color::Solid(rgb) = background_color {
                        let (width, height) = anim.size();
                        Some(RgbaImage::from_pixel(width, height, rgb.to_rgba()))
                    } else {
                        None
                    }
            },
    };

    let mut prev_frame = if let Some(term_canvas) = &term_canvas {
        RgbaImage::new(term_canvas.width(), term_canvas.height())
    } else {
        let (width, height) = anim.size();
        RgbaImage::new(width, height)
    };

    #[cfg(target_family = "unix")]
    let mut term = MaybeUninit::<libc::termios>::zeroed();

    #[cfg(target_family = "unix")]
    let mut term = if unsafe { libc::tcgetattr(libc::STDIN_FILENO, term.as_mut_ptr()) == 0 } {
        Some(unsafe { term.assume_init_mut() })
    } else {
        None
    };

    #[cfg(target_family = "unix")]
    if let Some(term) = &mut term {
        (*term).c_lflag &= !libc::ECHO;
        unsafe { libc::tcsetattr(libc::STDIN_FILENO, 0, *term); }
    }

    if inline {
        // Make sure everything is in view because when moving the cursor beyond
        // the bottom screen edge it will stay at the last line and not scroll
        // the screen.
        if args.no_padding {
            print!("\x1B[s");
        } else {
            let lines = (prev_frame.height() + 1) / 2;
            for _ in 0..lines {
                println!();
            }
            print!("\x1B[{lines}A\x1B[s");
        }
    } else {
        print!("\x1B[2J");
    }

    match anim {
        DecodedImage::Animated(width, height, frames) => {
            if !frames.is_empty() {
                let mut frame_canvas = RgbaImage::new(width, height);
                let mut loop_count = if args.loop_count < 0 { None } else { Some(args.loop_count )};
                let mut timestamp = Instant::now();

                while loop_count.unwrap_or(1) > 0 && run_anim.load(Ordering::Relaxed) {
                    for frame in &frames {
                        if !run_anim.load(Ordering::Relaxed) {
                            break;
                        }

                        let duration: Duration = frame.delay().into();

                        let frame_image = frame.buffer();
                        if let Color::Solid(rgb) = background_color {
                            let rgba = rgb.to_rgba();
                            for pixel in frame_canvas.pixels_mut() {
                                *pixel = rgba;
                            }
                            imageops::overlay(&mut frame_canvas, frame_image, frame.left() as i64, frame.top() as i64);
                        } else {
                            if frame_image.width() != frame_canvas.width() ||
                               frame_image.height() != frame_canvas.height() ||
                               frame.left() != 0 || frame.top() != 0
                            {
                                frame_canvas.fill(0);
                            }
                            frame_canvas.copy_from(frame_image, frame.left(), frame.top())?;
                        }

                        let term_size = term_size::dimensions();

                        if let Some(term_canvas) = &mut term_canvas {
                            let full_width;
                            if canvas_size.is_window() {
                                full_width = true;
                                if let Some((term_width, term_height)) = term_size {
                                    if term_width != term_canvas.width() as usize || (term_height * 2) != term_canvas.height() as usize {
                                        match background_color {
                                            Color::Transparent => {
                                                *term_canvas = RgbaImage::new(term_width as u32, term_height as u32 * 2);
                                            }
                                            Color::Solid(rgb) => {
                                                *term_canvas = RgbaImage::from_pixel(term_width as u32, term_height as u32 * 2, rgb.to_rgba());
                                            }
                                        }
                                        prev_frame = RgbaImage::new(term_canvas.width(), term_canvas.height());
                                        print!("\x1B[2J");
                                    } else {
                                        fill_color(term_canvas, background_color);
                                    }
                                } else {
                                    fill_color(term_canvas, background_color);
                                }
                            } else {
                                full_width = if let Some((term_width, _)) = term_size {
                                    term_canvas.width() as usize >= term_width
                                } else {
                                    true
                                };
                                fill_color(term_canvas, background_color);
                            }

                            style.paint(&frame_canvas, term_canvas, filter);

                            image_to_ansi_into(&prev_frame, term_canvas, alpha_threshold, full_width, &mut linebuf);
                            std::mem::swap(&mut prev_frame, term_canvas);
                        } else {
                            let full_width = if let Some((term_width, _)) = term_size {
                                frame_canvas.width() as usize >= term_width
                            } else {
                                true
                            };
                            image_to_ansi_into(&prev_frame, &frame_canvas, alpha_threshold, full_width, &mut linebuf);
                            std::mem::swap(&mut prev_frame, &mut frame_canvas);
                        }

                        if inline {
                            print!("\x1B[u{linebuf}");
                        } else {
                            print!("\x1B[1;1H{linebuf}");
                        }
                        let _ = lock.flush();

                        let now = Instant::now();

                        let elapsed = if timestamp > now {
                            // This would mean that it slept shorter than requested, but didn't
                            // signal any error!
                            Duration::ZERO
                        } else {
                            now - timestamp
                        };

                        timestamp += duration;

                        if duration > elapsed && !interruptable_sleep(duration - elapsed) {
                            run_anim.store(false, Ordering::Relaxed);
                            break;
                        }
                    }

                    if let Some(loop_count) = &mut loop_count {
                        *loop_count -= 1;
                    }
                }
            }
        }
        DecodedImage::Still(image) => {
            if let Some(term_canvas) = &mut term_canvas {
                style.paint(&image, term_canvas, filter);
                image_to_ansi_into(&prev_frame, &term_canvas, alpha_threshold, false, &mut linebuf);
            } else {
                image_to_ansi_into(&prev_frame, &image, alpha_threshold, false, &mut linebuf);
            }

            if inline {
                print!("{linebuf}");
            } else {
                print!("\x1B[1;1H{linebuf}");
            }
            let _ = lock.flush();
        }
    }

    // CSI 0 m        Reset or normal, all attributes become turned off
    // CSI ?  7 h     Auto-Wrap Mode (DECAWM), VT100
    // CSI ? 25 h     Show cursor (DECTCEM), VT220
    print!("\x1B[0m\x1B[?25h\x1B[?7h{endl}");

    #[cfg(target_family = "unix")]
    if let Some(term) = term {
        term.c_lflag |= libc::ECHO;
        unsafe { libc::tcsetattr(libc::STDIN_FILENO, 0, term); }
    }

    Ok(())
}
