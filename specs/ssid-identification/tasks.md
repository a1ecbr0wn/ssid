# Implementation Tasks: ssid

## Overview

Build the `ssid` standalone crate with cross-platform WiFi SSID resolution for
Linux, macOS, and Windows.

---

## Tasks

- [ ] 1. Crate scaffold
  - [ ] 1.1 Create `Cargo.toml` with package metadata and platform-gated dependencies
  - [ ] 1.2 Create `src/lib.rs` with public API and `#[cfg]` platform dispatch
  - [ ] 1.3 Gate Windows dep: `[target.'cfg(target_os="windows")'.dependencies]`
  - [ ] 1.4 Gate macOS dep: `[target.'cfg(target_os="macos")'.dependencies]`
  - [ ] 1.5 Gate Linux dep: `[target.'cfg(target_os="linux")'.dependencies]`

- [ ] 2. Unsupported platform fallback (`src/unsupported.rs`)
  - [ ] 2.1 Implement `get_ssid() -> Option<String>` returning `None`
  - [ ] 2.2 Implement `get_ssid_for_interface(_: &str) -> Option<String>` returning `None`

- [ ] 3. Linux implementation (`src/linux.rs`)
  - [ ] 3.1 Resolve `nl80211` generic netlink family id via `CTRL_CMD_GETFAMILY`
  - [ ] 3.2 Implement `get_ssid_for_interface` — send `NL80211_CMD_GET_INTERFACE` for the named interface, parse `NL80211_ATTR_SSID`
  - [ ] 3.3 Return `None` on `ENODEV`, missing SSID attribute, or any netlink error
  - [ ] 3.4 Implement `get_ssid` — send `NL80211_CMD_GET_INTERFACE` with `NLM_F_DUMP`, return first SSID found
  - [ ] 3.5 Unit test for SSID byte-buffer → String conversion

- [ ] 4. macOS implementation (`src/macos.rs`)
  - [ ] 4.1 Add `#[link(name = "CoreWLAN", kind = "framework")]`
  - [ ] 4.2 Implement `get_ssid_for_interface` via `objc2::msg_send!` to `CWWiFiClient`
  - [ ] 4.3 Implement `get_ssid` using the default `[CWWiFiClient interface]`
  - [ ] 4.4 Convert `NSString` → Rust `String` via `UTF8String`

- [ ] 5. Windows implementation (`src/windows.rs`)
  - [ ] 5.1 Implement `get_ssid_for_interface`: open handle → enum interfaces → find GUID → query connection → extract SSID
  - [ ] 5.2 Ensure `WlanFreeMemory` and `WlanCloseHandle` are called on all paths
  - [ ] 5.3 Implement `get_ssid` by iterating `WlanEnumInterfaces` and returning first SSID found

- [ ] 6. CLI tool (`src/main.rs`)
  - [ ] 6.1 Print result of `get_ssid()` when called with no arguments
  - [ ] 6.2 Print result of `get_ssid_for_interface(name)` when an interface name is passed
  - [ ] 6.3 Print `none` when the result is `None`

- [ ] 7. Tests
  - [ ] 7.1 Add `wifi_test` feature to `Cargo.toml`
  - [ ] 7.2 Add smoke test for `get_ssid()` gated on `#[cfg(feature = "wifi_test")]` — asserts no panic only
  - [ ] 7.3 Add smoke test for `get_ssid_for_interface` with a known-absent name — asserts `None`
  - [ ] 7.4 Run unit tests unconditionally: `cargo test` must pass in CI without hardware

- [ ] 8. Validation
  - [ ] 8.1 `cargo build` compiles on current platform
  - [ ] 8.2 `cargo clippy --all-targets -- -D warnings` is clean
  - [ ] 8.3 `cargo test` passes without WiFi hardware (no `wifi_test` feature)
  - [ ] 8.4 Manual smoke test on target platform: `cargo run` returns expected SSID
