use core::convert::TryInto;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_password: &'static str,
}
pub fn connect(modem: esp_idf_svc::hal::modem::Modem<'static>) -> BlockingWifi<EspWifi<'static>> {
    let sysloop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), Some(nvs)).unwrap(),
        sysloop,
    )
    .unwrap();

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: CONFIG.wifi_ssid.try_into().unwrap(),
        password: CONFIG.wifi_password.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))
    .unwrap();

    wifi.start().unwrap();
    println!("WLAN gestartet");
    wifi.connect().unwrap();
    println!("WLAN verbunden");
    wifi.wait_netif_up().unwrap();
    println!("WLAN bereit!");

    wifi
}

pub fn sync_time() -> EspSntp<'static> {
    let sntp = EspSntp::new_default().unwrap();
    println!("Warte auf Zeitsynchronisation...");
    while sntp.get_sync_status() != SyncStatus::Completed {
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(100);
    }
    println!("Zeit synchronisiert!");
    sntp
}
