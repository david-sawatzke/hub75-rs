# HUB75
Library for controlling the cheap RGB Displays with the interface colloquially
known as hub75 together with `embedded-graphics` & `embedded-hal` impls in rust.

Currently only supports panels with a resolution of 64x32 (tested on panel "P3-(2121)64*32-16S-D10").

See
(rpi-rgb-led-matrix)[https://github.com/hzeller/rpi-rgb-led-matrix/blob/master/wiring.md]
for hookup instructions.
Pinout: ![Hub 75 interface][hub75]

## Problem Solving
- It flickers

  Reduce the bits for the color output, call the `output` method more often or use a faster micro
- Some colors aren't displayed correctly/not at all

  If one of the rgb components after gamma correction has less than the provided
  bits, it isn't shown at all. For example, when using 3 color bits, having a
  value less than 124 leads to nothing being shown (as it's then gamma corrected
  to 31, which is less than 1<<5).
[hub75]: ./img/hub75.jpg
