use std::ffi::CString;
use std::thread;
use std::time::Duration;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::{
    esp, esp_wifi_connect, esp_wifi_get_mac, esp_wifi_start, wifi_interface_t_WIFI_IF_STA,
    wifi_prov_conn_cfg_t, wifi_prov_event_handler_t, wifi_prov_mgr_config_t,
    wifi_prov_mgr_deinit, wifi_prov_mgr_init, wifi_prov_mgr_is_provisioned,
    wifi_prov_mgr_start_provisioning, wifi_prov_mgr_wait, wifi_prov_scheme_ble,
    wifi_prov_scheme_ble_event_cb_free_btdm, wifi_prov_security_WIFI_PROV_SECURITY_1,
    EspError,
};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};

pub fn start() -> Result<(), EspError> {
    let peripherals = Peripherals::take().expect("Failed to initialize hardware peripherals");
    let sys_loop = EspSystemEventLoop::take().expect("Failed to initialize system event loop");
    let nvs = EspDefaultNvsPartition::take().expect("Failed to initialize default NVS partition");

    // Initialize WiFi driver
    let wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))
            .expect("Failed to initialize Wi-Fi driver"),
        sys_loop.clone(),
    )
    .expect("Failed to create BlockingWifi wrapper");

    // Check if the device is already provisioned
    let mut provisioned = false;
    unsafe {
        let scheme_event_handler = wifi_prov_event_handler_t {
            event_cb: Some(wifi_prov_scheme_ble_event_cb_free_btdm),
            user_data: core::ptr::null_mut(),
        };

        let config = wifi_prov_mgr_config_t {
            scheme: wifi_prov_scheme_ble,
            scheme_event_handler,
            app_event_handler: wifi_prov_event_handler_t::default(),
            wifi_prov_conn_cfg: wifi_prov_conn_cfg_t {
                wifi_conn_attempts: 5,
            },
        };

        esp!(wifi_prov_mgr_init(config))?;
        esp!(wifi_prov_mgr_is_provisioned(&mut provisioned))?;
    }

    if !provisioned {
        log::info!("Device is NOT provisioned yet. Starting BLE Provisioning service...");

        let mut mac = [0u8; 6];
        unsafe {
            esp_wifi_get_mac(wifi_interface_t_WIFI_IF_STA, mac.as_mut_ptr());
        }

        let service_name_str = format!("PROV_C6_{:02X}{:02X}", mac[4], mac[5]);
        let service_name = CString::new(service_name_str.as_str()).unwrap();
        let pop_str = "abcd1234";
        let pop = CString::new(pop_str).unwrap();

        log::info!("============================================================");
        log::info!("* OFFICIAL ESP BLE PROVISIONING STARTED");
        log::info!("* Device BLE Name: {}", service_name_str);
        log::info!("* Proof of Possession (PoP / PIN): {}", pop_str);
        log::info!("* Open 'ESP BLE Provisioning' app on iOS / Android");
        log::info!("* Scan for '{}' and enter PIN '{}'", service_name_str, pop_str);
        log::info!("============================================================");

        unsafe {
            esp!(wifi_prov_mgr_start_provisioning(
                wifi_prov_security_WIFI_PROV_SECURITY_1,
                pop.as_ptr() as *const core::ffi::c_void,
                service_name.as_ptr(),
                core::ptr::null(),
            ))?;

            // Wait until the mobile app finishes sending credentials and provisioning is done
            wifi_prov_mgr_wait();
            wifi_prov_mgr_deinit();
        }

        log::info!("Provisioning succeeded and BLE stopped! Connecting to Wi-Fi...");
    } else {
        log::info!("Device is already provisioned! Connecting to Wi-Fi...");
        unsafe {
            wifi_prov_mgr_deinit();
        }
    }

    // Start Wi-Fi station and connect with the provisioned credentials saved in NVS
    unsafe {
        esp!(esp_wifi_start())?;
        esp!(esp_wifi_connect())?;
    }

    log::info!("Waiting for IP address...");
    for _ in 0..30 {
        if wifi.is_connected().unwrap_or(false) {
            if let Ok(ip_info) = wifi.wifi().sta_netif().get_ip_info() {
                if !ip_info.ip.is_unspecified() {
                    log::info!("============================================================");
                    log::info!("* Wi-Fi CONNECTED!");
                    log::info!("* IP Address: {}", ip_info.ip);
                    log::info!("* Subnet:     {}", ip_info.subnet.mask);
                    log::info!("* Gateway:    {}", ip_info.subnet.gateway);
                    log::info!("============================================================");
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(500));
    }

    loop {
        thread::sleep(Duration::from_secs(10));
        let ip = wifi
            .wifi()
            .sta_netif()
            .get_ip_info()
            .map(|info| info.ip.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        log::info!("ESP32-C6 running normal firmware loop. Connected IP: {}", ip);
    }
}
