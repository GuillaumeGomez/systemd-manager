use std::env;

use systemd::dbus::BUS_TYPE;

extern crate gdk;
extern crate gtk;
mod systemd_gui; // Contains all of the heavy GUI-related work
mod systemd {
    pub mod analyze; // Support for systemd-analyze
    pub mod dbus; // The dbus backend for systemd
}

fn main() {
    for arg in env::args().skip(1) {
        match arg.as_ref() {
            "--user" => {
                *BUS_TYPE.lock().unwrap() = dbus::BusType::Session;
            }
            x => {
                panic!("Unrecognized CLI argument {:?}", x);
            }
        }
    }
    systemd_gui::launch();
}
