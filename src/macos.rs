use objc2_core_wlan::{CWInterface, CWWiFiClient};
use objc2_foundation::NSString;

#[cfg(target_os = "macos")]
fn interface_ssid(iface: &CWInterface) -> Option<String> {
    let ssid = unsafe { iface.ssid() }?;
    let s = ssid.to_string();
    if s.is_empty() { None } else { Some(s) }
}

/// Returns the SSID of the default WiFi interface via `CWWiFiClient`.
///
/// Returns `None` if no default interface is available, the interface is not
/// associated, or the SSID is empty (hidden network). On macOS 14+ the process
/// must hold the `com.apple.wifi.manager` entitlement; without it the OS
/// returns an empty SSID and this function returns `None`.
#[cfg(target_os = "macos")]
pub fn get_ssid() -> Option<String> {
    let client = unsafe { CWWiFiClient::sharedWiFiClient() };
    let iface = unsafe { client.interface() }.or_else(|| {
        log::warn!("macos: no default WiFi interface available");
        None
    })?;
    interface_ssid(&iface)
}

/// Returns the SSID of the named WiFi interface via `CWWiFiClient::interfaceWithName`.
///
/// Returns `None` if the interface name is not recognised, the interface is not
/// associated, or the SSID is empty. On macOS 14+ the `com.apple.wifi.manager`
/// entitlement is required (same as [`get_ssid`]).
#[cfg(target_os = "macos")]
pub fn get_ssid_for_interface(interface_name: &str) -> Option<String> {
    let client = unsafe { CWWiFiClient::sharedWiFiClient() };
    let name = NSString::from_str(interface_name);
    let iface = unsafe { client.interfaceWithName(Some(&name)) }.or_else(|| {
        log::warn!("macos: interface '{interface_name}' not found");
        None
    })?;
    interface_ssid(&iface)
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "wifi_test")]
    fn smoke_get_ssid_does_not_panic() {
        let _ = super::get_ssid();
    }

    #[test]
    #[cfg(feature = "wifi_test")]
    fn smoke_absent_interface_returns_none() {
        assert_eq!(super::get_ssid_for_interface("__nonexistent__"), None);
    }
}
