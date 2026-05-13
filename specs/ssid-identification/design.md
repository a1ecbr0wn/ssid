# Design Document: ssid

## Overview

The `ssid` crate exposes two synchronous functions for WiFi SSID resolution.
All platform-specific code is isolated in per-platform modules selected at compile time
via `#[cfg(target_os)]`. Callers see only the public API in `src/lib.rs`.

---

## Architecture

```text
ssid/
  src/
    lib.rs          (public API + cfg dispatch)
    linux.rs        (nl80211 via neli)
    macos.rs        (CoreWLAN via objc2-core-wlan)
    ios.rs          (NEHotspotNetwork via NetworkExtension)
    windows.rs      (WlanQueryInterface via windows crate)
    unsupported.rs  (returns None)
```

No binary, no async, no build script required on Linux/Windows.

---

## Public API (`src/lib.rs`)

```rust
pub fn get_ssid() -> Option<String> {
    platform::get_ssid()
}

pub fn get_ssid_for_interface(interface_name: &str) -> Option<String> {
    platform::get_ssid_for_interface(interface_name)
}
```

Platform dispatch:

```rust
#[cfg(target_os = "linux")]
mod platform { pub use super::linux::*; }

#[cfg(target_os = "macos")]
mod platform { pub use super::macos::*; }

#[cfg(target_os = "ios")]
mod platform { pub use super::ios::*; }

#[cfg(target_os = "windows")]
mod platform { pub use super::windows::*; }

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "ios", target_os = "windows")))]
mod platform { pub use super::unsupported::*; }
```

---

## Linux Implementation (`src/linux.rs`)

Uses `nl80211` over a generic netlink socket via the `neli` crate.

### Linux `get_ssid_for_interface`

```text
NlSocketHandle::connect(NlFamily::Generic, None, &[])
  → resolve nl80211 family id via CTRL_CMD_GETFAMILY
  → get_interface_list → find entry matching interface_name
      → if not found: log::warn!("nl80211: interface '...' not found") → return None
  → ssid_for_ifindex using the matched ifindex
      → parse NL80211_ATTR_SSID from the response attributes
      → String::from_utf8_lossy(ssid_bytes).into_owned()
      → on ENODEV / no SSID attr → return None
```

### Linux `get_ssid`

```text
send NL80211_CMD_GET_INTERFACE with NLM_F_DUMP flag
  → iterate all nl80211 interfaces in the response
  → for each, extract NL80211_ATTR_SSID if present
  → return the first SSID found, or None
```

Dependencies: `neli` (generic netlink socket and attribute parsing).

**Known limitation**: Kernels older than 2.6.22 lack `nl80211` and will always return
`None`. Wireless Extensions (`wext`) support for those kernels is out of scope.

---

## macOS Implementation (`src/macos.rs`)

Uses CoreWLAN via the `objc2-core-wlan` crate.

### macOS `get_ssid_for_interface`

```text
CWWiFiClient::sharedWiFiClient()         // shared singleton
  → interfaceWithName(name)              // → Option<CWInterface>
  → .ssid()                              // → Option<NSString>
  → NSString → String
```

### macOS `get_ssid`

```text
CWWiFiClient::sharedWiFiClient()
  → .interface()                         // default interface
  → .ssid()
```

`objc2-core-wlan` provides safe Rust wrappers around `CWWiFiClient` and `CWInterface`.
`NSString` is converted to `String` via `objc2-foundation`.

**Known limitation (macOS)**: `CWInterface.ssid()` requires the
`com.apple.wifi.manager` entitlement on macOS 14+ for unsigned processes. On macOS 26+
this is strictly enforced — unsigned binaries always receive `None`. To obtain the SSID
the binary must be code-signed with the `com.apple.wifi.manager` entitlement using a
valid Apple Developer ID certificate.

---

## iOS Implementation (`src/ios.rs`)

Uses `NEHotspotNetwork` from the `NetworkExtension` framework via `objc2`.

### iOS `get_ssid`

```text
NEHotspotNetwork::fetchCurrentWithCompletionHandler(block)
  → completion block receives Option<NEHotspotNetwork>
  → .ssid                                // → NSString?
  → NSString → String
```

