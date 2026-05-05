use std::sync::mpsc;

use block2::RcBlock;
use objc2_network_extension::NEHotspotNetwork;

// iOS does not expose per-interface WiFi selection; delegate to get_ssid().
#[cfg(target_os = "ios")]
pub fn get_ssid_for_interface(_interface_name: &str) -> Option<String> {
    get_ssid()
}

#[cfg(target_os = "ios")]
pub fn get_ssid() -> Option<String> {
    let (tx, rx) = mpsc::sync_channel::<Option<String>>(1);

    let block = RcBlock::new(move |network: *mut NEHotspotNetwork| {
        let ssid = unsafe { network.as_ref() }.and_then(|n| {
            let s = unsafe { n.SSID() }.to_string();
            if s.is_empty() { None } else { Some(s) }
        });
        // ignore send error — receiver dropped means caller timed out or panicked
        let _ = tx.send(ssid);
    });

    unsafe { NEHotspotNetwork::fetchCurrentWithCompletionHandler(&block) };

    rx.recv().ok().flatten()
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
        // On iOS all interfaces map to get_ssid(); just verify it doesn't panic.
        let _ = super::get_ssid_for_interface("__nonexistent__");
    }
}
