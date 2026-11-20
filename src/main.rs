use std::time::Duration;

use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::rmt::config::{TransmitConfig, TxChannelConfig};
use esp_idf_svc::hal::rmt::encoder::bytes_encoder::{BytesEncoder, BytesEncoderConfig};
use esp_idf_svc::hal::rmt::{PinState, Symbol, TxChannelDriver};
use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::sys::EspError;

const TICKS: Hertz = Hertz(20_000_000);
const ON: [u8; 3] = [16, 0, 0];
const OFF: [u8; 3] = [0, 0, 0];

fn bit(high: u64, low: u64) -> Result<Symbol, EspError> {
    Symbol::new_with(
        TICKS,
        PinState::High,
        Duration::from_nanos(high),
        PinState::Low,
        Duration::from_nanos(low),
    )
}

fn main() -> Result<(), EspError> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    match scene_esp32::block_count(scene_esp32::SCENE) {
        Ok(n) => log::info!("scene fixture: {n} blocks"),
        Err(e) => log::error!("scene fixture does not parse: {e}"),
    }

    let pins = Peripherals::take()?.pins;
    #[cfg(not(feature = "led-gpio38"))]
    let pin = pins.gpio48;
    #[cfg(feature = "led-gpio38")]
    let pin = pins.gpio38;
    let mut led = TxChannelDriver::new(
        pin,
        &TxChannelConfig {
            resolution: TICKS,
            ..Default::default()
        },
    )?;
    let mut ws2812 = BytesEncoder::with_config(&BytesEncoderConfig {
        bit0: bit(350, 800)?,
        bit1: bit(700, 600)?,
        msb_first: true,
        ..Default::default()
    })?;

    for grb in [ON, OFF].iter().cycle() {
        led.send_and_wait(&mut ws2812, grb, &TransmitConfig::default())?;
        std::thread::sleep(Duration::from_millis(500));
    }
    Ok(())
}
