use std::env;

extern crate gdk;
extern crate gtk;
mod systemd_gui; // Contains all of the heavy GUI-related work
mod systemd {
    pub mod analyze; // Support for systemd-analyze
    pub mod dbus; // The dbus backend for systemd
}

fn main() {
    let mut config = Config::default();
    for arg in env::args().skip(1) {
        match arg.as_ref() {
            "--user" => {
                config.bus_type = dbus::BusType::Session;
            }
            x => {
                panic!("Unrecognized CLI argument {:?}", x);
            }
        }
    }
    systemd_gui::launch(config);
}

#[derive(Debug, Clone)]
pub struct Config {
    /// The bus type to use. Defaults to System, can be instead Session to access the user dbus.
    bus_type: dbus::BusType,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bus_type: dbus::BusType::System,
        }
    }
}
