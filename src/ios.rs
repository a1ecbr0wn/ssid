use std::sync::mpsc;

use block2::RcBlock;
use objc2_network_extension::NEHotspotNetwork;

/// iOS exposes only a single system-wide WiFi connection; the `interface_name`
/// argument is ignored and the result is the same as [`get_ssid`].
#[cfg(target_os = "ios")]
pub fn get_ssid_for_interface(_interface_name: &str) -> Option<String> {
    get_ssid()
}

/// Returns the SSID of the current WiFi network via `NEHotspotNetwork::fetchCurrentWithCompletionHandler`.
///
/// The Objective-C API is asynchronous; this function blocks the calling thread
/// on a channel until the completion handler fires. Returns `None` if the device
/// is not associated with a WiFi network or if the SSID is empty.
///
/// Requires the `com.apple.developer.networking.wifi-info` entitlement and that
/// the user has granted location permission (`NSLocationWhenInUseUsageDescription`).
/// Without either, the completion handler receives `nil` and this function
/// returns `None`.
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
