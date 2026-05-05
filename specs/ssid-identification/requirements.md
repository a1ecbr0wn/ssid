# Requirements Document: ssid

## Introduction

This spec covers a standalone, cross-platform Rust crate (`ssid`) that identifies the
SSID of the WiFi network the host computer or device is currently connected to.

The crate abstracts platform-specific WiFi APIs behind a simple synchronous function,
returning `None` when no wireless connection is active or the platform is unsupported.

---

## Glossary

- **SSID**: The human-readable name of a WiFi network (Service Set Identifier), up to
  32 bytes.
- **ssid**: The standalone crate defined by this spec.
- **Active interface**: The wireless network interface currently associated with an
  access point.

---

## Requirements

### Requirement 1: Crate API

**User Story:** As a developer, I need a crate that returns the current WiFi SSID
without requiring me to know the active interface name upfront.

#### Acceptance Criteria

1. THE crate SHALL expose two public functions:

   ```rust
   /// Returns the SSID of the active WiFi connection, auto-detecting the interface.
   pub fn get_ssid() -> Option<String>

   /// Returns the SSID for the named interface, or `None` if it is not wireless
   /// or not associated.
   pub fn get_ssid_for_interface(interface_name: &str) -> Option<String>
   ```

2. `get_ssid()` SHALL return the SSID of whichever wireless interface is currently
   associated, or `None` if no wireless interface is active.
3. If multiple wireless interfaces are active, `get_ssid()` MAY return the SSID of
   any one of them (implementation-defined; typically the first found).
4. THE functions SHALL NOT spawn external processes.
5. THE functions SHALL be synchronous (blocking is acceptable; SSID resolution is fast).
6. THE functions SHALL return `None` gracefully — no panics, no propagated errors.

---

### Requirement 2: Platform Implementations

**User Story:** As a developer running on Linux, macOS, or Windows, I need SSID
resolution to work on my platform without additional configuration.

#### Platform Acceptance Criteria

1. **Linux** — THE Linux implementation SHALL use the `nl80211` netlink API
   (`NL80211_CMD_GET_INTERFACE`) via a generic netlink socket, using the `neli` crate.
   - `get_ssid()` SHALL enumerate all `nl80211` interfaces and return the SSID of the
     first one that is associated.
   - Kernels older than 2.6.22 (no `nl80211` support) are explicitly out of scope and
     will receive `None`.
2. **macOS** — THE macOS implementation SHALL use the CoreWLAN framework via
   `objc2-core-wlan`:
   - `get_ssid()`: `CWWiFiClient::sharedWiFiClient()` → `.interface()` → `.ssid()`
   - `get_ssid_for_interface()`: `CWWiFiClient::sharedWiFiClient()` →
     `.interfaceWithName(name)` → `.ssid()`
   - On macOS 14+ the binary must be code-signed with the `com.apple.wifi.manager`
     entitlement; unsigned binaries receive `None`.
3. **iOS** — THE iOS implementation SHALL use `NEHotspotNetwork.fetchCurrent()` from
   the `NetworkExtension` framework:
   - `get_ssid()` SHALL call `NEHotspotNetwork.fetchCurrent()` and return the SSID of
     the current network, or `None` if not associated.
   - `get_ssid_for_interface()` SHALL ignore `interface_name` and delegate to
     `get_ssid()`, as iOS does not expose per-interface selection.
   - The binary must be code-signed with the `com.apple.developer.networking.wifi-info`
     entitlement and the user must have granted location permission.
4. **Windows** — THE Windows implementation SHALL use `WlanOpenHandle`,
   `WlanQueryInterface` (opcode `wlan_intf_opcode_current_connection`), and
   `WlanCloseHandle` from the `windows` crate
   (`Win32_NetworkManagement_Wlan` feature).
   - `get_ssid()` SHALL call `WlanEnumInterfaces` and return the SSID of the first
     connected interface found.
5. **Other platforms** — THE fallback implementation SHALL return `None`.
6. ALL implementations SHALL return `None` when the interface is not found, not
   wireless, or not associated.

---

### Requirement 3: CLI Tool

**User Story:** As a developer, I need a small command-line program that calls the
crate API and prints the result, so I can quickly verify the implementation on a
target platform.

#### CLI Acceptance Criteria

1. THE crate SHALL include a binary target at `src/main.rs`.
2. WITH no arguments, it SHALL call `get_ssid()` and print the SSID or `none` if
   not connected.
3. WITH a single argument (interface name), it SHALL call
   `get_ssid_for_interface(name)` and print the SSID or `none`.
4. Exit code SHALL be `0` in both cases (absence of WiFi is not an error).

---

### Requirement 4: Testing

**User Story:** As a contributor, I need the test suite to run cleanly in CI
environments where no WiFi hardware is present.

#### Testing Acceptance Criteria

1. THE crate SHALL include a smoke test for each platform that calls `get_ssid()` and
   asserts only that it does not panic (no assertion on the returned value).
2. THE smoke test SHALL be gated so it only runs when the `wifi_test` feature or
   environment variable `SSID_INTEGRATION_TEST=1` is set, allowing CI to skip it by
   default.
3. THE crate SHALL include pure-unit tests for any non-platform helper logic
   (e.g., byte-buffer parsing, interface name conversion) that can run without hardware.

---
