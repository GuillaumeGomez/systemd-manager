extern crate gtk;           // Enable the creation of GTK windows and widgets
extern crate pango;         // Allows manipulating font styles
use systemctl;              // The command-line backend for handling systemctl
use gtk::traits::*;         // Enables the usage of GTK traits

// create_main_window() creates the main window for this program.
pub fn create_main_window() -> gtk::Window {
	let window = gtk::Window::new(gtk::WindowType::Toplevel).unwrap();
	window.set_title("System Services");
	window.set_default_size(500,500);

	window.connect_delete_event(|_, _| {
		gtk::main_quit();
		gtk::signal::Inhibit(true)
	});

	return window;
}

// generate_services() creates a gtk::ScrolledWindow widget containing the list of units available
// on the system. Each individual unit is created by get_unit_widget() and added to their respective
// gtk::Box.
pub fn generate_services() -> gtk::ScrolledWindow {
	let scrolled_window = gtk::ScrolledWindow::new(None, None).unwrap();

	let mut label_font = pango::FontDescription::new();
	label_font.set_weight(pango::Weight::Heavy);

	let service_list = {
		let list = gtk::Box::new(gtk::Orientation::Vertical, 0).unwrap();
		let service_label = gtk::Label::new("Services (Activate on Startup)").unwrap();
		service_label.override_font(&label_font);
		list.pack_start(&service_label, true, true, 0);
		list
	};

	let socket_list = {
		let list = gtk::Box::new(gtk::Orientation::Vertical, 0).unwrap();
		list.add(&gtk::Separator::new(gtk::Orientation::Horizontal).unwrap());
		let socket_label = gtk::Label::new("Sockets (Activate On Use)").unwrap();
		socket_label.override_font(&label_font);
		list.pack_start(&socket_label, true, true, 0);
		list
	};

	for unit in systemctl::get_unit_files() {
		match unit.unit_type {
			systemctl::UnitType::Service => {
				service_list.add(&gtk::Separator::new(gtk::Orientation::Horizontal).unwrap());
				service_list.pack_start(&get_unit_widget(unit), false, false, 3);
			},
			systemctl::UnitType::Socket => {
				socket_list.add(&gtk::Separator::new(gtk::Orientation::Horizontal).unwrap());
				socket_list.pack_start(&get_unit_widget(unit), false, false, 3);
			},
		};
	}

	service_list.add(&socket_list);
	scrolled_window.add(&service_list);
	return scrolled_window;
}

// get_unit_widget() takes a SystemdUnit and generates a gtk::Box widget from that information.
fn get_unit_widget(unit: systemctl::SystemdUnit) -> gtk::Box {
	let mut label_font = pango::FontDescription::new();
	label_font.set_weight(pango::Weight::Heavy);
	let label = gtk::Label::new(&unit.name).unwrap();
	label.override_font(&label_font);

	let new_button = |x: &str| gtk::Button::new_with_label(x).unwrap();
	let switch = if unit.status { new_button("Disable") } else { new_button(" Enable") };
	switch.set_halign(gtk::Align::End);

	{ // Defines action when clicking the button. Consider this to be it's own thread.
		let service_name: String = match unit.unit_type {
			systemctl::UnitType::Socket => unit.name.clone(),
			systemctl::UnitType::Service => unit.name.chars().take(unit.name.len()-8).collect(),
		};
		switch.connect_clicked(move |switch| {
			if &switch.get_label().unwrap() == "Disable" {
				if systemctl::run("disable", &service_name) { switch.set_label(" Enable"); }
			} else {
				if systemctl::run("enable", &service_name) { switch.set_label("Disable"); }
			}
		});
	}

	let layout = gtk::Box::new(gtk::Orientation::Horizontal, 0).unwrap();
	layout.pack_start(&label, false, false, 15);
	layout.pack_start(&switch, true, true, 15);

	return layout;
}
