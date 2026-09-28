mod ble_provisioning;

fn main() -> Result<(), esp_idf_svc::sys::EspError> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("wireless-config starting on ESP32-C6");
    ble_provisioning::start()
}