#![no_std]
// Inspired by
// - https://github.com/polyfloyd/ledcat/blob/master/src/device/hub75.rs
// - https://github.com/mmou/led-marquee/blob/8c88531a6938edff6db829ca21c15304515874ea/src/hub.rs
// - https://github.com/adafruit/RGB-matrix-Panel/blob/master/RGBmatrixPanel.cpp
// - https://www.mikrocontroller.net/topic/452187 (sorry, german only)

/// # Theory of Operation
/// This display is essentially split in half, with the top 16 rows being
/// controlled by one set of shift registers (r1, g1, b1) and the botton 16
/// rows by another set (r2, g2, b2). So, the best way to update it is to
/// show one of the botton and top rows in tandem. The row (between 0-15) is then
/// selected by the A, B, C, D pins, which are just, as one might expect, the bits 0 to 3.
/// Pin F is used by the 64x64 display to get 5 bit row addressing (1/32 row scan rate)
///
/// The display doesn't really do brightness, so we have to do it ourselves, by
/// rendering the same frame multiple times, with some pixels being turned of if
/// they are darker (pwm)
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb565, RgbColor as _},
    Pixel,
};
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::OutputPin;

#[cfg(feature = "size-64x64")]
const NUM_ROWS: usize = 32;
#[cfg(not(feature = "size-64x64"))]
const NUM_ROWS: usize = 16;

pub type DataRow = [(u8, u8, u8, u8, u8, u8); 64];

pub struct Hub75<PINS> {
    //       r1, g1, b1, r2, g2, b2, column, row
    data: [DataRow; NUM_ROWS],
    brightness_step: u8,
    brightness_count: u8,
    pins: PINS,
}

/// A trait, so that it's easier to reason about the pins
/// Implemented for a tuple `(r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe)`
/// with every element implementing `OutputPin`
/// f pin is needed for 64x64 matrix support
pub trait Outputs {
    type Error;
    type R1: OutputPin<Error = Self::Error>;
    type G1: OutputPin<Error = Self::Error>;
    type B1: OutputPin<Error = Self::Error>;
    type R2: OutputPin<Error = Self::Error>;
    type G2: OutputPin<Error = Self::Error>;
    type B2: OutputPin<Error = Self::Error>;
    type A: OutputPin<Error = Self::Error>;
    type B: OutputPin<Error = Self::Error>;
    type C: OutputPin<Error = Self::Error>;
    type D: OutputPin<Error = Self::Error>;
    #[cfg(feature = "size-64x64")]
    type F: OutputPin<Error = Self::Error>;
    type CLK: OutputPin<Error = Self::Error>;
    type LAT: OutputPin<Error = Self::Error>;
    type OE: OutputPin<Error = Self::Error>;
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
    #[cfg(feature = "size-64x64")]
    fn f(&mut self) -> &mut Self::F;
    fn clk(&mut self) -> &mut Self::CLK;
    fn lat(&mut self) -> &mut Self::LAT;
    fn oe(&mut self) -> &mut Self::OE;
}

#[cfg(feature = "size-64x64")]
impl<
        E,
        R1: OutputPin<Error = E>,
        G1: OutputPin<Error = E>,
        B1: OutputPin<Error = E>,
        R2: OutputPin<Error = E>,
        G2: OutputPin<Error = E>,
        B2: OutputPin<Error = E>,
        A: OutputPin<Error = E>,
        B: OutputPin<Error = E>,
        C: OutputPin<Error = E>,
        D: OutputPin<Error = E>,
        F: OutputPin<Error = E>,
        CLK: OutputPin<Error = E>,
        LAT: OutputPin<Error = E>,
        OE: OutputPin<Error = E>,
    > Outputs for (R1, G1, B1, R2, G2, B2, A, B, C, D, F, CLK, LAT, OE)
{
    type Error = E;
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
    type F = F;
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
    fn f(&mut self) -> &mut F {
        &mut self.10
    }
    fn clk(&mut self) -> &mut CLK {
        &mut self.11
    }
    fn lat(&mut self) -> &mut LAT {
        &mut self.12
    }
    fn oe(&mut self) -> &mut OE {
        &mut self.13
    }
}

