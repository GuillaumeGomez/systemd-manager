use systemd_dbus;    // The dbus-based backend for systemd
use gtk;
use gtk::prelude::*;
use gdk::enums::key;

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub fn launch() {
    gtk::init().unwrap_or_else(|_| panic!("tv-renamer: failed to initialize GTK."));

    let builder = gtk::Builder::new_from_string(include_str!("interface.glade"));
    let window: gtk::Window               = builder.get_object("main_window").unwrap();
    let notebook: gtk::Notebook           = builder.get_object("notebook").unwrap();
    let services_list: gtk::ListBox       = builder.get_object("services_list").unwrap();
    let sockets_list: gtk::ListBox        = builder.get_object("sockets_list").unwrap();
    let timers_list: gtk::ListBox         = builder.get_object("timers_list").unwrap();
    let refresh_units_button: gtk::Button = builder.get_object("refresh_units_button").unwrap();
    let unit_info: gtk::TextView          = builder.get_object("unit_info").unwrap();
    let ableness_button: gtk::Button      = builder.get_object("ableness_button").unwrap();
    let start_button: gtk::Button         = builder.get_object("start_button").unwrap();
    let stop_button: gtk::Button          = builder.get_object("stop_button").unwrap();
    let save_unit_file: gtk::Button       = builder.get_object("save_unit_file").unwrap();

    let unit_files = systemd_dbus::list_unit_files();

    // NOTE: Services
    let services = systemd_dbus::collect_togglable_services(&unit_files);
    let mut services_icons = Vec::new();
    for service in services.clone() {
        let mut unit_row = gtk::ListBoxRow::new();
        create_row(&mut unit_row, Path::new(service.name.as_str()), service.state, &mut services_icons);
        services_list.insert(&unit_row, -1);
    }

    {
        let services       = services.clone();
        let services_list  = services_list.clone();
        let unit_info      = unit_info.clone();
        let ableness_button = ableness_button.clone();
        services_list.connect_row_selected(move |_, row| {
            let index = row.clone().unwrap().get_index();
            let service = &services[index as usize];
            let description = get_unit_info(service.name.as_str());
            unit_info.get_buffer().unwrap().set_text(description.as_str());
            if systemd_dbus::get_unit_file_state(service.name.as_str()) {
                ableness_button.set_label("Disable");
            } else {
                ableness_button.set_label("Enable");
            }
        });
    }

    // NOTE: Sockets
    let sockets = systemd_dbus::collect_togglable_sockets(&unit_files);
    let mut sockets_icons = Vec::new();
    for socket in sockets.clone() {
        let mut unit_row = gtk::ListBoxRow::new();
        create_row(&mut unit_row, Path::new(socket.name.as_str()), socket.state, &mut sockets_icons);
        sockets_list.insert(&unit_row, -1);
    }

    {
        let sockets         = sockets.clone();
        let sockets_list    = sockets_list.clone();
        let unit_info       = unit_info.clone();
        let ableness_button = ableness_button.clone();
        sockets_list.connect_row_selected(move |_, row| {
            let index = row.clone().unwrap().get_index();
            let socket = &sockets[index as usize];
            let description = get_unit_info(socket.name.as_str());
            unit_info.get_buffer().unwrap().set_text(description.as_str());
            if systemd_dbus::get_unit_file_state(socket.name.as_str()) {
                ableness_button.set_label("Disable");
            } else {
                ableness_button.set_label("Enable");
            }
        });
    }

    // NOTE: Timers
    let timers = systemd_dbus::collect_togglable_timers(&unit_files);
    let mut timers_icons = Vec::new();
    for timer in timers.clone() {
        let mut unit_row = gtk::ListBoxRow::new();
        create_row(&mut unit_row, Path::new(timer.name.as_str()), timer.state, &mut timers_icons);
        timers_list.insert(&unit_row, -1);
    }

    {
        let timers          = timers.clone();
        let timers_list     = timers_list.clone();
        let unit_info       = unit_info.clone();
        let ableness_button = ableness_button.clone();
        timers_list.connect_row_selected(move |_, row| {
            let index = row.clone().unwrap().get_index();
            let timer = &timers[index as usize];
            let description = get_unit_info(timer.name.as_str());
            unit_info.get_buffer().unwrap().set_text(description.as_str());
            if systemd_dbus::get_unit_file_state(timer.name.as_str()) {
                ableness_button.set_label("Disable");
            } else {
                ableness_button.set_label("Enable");
            }
        });
    }

    { // NOTE: Implement the {dis, en}able button
    let services      = services.clone();
    let services_list = services_list.clone();
    let sockets       = sockets.clone();
    let sockets_list  = sockets_list.clone();
    let timers        = timers.clone();
    let timers_list   = timers_list.clone();
    let notebook      = notebook.clone();
    ableness_button.connect_clicked(move |button| {
        match notebook.get_current_page().unwrap() {
            0 => {
                let index   = services_list.get_selected_row().unwrap().get_index();
                let service = &services[index as usize];
                let service_path = Path::new(service.name.as_str()).file_name().unwrap().to_str().unwrap();
                if button.get_label().unwrap().as_str() == "Enable" {
                    if let None = systemd_dbus::enable_unit_files(service_path) {
                        button.set_label("Disable");
                    }
                } else {
                    if let None = systemd_dbus::disable_unit_files(service_path) {
                        button.set_label("Enable");
                    }
                }
            },
            1 => {
                let index   = sockets_list.get_selected_row().unwrap().get_index();
                let socket  = &sockets[index as usize];
                let socket_path = Path::new(socket.name.as_str()).file_name().unwrap().to_str().unwrap();
                if button.get_label().unwrap().as_str() == "Enable" {
                    if let None = systemd_dbus::enable_unit_files(socket_path) {
                        button.set_label("Disable");
                    }
                } else {
                    if let None = systemd_dbus::disable_unit_files(socket_path) {
                        button.set_label("Enable");
                    }
                }
            },
            2 => {
                let index   = timers_list.get_selected_row().unwrap().get_index();
                let timer  = &timers[index as usize];
                let timer_path = Path::new(timer.name.as_str()).file_name().unwrap().to_str().unwrap();
                if button.get_label().unwrap().as_str() == "Enable" {
                    if let None = systemd_dbus::enable_unit_files(timer_path) {
                        button.set_label("Disable");
                    }
                } else {
                    if let None = systemd_dbus::disable_unit_files(timer_path) {
                        button.set_label("Enable");
                    }
                }
            },
            _ => ()
        }
    });
    }

    { // NOTE: Implement the start button
        let services       = services.clone();
        let services_list  = services_list.clone();
        let sockets        = sockets.clone();
        let sockets_list   = sockets_list.clone();
        let timers         = timers.clone();
        let timers_list    = timers_list.clone();
        let notebook       = notebook.clone();
        let services_icons = services_icons.clone();
        let sockets_icons  = sockets_icons.clone();
        let timers_icons   = timers_icons.clone();
        start_button.connect_clicked(move |_| {
            match notebook.get_current_page().unwrap() {
                0 => {
                    let index   = services_list.get_selected_row().unwrap().get_index();
                    let service = &services[index as usize];
                    if let None = systemd_dbus::start_unit(Path::new(service.name.as_str()).file_name().unwrap().to_str().unwrap()) {
                        update_icon(&services_icons[index as usize], true);
                    }
                },
                1 => {
                    let index   = sockets_list.get_selected_row().unwrap().get_index();
                    let socket  = &sockets[index as usize];
                    if let None = systemd_dbus::start_unit(Path::new(socket.name.as_str()).file_name().unwrap().to_str().unwrap()) {
                        update_icon(&sockets_icons[index as usize], true);
                    }
                },
                2 => {
                    let index   = timers_list.get_selected_row().unwrap().get_index();
                    let timer  = &timers[index as usize];
                    if let None = systemd_dbus::start_unit(Path::new(timer.name.as_str()).file_name().unwrap().to_str().unwrap()) {
                        update_icon(&timers_icons[index as usize], true);
                    }
                },
                _ => ()
            }
        });
    }

    { // NOTE: Implement the stop button
        let services       = services.clone();
        let services_list  = services_list.clone();
        let sockets        = sockets.clone();
        let sockets_list   = sockets_list.clone();
        let timers         = timers.clone();
        let timers_list    = timers_list.clone();
        let notebook       = notebook.clone();
        let services_icons = services_icons.clone();
        let sockets_icons  = sockets_icons.clone();
        let timers_icons   = timers_icons.clone();
        stop_button.connect_clicked(move |_| {
            match notebook.get_current_page().unwrap() {
                0 => {
                    let index   = services_list.get_selected_row().unwrap().get_index();
                    let service = &services[index as usize];
                    if let None = systemd_dbus::stop_unit(Path::new(service.name.as_str()).file_name().unwrap().to_str().unwrap()) {
                        update_icon(&services_icons[index as usize], false);
                    }
                },
                1 => {
                    let index   = sockets_list.get_selected_row().unwrap().get_index();
                    let socket  = &sockets[index as usize];
                    if let None = systemd_dbus::stop_unit(Path::new(socket.name.as_str()).file_name().unwrap().to_str().unwrap()) {
                        update_icon(&sockets_icons[index as usize], false);
                    }
                },
                2 => {
                    let index   = timers_list.get_selected_row().unwrap().get_index();
                    let timer   = &timers[index as usize];
                    if let None = systemd_dbus::stop_unit(Path::new(timer.name.as_str()).file_name().unwrap().to_str().unwrap()) {
                        update_icon(&timers_icons[index as usize], false);
                    }
                },
                _ => ()
            }
        });
    }

    { // NOTE: Save Button
        let unit_info = unit_info.clone();
        let services      = services.clone();
        let services_list = services_list.clone();
        let sockets       = sockets.clone();
        let sockets_list  = sockets_list.clone();
        let timers        = timers.clone();
        let timers_list   = timers_list.clone();
        let notebook      = notebook.clone();
        save_unit_file.connect_clicked(move |_| {
            let buffer = unit_info.get_buffer().unwrap();
            let start  = buffer.get_start_iter();
            let end    = buffer.get_end_iter();
            let text   = buffer.get_text(&start, &end, true).unwrap();
            let path = match notebook.get_current_page().unwrap() {
                0 => &services[services_list.get_selected_row().unwrap().get_index() as usize].name,
                1 => &sockets[sockets_list.get_selected_row().unwrap().get_index() as usize].name,
                2 => &timers[timers_list.get_selected_row().unwrap().get_index() as usize].name,
                _ => unreachable!()
            };
            match fs::OpenOptions::new().write(true).open(&path) {
                Ok(mut file) => {
                    if let Err(message) = file.write(text.as_bytes()) {
                        println!("Unable to write to file: {:?}", message);
                    }
                },
                Err(message) => println!("Unable to open file: {:?}", message)
            }
        });

    }

    window.show_all();
    refresh_units_button.hide(); // TODO: Hide until it is implemented

    // Quit the program when the program has been exited
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Define custom actions on keypress
    window.connect_key_press_event(move |_, key| {
        if let key::Escape = key.get_keyval() { gtk::main_quit() }
        gtk::Inhibit(false)
    });

    gtk::main();
}

