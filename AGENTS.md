# ssid

## Product Overview

SSID is a cross-platform Rust-based crate that gets the SSID for a WiFi network.

### Key Features

- Retrieves the WiFi network SSID.

## Technology Stack

- Language: Rust (2024 edition)

### Build & Test Commands

```sh
cargo fmt                                  # format
cargo clippy --all-targets -- -D warnings  # lint (warnings are errors)
cargo build                                # build
cargo test --workspace                     # all tests
cargo run                                  # CLI: print SSID of active interface
cargo run -- <iface>                       # CLI: print SSID for named interface
```

CI enforces `RUSTFLAGS="--deny warnings"` on every build.

## Architecture

A single crate (`ssid`) with a binary entry point (`src/main.rs`) and a library.
Platform-specific WiFi code lives in dedicated modules selected at compile time:

```sh
src/lib.rs          — public API + #[cfg] dispatch to platform module
src/linux.rs        — nl80211 via netlink (neli crate), kernel ≥ 2.6.22
src/macos.rs        — CoreWLAN via objc2 Objective-C messaging
src/windows.rs      — WlanQueryInterface via windows crate
src/unsupported.rs  — returns None on all other platforms
src/main.rs         — CLI: no args → get_ssid(), one arg → get_ssid_for_interface()
```

Public API:

```rust
pub fn get_ssid() -> Option<String>                          // auto-detect active interface
pub fn get_ssid_for_interface(name: &str) -> Option<String>  // named interface
```

Both functions are infallible — they return `None` on any error, missing interface,
or non-wireless interface. No panics, no error propagation to the caller.

Platform deps are gated in `Cargo.toml` under
`[target.'cfg(target_os = "...")'.dependencies]` so they don't compile cross-platform.

Hardware-dependent smoke tests are gated behind the `wifi_test` Cargo feature and
must not run in CI. Unit tests for pure logic (e.g. byte-buffer parsing) run
unconditionally.

### Rust Coding conventions

- Write idiomatic Rust with proper error handling
- Follow Rust naming conventions: `snake_case` for functions/variables, `PascalCase` for types
- Avoid the `?` operator for error propagation — prefer explicitly handling error conditions
- Never just unwrap() an error, unless it is part of a unit test.
- Keep functions focused and well-documented, including parameter descriptions in doc comments
- When using format! and you can inline variables into {}, always do it.
- When possible, make `match` statements exhaustive and avoid wildcard arms.
- Where there are chunks of platform specific code, it is cleaner to isolate them into files that are specific to, and named after, that platform (e.g. unix.rs, linux.rs, macos.rs, windows.rs) and protect each function with a target_os specifcation for clarity, e.g. #[cfg(target_os = "linux")].
- Each platform should be tested on the correct platform by the pull request github action, so don't create tests of platform specific code that can run on different platforms.
- When writing tests, prefer comparing the equality of entire objects over fields one by one.
- Do not create small helper methods that are referenced only once.
- Avoid large modules:
  - Prefer adding new modules instead of growing existing ones.
  - Target Rust modules under 500 LoC, excluding tests.
  - If a file exceeds roughly 800 LoC, add new functionality in a new module instead of extending
    the existing file unless there is a strong documented reason not to.
  - When extracting code from a large module, move the related tests and module/type docs toward
    the new implementation so the invariants stay close to the code that owns them.
- When running Rust commands (e.g. `cargo test`) be patient with the command and never try to kill them using the PID. Rust lock can make the execution slow, this is expected.
