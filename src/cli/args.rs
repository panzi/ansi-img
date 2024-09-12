use std::ffi::OsString;

use clap::Parser;
use image::imageops;

use super::{canvas_size::CanvasSize, color::Color, filter::Filter, line_end::LineEnd, style::Style};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Times to loop the animation.
    /// 
    /// Negative values mean infinite looping.
    #[arg(short, long, default_value_t = -1)]
    pub loop_count: i64,

    /// Placement and scaling.
    /// 
    /// Values:{n}
    /// - center{n}
    /// - tile{n}
    /// - <x> <y> [z]{n}
    /// - <x> <y> <w> <h>{n}
    /// - cover{n}
    /// - contain{n}
    /// - shrink-to-fit (or shrinktofit)
    /// 
    /// x and y can be * to center within the canvas.
    /// 
    /// z is a zoom value. It is either a whole number >= 1 or a fraction <= 1/2.
    /// 
    /// w and h can be * so it's derived from the respective other value.
    #[arg(short, long, default_value_t = Style::ShrinkToFit)]
    pub style: Style,

    /// Size of the canvas.
    /// 
    /// Values:{n}
    /// - window{n}
    /// - image{n}
    /// - <width> <height>
    #[arg(short, long, default_value_t = CanvasSize::Window)]
    pub canvas_size: CanvasSize,

    #[arg(short, long, default_value_t = 127)]
    pub alpha_threshold: u8,

    /// Filter used when resizing images.
    /// 
    /// Values:{n}
    /// - nearest{n}
    /// - triangle{n}
    /// - catmull-rom (or catmullrom){n}
    /// - gaussian{n}
    /// - lanczos3
    #[arg(short, long, default_value_t = Filter::new(imageops::FilterType::Nearest))]
    pub filter: Filter,

    /// Set the background color.
    /// 
    /// Values:{n}
    /// - transparent{n}
    /// - #RRGGBB
    #[arg(short, long, default_value_t = Color::Transparent)]
    pub background_color: Color,

    /// Line ending to use.
    /// 
    /// Values:{n}
    /// - Cr{n}
    /// - Lf{n}
    /// - CrLf
    #[arg(short, long, default_value_t = LineEnd::Lf)]
    pub line_end: LineEnd,

    /// Don't clear screen and render image wherever the cursor currently is.
    #[arg(short, long, default_value_t = false)]
    pub inline: bool,

    /// When using `--inline` don't print newlines to scroll the screen to
    /// ensure the image is on screen.
    #[arg(short, long, default_value_t = false)]
    pub no_padding: bool,

    #[arg()]
    pub path: OsString,
}
