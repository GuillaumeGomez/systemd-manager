extern crate gtk;
extern crate gdk;
mod systemd_gui;    // Contains all of the heavy GUI-related work
mod systemd_dbus;   // The dbus-based backend for systemd

fn main() {
    systemd_gui::launch();
}
