use std::time::Duration;

use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::rmt::config::{TransmitConfig, TxChannelConfig};
use esp_idf_svc::hal::rmt::encoder::{BytesEncoder, BytesEncoderConfig};
use esp_idf_svc::hal::rmt::{PinState, Symbol, TxChannelDriver};
use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::sys::EspError;

const TICKS: Hertz = Hertz(20_000_000);
const RED: [u8; 3] = [0, 16, 0];
const OFF: [u8; 3] = [0, 0, 0];

/// The onboard WS2812: red while any row needs you, else off.
pub struct Led {
    tx: TxChannelDriver<'static>,
    ws2812: BytesEncoder,
}

impl Led {
    /// Drives the WS2812 on `pin` through the RMT.
    pub fn new(pin: impl OutputPin + 'static) -> Result<Self, EspError> {
        let config = TxChannelConfig {
            resolution: TICKS,
            ..Default::default()
        };
        let ws2812 = BytesEncoder::with_config(&BytesEncoderConfig {
            bit0: bit(350, 800)?,
            bit1: bit(700, 600)?,
            msb_first: true,
            ..Default::default()
        })?;
        Ok(Self {
            tx: TxChannelDriver::new(pin, &config)?,
            ws2812,
        })
    }

    /// Shows red when `needs_you`, else nothing.
    pub fn show(&mut self, needs_you: bool) -> Result<(), EspError> {
        let grb = if needs_you { RED } else { OFF };
        self.tx
            .send_and_wait(&mut self.ws2812, &grb, &TransmitConfig::default())
    }
}

fn bit(high: u64, low: u64) -> Result<Symbol, EspError> {
    Symbol::new_with(
        TICKS,
        PinState::High,
        Duration::from_nanos(high),
        PinState::Low,
        Duration::from_nanos(low),
    )
}
