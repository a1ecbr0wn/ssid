use std::ffi::c_void;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::NetworkManagement::WiFi::{
    WLAN_CONNECTION_ATTRIBUTES, WlanCloseHandle, WlanEnumInterfaces, WlanFreeMemory,
    WlanOpenHandle, WlanQueryInterface, wlan_intf_opcode_current_connection,
};
use windows::core::GUID;

fn open_handle() -> Option<HANDLE> {
    let mut negotiated_ver = 0u32;
    let mut handle = HANDLE::default();
    if unsafe { WlanOpenHandle(2, None, &mut negotiated_ver, &mut handle) } != 0 {
        return None;
    }
    Some(handle)
}

fn enum_interface_guids(handle: HANDLE) -> Vec<(String, GUID)> {
    let mut list_ptr = std::ptr::null_mut();
    if unsafe { WlanEnumInterfaces(handle, None, &mut list_ptr) } != 0 {
        return Vec::new();
    }
    let list = unsafe { &*list_ptr };
    let count = list.dwNumberOfItems as usize;
    let infos = unsafe { std::slice::from_raw_parts(list.InterfaceInfo.as_ptr(), count) };
    let result: Vec<(String, GUID)> = infos
        .iter()
        .map(|info| {
            let wide = &info.strInterfaceDescription;
            // strInterfaceDescription is a null-terminated UTF-16 array
            let null_pos = wide.iter().position(|&c| c == 0);
            let wide_str = match null_pos {
                Some(pos) => &wide[..pos],
                None => wide,
            };
            let name = String::from_utf16_lossy(wide_str);
            (name, info.InterfaceGuid)
        })
        .collect();
    unsafe { WlanFreeMemory(list_ptr as *mut c_void) };
    result
}

fn query_ssid(handle: HANDLE, guid: &GUID) -> Option<String> {
    let mut data_size = 0u32;
    let mut data_ptr: *mut c_void = std::ptr::null_mut();
    let mut opcode_type = Default::default();
    let result = unsafe {
        WlanQueryInterface(
            handle,
            guid,
            wlan_intf_opcode_current_connection,
            None,
            &mut data_size,
            &mut data_ptr as *mut *mut c_void,
            Some(&mut opcode_type),
        )
    };
    if result != 0 {
        return None;
    }
    if (data_size as usize) < std::mem::size_of::<WLAN_CONNECTION_ATTRIBUTES>() {
        unsafe { WlanFreeMemory(data_ptr) };
        return None;
    }
    let attrs = unsafe { &*(data_ptr as *const WLAN_CONNECTION_ATTRIBUTES) };
    let dot11 = &attrs.wlanAssociationAttributes.dot11Ssid;
    let len = dot11.uSSIDLength as usize;
    let ssid = String::from_utf8_lossy(&dot11.ucSSID[..len]).into_owned();
    unsafe { WlanFreeMemory(data_ptr) };
    if ssid.is_empty() { None } else { Some(ssid) }
}

#[cfg(target_os = "windows")]
pub fn get_ssid() -> Option<String> {
    let handle = open_handle()?;
    let guids = enum_interface_guids(handle);
    let result = guids.iter().find_map(|(_, guid)| query_ssid(handle, guid));
    unsafe { WlanCloseHandle(handle, None) };
    result
}

/// `interface_name` is matched against the adapter's description string
/// (`strInterfaceDescription` from `WLAN_INTERFACE_INFO`), e.g.
/// `"Intel(R) Wi-Fi 6 AX200 160MHz"` — not the friendly alias shown in
/// Windows Settings. To discover the exact string, run
/// `Get-NetAdapter | Select-Object Name,InterfaceDescription` in PowerShell.
#[cfg(target_os = "windows")]
pub fn get_ssid_for_interface(interface_name: &str) -> Option<String> {
    let handle = open_handle()?;
    let guids = enum_interface_guids(handle);
    // and_then ensures WlanCloseHandle is always reached, even when no GUID matches.
    let result = guids
        .iter()
        .find(|(name, _)| name == interface_name)
        .and_then(|(_, guid)| query_ssid(handle, guid));
    unsafe { WlanCloseHandle(handle, None) };
    result
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
