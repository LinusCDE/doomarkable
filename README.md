# DOOMarkable

[![rm1](https://img.shields.io/badge/rM1-supported-green)](https://remarkable.com/store/remarkable)
[![rm2](https://img.shields.io/badge/rM2-not_recommended-yellow)](https://remarkable.com/store/remarkable-2)
[![opkg](https://img.shields.io/badge/OPKG-doomarkable-blue)](https://github.com/toltec-dev/toltec)
[![launchers](https://img.shields.io/badge/Launchers-supported-green)](https://github.com/reHackable/awesome-reMarkable#launchers)

This is a doom port intended for the reMarkable 1.

<img src="https://transfer.cosmos-ink.net/3G3F8j/doom_title_screen.jpg" width="45%">&nbsp;<img src="https://transfer.cosmos-ink.net/ji7NIv/doom_screenshot.jpg" width="45%">

[Demo Video on rM 1](https://youtu.be/wdH3GFU74sM) | [Demo Video on rM 2 (not recommended)](https://youtu.be/PFR3QHZ7kGw)

## What's mainly used and how it's done

It is composed out of a lot of different compontents:

- [doomgeneric-rs](https://github.com/LinusCDE/doomgeneric-rs) - Rust bindings for doomgeneric
  - [doomgeneric](https://github.com/ozkl/doomgeneric) - An awesome and easy to use doom port made by @ozkl
- [libremarkable](https://github.com/canselcik/libremarkable/) - Drawing to the display and reading inputs
- [blue-noise](https://github.com/mblode/blue-noise/) - An amazing dithering algorithm to fake grayscale output

The meat of the work was to port doom to rust (doomgeneric-rs) and dithering the image and doing that as fast as possible!
The dither speed was achived through forcing better optimizations and caching the code. The dithering is actually done at compile time for a 320x200 source image and the results (for upscaling 4x) are put into the generated binary itself. The binary then just needs to decompress this and look up the results for each pixel.

## Current state

The game currently runs at about 11-14 FPS on the device.
It's not using the low latency drawing even though it's pretty simple to use since the image has no gray shades.
The reason is that using an A2-Like refresh has less artifacts and ghosting.
I personally find this worth the extra latency when playing.

The game currently runs fine but there are still some things to do:

- [ ] Making it easy to get the game resources **(semi done)**
- [ ] Properly exit the game without requiring killing the process **(semi done)**
- [ ] Adjusting gamma to make dithered visuals clearer for certain rooms **(semi done)**
- [x] Add an battery indicator (this sucks a lot of juice ..ahem.. blood)
- [ ] Package it up for [toltec](https://github.com/toltec-dev/toltec) and inclusion in [launchers](https://github.com/reHackable/awesome-reMarkable#launchers)
- [ ] Consider a smaller size for the rM 2, so the eink software driver doesn't die trying to update that many dots

## How to run

- Download the latest binary from the [release page](https://github.com/LinusCDE/doomarkable/releases) (the file without any extension)
- Copy the file to e.g. `/home/root` on your reMarkable (e.g. using FileZilla or WinSCP)
- Find an appropriate IWAD file (game resources) and put it in the same directory ([more details](https://github.com/LinusCDE/piston-doom#get-an-iwad-file))
- Log into the device using ssh (e.g. with Putty) and go into your chosen directory
- Make the binary executable by running `chmod +x doomarkable`
- Ensure no other UI is running (e.g. stop the default UI with `systemctl stop xochitl` and start it later using start instead of stop)
- Run the binary: `./doomarkable` (on the rM 2, you'll need [rm2fb](https://github.com/ddvk/remarkable2-framebuffer) and prefix that command with `rm2fb-client`)
- DOOM should now run on your device. If the game doesn't come up, view the output for any errors or enable debugging by adding `RUST_LOG=debug` before the command
