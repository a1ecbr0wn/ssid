//! Cross-platform crate to identify the WiFi SSID of the connected network.

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "ios",
    target_os = "windows"
)))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "ios")]
use ios as platform;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "ios",
    target_os = "windows"
)))]
use unsupported as platform;
#[cfg(target_os = "windows")]
use windows as platform;

/// Returns the SSID of the currently active WiFi network, auto-detecting the
/// wireless interface.
///
/// Returns `None` when no wireless interface is associated with a network, when
/// the platform is unsupported, or when the SSID cannot be read (e.g. missing
/// entitlement).
///
/// On machines with multiple WiFi adapters the returned SSID is
/// non-deterministic — typically the first interface enumerated by the OS.
/// Use [`get_ssid_for_interface`] to target a specific adapter.
///
/// # Platform notes
///
/// - **macOS 14+** — the process must be code-signed with the
///   `com.apple.wifi.manager` entitlement. Unsigned processes always receive
///   `None`.
/// - **iOS** — requires the `com.apple.developer.networking.wifi-info`
///   entitlement and that the user has granted location permission
///   (`NSLocationWhenInUseUsageDescription`). Without either, returns `None`.
/// - **Linux** — uses `nl80211` (kernel 2.6.22+). Older kernels always return
///   `None`.
/// - **Other platforms** — always returns `None`.
pub fn get_ssid() -> Option<String> {
    platform::get_ssid()
}

/// Returns the SSID of the WiFi network associated with the named interface,
/// or `None` if the interface is not found, not wireless, not associated, or
/// the SSID cannot be read.
///
/// A `log::warn` message is emitted when the interface name is not found, to
/// help distinguish "not connected" from a name mismatch.
///
/// # Platform notes
///
/// - **Windows** — `interface_name` must match `strInterfaceDescription` (e.g.
///   `"Intel(R) Wi-Fi 6 AX200 160MHz"`), not the friendly alias shown in
///   Windows Settings. Run `Get-NetAdapter | Select-Object Name,InterfaceDescription`
///   in PowerShell to find the exact string.
/// - **iOS** — `interface_name` is ignored; iOS exposes no per-interface
///   selection and the result is always the system-wide WiFi SSID, identical
///   to calling [`get_ssid`].
/// - **macOS 14+** — requires the `com.apple.wifi.manager` entitlement (same
///   as [`get_ssid`]).
/// - **Other platforms** — always returns `None`.
pub fn get_ssid_for_interface(interface_name: &str) -> Option<String> {
    platform::get_ssid_for_interface(interface_name)
}
