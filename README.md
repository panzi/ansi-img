ANSI IMG
========

Display images on the terminal using ANSI escape sequences and Unicode
characters.

This uses the [Unicode block elements](https://en.wikipedia.org/wiki/Block_Elements)
`█`, `▄`, and `▀` plus
[24-bit color ANSI escape sequences](https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit)
to set foreground and background colors in order to render two pixels for each
character. It also uses ANSI escape sequences to move the cursor around, so that
it doesn't have to re-paint the whole screen in an animation when only parts
change.

Not every terminal supports the 24-bit color escape sequences, and some that do
downsample the colors quite heavily. Also some terminals don't render the
characters perfectly and sometimes leave small gaps.

It supports displaying animated GIFs! (Press Ctrl+C to quit the animation.)

Demo
----

I filmed this video with my phone, because screen capture somehow ruins the
frame rate and produces artifacts all over.

[![Phone video of the display of the GIF in the terminal](https://img.youtube.com/vi/AqIT7vIFiDQ/maxresdefault.jpg)](https://www.youtube.com/watch?v=AqIT7vIFiDQ)

In this video I display this GIF (made it myself) in a 293 columns by 77 lines
Konsole terminal window on a 4k monitor and otherwise old hardware.

![Rotating cartoonish gold coin](https://i.imgur.com/A6ThmHM.gif)

Usage
-----

```plain
Usage: ansi-img [OPTIONS] <PATH>

Arguments:
  <PATH>
          

Options:
  -l, --loop-count <LOOP_COUNT>
          Times to loop the animation.
          
          Negative values mean infinite looping.
          
          [default: -1]

  -s, --style <STYLE>
          Placement and scaling.
          
          Values:
           - center
           - tile
           - <x> <y> [z]
           - <x> <y> <w> <h>
           - cover
           - contain
           - shrink-to-fit (or shrinktofit)
          
          x and y can be * to center within the canvas.
          
          z is a zoom value. It is either a whole number >= 1 or a fraction <= 1/2.
          
          w and h can be * so it's derived from the respective other value.
          
          [default: shrink-to-fit]

  -c, --canvas-size <CANVAS_SIZE>
          Size of the canvas.
          
          Values:
           - window
           - image
           - <width> <height>
          
          [default: window]

  -a, --alpha-threshold <ALPHA_THRESHOLD>
          [default: 127]

  -f, --filter <FILTER>
          Filter used when resizing images.
          
          Values:
           - nearest
           - triangle
           - catmull-rom (or catmullrom)
           - gaussian
           - lanczos3
          
          [default: Nearest]

  -b, --background-color <BACKGROUND_COLOR>
          Set the background color.
          
          Values:
           - transparent
           - #RRGGBB
          
          [default: transparent]

  -l, --line-end <LINE_END>
          Line ending to use.
          
          Values:
           - Cr
           - Lf
           - CrLf
          
          [default: Lf]

  -i, --inline
          Don't clear screen and render image wherever the cursor currently is

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
