use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver, I2C0};
use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::sys::EspError;
use scene_esp32::render;
use scene_esp32::view::Status;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;
use ssd1306::{I2CDisplayInterface, Ssd1306};

use super::fail;

type Panel = Ssd1306<
    I2CInterface<I2cDriver<'static>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

/// The SSD1306 OLED on I2C at 400 kHz.
pub struct Oled(Panel);

impl Oled {
    /// Brings up I2C0 and the panel.
    pub fn new(
        i2c: I2C0<'static>,
        sda: impl InputPin + OutputPin + 'static,
        scl: impl InputPin + OutputPin + 'static,
    ) -> Result<Self, EspError> {
        let config = I2cConfig::new().baudrate(Hertz(400_000));
        let i2c = I2cDriver::new(i2c, sda, scl, &config)?;
        let mut panel = Ssd1306::new(
            I2CDisplayInterface::new(i2c),
            DisplaySize128x64,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics_mode();
        panel.init().map_err(fail("oled init"))?;
        Ok(Self(panel))
    }

    /// Draws `status` and sends it.
    pub fn show(&mut self, status: &Status) -> Result<(), EspError> {
        render::status(status, &mut self.0).map_err(fail("oled draw"))?;
        self.0.flush().map_err(fail("oled flush"))
    }
}
