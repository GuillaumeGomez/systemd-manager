use systemd_dbus;    // The dbus-based backend for systemd
use gtk;
use gtk::prelude::*;
use gdk::enums::key;

/// Performs a loop over a list of units and creates a widget from each.
macro_rules! collect_units {
    ($filter_function:ident, $list:expr, $units:expr) => {
        for unit in systemd_dbus::$filter_function($units) {
            let unit_widget = get_unit_widget(unit);
            let row = gtk::ListBoxRow::new();
            row.add(&unit_widget);
            row.set_selectable(false);
            $list.insert(&row, -1);
        }
    }
}

/// Takes either a &str or String and returns a String with directory paths removed
macro_rules! rm_directory_path {
    ($input:expr) => {{
        let temp = $input;
        let mut split: Vec<&str> = temp.split('/').collect();
        String::from(split.pop().unwrap())
    }}
}

pub struct Tabs {
    notebook: gtk::Notebook,
    tabs:     Vec<gtk::Box>,
}

impl Default for Tabs {
    fn default() -> Tabs {
        Tabs {
            notebook: gtk::Notebook::new(),
            tabs: Vec::new(),
        }
    }
}

impl Tabs {
    pub fn create_tab(&mut self, title: &str, widget: &gtk::Widget) -> Option<u32> {
        let label = gtk::Label::new(Some(title));
        let tab = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        tab.pack_start(&label, true, true, 0);
        tab.show_all();

        let index = self.notebook.append_page(widget, Some(&tab));
        self.tabs.push(tab);
        Some(index)
    }
}

/// Launches the GTK3 GUI
pub fn launch() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    // A list of units available on the system
    let unit_files = systemd_dbus::list_unit_files(systemd_dbus::SortMethod::Name);

    // Creates the list of unit files as GTK widgets
    let services = gtk::ListBox::new();
    let sockets = gtk::ListBox::new();
    let timers = gtk::ListBox::new();
    collect_units!(collect_togglable_services, services, &unit_files.clone());
    collect_units!(collect_togglable_sockets, sockets, &unit_files.clone());
    collect_units!(collect_togglable_timers, timers, &unit_files.clone());

    // A structure for holding each of the tabs
    let mut tabs = Tabs::default();

    // Create the services tab
    let services_scroll = gtk::ScrolledWindow::new(None, None);
    let services_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    services_scroll.add(&services);
    services_box.pack_start(&services_scroll, true, true, 0);
    let services_container: gtk::Widget = services_box.upcast();
    tabs.create_tab("Services", &services_container);

    // Create the sockets tab
    let sockets_scroll = gtk::ScrolledWindow::new(None, None);
    let sockets_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    sockets_scroll.add(&sockets);
    sockets_box.pack_start(&sockets_scroll, true, true, 0);
    let sockets_container: gtk::Widget = sockets_box.upcast();
    tabs.create_tab("Sockets", &sockets_container);

    // Create the timers tab
    let timers_scroll = gtk::ScrolledWindow::new(None, None);
    let timers_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    timers_scroll.add(&timers);
    timers_box.pack_start(&timers_scroll, true, true, 0);
    let timers_container: gtk::Widget = timers_box.upcast();
    tabs.create_tab("Timers", &timers_container);

    // Add the tabs to a container
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.pack_start(&tabs.notebook, true, true, 0);

    // Create the window and add the container to the window
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("System Services");
    window.set_default_size(500,500);
    window.add(&container);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::Inhibit(true)
    });

    // Define action on key press
    window.connect_key_press_event(move |_, key| {
        if let key::Escape = key.get_keyval() { gtk::main_quit(); }
        gtk::Inhibit(false)
    });

    gtk::main();
}

/// Removes the directory path and extension from the unit name
fn get_unit_name(x: &str) -> String {
    let mut output = rm_directory_path!(x);
    let mut last_occurrence: usize = 0;
    for (index, value) in output.chars().enumerate() {
        if value == '.' { last_occurrence = index; }
    }
    output.truncate(last_occurrence);
    output
}

/// Takes a `SystemdUnit` and generates a `gtk::Box` widget from that information.
fn get_unit_widget(unit: systemd_dbus::SystemdUnit) -> gtk::Box {
    let switch = match unit.state {
        systemd_dbus::UnitState::Disabled => gtk::Button::new_with_label(" Enable"),
        systemd_dbus::UnitState::Enabled  => gtk::Button::new_with_label("Disable"),
        _ => unreachable!(), // This program currently only collects units that fit the above.
    };

    { // Defines action when clicking on the {en/dis}able toggle switch.
        let service = unit.name.clone();
        switch.connect_clicked(move |switch| {
            let filename = rm_directory_path!(&service);
            if &switch.get_label().unwrap() == "Disable" {
                match systemd_dbus::disable(&filename) {
                    Some(error) => print_dialog(&error),
                    None => switch.set_label(" Enable")
                }
            } else {
                match systemd_dbus::enable(&filename) {
                    Some(error) => print_dialog(&error),
                    None => switch.set_label("Disable")
                }
            }
        });
    }

    // Start Button
    let start_button = gtk::Button::new_with_label("Start"); {
        let unit = rm_directory_path!(unit.name.clone());
        start_button.connect_clicked(move |_| {
            if let Some(error) = systemd_dbus::start(&unit) {
                print_dialog(&error);
            }
        });
    }

    // Stop Button
    let stop_button = gtk::Button::new_with_label("Stop"); {
        let unit = rm_directory_path!(unit.name.clone());
        stop_button.connect_clicked(move |_| {
            if let Some(error) = systemd_dbus::stop(&unit) {
                print_dialog(&error);
            }
        });
    }

    let label = gtk::Label::new(Some(&get_unit_name(&unit.name)));
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    button_box.pack_start(&switch, false, false, 1);
    button_box.pack_start(&start_button, false, false, 1);
    button_box.pack_start(&stop_button, false, false, 1);
    button_box.set_halign(gtk::Align::End);

    let layout = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    layout.pack_start(&label, false, false, 5);
    layout.pack_start(&button_box, true, true, 15);

    layout
}

/// Prints an error dialog with the included message.
fn print_dialog(message: &str) {
    let dialog = gtk::Dialog::new();
    dialog.set_title("Systemd Error");
    let content = dialog.get_content_area();
    let text = gtk::TextView::new();
    text.get_buffer().unwrap().set_text(message);
    text.set_left_margin(5);
    text.set_right_margin(5);
    content.add(&text);
    dialog.show_all();
}
