extern crate gtk;    // Enable the creation of GTK windows and widgets
extern crate pango;  // Allows manipulating font styles
use systemd_dbus;    // The dbus-based backend for systemd
use gtk::traits::*;  // Enables the usage of GTK traits

// create_list_widget! creates the widgets for each section
macro_rules! create_list_widget {
    ($label:expr, $label_font:expr, $top:expr) => {{
        let list = gtk::Box::new(gtk::Orientation::Vertical, 0).unwrap();
        if !$top { list.add(&gtk::Separator::new(gtk::Orientation::Horizontal).unwrap()); }
        let label = gtk::Label::new($label).unwrap();
        label.override_font(&$label_font);
        list.pack_start(&label, true, true, 0);
        list
    }};
}

// collect_units performs a loop over a list of units and creates a widget from each.
macro_rules! collect_units {
    ($filter_function:ident, $list:expr, $units:expr) => {
        for unit in systemd_dbus::$filter_function($units) {
            $list.add(&gtk::Separator::new(gtk::Orientation::Horizontal).unwrap());
            $list.pack_start(&get_unit_widget(unit), false, false, 3);
        }
    }
}

// rm_directory_path takes either a &str or String and returns a String with directory paths removed
macro_rules! rm_directory_path {
    ($input:expr) => {{
        let temp = $input;
        let mut split: Vec<&str> = temp.split('/').collect();
        String::from(split.pop().unwrap())
    }}
}

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
    let mut label_font = pango::FontDescription::new();
    label_font.set_weight(pango::Weight::Heavy);

    let service_list = create_list_widget!("Services (Activate on Startup)", label_font, true);
    let socket_list = create_list_widget!("Sockets (Activate On Use)", label_font, false);
    let timer_list = create_list_widget!("Timers (Activate Periodically)", label_font, false);

    {
        let unit_files = systemd_dbus::list_unit_files();
        collect_units!(collect_togglable_services, service_list, unit_files.clone());
        collect_units!(collect_togglable_sockets, socket_list, unit_files.clone());
        collect_units!(collect_togglable_timers, timer_list, unit_files.clone());
    }

    service_list.add(&socket_list);
    service_list.add(&timer_list);
    let scrolled_window = gtk::ScrolledWindow::new(None, None).unwrap();
    scrolled_window.add(&service_list);
    return scrolled_window;
}

// get_unit_widget() takes a SystemdUnit and generates a gtk::Box widget from that information.
fn get_unit_widget(unit: systemd_dbus::SystemdUnit) -> gtk::Box {
    let switch = match unit.state {
        systemd_dbus::UnitState::Disabled => gtk::Button::new_with_label(" Enable").unwrap(),
        systemd_dbus::UnitState::Enabled  => gtk::Button::new_with_label("Disable").unwrap(),
        _ => unreachable!(), // This program currently only collects units that fit the above.
    };

    { // Defines action when clicking on the {en/dis}able toggle switch.
        let service = unit.name.clone();
        switch.connect_clicked(move |switch| {
            if &switch.get_label().unwrap() == "Disable" {
                if systemd_dbus::disable(&service) { switch.set_label(" Enable"); }
            } else {
                if systemd_dbus::enable(&service) { switch.set_label("Disable"); }
            }
        });
    }

    // Start Button
    let start_button = gtk::Button::new_with_label("Start").unwrap(); {
        let unit = rm_directory_path!(unit.name.clone());
        start_button.connect_clicked(move |_| { systemd_dbus::start(&unit); });
    }
    
    // Stop Button
    let stop_button = gtk::Button::new_with_label("Stop").unwrap(); {
        let unit = rm_directory_path!(unit.name.clone());
        stop_button.connect_clicked(move |_| { systemd_dbus::stop(&unit); });
    }

    // Removes the directory path and extension from the unit name
    fn get_unit_name(x: &str) -> String {
        let mut output = rm_directory_path!(x);
        let mut last_occurrence: usize = 0;
        for (index, value) in output.chars().enumerate() {
            if value == '.' { last_occurrence = index; }
        }
        output.truncate(last_occurrence);
        return output
    }

    let mut label_font = pango::FontDescription::new();
    label_font.set_weight(pango::Weight::Heavy);
    let label = gtk::Label::new(&get_unit_name(&unit.name)).unwrap();
    label.override_font(&label_font);
    
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0).unwrap();
    button_box.pack_start(&switch, false, false, 1);
    button_box.pack_start(&start_button, false, false, 1);
    button_box.pack_start(&stop_button, false, false, 1);
    button_box.set_halign(gtk::Align::End);

    let layout = gtk::Box::new(gtk::Orientation::Horizontal, 0).unwrap();
    layout.pack_start(&label, false, false, 5);
    layout.pack_start(&button_box, true, true, 15);

    return layout;
}
