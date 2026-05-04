use objc2_core_wlan::{CWInterface, CWWiFiClient};
use objc2_foundation::NSString;

fn nsstring_to_string(s: &NSString) -> String {
    s.to_string()
}

fn interface_ssid(iface: &CWInterface) -> Option<String> {
    let ssid = unsafe { iface.ssid() }?;
    let s = nsstring_to_string(&ssid);
    if s.is_empty() { None } else { Some(s) }
}

#[cfg(target_os = "macos")]
pub fn get_ssid() -> Option<String> {
    let client = unsafe { CWWiFiClient::sharedWiFiClient() };
    let iface = unsafe { client.interface() }?;
    interface_ssid(&iface)
}

#[cfg(target_os = "macos")]
pub fn get_ssid_for_interface(interface_name: &str) -> Option<String> {
    let client = unsafe { CWWiFiClient::sharedWiFiClient() };
    let name = NSString::from_str(interface_name);
    let iface = unsafe { client.interfaceWithName(Some(&name)) }?;
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
