// nl80211 attribute IDs from linux/nl80211.h
const NL80211_ATTR_IFINDEX: u16 = 3;
const NL80211_ATTR_IFNAME: u16 = 4;
const NL80211_ATTR_BSS: u16 = 47;

// nl80211 command IDs from linux/nl80211.h
const NL80211_CMD_GET_INTERFACE: u8 = 5;
const NL80211_CMD_GET_SCAN: u8 = 32;

// nl80211 BSS nested attribute IDs from linux/nl80211.h
const NL80211_BSS_INFORMATION_ELEMENTS: u16 = 6;
const NL80211_BSS_STATUS: u16 = 9;

// nl80211_bss_status values from linux/nl80211.h
const NL80211_BSS_STATUS_ASSOCIATED: u32 = 1;

// IEEE 802.11 SSID information element type
const IE_TYPE_SSID: u8 = 0;

use neli::{
    consts::{
        nl::{GenlId, NlmF},
        socket::NlFamily,
    },
    genl::{AttrTypeBuilder, Genlmsghdr, GenlmsghdrBuilder, Nlattr, NlattrBuilder, NoUserHeader},
    neli_enum,
    nl::NlPayload,
    router::synchronous::NlRouter,
    types::{Buffer, GenlBuffer},
    utils::Groups,
};

#[neli_enum(serialized_type = "u8")]
enum Nl80211Cmd {
    GetInterface = NL80211_CMD_GET_INTERFACE,
    GetScan = NL80211_CMD_GET_SCAN,
}
impl neli::consts::genl::Cmd for Nl80211Cmd {}

#[neli_enum(serialized_type = "u16")]
enum Nl80211Attr {
    Ifindex = NL80211_ATTR_IFINDEX,
    Ifname = NL80211_ATTR_IFNAME,
    Bss = NL80211_ATTR_BSS,
}
impl neli::consts::genl::NlAttrType for Nl80211Attr {}

#[neli_enum(serialized_type = "u16")]
enum Nl80211BssAttr {
    InformationElements = NL80211_BSS_INFORMATION_ELEMENTS,
    Status = NL80211_BSS_STATUS,
}
impl neli::consts::genl::NlAttrType for Nl80211BssAttr {}

/// Open a generic netlink socket connected to the nl80211 family.
///
/// Returns the router and the resolved nl80211 family ID, or `None` if the
/// nl80211 family is unavailable (no wireless hardware or insufficient permissions).
fn open_socket() -> Option<(NlRouter, u16)> {
    let (socket, _) = NlRouter::connect(NlFamily::Generic, Some(0), Groups::empty())
        .map_err(|e| log::warn!("nl80211: failed to open netlink socket: {e}"))
        .ok()?;
    let family_id = socket
        .resolve_genl_family("nl80211")
        .map_err(|e| log::warn!("nl80211: family not found (no wireless hardware?): {e}"))
        .ok()?;
    Some((socket, family_id))
}

/// Parse the SSID from a slice of 802.11 Information Elements.
///
/// IEs are encoded as `(type: u8, length: u8, data: [u8; length])` triples.
/// The SSID element has type 0. Returns `None` if no SSID element is found or
/// the SSID is empty (i.e. a hidden network).
fn ssid_from_ies(bytes: &[u8]) -> Option<String> {
    let mut i = 0;
    while i + 2 <= bytes.len() {
        let ie_type = bytes[i];
        let ie_len = bytes[i + 1] as usize;
        i += 2;
        if i + ie_len > bytes.len() {
            break;
        }
        if ie_type == IE_TYPE_SSID && ie_len > 0 {
            return Some(String::from_utf8_lossy(&bytes[i..i + ie_len]).into_owned());
        }
        i += ie_len;
    }
    None
}

