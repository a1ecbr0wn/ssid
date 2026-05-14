/// Retrieves and prints the SSID of the current WiFi network.
///
/// Accepts an optional network interface name as the first command-line argument.
/// If provided, retrieves the SSID for that specific interface; otherwise,
/// retrieves the SSID for the default interface.
///
/// Prints the SSID to stdout on success. On failure, prints an error message to stderr
/// and exits with code 1.
fn main() {
    let ssid = match std::env::args().nth(1) {
        Some(iface) => ssid::get_ssid_for_interface(&iface),
        None => ssid::get_ssid(),
    };
    match ssid {
        Some(s) => println!("{s}"),
        None => {
            eprintln!("failed to retrieve SSID");
            std::process::exit(1);
        }
    }
}
