use systemd::dbus::{self, UnitState};
use systemd::analyze::Analyze;
use gtk;
use gtk::prelude::*;
use gdk::enums::key;

use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

#[cfg(not(feature = "gtk_3_16"))]
macro_rules! with_gtk_3_16 {
    ($e:expr) => (
        ()
    );
    ($bl:block) => {
        ()
    }
}

#[cfg(feature = "gtk_3_16")]
macro_rules! with_gtk_3_16 {
    ($e:expr) => (
        $e
    );
    ($bl:block) => {
        $bl
    }
}

/// Updates the status icon for the selected unit
fn update_icon(icon: &gtk::Image, state: bool) {
    if state { icon.set_from_stock("gtk-yes", 4); } else { icon.set_from_stock("gtk-no", 4); }
}

/// Create a `gtk::ListboxRow` and add it to the `gtk::ListBox`, and then add the `gtk::Image` to a vector so that we can later modify
/// it when the state changes.
fn create_row(row: &mut gtk::ListBoxRow, path: &Path, state: UnitState, state_icons: &mut Vec<gtk::Image>) {
    let filename = path.file_stem().unwrap().to_str().unwrap();
    let unit_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let unit_label = gtk::Label::new(Some(filename));
    let image = if state == UnitState::Enabled {
        gtk::Image::new_from_stock("gtk-yes", 4)
    } else {
        gtk::Image::new_from_stock("gtk-no", 4)
    };
    unit_box.add(&unit_label);
    unit_box.pack_end(&image, false, false, 15);
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

/// Obtain the description from the unit file and return it.
fn get_unit_description(info: &str) -> Option<&str> {
    match info.lines().find(|x| x.starts_with("Description=")) {
        Some(description) => Some(description.split_at(12).1),
        None => None
    }
}

/// Use `systemd-analyze blame` to fill out the information for the Analyze `gtk::Stack`.
fn setup_systemd_analyze(builder: &gtk::Builder) {
    let analyze_tree: gtk::TreeView = builder.get_object("analyze_tree").unwrap();
    let analyze_store = gtk::ListStore::new(&[gtk::Type::U32, gtk::Type::String]);

    // A simple macro for adding a column to the preview tree.
    macro_rules! add_column {
        ($preview_tree:ident, $title:expr, $id:expr) => {{
            let column   = gtk::TreeViewColumn::new();
            let renderer = gtk::CellRendererText::new();
            column.set_title($title);
            column.set_resizable(true);
            column.pack_start(&renderer, true);
            column.add_attribute(&renderer, "text", $id);
            analyze_tree.append_column(&column);
        }}
    }

    add_column!(analyze_store, "Time (ms)", 0);
    add_column!(analyze_store, "Unit", 1);

    let units = Analyze::blame();
    for value in units.clone() {
        analyze_store.insert_with_values(None, &[0, 1], &[&value.time, &value.service]);
    }

    analyze_tree.set_model(Some(&analyze_store));

    let total_time_label: gtk::Label = builder.get_object("time_to_boot").unwrap();
    let time = (units.iter().last().unwrap().time as f32) / 1000f32;
    total_time_label.set_label(format!("{} seconds", time).as_str());
}

/// Updates the associated journal `TextView` with the contents of the unit's journal log.
fn update_journal(journal: &gtk::TextView, unit_path: &str) {
    journal.get_buffer().unwrap().set_text(get_unit_journal(unit_path).as_str());
}

/// Obtains the journal log for the given unit.
fn get_unit_journal(unit_path: &str) -> String {
    let log = String::from_utf8(Command::new("journalctl").arg("-b").arg("-u")
        .arg(Path::new(unit_path).file_stem().unwrap().to_str().unwrap())
        .output().unwrap().stdout).unwrap();
    log.lines().rev().map(|x| x.trim()).fold(String::with_capacity(log.len()), |acc, x| acc + "\n" + x)
}

fn get_filename(path: &str) -> &str {
    Path::new(path).file_name().unwrap().to_str().unwrap()
}

#[cfg(feature = "gtk_3_16")]
const GLADE_FILE: &'static str = include_str!("interface.glade");
#[cfg(not(feature = "gtk_3_16"))]
const GLADE_FILE: &'static str = include_str!("interface_3_10.glade");

pub fn launch() {
    gtk::init().unwrap_or_else(|_| panic!("systemd-manager: failed to initialize GTK."));

    let builder = gtk::Builder::new_from_string(GLADE_FILE);
    let window: gtk::Window                    = builder.get_object("main_window").unwrap();
    let unit_stack: gtk::Stack                 = builder.get_object("unit_stack").unwrap();
    let services_list: gtk::ListBox            = builder.get_object("services_list").unwrap();
    let sockets_list: gtk::ListBox             = builder.get_object("sockets_list").unwrap();
    let timers_list: gtk::ListBox              = builder.get_object("timers_list").unwrap();
    let unit_info: gtk::TextView               = builder.get_object("unit_info").unwrap();
    let ablement_switch: gtk::Switch           = builder.get_object("ablement_switch").unwrap();
    let start_button: gtk::Button              = builder.get_object("start_button").unwrap();
    let stop_button: gtk::Button               = builder.get_object("stop_button").unwrap();
    let save_unit_file: gtk::Button            = builder.get_object("save_button").unwrap();
    let unit_menu_label: gtk::Label            = builder.get_object("unit_menu_label").unwrap();
    let unit_popover: gtk::PopoverMenu         = builder.get_object("unit_menu_popover").unwrap();
    let services_button: gtk::Button           = builder.get_object("services_button").unwrap();
    let sockets_button: gtk::Button            = builder.get_object("sockets_button").unwrap();
    let timers_button: gtk::Button             = builder.get_object("timers_button").unwrap();
    let unit_journal: gtk::TextView            = builder.get_object("unit_journal_view").unwrap();
    let refresh_log_button: gtk::Button        = builder.get_object("refresh_log_button").unwrap();
    let header_service_label: gtk::Label       = builder.get_object("header_service_label").unwrap();
    let action_buttons: gtk::Box               = builder.get_object("action_buttons").unwrap();
    let systemd_menu_label: gtk::Label         = builder.get_object("systemd_menu_label").unwrap();
    let systemd_units_button: gtk::MenuButton  = builder.get_object("systemd_units_button").unwrap();
    let main_window_stack: gtk::Stack          = builder.get_object("main_window_stack").unwrap();
    let systemd_units: gtk::Button             = builder.get_object("systemd_units").unwrap();
    let systemd_analyze: gtk::Button           = builder.get_object("systemd_analyze").unwrap();
    let systemd_menu_popover: gtk::PopoverMenu = builder.get_object("systemd_menu_popover").unwrap();


    { // NOTE: Program the Systemd Analyze Button
        let systemd_analyze      = systemd_analyze.clone();
        let main_window_stack    = main_window_stack.clone();
        let systemd_menu_label   = systemd_menu_label.clone();
        let header_service_label = header_service_label.clone();
        let action_buttons       = action_buttons.clone();
        let systemd_units_button = systemd_units_button.clone();
        let popover              = systemd_menu_popover.clone();
        systemd_analyze.connect_clicked(move |_| {
            main_window_stack.set_visible_child_name("Systemd Analyze");
            systemd_menu_label.set_label("Systemd Analyze");
            systemd_units_button.set_visible(false);
            header_service_label.set_visible(false);
            action_buttons.set_visible(false);
            popover.set_visible(false);
        });
    }

    { // NOTE: Program the Systemd Unit Button
        let systemd_units_button = systemd_units_button.clone();
        let main_window_stack    = main_window_stack.clone();
        let systemd_menu_label   = systemd_menu_label.clone();
        let header_service_label = header_service_label.clone();
        let action_buttons       = action_buttons.clone();
        let systemd_units_button = systemd_units_button.clone();
        let popover              = systemd_menu_popover.clone();
        systemd_units.connect_clicked(move |_| {
            main_window_stack.set_visible_child_name("Systemd Units");
            systemd_menu_label.set_label("Systemd Units");
            systemd_units_button.set_visible(true);
            header_service_label.set_visible(true);
            action_buttons.set_visible(true);
            popover.set_visible(false);
        });
    }

    // List of all unit files on the system
    let unit_files = dbus::list_unit_files();
    let services   = dbus::collect_togglable_services(&unit_files);
    let sockets    = dbus::collect_togglable_sockets(&unit_files);
    let timers     = dbus::collect_togglable_timers(&unit_files);

    { // NOTE: Services Menu Button
        let label           = unit_menu_label.clone();
        let stack           = unit_stack.clone();
        let popover         = unit_popover.clone();
        let services        = services.clone();
        let services_list   = services_list.clone();
        let unit_info       = unit_info.clone();
        let ablement_switch = ablement_switch.clone();
        let unit_journal    = unit_journal.clone();
        let header          = header_service_label.clone();
        services_button.connect_clicked(move |_| {
            stack.set_visible_child_name("Services");
            label.set_text("Services");
            popover.set_visible(false);
            services_list.select_row(Some(&services_list.get_row_at_index(0).unwrap()));
            let service = &services[0];
            let info = get_unit_info(service.name.as_str());
            unit_info.get_buffer().unwrap().set_text(info.as_str());
            ablement_switch.set_active(dbus::get_unit_file_state(service.name.as_str()));
            ablement_switch.set_state(ablement_switch.get_active());
            update_journal(&unit_journal, service.name.as_str());
            match get_unit_description(&info) {
                Some(description) => header.set_label(description),
                None              => header.set_label(get_filename(service.name.as_str()))
            }
        });
    }

    { // NOTE: Sockets Menu Button
        let label           = unit_menu_label.clone();
        let stack           = unit_stack.clone();
        let popover         = unit_popover.clone();
        let sockets         = sockets.clone();
        let sockets_list    = sockets_list.clone();
        let unit_info       = unit_info.clone();
        let ablement_switch = ablement_switch.clone();
        let unit_journal    = unit_journal.clone();
        let header          = header_service_label.clone();
        sockets_button.connect_clicked(move |_| {
            stack.set_visible_child_name("Sockets");
            label.set_text("Sockets");
            popover.set_visible(false);
            sockets_list.select_row(Some(&sockets_list.get_row_at_index(0).unwrap()));
            let socket = &sockets[0];
            let info = get_unit_info(socket.name.as_str());
            unit_info.get_buffer().unwrap().set_text(info.as_str());
            ablement_switch.set_active(dbus::get_unit_file_state(socket.name.as_str()));
            ablement_switch.set_state(true);
            update_journal(&unit_journal, socket.name.as_str());
            match get_unit_description(&info) {
                Some(description) => header.set_label(description),
                None              => header.set_label(get_filename(socket.name.as_str()))
            }
        });
    }

    { // NOTE: Timers Menu Button
        let label           = unit_menu_label.clone();
        let stack           = unit_stack.clone();
        let popover         = unit_popover.clone();
        let timers          = timers.clone();
        let timers_list     = timers_list.clone();
        let unit_info       = unit_info.clone();
        let ablement_switch = ablement_switch.clone();
        let unit_journal    = unit_journal.clone();
        let header          = header_service_label.clone();
        timers_button.connect_clicked(move |_| {
            stack.set_visible_child_name("Timers");
            label.set_text("Timers");
            popover.set_visible(false);
            timers_list.select_row(Some(&timers_list.get_row_at_index(0).unwrap()));
            let timer = &timers[0];
            let info = get_unit_info(timer.name.as_str());
            unit_info.get_buffer().unwrap().set_text(info.as_str());
            ablement_switch.set_active(dbus::get_unit_file_state(timer.name.as_str()));
            ablement_switch.set_state(true);
            update_journal(&unit_journal, timer.name.as_str());
            header.set_label(get_filename(timer.name.as_str()));
            match get_unit_description(&info) {
                Some(description) => header.set_label(description),
                None              => header.set_label(get_filename(timer.name.as_str()))
            }
        });
    }

    // Setup the Analyze stack
    setup_systemd_analyze(&builder);

    // NOTE: Services
    let mut services_icons = Vec::new();
    for service in services.clone() {
        let mut unit_row = gtk::ListBoxRow::new();
        create_row(&mut unit_row, Path::new(service.name.as_str()), service.state, &mut services_icons);
        services_list.insert(&unit_row, -1);
    }

    {
        let services        = services.clone();
        let services_list   = services_list.clone();
        let unit_info       = unit_info.clone();
        let ablement_switch = ablement_switch.clone();
        let unit_journal    = unit_journal.clone();
        let header          = header_service_label.clone();
        services_list.connect_row_selected(move |_, row| {
            let index = row.clone().unwrap().get_index();
            let service = &services[index as usize];
            let description = get_unit_info(service.name.as_str());
            unit_info.get_buffer().unwrap().set_text(description.as_str());
            ablement_switch.set_active(dbus::get_unit_file_state(service.name.as_str()));
            ablement_switch.set_state(ablement_switch.get_active());
            update_journal(&unit_journal, service.name.as_str());
            header.set_label(get_filename(service.name.as_str()));
            match get_unit_description(&description) {
                Some(description) => header.set_label(description),
                None              => header.set_label(get_filename(service.name.as_str()))
            }
        });
    }

    // NOTE: Sockets
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
        let ablement_switch = ablement_switch.clone();
        let unit_journal    = unit_journal.clone();
        let header          = header_service_label.clone();
        sockets_list.connect_row_selected(move |_, row| {
            let index = row.clone().unwrap().get_index();
            let socket = &sockets[index as usize];
            let info = get_unit_info(socket.name.as_str());
            unit_info.get_buffer().unwrap().set_text(info.as_str());
            ablement_switch.set_active(dbus::get_unit_file_state(socket.name.as_str()));
            ablement_switch.set_state(true);
            update_journal(&unit_journal, socket.name.as_str());
            header.set_label(get_filename(socket.name.as_str()));
            match get_unit_description(&info) {
                Some(description) => header.set_label(description),
                None              => header.set_label(get_filename(socket.name.as_str()))
            }
        });
    }

    // NOTE: Timers
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
        let ablement_switch = ablement_switch.clone();
        let unit_journal    = unit_journal.clone();
        let header          = header_service_label.clone();
        timers_list.connect_row_selected(move |_, row| {
            let index = row.clone().unwrap().get_index();
            let timer = &timers[index as usize];
            let info = get_unit_info(timer.name.as_str());
            unit_info.get_buffer().unwrap().set_text(info.as_str());
            ablement_switch.set_active(dbus::get_unit_file_state(timer.name.as_str()));
            ablement_switch.set_state(true);
            update_journal(&unit_journal, timer.name.as_str());
            header.set_label(get_filename(timer.name.as_str()));
            match get_unit_description(&info) {
                Some(description) => header.set_label(description),
                None              => header.set_label(get_filename(timer.name.as_str()))
            }
        });
    }

    { // NOTE: Implement the {dis, en}able button
    let services        = services.clone();
    let services_list   = services_list.clone();
    let sockets         = sockets.clone();
    let sockets_list    = sockets_list.clone();
    let timers          = timers.clone();
    let timers_list     = timers_list.clone();
    let unit_stack      = unit_stack.clone();
    ablement_switch.connect_state_set(move |switch, enabled| {
        match unit_stack.get_visible_child_name().unwrap().as_str() {
            "Services" => {
                let index   = match services_list.get_selected_row() {
                    Some(row) => row.get_index(),
                    None      => 0
                };
                let service = &services[index as usize];
                let service_path = get_filename(service.name.as_str());
                if enabled && !dbus::get_unit_file_state(service.name.as_str()) {
                    dbus::enable_unit_files(service_path);
                    switch.set_state(true);
                } else if !enabled && dbus::get_unit_file_state(service.name.as_str()) {
                    dbus::disable_unit_files(service_path);
                    switch.set_state(false);
                }
            },
            "Sockets" => {
                let index   = match sockets_list.get_selected_row() {
                    Some(row) => row.get_index(),
                    None      => 0
                };
                let socket  = &sockets[index as usize];
                let socket_path = get_filename(socket.name.as_str());
                if enabled && !dbus::get_unit_file_state(socket.name.as_str()) {
                    dbus::enable_unit_files(socket_path);
                    switch.set_state(true);
                } else if !enabled && dbus::get_unit_file_state(socket.name.as_str()) {
                    dbus::disable_unit_files(socket_path);
                    switch.set_state(false);
                }
            },
            "Timers" => {
                let index   = match timers_list.get_selected_row() {
                    Some(row) => row.get_index(),
                    None      => 0
                };
                let timer  = &timers[index as usize];
                let timer_path = get_filename(timer.name.as_str());
                if enabled && !dbus::get_unit_file_state(timer.name.as_str()) {
                    dbus::enable_unit_files(timer_path);
                    switch.set_state(true);
                } else if !enabled && dbus::get_unit_file_state(timer.name.as_str()) {
                    dbus::disable_unit_files(timer_path);
                    switch.set_state(false);
                }
            },
            _ => unreachable!()
        }
        gtk::Inhibit(true)
    });
    }

    { // NOTE: Implement the start button
        let services       = services.clone();
        let services_list  = services_list.clone();
        let sockets        = sockets.clone();
        let sockets_list   = sockets_list.clone();
        let timers         = timers.clone();
        let timers_list    = timers_list.clone();
        let services_icons = services_icons.clone();
        let sockets_icons  = sockets_icons.clone();
        let timers_icons   = timers_icons.clone();
        let unit_stack    = unit_stack.clone();
        start_button.connect_clicked(move |_| {
            match unit_stack.get_visible_child_name().unwrap().as_str() {
                "Services" => {
                    let index   = match services_list.get_selected_row() {
                        Some(row) => row.get_index(),
                        None      => 0
                    };
                    let service = &services[index as usize];
                    if let None = dbus::start_unit(get_filename(service.name.as_str())) {
                        update_icon(&services_icons[index as usize], true);
                    }
                },
                "Sockets" => {
                    let index   = match sockets_list.get_selected_row() {
                        Some(row) => row.get_index(),
                        None      => 0
                    };
                    let socket  = &sockets[index as usize];
                    if let None = dbus::start_unit(get_filename(socket.name.as_str())) {
                        update_icon(&sockets_icons[index as usize], true);
                    }
                },
                "Timers" => {
                    let index   = match timers_list.get_selected_row() {
                        Some(row) => row.get_index(),
                        None      => 0
                    };
                    let timer  = &timers[index as usize];
                    if let None = dbus::start_unit(get_filename(timer.name.as_str())) {
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
        let services_icons = services_icons.clone();
        let sockets_icons  = sockets_icons.clone();
        let timers_icons   = timers_icons.clone();
        let unit_stack    = unit_stack.clone();
        stop_button.connect_clicked(move |_| {
            match unit_stack.get_visible_child_name().unwrap().as_str() {
                "Services" => {
                    let index   = match services_list.get_selected_row() {
                        Some(row) => row.get_index(),
                        None      => 0
                    };
                    let service = &services[index as usize];
                    if let None = dbus::stop_unit(get_filename(service.name.as_str())) {
                        update_icon(&services_icons[index as usize], false);
                    }
                },
                "Sockets" => {
                    let index   = match sockets_list.get_selected_row() {
                        Some(row) => row.get_index(),
                        None      => 0
                    };
                    let socket  = &sockets[index as usize];
                    if let None = dbus::stop_unit(get_filename(socket.name.as_str())) {
                        update_icon(&sockets_icons[index as usize], false);
                    }
                },
                "Timers" => {
                    let index   = match timers_list.get_selected_row() {
                        Some(row) => row.get_index(),
                        None      => 0
                    };
                    let timer   = &timers[index as usize];
                    if let None = dbus::stop_unit(get_filename(timer.name.as_str())) {
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
        let unit_stack    = unit_stack.clone();
        save_unit_file.connect_clicked(move |_| {
            let buffer = unit_info.get_buffer().unwrap();
            let start  = buffer.get_start_iter();
            let end    = buffer.get_end_iter();
            let text   = buffer.get_text(&start, &end, true).unwrap();
            let path = match unit_stack.get_visible_child_name().unwrap().as_str() {
                "Services" => &services[services_list.get_selected_row().unwrap().get_index() as usize].name,
                "Sockets" => &sockets[sockets_list.get_selected_row().unwrap().get_index() as usize].name,
                "Timers" => &timers[timers_list.get_selected_row().unwrap().get_index() as usize].name,
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

    { // NOTE: Journal Refresh Button
        let services       = services.clone();
        let services_list  = services_list.clone();
        let sockets        = sockets.clone();
        let sockets_list   = sockets_list.clone();
        let timers         = timers.clone();
        let timers_list    = timers_list.clone();
        let unit_stack     = unit_stack.clone();
        let refresh_button = refresh_log_button.clone();
        let unit_journal   = unit_journal.clone();
        refresh_button.connect_clicked(move |_| {
            match unit_stack.get_visible_child_name().unwrap().as_str() {
                "Services" => {
                    let index   = match services_list.get_selected_row() {
                        Some(row) => row.get_index(),
                        None      => 0
                    };
                    let service = &services[index as usize];
                    update_journal(&unit_journal, service.name.as_str());
                },
                "Sockets" => {
                    let index   = match sockets_list.get_selected_row() {
                        Some(row) => row.get_index(),
                        None      => 0
                    };
                    let socket = &sockets[index as usize];
                    update_journal(&unit_journal, socket.name.as_str());
                },
                "Timers" => {
                    let index   = match timers_list.get_selected_row() {
                        Some(row) => row.get_index(),
                        None      => 0
                    };
                    let timer = &timers[index as usize];
                    update_journal(&unit_journal, timer.name.as_str());
                },
                _ => unreachable!()
            }
        });
    }

    window.show_all();

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
