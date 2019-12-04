#![no_std]
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::OutputPin;
// Inspired by
// - https://github.com/polyfloyd/ledcat/blob/master/src/device/hub75.rs
// - https://github.com/mmou/led-marquee/blob/8c88531a6938edff6db829ca21c15304515874ea/src/hub.rs
// - https://github.com/adafruit/RGB-matrix-Panel/blob/master/RGBmatrixPanel.cpp
// - https://www.mikrocontroller.net/topic/452187 (sorry, german only)

// # How this works
// This display is essentially split in half, with the top 16 rows being
// controlled by one set of shift registers (r1, g1, b1) and the botton 16
// rows by another set (r2, g2, b2). So, the best way to update it is to
// show one of the botton and top rows in tandem. The row (between 0-15) is then
// selected by the A, B, C, D pins, which are just, as one might expect, the bits 0 to 3.
//
// The display doesn't really do brightness, so we have to do it ourselves, by
// rendering the same frame multiple times, with some pixels being turned of if
// they are darker (pwm)

pub struct Hub75<PINS> {
    //       r1, g1, b1, r2, g2, b2, column, row
    data: [[(u8, u8, u8, u8, u8, u8); 64]; 16],
    brightness_step: u8,
    brightness_count: u8,
    pins: PINS,
}

pub trait Outputs {
    type R1: OutputPin;
    type G1: OutputPin;
    type B1: OutputPin;
    type R2: OutputPin;
    type G2: OutputPin;
    type B2: OutputPin;
    type A: OutputPin;
    type B: OutputPin;
    type C: OutputPin;
    type D: OutputPin;
    type CLK: OutputPin;
    type LAT: OutputPin;
    type OE: OutputPin;
    fn r1(&mut self) -> &mut Self::R1;
    fn g1(&mut self) -> &mut Self::G1;
    fn b1(&mut self) -> &mut Self::B1;
    fn r2(&mut self) -> &mut Self::R2;
    fn g2(&mut self) -> &mut Self::G2;
    fn b2(&mut self) -> &mut Self::B2;
    fn a(&mut self) -> &mut Self::A;
    fn b(&mut self) -> &mut Self::B;
    fn c(&mut self) -> &mut Self::C;
    fn d(&mut self) -> &mut Self::D;
    fn clk(&mut self) -> &mut Self::CLK;
    fn lat(&mut self) -> &mut Self::LAT;
    fn oe(&mut self) -> &mut Self::OE;
}

impl<
        R1: OutputPin,
        G1: OutputPin,
        B1: OutputPin,
        R2: OutputPin,
        G2: OutputPin,
        B2: OutputPin,
        A: OutputPin,
        B: OutputPin,
        C: OutputPin,
        D: OutputPin,
        CLK: OutputPin,
        LAT: OutputPin,
        OE: OutputPin,
    > Outputs for (R1, G1, B1, R2, G2, B2, A, B, C, D, CLK, LAT, OE)
{
    type R1 = R1;
    type G1 = G1;
    type B1 = B1;
    type R2 = R2;
    type G2 = G2;
    type B2 = B2;
    type A = A;
    type B = B;
    type C = C;
    type D = D;
    type CLK = CLK;
    type LAT = LAT;
    type OE = OE;
    fn r1(&mut self) -> &mut R1 {
        &mut self.0
    }
    fn g1(&mut self) -> &mut G1 {
        &mut self.1
    }
    fn b1(&mut self) -> &mut B1 {
        &mut self.2
    }
    fn r2(&mut self) -> &mut R2 {
        &mut self.3
    }
    fn g2(&mut self) -> &mut G2 {
        &mut self.4
    }
    fn b2(&mut self) -> &mut B2 {
        &mut self.5
    }
    fn a(&mut self) -> &mut A {
        &mut self.6
    }
    fn b(&mut self) -> &mut B {
        &mut self.7
    }
    fn c(&mut self) -> &mut C {
        &mut self.8
    }
    fn d(&mut self) -> &mut D {
        &mut self.9
    }
    fn clk(&mut self) -> &mut CLK {
        &mut self.10
    }
    fn lat(&mut self) -> &mut LAT {
        &mut self.11
    }
    fn oe(&mut self) -> &mut OE {
        &mut self.12
    }
}

impl<PINS: Outputs> Hub75<PINS> {
    /// Create a new hub instance
    ///
    /// Takes an implementation of the Outputs trait,
    /// using a tuple `(r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe)`,
    /// with every member implementing `OutputPin` is usually the right choice.
    ///
    /// `brightness_bits` provides the number of brightness_bits for each color (1-8).
    /// More bits allow for much more colors, especially in combination with the gamma correction,
    /// but each extra bit doubles the time `output` will take. This might lead to noticable flicker.
    ///
    /// 3-4 bits are usually a good choice.
    pub fn new(pins: PINS, brightness_bits: u8) -> Self {
        assert!(brightness_bits < 9 && brightness_bits > 0);
        let data = [[(0, 0, 0, 0, 0, 0); 64]; 16];
        let brightness_step = 1 << (8 - brightness_bits);
        let brightness_count = ((1 << brightness_bits as u16) - 1) as u8;
        Self {
            data,
            brightness_step,
            brightness_count,
            pins,
        }
    }

