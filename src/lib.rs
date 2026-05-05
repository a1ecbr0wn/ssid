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

/// Returns the SSID of the active WiFi connection, auto-detecting the interface.
///
/// Returns `None` if no wireless interface is active, the platform is unsupported,
/// or the SSID cannot be determined.
pub fn get_ssid() -> Option<String> {
    platform::get_ssid()
}

/// Returns the SSID of the WiFi network associated with the named interface.
///
/// Returns `None` if the interface is not found, not wireless, not associated,
/// or the SSID cannot be determined.
pub fn get_ssid_for_interface(interface_name: &str) -> Option<String> {
    platform::get_ssid_for_interface(interface_name)
}
