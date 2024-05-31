ANSI IMG
========

Display images on the terminal using ANSI escape sequences and Unicode
characters.

This uses the [Unicode block elements](https://en.wikipedia.org/wiki/Block_Elements)
`█`, `▄`, and `▀` plus
[24-bit color ANSI escape sequences](https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit)
to set foreground and background colors in order to render two pixels for each
character. Not every terminal supports these escape sequences, and some that do
downsample the color quite heavily. Also some terminals don't render the
characters perfectly and sometimes leave small gaps.

It supports displaying animated GIFs! (Press Ctrl+C to quit the animation.)

Demo
----

I filmed it with my phone, because screen capture somehow ruins the frame rate
and produces artifacts all over.

[![Phone video of the display of the GIF in the terminal](https://img.youtube.com/vi/AqIT7vIFiDQ/maxresdefault.jpg)](https://www.youtube.com/watch?v=AqIT7vIFiDQ)

In this video I display this GIF (made it myself) in a 293 columns by 77 lines
Konsole terminal window.

![Rotating cartoonish gold coin](https://i.imgur.com/A6ThmHM.gif)
