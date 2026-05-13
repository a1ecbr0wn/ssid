# Implementation Tasks: ssid

## Overview

Build the `ssid` standalone crate with cross-platform WiFi SSID resolution for
Linux, macOS, iOS, and Windows.

---

## Tasks

- [x] 1. Crate scaffold
  - [x] 1.1 Create `Cargo.toml` with package metadata and platform-gated dependencies
  - [x] 1.2 Create `src/lib.rs` with public API and `#[cfg]` platform dispatch
  - [x] 1.3 Gate Windows dep: `[target.'cfg(target_os="windows")'.dependencies]`
  - [x] 1.4 Gate macOS dep: `[target.'cfg(target_os="macos")'.dependencies]`
  - [x] 1.5 Gate Linux dep: `[target.'cfg(target_os="linux")'.dependencies]`

- [x] 2. Unsupported platform fallback (`src/unsupported.rs`)
  - [x] 2.1 Implement `get_ssid() -> Option<String>` returning `None`
  - [x] 2.2 Implement `get_ssid_for_interface(_: &str) -> Option<String>` returning `None`

- [x] 3. Linux implementation (`src/linux.rs`)
  - [x] 3.1 Resolve `nl80211` generic netlink family id via `CTRL_CMD_GETFAMILY`
  - [x] 3.2 Implement `get_ssid_for_interface` — send `NL80211_CMD_GET_INTERFACE` for the named interface, parse `NL80211_ATTR_SSID`
  - [x] 3.3 Return `None` on `ENODEV`, missing SSID attribute, or any netlink error
  - [x] 3.4 Implement `get_ssid` — send `NL80211_CMD_GET_INTERFACE` with `NLM_F_DUMP`, return first SSID found
  - [x] 3.5 Unit test for SSID byte-buffer → String conversion

- [x] 4. macOS implementation (`src/macos.rs`)
  - [x] 4.1 Add `objc2-core-wlan`, `objc2-foundation`, `objc2` as macOS-gated deps
  - [x] 4.2 Implement `get_ssid` via `CWWiFiClient::sharedWiFiClient().interface().ssid()`
  - [x] 4.3 Implement `get_ssid_for_interface` via `CWWiFiClient::sharedWiFiClient().interfaceWithName(name).ssid()`
  - [x] 4.4 Return `None` for empty SSID string

- [x] 5. iOS implementation (`src/ios.rs`)
  - [x] 5.1 Add `objc2-network-extension`, `objc2-foundation`, `objc2` as iOS-gated deps in `Cargo.toml`
  - [x] 5.2 Add `#[cfg(target_os = "ios")]` dispatch in `src/lib.rs`
  - [x] 5.3 Implement `get_ssid` using `NEHotspotNetwork::fetchCurrentWithCompletionHandler`, blocking with a semaphore
  - [x] 5.4 Implement `get_ssid_for_interface` delegating to `get_ssid()` (iOS has no per-interface selection)
  - [x] 5.5 Return `None` for empty or missing SSID

- [x] 6. Windows implementation (`src/windows.rs`)
  - [x] 6.1 Implement `get_ssid_for_interface`: open handle → enum interfaces → find GUID → query connection → extract SSID
  - [x] 6.2 Ensure `WlanFreeMemory` and `WlanCloseHandle` are called on all paths
  - [x] 6.3 Implement `get_ssid` by iterating `WlanEnumInterfaces` and returning first SSID found

- [x] 7. CLI tool (`src/main.rs`)
  - [x] 7.1 Print result of `get_ssid()` when called with no arguments
  - [x] 7.2 Print result of `get_ssid_for_interface(name)` when an interface name is passed
  - [x] 7.3 Print `none` when the result is `None`

- [x] 8. Tests
  - [x] 8.1 Add `wifi_test` feature to `Cargo.toml`
  - [x] 8.2 Add smoke test for `get_ssid()` gated on `#[cfg(feature = "wifi_test")]` — asserts no panic only
  - [x] 8.3 Add smoke test for `get_ssid_for_interface` with a known-absent name — asserts `None`
  - [x] 8.4 Run unit tests unconditionally: `cargo test` must pass in CI without hardware

- [x] 9. Validation
  - [x] 9.1 `cargo build` compiles on current platform
  - [x] 9.2 `cargo clippy --all-targets -- -D warnings` is clean
  - [x] 9.3 `cargo test` passes without WiFi hardware (no `wifi_test` feature)
  - [ ] 9.4 Manual smoke test on target platform: `cargo run` returns expected SSID
