use objc2_core_wlan::{CWInterface, CWWiFiClient};
use objc2_foundation::NSString;

fn interface_ssid(iface: &CWInterface) -> Option<String> {
    let ssid = unsafe { iface.ssid() }?;
    let s = ssid.to_string();
    if s.is_empty() { None } else { Some(s) }
}

#[cfg(target_os = "macos")]
pub fn get_ssid() -> Option<String> {
    let client = unsafe { CWWiFiClient::sharedWiFiClient() };
    let iface = unsafe { client.interface() }.or_else(|| {
        log::warn!("macos: no default WiFi interface available");
        None
    })?;
    interface_ssid(&iface)
}

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
