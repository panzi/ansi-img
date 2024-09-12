use std::fmt::Write;

use image::{Rgba, RgbaImage};

#[inline]
pub fn image_to_ansi(prev_frame: &RgbaImage, image: &RgbaImage, alpha_threshold: u8, full_width: bool) -> String {
    let mut lines = String::new();
    image_to_ansi_into(prev_frame, image, alpha_threshold, full_width, &mut lines);
    lines
}

#[inline]
fn move_cursor(curr_x: u32, curr_line_y: u32, x: u32, line_y: u32, lines: &mut String) {
    if x != curr_x {
        if x > curr_x {
            let dx = x - curr_x;
            if dx == 1 {
                lines.push_str("\x1B[C");
            } else {
                let _ = write!(lines, "\x1B[{dx}C");
            }
        } else {
            let dx = curr_x - x;
            if dx == 1 {
                lines.push_str("\x1B[D");
            } else {
                let _ = write!(lines, "\x1B[{dx}D");
            }
        }
    }

    if line_y != curr_line_y {
        if line_y > curr_line_y {
            let dy = line_y - curr_line_y;
            if dy == 1 {
                lines.push_str("\x1B[B");
            } else {
                let _ = write!(lines, "\x1B[{dy}B");
            }
        } else {
            let dy = curr_line_y - line_y;
            if dy == 1 {
                lines.push_str("\x1B[A");
            } else {
                let _ = write!(lines, "\x1B[{dy}A");
            }
        }
    }
}

pub fn image_to_ansi_into(prev_frame: &RgbaImage, image: &RgbaImage, alpha_threshold: u8, full_width: bool, lines: &mut String) {
    let width = image.width();
    let line_len = (width as usize) * "\x1B[38;2;255;255;255\x1B[48;2;255;255;255m▄".len() + "\x1B[0m".len();
    let row_count = (image.height() + 1) / 2;

    lines.clear();

    if row_count == 0 {
        return;
    }

    lines.reserve(line_len * row_count as usize + "\x1B[0m".len());

    let mut curr_line_y = 0;
    let mut curr_x = 0;

    for line_y in 0..row_count {
        let y = line_y * 2;
        if y + 1 == image.height() {
            let mut prev_color = Rgba([0, 0, 0, 0]);
            for x in 0..image.width() {
                let color = *image.get_pixel(x, y);
                if color != *prev_frame.get_pixel(x, y) {
                    move_cursor(curr_x, curr_line_y, x, line_y, lines);
                    let Rgba([r, g, b, a]) = color;
                    if a < alpha_threshold {
                        if prev_color[3] < alpha_threshold {
                            lines.push_str(" ");
                        } else {
                            lines.push_str("\x1B[0m ");
                        }
                    } else if color == prev_color {
                        lines.push_str("▀");
                    } else {
                        let _ = write!(lines, "\x1B[38;2;{r};{g};{b}m▀");
                    }
                    prev_color = color;
                    // NOTE: Cursor location doesn't update at the end of the screen.
                    // This assumes that the image is rendered up to the end of the screen!
                    if full_width && (x + 1) == width {
                        curr_x = x;
                    } else {
                        curr_x = x + 1;
                    }
                    curr_line_y = line_y;
                }
            }
        } else {
            let mut prev_bg = Rgba([0, 0, 0, 0]);
            let mut prev_fg = Rgba([0, 0, 0, 0]);
            for x in 0..image.width() {
                let color_top    = *image.get_pixel(x, y);
                let color_bottom = *image.get_pixel(x, y + 1);

                if color_top != *prev_frame.get_pixel(x, y) || color_bottom != *prev_frame.get_pixel(x, y + 1) {
                    move_cursor(curr_x, curr_line_y, x, line_y, lines);
                    let Rgba([r1, g1, b1, a1]) = color_top;

                    if color_top == color_bottom {
                        if a1 < alpha_threshold {
                            if prev_bg.0[3] < alpha_threshold && prev_fg.0[3] < alpha_threshold {
                                lines.push_str(" ");
                            } else {
                                lines.push_str("\x1B[0m ");
                            }
                        } else {
                            let _ = write!(lines, "\x1B[38;2;{r1};{g1};{b1}m█");
                        }
                        prev_fg = color_top;
                        prev_bg = color_top;
                    } else {
                        let Rgba([r2, g2, b2, a2]) = color_bottom;
                        if a1 < alpha_threshold && a2 < alpha_threshold {
                            if prev_bg.0[3] < alpha_threshold && prev_fg.0[3] < alpha_threshold {
                                lines.push_str(" ");
                            } else {
                                lines.push_str("\x1B[0m ");
                            }
                            prev_fg = color_top;
                            prev_bg = color_bottom;
                        } else if a1 < alpha_threshold {
                            let _ = write!(lines, "\x1B[0m\x1B[38;2;{r2};{g2};{b2}m▄");
                            prev_fg = color_bottom;
                            prev_bg = color_top;
                        } else if a2 < alpha_threshold {
                            let _ = write!(lines, "\x1B[0m\x1B[38;2;{r1};{g1};{b1}m▀");
                            prev_fg = color_top;
                            prev_bg = color_bottom;
                        } else {
                            if prev_fg == color_bottom && prev_bg == color_top {
                                let _ = write!(lines, "▄");
                            } else if prev_fg == color_top && prev_bg == color_bottom {
                                let _ = write!(lines, "▀");
                            } else if prev_fg == color_bottom {
                                let _ = write!(lines, "\x1B[48;2;{r1};{g1};{b1}m▄");
                                prev_bg = color_top;
                            } else if prev_fg == color_top {
                                let _ = write!(lines, "\x1B[48;2;{r2};{g2};{b2}m▀");
                                prev_bg = color_bottom;
                            } else if prev_bg == color_top {
                                let _ = write!(lines, "\x1B[38;2;{r2};{g2};{b2}m▄");
                                prev_fg = color_bottom;
                            } else if prev_bg == color_bottom {
                                let _ = write!(lines, "\x1B[38;2;{r1};{g1};{b1}m▀");
                                prev_fg = color_top;
                            } else {
                                let _ = write!(lines, "\x1B[48;2;{r1};{g1};{b1}m\x1B[38;2;{r2};{g2};{b2}m▄");
                                prev_fg = color_bottom;
                                prev_bg = color_top;
                            }
                        }
                    }
                    // NOTE: Cursor location doesn't update at the end of the screen.
                    // This assumes that the image is rendered up to the end of the screen!
                    if full_width && (x + 1) == width {
                        curr_x = x;
                    } else {
                        curr_x = x + 1;
                    }
                    curr_line_y = line_y;
                }
            }
        }

        if curr_line_y == line_y {
            lines.push_str("\x1B[0m");
        }
    }

    // Just to ensure that the cursor is at the correct position after
    // the image is rendered or when hitting Ctrl+C during sleep.
    let dx = image.width() - curr_x;
    if dx > 0 {
        if dx == 1 {
            lines.push_str("\x1B[C");
        } else {
            let _ = write!(lines, "\x1B[{dx}C");
        }
    }

    let dy = row_count - 1 - curr_line_y;
    if dy > 0 {
        if dy == 1 {
            lines.push_str("\x1B[B");
        } else {
            let _ = write!(lines, "\x1B[{dy}B");
        }
    }
}
