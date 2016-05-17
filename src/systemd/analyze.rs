use std::process::Command;

#[derive(Clone, Debug, PartialEq)]
pub struct Analyze {
    pub time: u32,
    pub service: String,
}

impl Analyze {
    /// Returns the results of `systemd-analyze blame`
    pub fn blame() -> Vec<Analyze> {
        String::from_utf8(Command::new("systemd-analyze").arg("blame").output().unwrap().stdout).unwrap()
            .lines().rev().map(|x| parse_analyze(x)).collect::<Vec<Analyze>>()
    }
}

fn parse_analyze(x: &str) -> Analyze {
    let mut values: Vec<&str> = x.trim().split_whitespace().collect();
    let service = values.pop().unwrap();
    let time = values.iter().fold(0u32, |acc, x| acc + parse_time(x));
    Analyze {
        time: time,
        service: String::from(service)
    }
}

fn parse_time(input: &str) -> u32 {
    if input.ends_with("ms") {
        input[0..input.len()-2].parse::<u32>().unwrap_or(0)
    } else if input.ends_with('s') {
        (input[0..input.len()-1].parse::<f32>().unwrap_or(0f32) * 1000f32) as u32
    } else if input.ends_with("min") {
        input[0..input.len()-3].parse::<u32>().unwrap_or(0) * 60000u32
    } else {
        0u32
    }
}

#[test]
fn test_analyze_minutes() {
    let correct = Analyze{time: 218514, service: String::from("updatedb.service")};
    assert_eq!(correct, parse_analyze("3min 38.514s updatedb.service"));
}

#[test]
fn test_analyze_seconds() {
    let correct = Analyze{time: 15443, service: String::from("openntpd.service")};
    assert_eq!(correct, parse_analyze("15.443s openntpd.service"));
}

#[test]
fn test_analyze_milliseconds() {
    let correct = Analyze{time: 1989, service: String::from("systemd-sysctl.service")};
    assert_eq!(correct, parse_analyze("1989ms systemd-sysctl.service"));
}
