# pdp12 emulator for the web

![image](https://user-images.githubusercontent.com/1576660/203506524-54a6beaf-112b-4d48-bd27-c44aa324995e.png)

I'm building a PDP-12 emulator and this is my current UI. The top decal/light panel is an SVG. The lights on the top panel are turned on by switching out one gradient url with another using Javascript.

The switches are html input type="checkbox" that are styled with more svgs as background images. They make a nice clicking sound when you switch them.

Currently only the PDP-8 instructions of the computer are implemented, and they are implemented using Rust compiled to WASM. I want this project to be usable by all, so that's why it's web based. But I also want it to run at original speed at minimum, so I wanted to use a "fast language".

I want to add all the LINC instructions, and some IO peripherals as well. I want at least the ASR-32 teletype to work, so you can load in programs using it's paper tape loader. Then I'll start looking at the CRT module the PDP-12 has and the magnetic tape module.
