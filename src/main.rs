use std::ffi::OsString;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::fmt::{Display, Write};

use clap::Parser;

use image::codecs::gif::GifDecoder;
use image::codecs::png::PngDecoder;
use image::codecs::webp::WebPDecoder;
use image::io::Reader as ImageReader;
use image::error::ImageResult;
use image::{AnimationDecoder, DynamicImage, Frame, GenericImage, ImageDecoder, Pixel, Rgb, Rgba, RgbaImage};
use image::imageops;

pub fn image_to_ansi(image: &RgbaImage, alpha_threshold: u8) -> Vec<String> {
    let mut lines = vec![];
    image_to_ansi_into(image, alpha_threshold, &mut lines);
    lines
}

pub fn image_to_ansi_into(image: &RgbaImage, alpha_threshold: u8, lines: &mut Vec<String>) {
    let line_len = (image.width() as usize) * "\x1B[38;2;255;255;255\x1B[48;2;255;255;255m▄".len() + "\x1B[0m".len();

    for line_y in 0..((image.height() + 1) / 2) {
        let mut line = String::with_capacity(line_len);

        let y = line_y * 2;
        if y + 1 == image.height() {
            let mut prev_color = Rgba([0, 0, 0, 0]);
            for x in 0..image.width() {
                let color = *image.get_pixel(x, y);
                let Rgba([r, g, b, a]) = color;
                if a < alpha_threshold {
                    if prev_color[3] < alpha_threshold {
                        line.push(' ');
                    } else {
                        line.push_str("\x1B[0m ");
                    }
                } else if color == prev_color {
                    line.push('▀');
                } else {
                    let _ = write!(line, "\x1B[38;2;{r};{g};{b}m▀");
                }
                prev_color = color;
            }
        } else {
            for x in 0..image.width() {
                let color_top    = *image.get_pixel(x, y);
                let color_bottom = *image.get_pixel(x, y + 1);
                let Rgba([r1, g1, b1, a1]) = color_top;

                if color_top == color_bottom {
                    if a1 < alpha_threshold {
                        line.push_str("\x1B[0m ");
                    } else {
                        let _ = write!(line, "\x1B[38;2;{r1};{g1};{b1}m█");
                    }
                } else {
                    let Rgba([r2, g2, b2, a2]) = color_bottom;
                    if a1 < alpha_threshold && a2 < alpha_threshold {
                        line.push_str("\x1B[0m ");
                    } else if a1 < alpha_threshold {
                        let _ = write!(line, "\x1B[0m\x1B[38;2;{r2};{g2};{b2}m▄");
                    } else if a2 < alpha_threshold {
                        let _ = write!(line, "\x1B[0m\x1B[38;2;{r1};{g1};{b1}m▀");
                    } else {
                        let _ = write!(line, "\x1B[48;2;{r1};{g1};{b1}m\x1B[38;2;{r2};{g2};{b2}m▄");
                    }
                }
            }
        }

        line.push_str("\x1B[0m");
        lines.push(line);
    }
}

