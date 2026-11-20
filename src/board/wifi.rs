use std::sync::mpsc::SyncSender;
use std::time::Duration;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::http::client::{Configuration as Http, EspHttpConnection};
use esp_idf_svc::http::Method;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::{EspError, ESP_ERR_HTTP_CONNECT, ESP_FAIL};
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use scene_esp32::app::Event;
use scene_esp32::net::{read_body, Fail, Feed};

const SSID: Option<&str> = option_env!("SCENE_WIFI_SSID");
const PASS: &str = match option_env!("SCENE_WIFI_PASS") {
    Some(pass) => pass,
    None => "",
};
const ADDR: Option<&str> = option_env!("SCENE_ADDR");

/// Starts the net thread: it joins Wi-Fi, polls drover's scene and sends each poll to `tx`.
pub fn spawn(modem: Modem<'static>, tx: SyncSender<Event>) -> Result<(), EspError> {
    let wifi = match (SSID, ADDR) {
        (Some(ssid), Some(addr)) => {
            Some((join(modem, ssid)?, format!("http://{addr}/screens/scene")))
        }
        _ => None,
    };
    std::thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(move || run(wifi, tx))
        .map(drop)
        .map_err(|_| EspError::from_infallible::<ESP_FAIL>())
}

fn join(modem: Modem<'static>, ssid: &str) -> Result<BlockingWifi<EspWifi<'static>>, EspError> {
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let mut wifi = BlockingWifi::wrap(EspWifi::new(modem, sysloop.clone(), Some(nvs))?, sysloop)?;
    let bad = |_| EspError::from_infallible::<ESP_FAIL>();
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().map_err(bad)?,
        password: PASS.try_into().map_err(bad)?,
        auth_method: match PASS {
            "" => AuthMethod::None,
            _ => AuthMethod::WPA2Personal,
        },
        ..Default::default()
    }))?;
    wifi.start()?;
    Ok(wifi)
}

fn run(mut wifi: Option<(BlockingWifi<EspWifi<'static>>, String)>, tx: SyncSender<Event>) {
    let mut feed = Feed::new();
    loop {
        let got = match &mut wifi {
            Some((wifi, url)) => get(wifi, url),
            None => Err(Fail::NotConfigured),
        };
        if let Err(fail) = &got {
            log::warn!("poll: {fail}");
        }
        let (poll, wait) = feed.take(got);
        if tx.send(Event::Poll(poll)).is_err() {
            return;
        }
        std::thread::sleep(wait);
    }
}

fn get(wifi: &mut BlockingWifi<EspWifi<'static>>, url: &str) -> Result<Vec<u8>, Fail> {
    if !wifi.is_connected().unwrap_or(false) {
        wifi.connect().map_err(|_| Fail::NoWifi)?;
        wifi.wait_netif_up().map_err(|_| Fail::NoWifi)?;
    }
    let http = Http {
        timeout: Some(Duration::from_secs(4)),
        ..Default::default()
    };
    let mut conn = EspHttpConnection::new(&http).map_err(net)?;
    conn.initiate_request(Method::Get, url, &[]).map_err(net)?;
    conn.initiate_response().map_err(net)?;
    match conn.status() {
        200 => read_body(|buf| conn.read(buf).map_err(net)),
        code => Err(Fail::Http(code)),
    }
}

fn net(e: EspError) -> Fail {
    match e.code() {
        ESP_ERR_HTTP_CONNECT => Fail::Refused,
        _ => Fail::Timeout,
    }
}
