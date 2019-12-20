# HUB75

![Example image](example.jpg)

Library for controlling the cheap RGB matrix displays with the interface colloquially
known as hub75 with `embedded-graphics` & `embedded-hal` impls in rust.

Currently only supports panels with a resolution of 64x32 (tested on panel "P3-(2121)64*32-16S-D10").

See
(rpi-rgb-led-matrix)[https://github.com/hzeller/rpi-rgb-led-matrix/blob/master/wiring.md]
for hookup instructions.

Pinout:

![Hub 75 interface](hub75.jpg)

## Problem Solving
- It flickers

  Reduce the bits for the color output, call the `output` method more often or use a faster micro
- Some colors aren't displayed correctly/not at all

  If one of the rgb components after gamma correction has less than the provided
  bits, it isn't shown at all. For example, when using 3 color bits, having a
  value less than 124 leads to nothing being shown (as it's then gamma corrected
  to 31, which is less than 1<<5).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