#[cfg(not(feature = "size-64x64"))]
impl<
        E,
        R1: OutputPin<Error = E>,
        G1: OutputPin<Error = E>,
        B1: OutputPin<Error = E>,
        R2: OutputPin<Error = E>,
        G2: OutputPin<Error = E>,
        B2: OutputPin<Error = E>,
        A: OutputPin<Error = E>,
        B: OutputPin<Error = E>,
        C: OutputPin<Error = E>,
        D: OutputPin<Error = E>,
        CLK: OutputPin<Error = E>,
        LAT: OutputPin<Error = E>,
        OE: OutputPin<Error = E>,
    > Outputs for (R1, G1, B1, R2, G2, B2, A, B, C, D, CLK, LAT, OE)
{
    type Error = E;
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
        let data = [[(0, 0, 0, 0, 0, 0); 64]; NUM_ROWS];
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
    pub fn output<DELAY: DelayUs<u8>>(&mut self, delay: &mut DELAY) -> Result<(), PINS::Error> {
        // Enable the output
        // The previous last row will continue to display
        self.pins.oe().set_low()?;
        // PWM cycle
        for mut brightness in 0..self.brightness_count {
            brightness = (brightness + 1).saturating_mul(self.brightness_step);
            for (count, row) in self.data.iter().enumerate() {
                for element in row.iter() {
                    if element.0 >= brightness {
                        self.pins.r1().set_high()?;
                    } else {
                        self.pins.r1().set_low()?;
                    }
                    if element.1 >= brightness {
                        self.pins.g1().set_high()?;
                    } else {
                        self.pins.g1().set_low()?;
                    }
                    if element.2 >= brightness {
                        self.pins.b1().set_high()?;
                    } else {
                        self.pins.b1().set_low()?;
                    }
                    if element.3 >= brightness {
                        self.pins.r2().set_high()?;
                    } else {
                        self.pins.r2().set_low()?;
                    }
                    if element.4 >= brightness {
                        self.pins.g2().set_high()?;
                    } else {
                        self.pins.g2().set_low()?;
                    }
                    if element.5 >= brightness {
                        self.pins.b2().set_high()?;
                    } else {
                        self.pins.b2().set_low()?;
                    }
                    self.pins.clk().set_high()?;
                    self.pins.clk().set_low()?;
                }
                self.pins.oe().set_high()?;
                // Prevents ghosting, no idea why
                delay.delay_us(2);
                self.pins.lat().set_low()?;
                delay.delay_us(2);
                self.pins.lat().set_high()?;
                // Select row
                if count & 1 != 0 {
                    self.pins.a().set_high()?;
                } else {
                    self.pins.a().set_low()?;
                }
                if count & 2 != 0 {
                    self.pins.b().set_high()?;
                } else {
                    self.pins.b().set_low()?;
                }
                if count & 4 != 0 {
                    self.pins.c().set_high()?;
                } else {
                    self.pins.c().set_low()?;
                }
                if count & 8 != 0 {
                    self.pins.d().set_high()?;
                } else {
                    self.pins.d().set_low()?;
                }
                #[cfg(feature = "size-64x64")]
                if count & 16 != 0 {
                    self.pins.f().set_high()?;
                } else {
                    self.pins.f().set_low()?;
                }
                delay.delay_us(2);
                self.pins.oe().set_low()?;
            }
        }
        // Disable the output
        // Prevents one row from being much brighter than the others
        self.pins.oe().set_high()?;
        Ok(())
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

pub type Error = &'static str;
// This table remaps linear input values
// (the numbers we’d like to use; e.g. 127 = half brightness)
// to nonlinear gamma-corrected output values
// (numbers producing the desired effect on the LED;
// e.g. 36 = half brightness).
const GAMMA8: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5,
    5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14,
    14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25, 25, 26, 27,
    27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46,
    47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72,
    73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104,
    105, 107, 109, 110, 112, 114, 115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137,
    138, 140, 142, 144, 146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175,
    177, 180, 182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
    223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,
];

impl<PINS> OriginDimensions for Hub75<PINS> {
    fn size(&self) -> Size {
        #[cfg(feature = "size-64x64")]
        {
            Size {
                width: 64,
                height: 64,
            }
        }
        #[cfg(not(feature = "size-64x64"))]
        {
            Size {
                width: 64,
                height: 32,
            }
        }
    }
}

impl<PINS: Outputs> DrawTarget for Hub75<PINS> {
    type Color = Rgb565;
    type Error = Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            let row = coord[1] % NUM_ROWS as i32;
            let data = &mut self.data[row as usize][coord[0] as usize];
            if coord[1] >= NUM_ROWS as i32 {
                data.3 = GAMMA8[color.r() as usize];
                data.4 = GAMMA8[color.g() as usize];
                data.5 = GAMMA8[color.b() as usize];
            } else {
                data.0 = GAMMA8[color.r() as usize];
                data.1 = GAMMA8[color.g() as usize];
                data.2 = GAMMA8[color.b() as usize];
            }
        }
        Ok(())
    }
}
