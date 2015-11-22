extern crate dbus;
use std::path::Path;

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
pub enum UnitState { Disabled, Enabled, Masked, Static, Indirect }
impl UnitState {
	// Takes the string containing the state information from the dbus message and converts it
	// into a UnitType by matching the first character.
	pub fn new(x: &str) -> UnitState {
		let x_as_chars: Vec<char> = x.chars().skip(6).take_while(|x| *x != '\"').collect();
		match x_as_chars[0] {
			's' => UnitState::Static,
			'd' => UnitState::Disabled,
			'e' => UnitState::Enabled,
			'm' => UnitState::Masked,
			'i' => UnitState::Indirect,
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

	let message = dbus::Message::new_method_call("org.freedesktop.systemd1",
		"/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "ListUnitFiles").
		unwrap_or_else(|e| panic!("{}", e));
	let connection = dbus::Connection::get_private(dbus::BusType::System).unwrap();
	let reply = connection.send_with_reply_and_block(message, 4000).unwrap().get_items();

	parse_message(&format!("{:?}", reply))
}

// collect_togglable_services takes a Vec<SystemdUnit> as input and returns a new vector only
// containing services which can be enabled and disabled.
pub fn collect_togglable_services(units: Vec<SystemdUnit>) -> Vec<SystemdUnit> {
	units.into_iter().filter(|x| x.utype == UnitType::Service && x.state != UnitState::Static &&
		x.state != UnitState::Masked && x.state != UnitState::Indirect &&
		!x.name.contains("/etc/")).collect()
}

// collect_togglable_sockets takes a Vec<SystemdUnit> as input and returns a new vector only
// containing sockets which can be enabled and disabled.
pub fn collect_togglable_sockets(units: Vec<SystemdUnit>) -> Vec<SystemdUnit> {
	units.into_iter().filter(|x| x.utype == UnitType::Socket && x.state != UnitState::Static &&
		x.state != UnitState::Masked && x.state != UnitState::Indirect).collect()
}

// collect_togglable_timers takes a Vec<SystemdUnit> as input and returns a new vector only
// containing timers which can be enabled and disabled.
pub fn collect_togglable_timers(units: Vec<SystemdUnit>) -> Vec<SystemdUnit> {
	units.into_iter().filter(|x| x.utype == UnitType::Timer && x.state != UnitState::Static &&
		x.state != UnitState::Masked && x.state != UnitState::Indirect).collect()
}

// enable takes the pathname of a service and enables it via dbus.
// If dbus replies with `[Bool(true), Array([], "(sss)")]`, the service is already enabled.
pub fn enable(service_path: &str) -> bool {
	let mut message = dbus::Message::new_method_call("org.freedesktop.systemd1",
		"/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "EnableUnitFiles").
		unwrap_or_else(|e| panic!("{}", e));
	message.append_items(&[[service_path][..].into(), false.into(), true.into()]);
	let connection = dbus::Connection::get_private(dbus::BusType::System).unwrap();

	match connection.send_with_reply_and_block(message, 4000) {
		Ok(reply) => {
			let message = format!("{:?}", reply.get_items());
			if message == "[Bool(true), Array([], \"(sss)\")]" {
				println!("Service already enabled: {}", service_path);
			} else {
				println!("Service has been enabled: {}", service_path);
			}
			return true;
		},
		Err(reply) => {
			println!("Error enabling service: {:?}", reply);
			return false;
		},
	}
}

// disable takes a pathname as input and disables it via dbus.
// If dbus replies with `[Array([], "(sss)")]`, the service is already disabled.
pub fn disable(service_path: &str) -> bool {
	let mut message = dbus::Message::new_method_call("org.freedesktop.systemd1",
		"/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "DisableUnitFiles").
		unwrap_or_else(|e| panic!("{}", e));
	message.append_items(&[[service_path][..].into(), false.into()]);
	let connection = dbus::Connection::get_private(dbus::BusType::System).unwrap();

	match connection.send_with_reply_and_block(message, 4000) {
		Ok(reply) => {
			let message = format!("{:?}", reply.get_items());
			if message == "[Array([], \"(sss)\")]" {
				println!("Service is already disabled: {}", service_path);
			} else {
				println!("Service has been disabled: {}", service_path);
			}
			return true;
		},
		Err(reply) => {
			println!("Error disabling service: {:?}", reply);
			return false
		},
	}
}
