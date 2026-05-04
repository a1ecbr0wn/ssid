# ssid

Cross-platform Rust crate to identify the WiFi SSID of the connected network.

## Usage

```rust
// Auto-detect the active interface
if let Some(ssid) = ssid::get_ssid() {
    println!("Connected to: {ssid}");
}

// Named interface
if let Some(ssid) = ssid::get_ssid_for_interface("en0") {
    println!("en0 SSID: {ssid}");
}
```

Both functions return `None` on any error, missing interface, or non-wireless interface — they never panic.

## CLI

```sh
cargo run            # print SSID of the active interface
cargo run -- en0     # print SSID for a named interface
```

## Platform notes

### Linux

Uses `nl80211` via generic netlink (requires kernel ≥ 2.6.22, which covers all practical deployments). Returns `None` if no interface is associated.

### macOS

Uses `CWWiFiClient` from the CoreWLAN framework. On macOS 14+ the binary must be **code-signed with the `com.apple.wifi.manager` entitlement** to read the SSID; unsigned binaries receive `None`. On macOS 26+ this is strictly enforced for all processes regardless of privilege level.

To sign your binary with the required entitlement:

1. Create `ssid.entitlements`:
   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
     "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0"><dict>
     <key>com.apple.wifi.manager</key><true/>
   </dict></plist>
   ```

2. Sign with a Developer ID Application certificate (requires [Apple Developer Program](https://developer.apple.com/programs/) membership):
   ```sh
   codesign --force \
     --sign "Developer ID Application: Your Name (TEAMID)" \
     --entitlements ssid.entitlements \
     /path/to/your/binary
   ```

Ad-hoc signing (`--sign -`) is not sufficient — macOS kills the process immediately when it tries to claim this entitlement without a valid certificate.

### Windows

Uses `WlanQueryInterface` from the Win32 WLAN API. `interface_name` is matched against the adapter's description string (e.g. `"Intel(R) Wi-Fi 6 AX200 160MHz"`); use `Get-NetAdapter | Select-Object Name,InterfaceDescription` in PowerShell to list available names.

## Testing

Hardware-dependent smoke tests are gated behind the `wifi_test` Cargo feature and are not run in CI:

```sh
cargo test --features wifi_test
```
