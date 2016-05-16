extern crate gtk;
extern crate gdk;
mod systemd_analyze; // Obtains information for `systemd-analyze blame`
mod systemd_dbus;    // The dbus-based backend for systemd
mod systemd_gui;     // Contains all of the heavy GUI-related work

fn main() {
    systemd_gui::launch();
}