    /// Output the buffer to the display
    ///
    /// Takes some time and should be called quite often, otherwise the output
    /// will flicker
    pub fn output<DELAY: DelayUs<u8>>(&mut self, delay: &mut DELAY) {
        // Enable the output
        // The previous last row will continue to display
        self.pins.oe().set_low().ok();
        // PWM cycle
        for mut brightness in 0..self.brightness_count {
            brightness = (brightness + 1).saturating_mul(self.brightness_step);
            for (count, row) in self.data.iter().enumerate() {
                for element in row.iter() {
                    if element.0 >= brightness {
                        self.pins.r1().set_high().ok();
                    } else {
                        self.pins.r1().set_low().ok();
                    }
                    if element.1 >= brightness {
                        self.pins.g1().set_high().ok();
                    } else {
                        self.pins.g1().set_low().ok();
                    }
                    if element.2 >= brightness {
                        self.pins.b1().set_high().ok();
                    } else {
                        self.pins.b1().set_low().ok();
                    }
                    if element.3 >= brightness {
                        self.pins.r2().set_high().ok();
                    } else {
                        self.pins.r2().set_low().ok();
                    }
                    if element.4 >= brightness {
                        self.pins.g2().set_high().ok();
                    } else {
                        self.pins.g2().set_low().ok();
                    }
                    if element.5 >= brightness {
                        self.pins.b2().set_high().ok();
                    } else {
                        self.pins.b2().set_low().ok();
                    }
                    self.pins.clk().set_high().ok();
                    self.pins.clk().set_low().ok();
                }
                self.pins.oe().set_high().ok();
                // Prevents ghosting, no idea why
                delay.delay_us(2);
                self.pins.lat().set_low().ok();
                delay.delay_us(2);
                self.pins.lat().set_high().ok();
                // Select row
                if count & 1 != 0 {
                    self.pins.a().set_high().ok();
                } else {
                    self.pins.a().set_low().ok();
                }
                if count & 2 != 0 {
                    self.pins.b().set_high().ok();
                } else {
                    self.pins.b().set_low().ok();
                }
                if count & 4 != 0 {
                    self.pins.c().set_high().ok();
                } else {
                    self.pins.c().set_low().ok();
                }
                if count & 8 != 0 {
                    self.pins.d().set_high().ok();
                } else {
                    self.pins.d().set_low().ok();
                }
                delay.delay_us(2);
                self.pins.oe().set_low().ok();
            }
        }
        // Disable the output
        // Prevents one row from being much brighter than the others
        self.pins.oe().set_high().ok();
    }
    /// Clear the output
    ///
    /// It's a bit faster than using the embedded_graphics interface
    /// to do the same
    pub fn clear(&mut self) {
        for row in self.data.iter_mut() {
            for e in row.iter_mut() {
                e.0 = 0;
                e.1 = 0;
                e.2 = 0;
                e.3 = 0;
                e.4 = 0;
                e.5 = 0;
            }
        }
    }
}

use embedded_graphics::{
    drawable::{Dimensions, Pixel},
    pixelcolor::Rgb565,
    Drawing, SizedDrawing,
};
impl<PINS: Outputs> Drawing<Rgb565> for Hub75<PINS> {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: IntoIterator<Item = Pixel<Rgb565>>,
    {
        // This table remaps linear input values
        // (the numbers we’d like to use; e.g. 127 = half brightness)
        // to nonlinear gamma-corrected output values
        // (numbers producing the desired effect on the LED;
        // e.g. 36 = half brightness).
        const GAMMA8: [u8; 256] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4,
            4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11,
            12, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22,
            22, 23, 24, 24, 25, 25, 26, 27, 27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37,
            38, 39, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58,
            59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72, 73, 74, 75, 77, 78, 79, 81, 82, 83, 85,
            86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104, 105, 107, 109, 110, 112, 114,
            115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137, 138, 140, 142, 144,
            146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175, 177, 180,
            182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
            223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,
        ];
        for Pixel(coord, color) in item_pixels {
            let row = coord[1] % 16;
            let data = &mut self.data[row as usize][coord[0] as usize];
            if coord[1] >= 16 {
                data.3 = GAMMA8[color.r() as usize];
                data.4 = GAMMA8[color.g() as usize];
                data.5 = GAMMA8[color.b() as usize];
            } else {
                data.0 = GAMMA8[color.r() as usize];
                data.1 = GAMMA8[color.g() as usize];
                data.2 = GAMMA8[color.b() as usize];
            }
        }
    }
}

// TODO Does it make sense to include this?
impl<PINS: Outputs> SizedDrawing<Rgb565> for Hub75<PINS> {
    fn draw_sized<T>(&mut self, item_pixels: T)
    where
        T: IntoIterator<Item = Pixel<Rgb565>> + Dimensions,
    {
        self.draw(item_pixels);
    }
}
