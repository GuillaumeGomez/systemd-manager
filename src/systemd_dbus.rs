extern crate dbus;
use std::path::Path;

// dbus_message takes a systemd dbus function as input and returns the result as a dbus::Message.
macro_rules! dbus_message {
    ($function:expr) => {{
        dbus::Message::new_method_call("org.freedesktop.systemd1",
            "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", $function).
            unwrap_or_else(|e| panic!("{}", e))
    }}
}

// dbus_connect takes a dbus::Message as input and makes a connection to dbus, returning the reply
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
    // Takes the pathname of the unit as input to determine what type of unit it is.
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
    // Takes the string containing the state information from the dbus message and converts it
    // into a UnitType by matching the first character.
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

// list_unit_files() communicates with dbus to obtain a list of unit files and returns them as a
// vector of SystemdUnits.
pub fn list_unit_files() -> Vec<SystemdUnit> {
    // parse_message takes the dbus message as input and maps the information to a Vec<SystemdUnit>
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
        let mut iterator = message.split(','); loop {
            let name: String = match iterator.next() {
                Some(x) => x.chars().skip(14).take_while(|x| *x != '\"').collect(),
                None => break,
            };
            let utype = UnitType::new(&name);
            let state = UnitState::new(iterator.next().unwrap());
            systemd_units.push(SystemdUnit{name: name, state: state, utype: utype});
        }

        systemd_units.sort_by(|a, b| a.name.cmp(&b.name)); // Sort in ascending order
        return systemd_units;
    }

    let message = dbus_connect!(dbus_message!("ListUnitFiles")).unwrap().get_items();
    parse_message(&format!("{:?}", message))
}

// collect_togglable_services takes a Vec<SystemdUnit> as input and returns a new vector only
// containing services which can be enabled and disabled.
pub fn collect_togglable_services(units: Vec<SystemdUnit>) -> Vec<SystemdUnit> {
    units.into_iter().filter(|x| x.utype == UnitType::Service && (x.state == UnitState::Enabled ||
        x.state == UnitState::Disabled) && !x.name.contains("/etc/")).collect()
}

// collect_togglable_sockets takes a Vec<SystemdUnit> as input and returns a new vector only
// containing sockets which can be enabled and disabled.
pub fn collect_togglable_sockets(units: Vec<SystemdUnit>) -> Vec<SystemdUnit> {
    units.into_iter().filter(|x| x.utype == UnitType::Socket && (x.state == UnitState::Enabled ||
        x.state == UnitState::Disabled)).collect()
}

// collect_togglable_timers takes a Vec<SystemdUnit> as input and returns a new vector only
// containing timers which can be enabled and disabled.
pub fn collect_togglable_timers(units: Vec<SystemdUnit>) -> Vec<SystemdUnit> {
    units.into_iter().filter(|x| x.utype == UnitType::Timer && (x.state == UnitState::Enabled ||
        x.state == UnitState::Disabled)).collect()
}

// enable takes the unit pathname of a service and enables it via dbus.
// If dbus replies with `[Bool(true), Array([], "(sss)")]`, the service is already enabled.
pub fn enable(unit: &str) -> bool {
    let mut message = dbus_message!("EnableUnitFiles");
    message.append_items(&[[unit][..].into(), false.into(), true.into()]);
    match dbus_connect!(message) {
        Ok(reply) => {
            if format!("{:?}", reply.get_items()) == "[Bool(true), Array([], \"(sss)\")]" {
                println!("{} already enabled", unit);
            } else {
                println!("{} has been enabled", unit);
            }
            return true;
        },
        Err(reply) => {
            println!("Error enabling service: {:?}", reply);
            return false;
        },
    }
}

// disable takes the unit pathname as input and disables it via dbus.
// If dbus replies with `[Array([], "(sss)")]`, the service is already disabled.
pub fn disable(unit: &str) -> bool {
    let mut message = dbus_message!("DisableUnitFiles");
    message.append_items(&[[unit][..].into(), false.into()]);
    match dbus_connect!(message) {
        Ok(reply) => {
            if format!("{:?}", reply.get_items()) == "[Array([], \"(sss)\")]" {
                println!("{} is already disabled", unit);
            } else {
                println!("{} has been disabled", unit);
            }
            return true;
        },
        Err(reply) => {
            println!("Error disabling service: {:?}", reply);
            return false
        },
    }
}

// start takes a unit name as input and attempts to start it
pub fn start(unit: &str) {
    let mut message = dbus_message!("StartUnit");
    message.append_items(&[unit.into(), "fail".into()]);
    match dbus_connect!(message) {
        Ok(_) => println!("{} successfully started", unit),
        Err(_) => println!("{} failed to start", unit),
    }
}

// stop takes a unit name as input and attempts to stop it.
pub fn stop(unit: &str) {
    let mut message = dbus_message!("StopUnit");
    message.append_items(&[unit.into(), "fail".into()]);
    match dbus_connect!(message) {
        Ok(_) => println!("{} was successfully stopped", unit),
        Err(_) => println!("{} failed to stop", unit),
    }
}
