extern crate dbus;
use std::{path::Path, sync::Mutex};

/// Whether to print debug messages in DbusHandle::send.
const SEND_DEBUG: bool = false;

/// Takes a systemd dbus function as input and returns the result as a `dbus::Message`.
macro_rules! dbus_message {
    ($function:expr) => {{
        let dest = "org.freedesktop.systemd1";
        let node = "/org/freedesktop/systemd1";
        let interface = "org.freedesktop.systemd1.Manager";
        dbus::Message::new_method_call(dest, node, interface, $function).unwrap()
    }};
}

#[derive(Clone)]
pub struct SystemdUnit {
    pub name: String,
    pub state: UnitState,
    pub utype: UnitType,
}

#[derive(Clone, PartialEq, Eq)]
pub enum UnitType {
    Automount,
    Busname,
    Mount,
    Path,
    Scope,
    Service,
    Slice,
    Socket,
    Target,
    Timer,
    Swap,
}
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
            "swap" => UnitType::Swap,
            _ => panic!("Unknown Type: {}", pathname),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum UnitState {
    Bad,
    Disabled,
    Enabled,
    Indirect,
    Linked,
    Masked,
    Static,
    Generated,
    Alias,
    Transient,
}
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
            'g' => UnitState::Generated,
            'a' => UnitState::Alias,
            't' => UnitState::Transient,
            _ => panic!("Unknown State: {}", x),
        }
    }
}

#[derive(Debug)]
pub struct DbusHandle {
    bus_type: dbus::BusType,
    connection: Mutex<Option<dbus::Connection>>,
}
impl DbusHandle {
    pub fn new(bus_type: dbus::BusType) -> Self {
        Self {
            bus_type,
            connection: None.into(),
        }
    }

    /// Obtain a reference to the dbus::Connection, establishing it if necessary.
    pub fn con(&self) -> std::sync::MutexGuard<'_, Option<dbus::Connection>> {
        let mut conn = self.connection.lock().unwrap();
        if conn.is_none() {
            *conn = dbus::Connection::get_private(self.bus_type)
                .expect("Failed to establish dbus connection")
                .into();
        }
        // TODO: When MappedMutexGuard gets stabilized, can unwrap the option here.
        conn
    }
    /// Sends a dbus message and waits for a reply.
    pub fn send(&self, message: dbus::Message) -> Result<dbus::Message, dbus::Error> {
        if SEND_DEBUG {
            println!(
                "Sending message {:?} from thread {:?}",
                message,
                std::thread::current().id()
            );
        }
        self.con()
            .as_ref()
            .unwrap()
            .send_with_reply_and_block(message, 4000)
    }
    /// Sends a function call message and waits for a reply.
    pub fn call(&self, function_name: &str) -> Result<dbus::Message, dbus::Error> {
        self.send(dbus_message!(function_name))
    }
    /// Communicates with dbus to obtain a list of unit files and returns them as a `Vec<SystemdUnit>`.
    pub fn list_unit_files(&self) -> Vec<SystemdUnit> {
        let message = self.call("ListUnitFiles").unwrap().get_items();
        parse_units_from_message(&format!("{:?}", message))
    }

    /// Returns the current enablement status of the unit. Should be called with a unit name, not a path.
    pub fn get_unit_file_state(&self, name: &str) -> bool {
        //GetUnitFileState(in  s file, out s state);
        let mut msg = dbus_message!("GetUnitFileState");
        let unitname = if name.contains('/') {
            let stripped = name.split("/").last().unwrap();
            println!("Warning: instead of a name, a path {name:?} was passed to get_unit_file_state. Stripping it to {stripped:?}");
            stripped
        } else {
            name
        };
        msg.append_items(&[unitname.into()]);
        let reply = self
            .send(msg)
            .unwrap_or_else(|e| panic!("Failure getting the state of unit {}: {:?}", unitname, e));
        let status: String = reply.get1().unwrap();
        status.to_lowercase() == "enabled"
    }

    /// Takes the unit pathname of a service and enables it via dbus.
    /// If dbus replies with `[Bool(true), Array([], "(sss)")]`, the service is already enabled.
    pub fn enable_unit_files(&self, unit: &str) -> Option<String> {
        let mut message = dbus_message!("EnableUnitFiles");
        message.append_items(&[[unit][..].into(), false.into(), true.into()]);
        match self.send(message) {
            Ok(reply) => {
                if format!("{:?}", reply.get_items()) == "[Bool(true), Array([], \"(sss)\")]" {
                    println!("{} already enabled", unit);
                } else {
                    println!("{} has been enabled", unit);
                }
                None
            }
            Err(reply) => {
                let error = format!("Error enabling {}:\n{:?}", unit, reply);
                println!("{}", error);
                Some(error)
            }
        }
    }

    /// Takes the unit pathname as input and disables it via dbus.
    /// If dbus replies with `[Array([], "(sss)")]`, the service is already disabled.
    pub fn disable_unit_files(&self, unit: &str) -> Option<String> {
        let mut message = dbus_message!("DisableUnitFiles");
        message.append_items(&[[unit][..].into(), false.into()]);
        match self.send(message) {
            Ok(reply) => {
                if format!("{:?}", reply.get_items()) == "[Array([], \"(sss)\")]" {
                    println!("{} is already disabled", unit);
                } else {
                    println!("{} has been disabled", unit);
                }
                None
            }
            Err(reply) => {
                let error = format!("Error disabling {}:\n{:?}", unit, reply);
                println!("{}", error);
                Some(error)
            }
        }
    }

    /// Takes a unit name as input and attempts to start it
    pub fn start_unit(&self, unit: &str) -> Option<String> {
        let mut message = dbus_message!("StartUnit");
        message.append_items(&[unit.into(), "fail".into()]);
        match self.send(message) {
            Ok(_) => {
                println!("{} successfully started", unit);
                None
            }
            Err(error) => {
                let output = format!("{} failed to start:\n{:?}", unit, error);
                println!("{}", output);
                Some(output)
            }
        }
    }

    /// Takes a unit name as input and attempts to stop it.
    pub fn stop_unit(&self, unit: &str) -> Option<String> {
        let mut message = dbus_message!("StopUnit");
        message.append_items(&[unit.into(), "fail".into()]);
        match self.send(message) {
            Ok(_) => {
                println!("{} successfully stopped", unit);
                None
            }
            Err(error) => {
                let output = format!("{} failed to stop:\n{:?}", unit, error);
                println!("{}", output);
                Some(output)
            }
        }
    }
}