fn write_frame_to_buf(lines: &[impl AsRef<str>], linebuf: &mut String) {
    linebuf.clear();
    linebuf.push_str("\x1B[1;1H\x1B[2J");
    let mut first = true;
    for line in lines {
        if first {
            first = false;
        } else {
            linebuf.push('\n');
        }
        linebuf.push_str(line.as_ref());
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Style {
    Center,
    Tile,
    Position (u32, u32),
    Cover,
    Contain,
    ShrinkToFit,
}

#[inline]
fn draw_contain(image: &RgbaImage, canvas: &mut RgbaImage, filter: imageops::FilterType) {
    if canvas.width() == image.width() && canvas.height() == image.height() {
        imageops::overlay(canvas, image, 0, 0);
    } else {
        let mut width = canvas.width();
        let mut height = image.height() * width / image.width();
        let x;
        let y;
        if height > canvas.height() {
            height = canvas.height();
            width = image.width() * height / image.height();
            x = (canvas.width() as i64 - width as i64) / 2;
            y = 0;
        } else {
            x = 0;
            y = (canvas.height() as i64 - height as i64) / 2;
        }
        let image = imageops::resize(image, width, height, filter);
        imageops::overlay(canvas, &image, x, y);
    }
}

#[inline]
fn draw_center(image: &RgbaImage, canvas: &mut RgbaImage) {
    let x = (canvas.width() as i64 - image.width() as i64) / 2;
    let y = (canvas.height() as i64 - image.height() as i64) / 2;
    imageops::overlay(canvas, image, x, y);
}

impl Style {
    pub fn paint(&self, image: &RgbaImage, canvas: &mut RgbaImage, filter: imageops::FilterType) {
        match *self {
            Style::Center => {
                draw_center(image, canvas);
            },
            Style::Tile => {
                for y in (0..canvas.height()).step_by(image.height() as usize) {
                    for x in (0..canvas.width()).step_by(image.width() as usize) {
                        imageops::overlay(canvas, image, x.into(), y.into());
                    }
                }
            },
            Style::Position(x, y) => {
                imageops::overlay(canvas, image, x.into(), y.into());
            },
            Style::Cover => {
                if canvas.width() == image.width() && canvas.height() == image.height() {
                    imageops::overlay(canvas, image, 0, 0);
                } else {
                    let mut width = canvas.width();
                    let mut height = image.height() * width / image.width();
                    let x;
                    let y;
                    if height < canvas.height() {
                        height = canvas.height();
                        width = image.width() * height / image.height();
                        x = (canvas.width() as i64 - width as i64) / 2;
                        y = 0;
                    } else {
                        x = 0;
                        y = (canvas.height() as i64 - height as i64) / 2;
                    }
                    let image = imageops::resize(image, width, height, filter);
                    imageops::overlay(canvas, &image, x, y);
                }
            },
            Style::Contain => {
                draw_contain(image, canvas, filter);
            },
            Style::ShrinkToFit => {
                if image.width() <= canvas.width() && image.height() <= canvas.height() {
                    draw_center(image, canvas);
                } else {
                    draw_contain(image, canvas, filter);
                }
            },
        }
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Center => {
                "center".fmt(f)
            },
            Self::Tile => {
                "tile".fmt(f)
            },
            Self::Position(x, y) => {
                write!(f, "position {x} {y}")
            },
            Self::Cover => {
                "cover".fmt(f)
            },
            Self::Contain => {
                "contain".fmt(f)
            },
            Self::ShrinkToFit => {
                "shrink-to-fit".fmt(f)
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StyleParseError();

impl Display for StyleParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "illegal style value".fmt(f)
    }
}

impl std::error::Error for StyleParseError {}

impl FromStr for Style {
    type Err = StyleParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("center") {
            Ok(Style::Center)
        } else if value.eq_ignore_ascii_case("tile") {
            Ok(Style::Tile)
        } else if value.eq_ignore_ascii_case("cover") {
            Ok(Style::Cover)
        } else if value.eq_ignore_ascii_case("contain") {
            Ok(Style::Contain)
        } else if value.eq_ignore_ascii_case("shrink-to-fit") || value.eq_ignore_ascii_case("shrinktofit") {
            Ok(Style::ShrinkToFit)
        } else {
            let mut items = value.split_ascii_whitespace();
            let Some(first) = items.next() else {
                return Err(StyleParseError());
            };

            if !first.eq_ignore_ascii_case("position") {
                return Err(StyleParseError());
            }

            let Some(Ok(x)) = items.next().map(|value| value.parse()) else {
                return Err(StyleParseError());
            };

            let Some(Ok(y)) = items.next().map(|value| value.parse()) else {
                return Err(StyleParseError());
            };

            if let Some(_) = items.next() {
                return Err(StyleParseError());
            }

            Ok(Style::Position(x, y))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanvasSize {
    Window,
    Image,
    Exact(u32, u32)
}

impl CanvasSize {
    #[inline]
    pub fn is_window(&self) -> bool {
        matches!(self, CanvasSize::Window)
    }

    #[inline]
    pub fn is_image(&self) -> bool {
        matches!(self, CanvasSize::Image)
    }

    #[inline]
    pub fn is_exact(&self) -> bool {
        matches!(self, CanvasSize::Exact(_, _))
    }
}

impl Display for CanvasSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CanvasSize::Window => "window".fmt(f),
            CanvasSize::Image => "image".fmt(f),
            CanvasSize::Exact(width, height) => write!(f, "{width} {height}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CanvasSizeParseError();

impl Display for CanvasSizeParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "illegal canvas size".fmt(f)
    }
}

impl std::error::Error for CanvasSizeParseError {}

impl FromStr for CanvasSize {
    type Err = CanvasSizeParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("window") {
            Ok(CanvasSize::Window)
        } else if value.eq_ignore_ascii_case("image") {
            Ok(CanvasSize::Image)
        } else {
            let mut items = value.split_ascii_whitespace();
            let Some(Ok(width)) = items.next().map(|value| value.parse()) else {
                return Err(CanvasSizeParseError());
            };

            let Some(Ok(height)) = items.next().map(|value| value.parse()) else {
                return Err(CanvasSizeParseError());
            };

            if let Some(_) = items.next() {
                return Err(CanvasSizeParseError());
            }

            Ok(CanvasSize::Exact(width, height))
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Filter(imageops::FilterType);

impl Display for Filter {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

#[derive(Debug, PartialEq)]
pub struct FilterParseError();

impl Display for FilterParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "illegal filter type".fmt(f)
    }
}

impl std::error::Error for FilterParseError {}

impl FromStr for Filter {
    type Err = FilterParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("catmull-rom") || value.eq_ignore_ascii_case("catmullrom") {
            Ok(Filter(imageops::FilterType::CatmullRom))
        } else if value.eq_ignore_ascii_case("gaussian") {
            Ok(Filter(imageops::FilterType::Gaussian))
        } else if value.eq_ignore_ascii_case("lanczos3") {
            Ok(Filter(imageops::FilterType::Lanczos3))
        } else if value.eq_ignore_ascii_case("nearest") {
            Ok(Filter(imageops::FilterType::Nearest))
        } else if value.eq_ignore_ascii_case("triangle") {
            Ok(Filter(imageops::FilterType::Triangle))
        } else {
            Err(FilterParseError())
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Color {
    Transparent,
    Solid(image::Rgb<u8>)
}

impl Display for Color {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Transparent => "transparent".fmt(f),
            Color::Solid(Rgb([r, g, b])) => write!(f, "#{r:02x}{g:02x}{b:02x}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ColorParseError();

impl Display for ColorParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "illegal color value".fmt(f)
    }
}

impl std::error::Error for ColorParseError {}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("transparent") {
            Ok(Color::Transparent)
        } else if value.starts_with('#') && value.len() == 7 {
            let Ok(r) = u8::from_str_radix(&value[1..3], 16) else { return Err(ColorParseError()); };
            let Ok(g) = u8::from_str_radix(&value[3..5], 16) else { return Err(ColorParseError()); };
            let Ok(b) = u8::from_str_radix(&value[5..7], 16) else { return Err(ColorParseError()); };

            Ok(Color::Solid(Rgb([r, g, b])))
        } else {
            Err(ColorParseError())
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Times to loop the animation.
    /// 
    /// Negative values mean infinite looping.
    #[arg(short, long, default_value_t = -1)]
    loop_count: i64,

    /// Placement and scaling.
    /// 
    /// Values:{n}
    /// - center{n}
    /// - tile{n}
    /// - position <x> <y>{n}
    /// - cover{n}
    /// - contain{n}
    /// - shrink-to-fit (or shrinktofit)
    #[arg(short, long, default_value_t = Style::ShrinkToFit)]
    style: Style,

    /// Size of the canvas.
    /// 
    /// Values:{n}
    /// - window{n}
    /// - image{n}
    /// - <width> <height>
    #[arg(short, long, default_value_t = CanvasSize::Window)]
    canvas_size: CanvasSize,

    #[arg(short, long, default_value_t = 127)]
    alpha_threshold: u8,

    /// Filter used when resizing images.
    /// 
    /// Values:{n}
    /// - nearest{n}
    /// - triangle{n}
    /// - catmull-rom (or catmullrom){n}
    /// - caussian{n}
    /// - lanczos3
    #[arg(short, long, default_value_t = Filter(imageops::FilterType::Nearest))]
    filter: Filter,

    /// Set the background color.
    /// 
    /// Values:{n}
    /// - transparent{n}
    /// - #RRGGBB
    #[arg(short, long, default_value_t = Color::Transparent)]
    background_color: Color,

    #[arg()]
    path: OsString,
}

fn interruptable_sleep(duration: Duration) -> bool {
    #[cfg(target_family = "unix")]
    {
        let nanos = duration.as_nanos();
        let sec = nanos / 1_000_000_000u128;
        let req = libc::timespec {
            tv_sec:  sec as i64,
            tv_nsec: (nanos - (sec * 1_000_000_000u128)) as i64,
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
    let filter = args.filter.0;
    let background_color = args.background_color;

    {
        let run_anim = run_anim.clone();
        let _ = ctrlc::set_handler(move || {
            run_anim.store(false, Ordering::Relaxed);
        });
    }

    let reader = ImageReader::open(path)?.with_guessed_format()?;

    let mut lock = std::io::stdout().lock();
    print!("\x1B[?25l\x1B[?7l");

    let mut linebuf = String::new();
    let mut lines: Vec<String> = vec![];
    let mut term_canvas = match canvas_size {
        CanvasSize::Exact(width, height) => Some(RgbaImage::new(width, height * 2)),
        CanvasSize::Window =>
            term_size::dimensions().map(|(width, height)|
                match background_color {
                    Color::Transparent => RgbaImage::new(width as u32, height as u32 * 2),
                    Color::Solid(rgb) => RgbaImage::from_pixel(width as u32, height as u32 * 2, rgb.to_rgba()),
                }),
        CanvasSize::Image => None,
    };

    enum DecodedImage {
        Animated(u32, u32, Vec<Frame>),
        Still(RgbaImage)
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

                        lines.clear();
                        if let Some(term_canvas) = &mut term_canvas {
                            if canvas_size.is_window() {
                                if let Some((term_width, term_height)) = term_size::dimensions() {
                                    if term_width != term_canvas.width() as usize || term_height != term_canvas.height() as usize {
                                        match background_color {
                                            Color::Transparent => {
                                                *term_canvas = RgbaImage::new(term_width as u32, term_height as u32 * 2);
                                            }
                                            Color::Solid(rgb) => {
                                                *term_canvas = RgbaImage::from_pixel(term_width as u32, term_height as u32 * 2, rgb.to_rgba());
                                            }
                                        }
                                    } else {
                                        fill_color(term_canvas, background_color);
                                    }
                                } else {
                                    fill_color(term_canvas, background_color);
                                }
                            } else {
                                fill_color(term_canvas, background_color);
                            }

                            style.paint(&frame_canvas, term_canvas, filter);

                            image_to_ansi_into(term_canvas, alpha_threshold, &mut lines);
                        } else {
                            image_to_ansi_into(&frame_canvas, alpha_threshold, &mut lines);
                        }
                        write_frame_to_buf(&lines, &mut linebuf);

                        print!("{linebuf}");
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
                image_to_ansi_into(&term_canvas, alpha_threshold, &mut lines);
            } else {
                image_to_ansi_into(&image, alpha_threshold, &mut lines);
            }
            write_frame_to_buf(&lines, &mut linebuf);

            print!("{linebuf}");
            let _ = lock.flush();
        }
    }

    print!("\x1B[0m\x1B[?25h\x1B[?7h\n");

    Ok(())
}
