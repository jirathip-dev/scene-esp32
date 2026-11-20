use std::sync::mpsc::SyncSender;
use std::thread::sleep;
use std::time::{Duration, Instant};

use esp_idf_svc::hal::adc::attenuation::DB_12;
use esp_idf_svc::hal::adc::oneshot::config::{AdcChannelConfig, Calibration};
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::sys::{EspError, ESP_FAIL};
use scene_esp32::app::Event;
use scene_esp32::input::{Button, Debounce, Input, Stick};

use super::pins::Controls;

const PERIOD: Duration = Duration::from_millis(5);
const FULL_MV: u16 = 3100;
const BUTTONS: [Button; 5] = [
    Button::B1,
    Button::B2,
    Button::B3,
    Button::B4,
    Button::Stick,
];

/// Starts the input thread: it reads the buttons and the stick every 5 ms and sends each input to `tx`.
pub fn spawn(controls: Controls, tx: SyncSender<Event>) -> Result<(), EspError> {
    std::thread::Builder::new()
        .stack_size(4 * 1024)
        .spawn(move || {
            if let Err(e) = run(controls, &tx) {
                log::error!("controls: {e:?}");
            }
        })
        .map(drop)
        .map_err(|_| EspError::from_infallible::<ESP_FAIL>())
}

fn run(c: Controls, tx: &SyncSender<Event>) -> Result<(), EspError> {
    let [b1, b2, b3, b4] = c.buttons;
    let pins = [b1, b2, b3, b4, c.stick]
        .into_iter()
        .map(|pin| PinDriver::input(pin, Pull::Up))
        .collect::<Result<Vec<_>, _>>()?;
    let adc = AdcDriver::new(c.adc)?;
    let config = AdcChannelConfig {
        attenuation: DB_12,
        calibration: Calibration::Curve,
        ..Default::default()
    };
    let mut x = AdcChannelDriver::new(&adc, c.vrx, &config)?;
    let mut y = AdcChannelDriver::new(&adc, c.vry, &config)?;
    let mut read = || Ok::<_, EspError>([x.read()?, y.read()?]);
    let boot = (0..32)
        .map(|_| {
            sleep(PERIOD);
            read()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut stick = Stick::learn(&boot, FULL_MV);
    log::info!("stick centre {:?} mV", stick.centre());
    let mut buttons: [Debounce; 5] = Default::default();
    let started = Instant::now();
    let mut next = started;
    loop {
        let pressed = pins.iter().zip(&mut buttons).zip(BUTTONS);
        let mut inputs: Vec<Input> = pressed
            .filter_map(|((pin, button), b)| button.read(pin.is_low()).then_some(Input::Press(b)))
            .collect();
        inputs.extend(stick.read(read()?, started.elapsed()).map(Input::Tilt));
        for input in inputs {
            log::info!("input {input:?}");
            if tx.send(Event::Input(input)).is_err() {
                return Ok(());
            }
        }
        next += PERIOD;
        sleep(next.saturating_duration_since(Instant::now()));
    }
}
