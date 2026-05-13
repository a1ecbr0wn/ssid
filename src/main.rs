fn main() {
    let ssid = match std::env::args().nth(1) {
        Some(iface) => ssid::get_ssid_for_interface(&iface),
        None => ssid::get_ssid(),
    };
    match ssid {
        Some(s) => println!("{s}"),
        None => println!("none"),
    }
}