`fetchCurrentWithCompletionHandler` is async at the Objective-C level; the
implementation blocks a thread using a `DispatchSemaphore` (or equivalent) to
satisfy the synchronous Rust API contract.

### iOS `get_ssid_for_interface`

iOS does not expose per-interface selection. This function ignores `interface_name`
and delegates to `get_ssid()`.

**Known limitation (iOS)**: The binary must be code-signed with the
`com.apple.developer.networking.wifi-info` entitlement and the user must have granted
location permission (`NSLocationWhenInUseUsageDescription`) at runtime. Without either,
`NEHotspotNetwork.fetchCurrent()` returns `nil` and `get_ssid()` returns `None`.

---

## Windows Implementation (`src/windows.rs`)

Uses the `windows` crate with feature `Win32_NetworkManagement_Wlan`.

### Windows `get_ssid_for_interface`

```text
WlanOpenHandle(dwClientVersion=2) → client_handle
WlanEnumInterfaces(client_handle) → interface list
  → find entry whose strInterfaceDescription matches interface_name
      → if not found: log::warn!("wlan: interface '...' not found")
                      WlanCloseHandle(client_handle) → return None
  → get GUID
WlanQueryInterface(
    client_handle,
    &guid,
    wlan_intf_opcode_current_connection,
    ...,
    &mut data_ptr,
    ...,
) → *mut WLAN_CONNECTION_ATTRIBUTES
  → wlanAssociationAttributes.dot11Ssid.ucSSID[..uSSIDLength]
  → String::from_utf8_lossy(...)
WlanFreeMemory(data_ptr)
WlanCloseHandle(client_handle)
```

`WlanFreeMemory` and `WlanCloseHandle` are called on all exit paths (including
error paths and the no-GUID-match early return) to prevent handle leaks.

### Windows `get_ssid`

Calls `WlanEnumInterfaces`, iterates all interfaces, calls
`get_ssid_for_interface` on each, returns the first `Some`.

New dependency: `windows = { version = "0.58", features = ["Win32_NetworkManagement_Wlan"] }`.

---

## Unsupported Platforms (`src/unsupported.rs`)

```rust
pub fn get_ssid() -> Option<String> { None }
pub fn get_ssid_for_interface(_: &str) -> Option<String> { None }
```

---

## Dependencies

| Crate                     | Platform     | Notes                                                 |
| ------------------------- | ------------ | ----------------------------------------------------- |
| `log`                     | all          | diagnostic `warn!` messages for silent `None` returns |
| `neli`                    | Linux        | generic netlink socket and nl80211 attribute parsing  |
| `objc2-core-wlan`         | macOS        | safe Rust bindings for `CWWiFiClient` / `CWInterface` |
| `objc2-foundation`        | macOS        | `NSString` conversion                                 |
| `objc2`                   | macOS        | Objective-C runtime support                           |
| `objc2-network-extension` | iOS          | `NEHotspotNetwork` bindings                           |
| `objc2-foundation`        | iOS          | `NSString` conversion                                 |
| `objc2`                   | iOS          | Objective-C runtime support                           |
| `windows`                 | Windows      | WiFi API — feature `Win32_NetworkManagement_Wlan`     |

Platform-specific deps are gated with `[target.'cfg(target_os = "...")'.dependencies]`
in `Cargo.toml` so they don't compile on other platforms.

---

## CLI Tool (`src/main.rs`)

```rust
fn main() {
    let ssid = match std::env::args().nth(1) {
        Some(iface) => ssid::get_ssid_for_interface(&iface),
        None        => ssid::get_ssid(),
    };
    println!("{}", ssid.as_deref().unwrap_or("none"));
}
```

Run with `cargo run` (auto-detect) or `cargo run -- en0` (named interface).

---

## Testing Strategy

### CI (no hardware)

- Smoke tests are gated behind `SSID_INTEGRATION_TEST=1` or a `wifi_test` Cargo
  feature, so CI runs pass without WiFi.
- Unit tests for pure helper logic (byte parsing, interface name extraction from
  `/proc/net/wireless`) run unconditionally.

### Local / hardware

Set `SSID_INTEGRATION_TEST=1` (or `--features wifi_test`) to enable the smoke test
that calls `get_ssid()` and asserts only that it does not panic.
