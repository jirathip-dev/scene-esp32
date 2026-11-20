use esp_idf_svc::hal::adc::ADC1;
use esp_idf_svc::hal::gpio::{AnyIOPin, Gpio1, Gpio2};
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::sys::EspError;

use super::led::Led;
use super::oled::Oled;
use super::tft::{Tft, TftPins};

/// The controls' parts: buttons 1–4 and the stick's switch (pull-up, active low), the stick's ADC1 axes.
pub struct Controls {
    pub buttons: [AnyIOPin<'static>; 4],
    pub stick: AnyIOPin<'static>,
    pub vrx: Gpio1<'static>,
    pub vry: Gpio2<'static>,
    pub adc: ADC1<'static>,
}

/// The board's parts, each on its pins from the pin map in `docs/board.md`.
pub struct Board {
    pub tft: Tft,
    pub oled: Oled,
    pub led: Led,
    pub controls: Controls,
    pub wifi: Modem<'static>,
}

impl Board {
    /// Takes the chip's peripherals and brings up each part: the one place a GPIO number is named.
    pub fn take() -> Result<Self, EspError> {
        let p = Peripherals::take()?;
        let g = p.pins;
        let tft = TftPins {
            sck: g.gpio12.degrade_output(),
            mosi: g.gpio11.degrade_output(),
            cs: g.gpio10.degrade_output(),
            dc: g.gpio9.degrade_output(),
            rst: g.gpio14.degrade_output(),
            backlight: g.gpio21.degrade_output(),
            touch_cs: g.gpio15.degrade_output(),
        };
        #[cfg(not(feature = "led-gpio38"))]
        let led = g.gpio48;
        #[cfg(feature = "led-gpio38")]
        let led = g.gpio38;
        Ok(Self {
            tft: Tft::new(p.spi2, tft, p.ledc.timer0, p.ledc.channel0)?,
            oled: Oled::new(p.i2c0, g.gpio17, g.gpio18)?,
            led: Led::new(led)?,
            controls: Controls {
                buttons: [
                    g.gpio5.degrade_input_output(),
                    g.gpio6.degrade_input_output(),
                    g.gpio7.degrade_input_output(),
                    g.gpio8.degrade_input_output(),
                ],
                stick: g.gpio4.degrade_input_output(),
                vrx: g.gpio1,
                vry: g.gpio2,
                adc: p.adc1,
            },
            wifi: p.modem,
        })
    }
}
