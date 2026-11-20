pub mod led;
pub mod oled;
pub mod pins;
pub mod tft;
pub mod wifi;

use std::fmt::Debug;

use esp_idf_svc::sys::{EspError, ESP_FAIL};

pub use pins::Board;

fn fail<E: Debug>(what: &'static str) -> impl FnOnce(E) -> EspError {
    move |e| {
        log::error!("{what}: {e:?}");
        EspError::from_infallible::<ESP_FAIL>()
    }
}
