# wireless-config

Rust firmware for ESP32-C6 using ESP-IDF, featuring official Espressif Bluetooth LE (BLE) Wi-Fi provisioning and an OTA-ready partition table for a 4 MiB flash board.

## Features

- **Target Architecture**: ESP-IDF Rust target `riscv32imac-esp-espidf` for ESP32-C6.
- **Official BLE Wi-Fi Provisioning**: Integrates with Espressif's native `wifi_prov_mgr` and NimBLE stack. Compatible with the official "ESP BLE Provisioning" mobile app on iOS and Android.
- **Persistent Storage**: 36 KiB NVS partition for Wi-Fi credentials and configuration.
- **OTA Ready**: Two 1.94 MiB OTA application slots (`ota_0`, `ota_1`) and the required `otadata` partition.
- **Flashing & Monitoring**: Pre-configured Cargo runner using `espflash`.

---

## Toolchain Setup

### 1. Prerequisites (macOS)

Install base tools via Homebrew:
```sh
brew install cmake ninja espflash cargo-binstall
```

Install Espressif toolchain installer and linker proxy:
```sh
cargo binstall -y espup ldproxy
espup install
```

### 2. Rust Toolchain Configuration

This project requires the `esp` toolchain (installed by `espup`). The project repository includes `rust-toolchain.toml` to automatically select `esp`. You can also ensure the directory override is active with:
```sh
rustup override set esp
```

---

## Build and Flash over USB

Connect your ESP32-C6 board via USB-C, then run:

```sh
# Build firmware
cargo build

# Flash the device and start the serial monitor
cargo run
```

The configured Cargo runner invokes `espflash flash --monitor --partition-table partitions.csv`. If the board does not enter download mode automatically, hold its **BOOT** button while pressing **RESET**.

---

## Official BLE Wi-Fi Provisioning

When the device boots and detects that no Wi-Fi credentials are saved in NVS, it starts advertising over Bluetooth LE.

### Provisioning Steps

1. Install the official **"ESP BLE Provisioning"** app from Espressif on your phone:
   - [iOS App Store](https://apps.apple.com/app/esp-ble-provisioning/id1473530145)
   - [Android Google Play](https://play.google.com/store/apps/details?id=com.espressif.provble)
2. Open the app and tap **Provision New Device**.
3. Scan for devices and select your board:
   - **Device BLE Name**: `PROV_C6_XXXX` (e.g. `PROV_C6_E93C`, where `XXXX` is derived from the Wi-Fi MAC address).
4. Enter the **Proof of Possession (PoP / PIN)**:
   ```text
   abcd1234
   ```
5. Select your 2.4 GHz Wi-Fi network from the scan list and enter the password.
6. The ESP32-C6 verifies the connection, acquires a DHCP IP address, and stores the credentials securely in the `nvs` partition.

> [!WARNING]
> **BLE Shuts Down After Provisioning**:  
> As soon as the Wi-Fi credentials are confirmed and saved, the ESP-IDF provisioning manager **automatically stops and deinitializes the BLE stack** to release memory and save power.  
> On subsequent reboots, the firmware detects that credentials already exist in NVS and connects directly to Wi-Fi without starting BLE advertising.

---

## Starting Over: Resetting Wi-Fi & Re-flashing

Because Wi-Fi credentials are preserved in the `nvs` flash partition, simply running `cargo run` will **not** trigger the BLE provisioning flow again.

To completely reset the device and re-provision via BLE:

```sh
# 1. Completely erase flash (wipes NVS credentials, OTA states, and firmware)
espflash erase-flash

# 2. Re-flash the firmware and restart the serial monitor
cargo run
```

Upon rebooting after the erase, the device will detect no credentials and start BLE advertising (`PROV_C6_XXXX`) immediately.

---

## Partition Layout

[partitions.csv](file:///Users/manabutokunaga/development/esp32/wireless-config/partitions.csv) uses a fixed 4 MiB flash map:

| Partition | Type | SubType | Offset | Size | Purpose |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `nvs` | `data` | `nvs` | `0x9000` | 36 KiB | Wi-Fi credentials and persistent settings |
| `otadata` | `data` | `ota` | `0x12000` | 8 KiB | Active OTA boot slot selection |
| `phy_init`| `data` | `phy` | `0x14000` | 4 KiB | RF calibration data |
| `ota_0` | `app` | `ota_0` | `0x20000` | 1.94 MiB | First OTA application slot |
| `ota_1` | `app` | `ota_1` | `0x210000` | 1.94 MiB | Second OTA application slot |

If your board has a different flash size, adjust both OTA slot sizes and offsets before flashing.