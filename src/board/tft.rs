use std::time::Instant;

use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{AnyIOPin, AnyOutputPin, Output, PinDriver};
use esp_idf_svc::hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, LowSpeed};
use esp_idf_svc::hal::ledc::{CHANNEL0, TIMER0};
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::{Dma, SpiDeviceDriver, SpiDriver, SPI2};
use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::sys::EspError;
#[cfg(feature = "ili9341")]
use mipidsi::models::ILI9341Rgb565 as Model;
#[cfg(feature = "st7789")]
use mipidsi::models::ST7789 as Model;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{interface::SpiInterface, Builder, Display};
use scene_esp32::app::Backlight;
use scene_esp32::render::FrameBuf;

use super::fail;

const SPI_HZ: Hertz = Hertz(20_000_000);
const ORDER: ColorOrder = if cfg!(feature = "st7789") {
    ColorOrder::Rgb
} else {
    ColorOrder::Bgr
};
const INVERSION: ColorInversion = if cfg!(feature = "st7789") {
    ColorInversion::Inverted
} else {
    ColorInversion::Normal
};

type Bus = SpiDeviceDriver<'static, SpiDriver<'static>>;
type Pin = PinDriver<'static, Output>;
type Panel = Display<SpiInterface<'static, Bus, Pin>, Model, Pin>;

/// The TFT's pins, named as the README's pin map names them.
pub struct TftPins {
    pub sck: AnyOutputPin<'static>,
    pub mosi: AnyOutputPin<'static>,
    pub cs: AnyOutputPin<'static>,
    pub dc: AnyOutputPin<'static>,
    pub rst: AnyOutputPin<'static>,
    pub backlight: AnyOutputPin<'static>,
    pub touch_cs: AnyOutputPin<'static>,
}

/// The TFT in landscape on SPI2, its PWM backlight, and the touch chip held off the shared bus.
pub struct Tft {
    panel: Panel,
    backlight: LedcDriver<'static>,
    _timer: LedcTimerDriver<'static, LowSpeed>,
    _touch_cs: Pin,
}

impl Tft {
    /// Brings up SPI2 at 20 MHz and the panel the build's feature names.
    pub fn new(
        spi: SPI2<'static>,
        pins: TftPins,
        timer: TIMER0<'static>,
        channel: CHANNEL0<'static>,
    ) -> Result<Self, EspError> {
        let mut touch_cs = PinDriver::output(pins.touch_cs)?;
        touch_cs.set_high()?;
        let bus_config = DriverConfig::new().dma(Dma::Auto(4096));
        let bus = SpiDriver::new(spi, pins.sck, pins.mosi, None::<AnyIOPin>, &bus_config)?;
        let config = Config::new().baudrate(SPI_HZ).write_only(true);
        let device = SpiDeviceDriver::new(bus, Some(pins.cs), &config)?;
        let di = SpiInterface::new(device, PinDriver::output(pins.dc)?, vec![0; 4096].leak());
        let panel = Builder::new(Model, di)
            .reset_pin(PinDriver::output(pins.rst)?)
            .color_order(ORDER)
            .invert_colors(INVERSION)
            .orientation(Orientation::new().rotate(Rotation::Deg90))
            .init(&mut Ets)
            .map_err(fail("tft init"))?;
        let timer = LedcTimerDriver::new(timer, &TimerConfig::default())?;
        let backlight = LedcDriver::new(channel, &timer, pins.backlight)?;
        Ok(Self {
            panel,
            backlight,
            _timer: timer,
            _touch_cs: touch_cs,
        })
    }

    /// Sends the rectangles `fb` touched since the last flush and logs how long it took.
    pub fn flush(&mut self, fb: &mut FrameBuf) -> Result<(), EspError> {
        let touched = fb.take_touched();
        if touched.is_empty() {
            return Ok(());
        }
        let started = Instant::now();
        let mut sent = 0;
        for area in touched {
            let Some(end) = area.bottom_right() else {
                continue;
            };
            let (x, y) = (area.top_left.x as u16, area.top_left.y as u16);
            self.panel
                .set_pixels(x, y, end.x as u16, end.y as u16, fb.pixels(area))
                .map_err(fail("tft flush"))?;
            sent += area.size.width * area.size.height;
        }
        let us = started.elapsed().as_micros();
        log::info!("flush {sent} px in {us} us");
        Ok(())
    }

    /// Sets the backlight's duty for `level`.
    pub fn backlight(&mut self, level: Backlight) -> Result<(), EspError> {
        let max = self.backlight.get_max_duty();
        self.backlight.set_duty(match level {
            Backlight::Full => max,
            Backlight::Dim => max / 8,
            Backlight::Off => 0,
        })
    }
}