/// Takes the dbus message as input and maps the information to a `Vec<SystemdUnit>`.
fn parse_units_from_message(input: &str) -> Vec<SystemdUnit> {
    let message = {
        let mut output: String = input.chars().skip(7).collect();
        let len = output.len() - 10;
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
        systemd_units.push(SystemdUnit { name, state, utype });
    }

    systemd_units.sort_by(|a, b| a.name.cmp(&b.name));
    systemd_units
}

/// Takes a `Vec<SystemdUnit>` as input and returns a new vector only containing services which can be enabled and
/// disabled.
pub fn collect_togglable_services(units: &[SystemdUnit]) -> Vec<SystemdUnit> {
    units
        .iter()
        .filter(|x| {
            x.utype == UnitType::Service
                && (x.state == UnitState::Enabled || x.state == UnitState::Disabled)
                && !x.name.contains("/etc/")
        })
        .cloned()
        .collect()
}

/// Takes a `Vec<SystemdUnit>` as input and returns a new vector only containing sockets which can be enabled and
/// disabled.
pub fn collect_togglable_sockets(units: &[SystemdUnit]) -> Vec<SystemdUnit> {
    units
        .iter()
        .filter(|x| {
            x.utype == UnitType::Socket
                && (x.state == UnitState::Enabled || x.state == UnitState::Disabled)
        })
        .cloned()
        .collect()
}

/// Takes a `Vec<SystemdUnit>` as input and returns a new vector only containing timers which can be enabled and
/// disabled.
pub fn collect_togglable_timers(units: &[SystemdUnit]) -> Vec<SystemdUnit> {
    units
        .iter()
        .filter(|x| {
            x.utype == UnitType::Timer
                && (x.state == UnitState::Enabled || x.state == UnitState::Disabled)
        })
        .cloned()
        .collect()
}