/// Extract the SSID from a scan result's BSS nested attribute.
///
/// Parses the `NL80211_ATTR_BSS` nested payload, checks that `NL80211_BSS_STATUS`
/// equals `NL80211_BSS_STATUS_ASSOCIATED`, and extracts the SSID from the
/// `NL80211_BSS_INFORMATION_ELEMENTS` binary IEs.  Returns `None` if the BSS is
/// not the currently associated one or contains no SSID.
fn ssid_from_bss_attr(bss_attr: &Nlattr<Nl80211Attr, Buffer>) -> Option<String> {
    let handle = bss_attr.get_attr_handle::<Nl80211BssAttr>().ok()?;

    let mut status: Option<u32> = None;
    let mut ssid: Option<String> = None;

    for attr in handle.iter() {
        match attr.nla_type().nla_type() {
            Nl80211BssAttr::Status => {
                let bytes = attr.nla_payload().as_ref();
                if bytes.len() >= 4 {
                    // netlink delivers integers in host byte order
                    status = Some(u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
                }
            }
            Nl80211BssAttr::InformationElements => {
                ssid = ssid_from_ies(attr.nla_payload().as_ref());
            }
            // neli may surface kernel attribute IDs not mapped in this enum
            _ => {}
        }
    }

    if status == Some(NL80211_BSS_STATUS_ASSOCIATED) {
        ssid
    } else {
        None
    }
}

/// Enumerate all nl80211 wireless interfaces, returning `(ifindex, ifname)` pairs.
fn get_interface_list(socket: &NlRouter, family_id: u16) -> Vec<(u32, String)> {
    let genl_hdr = match GenlmsghdrBuilder::<Nl80211Cmd, Nl80211Attr, NoUserHeader>::default()
        .cmd(Nl80211Cmd::GetInterface)
        .version(1)
        .build()
    {
        Ok(h) => h,
        Err(e) => {
            log::warn!("nl80211: failed to build GetInterface message: {e}");
            return Vec::new();
        }
    };

    let recv = match socket.send::<_, _, GenlId, Genlmsghdr<Nl80211Cmd, Nl80211Attr>>(
        family_id,
        NlmF::REQUEST | NlmF::DUMP,
        NlPayload::Payload(genl_hdr),
    ) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("nl80211: GetInterface send failed: {e}");
            return Vec::new();
        }
    };

    let mut interfaces = Vec::new();
    for response in recv {
        let msg = match response {
            Ok(m) => m,
            Err(e) => {
                log::warn!("nl80211: recv error in GetInterface, stopping iteration: {e}");
                break;
            }
        };
        if let Some(genl) = msg.get_payload() {
            let mut ifindex: Option<u32> = None;
            let mut ifname: Option<String> = None;
            for attr in genl.attrs().get_attr_handle().iter() {
                match attr.nla_type().nla_type() {
                    Nl80211Attr::Ifindex => {
                        let bytes = attr.nla_payload().as_ref();
                        if bytes.len() >= 4 {
                            // netlink delivers integers in host byte order
                            ifindex =
                                Some(u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
                        }
                    }
                    Nl80211Attr::Ifname => {
                        let bytes = attr.nla_payload().as_ref();
                        let end = match bytes.iter().position(|&b| b == 0) {
                            Some(pos) => pos,
                            None => bytes.len(),
                        };
                        let name = String::from_utf8_lossy(&bytes[..end]).into_owned();
                        if !name.is_empty() {
                            ifname = Some(name);
                        }
                    }
                    // neli may surface kernel attribute IDs not mapped in this enum
                    _ => {}
                }
            }
            if let (Some(idx), Some(name)) = (ifindex, ifname) {
                interfaces.push((idx, name));
            }
        }
    }
    interfaces
}