/// Updates the status icon for the selected unit
fn update_icon(icon: &gtk::Image, state: bool) {
    if state { icon.set_from_stock("gtk-yes", 5); } else { icon.set_from_stock("gtk-no", 5); }
}

/// Create a `gtk::ListboxRow` and add it to the `gtk::ListBox`, and then add the `gtk::Image` to a vector so that we can later modify
/// it when the state changes.
fn create_row(row: &mut gtk::ListBoxRow, path: &Path, state: systemd_dbus::UnitState, state_icons: &mut Vec<gtk::Image>) {
    let filename = path.file_stem().unwrap().to_str().unwrap();
    let unit_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let unit_label = gtk::Label::new(Some(filename));
    let image = if state == systemd_dbus::UnitState::Enabled {
        gtk::Image::new_from_stock("gtk-yes", 5)
    } else {
        gtk::Image::new_from_stock("gtk-no", 5)
    };
    unit_box.add(&unit_label);
    unit_box.pack_end(&image, false, false, 5);
    row.add(&unit_box);
    state_icons.push(image);
}

/// Read the unit file and return it's contents so that we can display it in the `gtk::TextView`.
fn get_unit_info<P: AsRef<Path>>(path: P) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut output = String::new();
    let _ = file.read_to_string(&mut output);
    output
}
