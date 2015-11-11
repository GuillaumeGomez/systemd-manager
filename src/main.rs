extern crate gtk;   // Enable the creation of GTK windows and widgets
mod systemctl_gui;  // Contains all of the heavy GUI-related work
mod systemctl;      // The command-line backend for handling systemctl
use gtk::traits::*; // Enables the usage of GTK traits

fn main() {
	gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

	let window = systemctl_gui::create_main_window();
	let services_window = systemctl_gui::generate_services();
	window.add(&services_window);
	window.show_all();

	gtk::main();
}
