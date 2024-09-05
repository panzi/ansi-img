use std::fmt::Write;

use image::{Rgba, RgbaImage};

#[inline]
pub fn image_to_ansi(prev_frame: &RgbaImage, image: &RgbaImage, alpha_threshold: u8) -> Vec<String> {
    let mut lines = vec![];
    image_to_ansi_into(prev_frame, image, alpha_threshold, &mut lines);
    lines
}

pub fn image_to_ansi_into(prev_frame: &RgbaImage, image: &RgbaImage, alpha_threshold: u8, lines: &mut Vec<String>) {
    let line_len = (image.width() as usize) * "\x1B[38;2;255;255;255\x1B[48;2;255;255;255m▄".len() + "\x1B[0m".len();
    let row_count = (image.height() + 1) / 2;

    lines.resize_with(row_count as usize, || String::with_capacity(line_len));

    for line_y in 0..row_count {
        let line = &mut lines[line_y as usize];
        line.clear();

        let y = line_y * 2;
        if y + 1 == image.height() {
            let mut prev_color = Rgba([0, 0, 0, 0]);
            let mut x_skip = 0;
            for x in 0..image.width() {
                let color = *image.get_pixel(x, y);
                if color == *prev_frame.get_pixel(x, y) {
                    x_skip += 1;
                } else {
                    if x_skip > 0 {
                        if x_skip == 1 {
                            line.push_str("\x1B[C");
                        } else {
                            let _ = write!(line, "\x1B[{x_skip}C");
                        }
                        x_skip = 0;
                    }
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
            }
        } else {
            let mut prev_bg = Rgba([0, 0, 0, 0]);
            let mut prev_fg = Rgba([0, 0, 0, 0]);
            let mut x_skip = 0;
            for x in 0..image.width() {
                let color_top    = *image.get_pixel(x, y);
                let color_bottom = *image.get_pixel(x, y + 1);

                if color_top == *prev_frame.get_pixel(x, y) && color_bottom == *prev_frame.get_pixel(x, y + 1) {
                    x_skip += 1;
                } else {
                    if x_skip > 0 {
                        if x_skip == 1 {
                            line.push_str("\x1B[C");
                        } else {
                            let _ = write!(line, "\x1B[{x_skip}C");
                        }
                        x_skip = 0;
                    }
                    let Rgba([r1, g1, b1, a1]) = color_top;

                    if color_top == color_bottom {
                        if a1 < alpha_threshold {
                            if prev_bg.0[3] < alpha_threshold && prev_fg.0[3] < alpha_threshold {
                                line.push_str(" ");
                            } else {
                                line.push_str("\x1B[0m ");
                            }
                        } else {
                            let _ = write!(line, "\x1B[38;2;{r1};{g1};{b1}m█");
                        }
                        prev_fg = color_top;
                        prev_bg = color_top;
                    } else {
                        let Rgba([r2, g2, b2, a2]) = color_bottom;
                        if a1 < alpha_threshold && a2 < alpha_threshold {
                            if prev_bg.0[3] < alpha_threshold && prev_fg.0[3] < alpha_threshold {
                                line.push_str(" ");
                            } else {
                                line.push_str("\x1B[0m ");
                            }
                            prev_fg = color_top;
                            prev_bg = color_bottom;
                        } else if a1 < alpha_threshold {
                            let _ = write!(line, "\x1B[0m\x1B[38;2;{r2};{g2};{b2}m▄");
                            prev_fg = color_bottom;
                            prev_bg = color_top;
                        } else if a2 < alpha_threshold {
                            let _ = write!(line, "\x1B[0m\x1B[38;2;{r1};{g1};{b1}m▀");
                            prev_fg = color_top;
                            prev_bg = color_bottom;
                        } else {
                            if prev_fg == color_bottom && prev_bg == color_top {
                                let _ = write!(line, "▄");
                            } else if prev_fg == color_top && prev_bg == color_bottom {
                                let _ = write!(line, "▀");
                            } else if prev_fg == color_bottom {
                                let _ = write!(line, "\x1B[48;2;{r1};{g1};{b1}m▄");
                                prev_bg = color_top;
                            } else if prev_fg == color_top {
                                let _ = write!(line, "\x1B[48;2;{r2};{g2};{b2}m▀");
                                prev_bg = color_bottom;
                            } else if prev_bg == color_top {
                                let _ = write!(line, "\x1B[38;2;{r2};{g2};{b2}m▄");
                                prev_fg = color_bottom;
                            } else if prev_bg == color_bottom {
                                let _ = write!(line, "\x1B[38;2;{r1};{g1};{b1}m▀");
                                prev_fg = color_top;
                            } else {
                                let _ = write!(line, "\x1B[48;2;{r1};{g1};{b1}m\x1B[38;2;{r2};{g2};{b2}m▄");
                                prev_fg = color_bottom;
                                prev_bg = color_top;
                            }
                        }
                    }
                }
            }
        }

        line.push_str("\x1B[0m");
    }
}
