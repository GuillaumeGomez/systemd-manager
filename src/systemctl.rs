use std::process::Command; // Allow executing commands directly

pub enum UnitType {
	Service,
	Socket,
}

pub struct SystemdUnit {
	pub name:   String,
	pub status: bool,
	pub unit_type: UnitType,
}

pub fn get_unit_files() -> Vec<SystemdUnit> {
	let cmd = Command::new("systemctl").arg("list-unit-files").
		output().unwrap_or_else(|x| { panic!("Failed to execute process: {}", x) });

	String::from_utf8_lossy(&cmd.stdout).into_owned().lines().skip(1).
		filter(|x| !x.contains("target") && !x.contains("path") && !x.contains("static")).
		take_while(|x| x.len() != 0).map(|x| SystemdUnit{
			name: x.chars().take_while(|x| *x != ' ').collect(),
			status: x.contains("enabled"),
			unit_type: if x.contains(".socket") { UnitType::Socket } else { UnitType::Service },
		}).collect()
}

pub fn run(operation: &str, service: &str) -> bool {
	println!("systemctl {} {}", operation, service);
	let output = Command::new("sh").arg("-c").arg(format!("systemctl {} {}", operation, service)).
		output().unwrap_or_else(|_| panic!("Failed to execute systemctl."));
	println!("{}", String::from_utf8_lossy(&output.stdout));
	println!("{}", String::from_utf8_lossy(&output.stderr));
	output.status.success() // return exit status: {true, false}
}