/// Query nl80211 scan results for a specific interface and return the SSID of
/// the currently associated BSS, or `None` if the interface is not connected.
fn ssid_for_ifindex(socket: &NlRouter, family_id: u16, ifindex: u32) -> Option<String> {
    let ifindex_attr = NlattrBuilder::default()
        .nla_type(
            AttrTypeBuilder::default()
                .nla_type(Nl80211Attr::Ifindex)
                .build()
                .ok()?,
        )
        .nla_payload(ifindex)
        .build()
        .ok()?;
    let attrs: GenlBuffer<Nl80211Attr, Buffer> = std::iter::once(ifindex_attr).collect();

    let genl_hdr = GenlmsghdrBuilder::<Nl80211Cmd, Nl80211Attr, NoUserHeader>::default()
        .cmd(Nl80211Cmd::GetScan)
        .version(1)
        .attrs(attrs)
        .build()
        .ok()?;

    let recv = socket
        .send::<_, _, GenlId, Genlmsghdr<Nl80211Cmd, Nl80211Attr>>(
            family_id,
            NlmF::REQUEST | NlmF::DUMP,
            NlPayload::Payload(genl_hdr),
        )
        .map_err(|e| log::warn!("nl80211: GetScan send failed for ifindex {ifindex}: {e}"))
        .ok()?;

    for response in recv {
        let msg = match response {
            Ok(m) => m,
            Err(e) => {
                log::warn!(
                    "nl80211: recv error in GetScan for ifindex {ifindex}, stopping iteration: {e}"
                );
                break;
            }
        };
        if let Some(genl) = msg.get_payload() {
            for attr in genl.attrs().get_attr_handle().iter() {
                if let Nl80211Attr::Bss = attr.nla_type().nla_type() {
                    if let Some(ssid) = ssid_from_bss_attr(attr) {
                        return Some(ssid);
                    }
                }
            }
        }
    }
    None
}

/// Return the SSID of the first connected wireless interface found, or `None`
/// if no wireless interface is currently associated with a network.
///
/// On machines with multiple WiFi adapters the result is non-deterministic;
/// use [`get_ssid_for_interface`] to target a specific adapter.
#[cfg(target_os = "linux")]
pub fn get_ssid() -> Option<String> {
    let (socket, family_id) = open_socket()?;
    for (ifindex, _) in get_interface_list(&socket, family_id) {
        if let Some(ssid) = ssid_for_ifindex(&socket, family_id, ifindex) {
            return Some(ssid);
        }
    }
    None
}

/// Return the SSID of the named wireless interface, or `None` if it is not
/// connected or does not exist.
#[cfg(target_os = "linux")]
pub fn get_ssid_for_interface(interface_name: &str) -> Option<String> {
    let (socket, family_id) = open_socket()?;
    let ifindex = get_interface_list(&socket, family_id)
        .into_iter()
        .find(|(_, name)| name == interface_name)
        .map(|(idx, _)| idx)?;
    ssid_for_ifindex(&socket, family_id, ifindex)
}

#[cfg(test)]
mod tests {
    #[test]
    fn ssid_bytes_to_string_valid_utf8() {
        let bytes = b"MyNetwork";
        let result = String::from_utf8_lossy(bytes).into_owned();
        assert_eq!(result, "MyNetwork");
    }

    #[test]
    fn ssid_bytes_to_string_non_utf8() {
        // SSIDs can contain arbitrary bytes; from_utf8_lossy replaces invalid
        // sequences with U+FFFD. Bytes: 'M','y',0xFF,'W','i','f','i'
        let bytes = [0x4d, 0x79, 0xff, 0x57, 0x69, 0x66, 0x69];
        let result = String::from_utf8_lossy(&bytes).into_owned();
        assert_eq!(result, "My\u{FFFD}Wifi");
    }

    #[test]
    fn ssid_from_ies_finds_ssid() {
        // IE: type=0 (SSID), len=8, data="TestWifi"
        let mut ies = vec![0x00, 0x08];
        ies.extend_from_slice(b"TestWifi");
        // Add another IE to ensure we stop at the right one
        ies.extend_from_slice(&[0x01, 0x02, 0x82, 0x84]);
        assert_eq!(super::ssid_from_ies(&ies), Some("TestWifi".to_string()));
    }

    #[test]
    fn ssid_from_ies_hidden_network() {
        // SSID IE with length 0 = hidden network
        let ies = vec![0x00, 0x00, 0x01, 0x02, 0x82, 0x84];
        assert_eq!(super::ssid_from_ies(&ies), None);
    }

    #[test]
    fn ssid_from_ies_empty() {
        assert_eq!(super::ssid_from_ies(&[]), None);
    }

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
