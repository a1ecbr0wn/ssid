fn main() {
    let ssid = match std::env::args().nth(1) {
        Some(iface) => ssid::get_ssid_for_interface(&iface),
        None => ssid::get_ssid(),
    };
    println!("{}", ssid.as_deref().unwrap_or("none"));
}
