extern crate gtk;
extern crate gdk;
mod systemd_gui;     // Contains all of the heavy GUI-related work
mod systemd {
    pub mod analyze; // Support for systemd-analyze
    pub mod dbus;    // The dbus backend for systemd
}

fn main() {
    systemd_gui::launch();
}
