extern crate dbus;
use std::path::Path;

/// Takes a systemd dbus function as input and returns the result as a `dbus::Message`.
macro_rules! dbus_message {
    ($function:expr) => {{
        let dest      = "org.freedesktop.systemd1";
        let node      = "/org/freedesktop/systemd1";
        let interface = "org.freedesktop.systemd1.Manager";
        dbus::Message::new_method_call(dest, node, interface, $function).
            unwrap_or_else(|e| panic!("{}", e))
    }}
}

/// Takes a `dbus::Message` as input and makes a connection to dbus, returning the reply.
macro_rules! dbus_connect {
    ($message:expr) => {
        dbus::Connection::get_private(dbus::BusType::System).unwrap().
            send_with_reply_and_block($message, 4000)
    }
}

#[derive(Clone)]
pub struct SystemdUnit {
    pub name: String,
    pub state: UnitState,
    pub utype: UnitType,
}

#[derive(Clone, PartialEq, Eq)]
pub enum UnitType { Automount, Busname, Mount, Path, Scope, Service, Slice, Socket, Target, Timer }
impl UnitType {
    /// Takes the pathname of the unit as input to determine what type of unit it is.
    pub fn new(pathname: &str) -> UnitType {
        match Path::new(pathname).extension().unwrap().to_str().unwrap() {
            "automount" => UnitType::Automount,
            "busname" => UnitType::Busname,
            "mount" => UnitType::Mount,
            "path" => UnitType::Path,
            "scope" => UnitType::Scope,
            "service" => UnitType::Service,
            "slice" => UnitType::Slice,
            "socket" => UnitType::Socket,
            "target" => UnitType::Target,
            "timer" => UnitType::Timer,
            _ => panic!("Unknown Type: {}", pathname),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum UnitState { Bad, Disabled, Enabled, Indirect, Linked, Masked, Static }
impl UnitState {
    /// Takes the string containing the state information from the dbus message and converts it
    /// into a UnitType by matching the first character.
    pub fn new(x: &str) -> UnitState {
        let x_as_chars: Vec<char> = x.chars().skip(6).take_while(|x| *x != '\"').collect();
        match x_as_chars[0] {
            's' => UnitState::Static,
            'd' => UnitState::Disabled,
            'e' => UnitState::Enabled,
            'i' => UnitState::Indirect,
            'l' => UnitState::Linked,
            'm' => UnitState::Masked,
            'b' => UnitState::Bad,
            _ => panic!("Unknown State: {}", x),
        }
    }
}

/// Communicates with dbus to obtain a list of unit files and returns them as a `Vec<SystemdUnit>`.
pub fn list_unit_files() -> Vec<SystemdUnit> {
    /// Takes the dbus message as input and maps the information to a `Vec<SystemdUnit>`.
    fn parse_message(input: &str) -> Vec<SystemdUnit> {
        let message = {
            let mut output: String = input.chars().skip(7).collect();
            let len = output.len()-10;
            output.truncate(len);
            output
        };

        // This custom loop iterates across two variables at a time. The first variable contains the
        // pathname of the unit, while the second variable contains the state of that unit.
        let mut systemd_units: Vec<SystemdUnit> = Vec::new();
        let mut iterator = message.split(',');
        while let Some(name) = iterator.next() {
            let name: String = name.chars().skip(14).take_while(|x| *x != '\"').collect();
            let utype = UnitType::new(&name);
            let state = UnitState::new(iterator.next().unwrap());
            systemd_units.push(SystemdUnit{name: name, state: state, utype: utype});
        }

        systemd_units.sort_by(|a, b| a.name.cmp(&b.name));
        systemd_units
    }

    let message = dbus_connect!(dbus_message!("ListUnitFiles")).unwrap().get_items();
    parse_message(&format!("{:?}", message))
}

/// Returns the current enablement status of the unit
pub fn get_unit_file_state(path: &str) -> bool {
    for unit in list_unit_files() {
        if unit.name.as_str() == path { return unit.state == UnitState::Enabled; }
    }
    false
}

/// Takes a `Vec<SystemdUnit>` as input and returns a new vector only containing services which can be enabled and
/// disabled.
pub fn collect_togglable_services(units: &[SystemdUnit]) -> Vec<SystemdUnit> {
    units.iter().filter(|x| x.utype == UnitType::Service && (x.state == UnitState::Enabled ||
        x.state == UnitState::Disabled) && !x.name.contains("/etc/")).cloned().collect()
}

/// Takes a `Vec<SystemdUnit>` as input and returns a new vector only containing sockets which can be enabled and
/// disabled.
pub fn collect_togglable_sockets(units: &[SystemdUnit]) -> Vec<SystemdUnit> {
    units.iter().filter(|x| x.utype == UnitType::Socket && (x.state == UnitState::Enabled ||
        x.state == UnitState::Disabled)).cloned().collect()
}

/// Takes a `Vec<SystemdUnit>` as input and returns a new vector only containing timers which can be enabled and
/// disabled.
pub fn collect_togglable_timers(units: &[SystemdUnit]) -> Vec<SystemdUnit> {
    units.iter().filter(|x| x.utype == UnitType::Timer && (x.state == UnitState::Enabled ||
        x.state == UnitState::Disabled)).cloned().collect()
}

/// Takes the unit pathname of a service and enables it via dbus.
/// If dbus replies with `[Bool(true), Array([], "(sss)")]`, the service is already enabled.
pub fn enable_unit_files(unit: &str) -> Option<String> {
    let mut message = dbus_message!("EnableUnitFiles");
    message.append_items(&[[unit][..].into(), false.into(), true.into()]);
    match dbus_connect!(message) {
        Ok(reply) => {
            if format!("{:?}", reply.get_items()) == "[Bool(true), Array([], \"(sss)\")]" {
                println!("{} already enabled", unit);
            } else {
                println!("{} has been enabled", unit);
            }
            None
        },
        Err(reply) => {
            let error = format!("Error enabling {}:\n{:?}", unit, reply);
            println!("{}", error);
            Some(error)
        }
    }
}

/// Takes the unit pathname as input and disables it via dbus.
/// If dbus replies with `[Array([], "(sss)")]`, the service is already disabled.
pub fn disable_unit_files(unit: &str) -> Option<String> {
    let mut message = dbus_message!("DisableUnitFiles");
    message.append_items(&[[unit][..].into(), false.into()]);
    match dbus_connect!(message) {
        Ok(reply) => {
            if format!("{:?}", reply.get_items()) == "[Array([], \"(sss)\")]" {
                println!("{} is already disabled", unit);
            } else {
                println!("{} has been disabled", unit);
            }
            None
        },
        Err(reply) => {
            let error = format!("Error disabling {}:\n{:?}", unit, reply);
            println!("{}", error);
            Some(error)
        }
    }
}

/// Takes a unit name as input and attempts to start it
pub fn start_unit(unit: &str) -> Option<String> {
    let mut message = dbus_message!("StartUnit");
    message.append_items(&[unit.into(), "fail".into()]);
    match dbus_connect!(message) {
        Ok(_) => {
            println!("{} successfully started", unit);
            None
        },
        Err(error) => {
            let output = format!("{} failed to start:\n{:?}", unit, error);
            println!("{}", output);
            Some(output)
        }

    }
}

/// Takes a unit name as input and attempts to stop it.
pub fn stop_unit(unit: &str) -> Option<String> {
    let mut message = dbus_message!("StopUnit");
    message.append_items(&[unit.into(), "fail".into()]);
    match dbus_connect!(message) {
        Ok(_) => {
            println!("{} successfully stopped", unit);
            None
        },
        Err(error) => {
            let output = format!("{} failed to stop:\n{:?}", unit, error);
            println!("{}", output);
            Some(output)
        }
    }
}
