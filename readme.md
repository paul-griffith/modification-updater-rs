# Modification Updater

A rudimentary CLI for modifying Ignition project resource files.
Written in Rust, for speed and learning purposes.

# Build
1. Clone this repository.
2. Install Rust.
3. `cargo build --release` in the root of this repository.
4. A native executable will be generated in `target/release`.
5. This self-contained executable can be moved anywhere.

# Usage
`./modification-updater path actor` - updates the project resource directory at `path` with the new actor `actor`.
The current time is used as the timestamp.
