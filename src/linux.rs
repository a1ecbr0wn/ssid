// nl80211 constants from linux/nl80211.h
const NL80211_ATTR_IFNAME: u16 = 4;
const NL80211_ATTR_SSID: u16 = 52;
const NL80211_CMD_GET_INTERFACE: u8 = 5;

use neli::{
    consts::{
        nl::{NlmF, NlmFFlags},
        socket::NlFamily,
    },
    genl::{Genlmsghdr, Nlattr},
    neli_enum,
    nl::{NlPayload, Nlmsghdr},
    socket::NlSocketHandle,
    types::GenlBuffer,
};

#[neli_enum(serialized_type = "u8")]
enum Nl80211Cmd {
    GetInterface = NL80211_CMD_GET_INTERFACE,
}
impl neli::consts::genl::Cmd for Nl80211Cmd {}

#[neli_enum(serialized_type = "u16")]
enum Nl80211Attr {
    Ifname = NL80211_ATTR_IFNAME,
    Ssid = NL80211_ATTR_SSID,
}
impl neli::consts::genl::NlAttrType for Nl80211Attr {}

fn open_socket() -> Option<(NlSocketHandle, u16)> {
    let mut socket = NlSocketHandle::connect(NlFamily::Generic, None, &[]).ok()?;
    let family_id = socket.resolve_genl_family("nl80211").ok()?;
    Some((socket, family_id))
}

fn extract_ssid(genl: &Genlmsghdr<Nl80211Cmd, Nl80211Attr>) -> Option<String> {
    for attr in genl.get_attr_handle().iter() {
        if attr.nla_type().nla_type() == &Nl80211Attr::Ssid {
            let bytes: &[u8] = attr.nla_payload().as_ref();
            if !bytes.is_empty() {
                return Some(String::from_utf8_lossy(bytes).into_owned());
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
pub fn get_ssid() -> Option<String> {
    let (mut socket, family_id) = open_socket()?;

    let genl_hdr: Genlmsghdr<Nl80211Cmd, Nl80211Attr> =
        Genlmsghdr::new(Nl80211Cmd::GetInterface, 1, GenlBuffer::new());
    let nl_hdr = Nlmsghdr::new(
        None,
        family_id,
        NlmFFlags::new(&[NlmF::Request, NlmF::Dump]),
        None,
        None,
        NlPayload::Payload(genl_hdr),
    );
    socket.send(nl_hdr).ok()?;

    // Returns the SSID of the first associated interface found in the dump.
    // On machines with multiple WiFi adapters the result is non-deterministic;
    // use get_ssid_for_interface to target a specific adapter.
    for response in socket.iter::<Genlmsghdr<Nl80211Cmd, Nl80211Attr>>(false) {
        let msg = match response {
            Ok(m) => m,
            Err(_) => break,
        };
        if let NlPayload::Payload(genl) = msg.nl_payload() {
            if let Some(ssid) = extract_ssid(genl) {
                return Some(ssid);
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
pub fn get_ssid_for_interface(interface_name: &str) -> Option<String> {
    let (mut socket, family_id) = open_socket()?;

    let mut attrs: GenlBuffer<Nl80211Attr, neli::types::Buffer> = GenlBuffer::new();
    attrs.push(Nlattr::new(false, false, Nl80211Attr::Ifname, interface_name).ok()?);

    let genl_hdr = Genlmsghdr::new(Nl80211Cmd::GetInterface, 1, attrs);
    let nl_hdr = Nlmsghdr::new(
        None,
        family_id,
        NlmFFlags::new(&[NlmF::Request]),
        None,
        None,
        NlPayload::Payload(genl_hdr),
    );
    socket.send(nl_hdr).ok()?;

    for response in socket.iter::<Genlmsghdr<Nl80211Cmd, Nl80211Attr>>(false) {
        let msg = match response {
            Ok(m) => m,
            Err(_) => break,
        };
        if let NlPayload::Payload(genl) = msg.nl_payload() {
            if let Some(ssid) = extract_ssid(genl) {
                return Some(ssid);
            }
        }
    }
    None
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
        assert_eq!(result, "My\u{FFFD}WiFi");
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
