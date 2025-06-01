use std::fmt::Display;

use crate::{command::{Command, CompleteAction, CompleteCommand, CompleteRelOrAbsolute}, direction::{CardinalDirection, OrdinalDirection}, location::{AirLocation, Destination, GroundLocation, Location}, map_objects::{GridRenderable, ListRenderable, COMMAND_TARGET_EMPHASIS, COMMAND_TARGET_EMPHASIS_RESET}};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    Marked,
    Unmarked,
    Ignored,
} impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Marked => Ok(()),
            Visibility::Unmarked | Visibility::Ignored => write!(f, "\x1b[2m"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Plane {
    pub location: Location,
    pub destination: Destination,
    pub target_flight_level: u16,
    pub callsign: char,
    pub is_jet: bool,
    pub ticks_active: u32,
    pub target_direction: OrdinalDirection,
    pub current_direction: OrdinalDirection,
    pub show: Visibility,
    pub command: Option<CompleteCommand>,
} impl Plane {
    pub fn accept_cmd(&mut self, cmd: CompleteCommand, is_at_beacon: bool) {
        let CompleteCommand { action, at, .. } = cmd;
        if at.is_none() || is_at_beacon {
            match action {
                CompleteAction::Altitude(CompleteRelOrAbsolute::To(val)) => self.target_flight_level = val,
                CompleteAction::Altitude(CompleteRelOrAbsolute::Plus(val)) => self.target_flight_level += val,
                CompleteAction::Altitude(CompleteRelOrAbsolute::Minus(val)) => self.target_flight_level -= val,
                CompleteAction::Heading(targ) => {
                    self.target_direction = targ;
                    if let Some(CompleteCommand { action: CompleteAction::Circle(_), .. }) = self.command {
                        self.command = None;
                    }
                },
                CompleteAction::Circle(dir) => {
                    self.target_direction = self.current_direction.rotated_90(dir);
                    self.command = Some(CompleteCommand {
                      plane: self.callsign,
                        action: CompleteAction::Circle(dir),
                        at: None,
                    });
                },
                CompleteAction::SetVisiblity(v) => self.show = v,
            }
            if is_at_beacon {
                if let Some(c) = self.command {
                    match c {
                        CompleteCommand { action: CompleteAction::SetVisiblity(_), .. } => {},
                        CompleteCommand { action: CompleteAction::Circle(_), .. } => {},
                        _ => self.command = None,
                    };
                }
            }
        } else if at.is_some() {
            self.command = Some(cmd);
        }

    }
    pub fn tick(&mut self, is_at_beacon: bool) {
        if let Some(cmd) = self.command {
            self.accept_cmd(cmd, is_at_beacon);
        }
        match self.location {
            Location::Flight(loc) => {
                let AirLocation(mut x, mut y, mut flight_level) = loc;


                if self.is_jet || self.ticks_active % 2 == 0 {
                    match (self.target_flight_level).cmp(&flight_level) {
                        std::cmp::Ordering::Less => {
                            flight_level -= 1;
                        }
                        std::cmp::Ordering::Greater => {
                            flight_level += 1;
                        }
                        std::cmp::Ordering::Equal => {}
                    }
                    if self.target_direction != self.current_direction {
                        self.current_direction = self.current_direction.rotate_toward(self.target_direction);
                    }
                    let (offset_x, offset_y) = self.current_direction.as_offset();
                    x = (x as i16 + offset_x) as u16;
                    y = (y as i16 + offset_y) as u16;
                    self.location = Location::Flight(AirLocation(x, y, flight_level));
                }
            },
            Location::Airport(port) => {
                if self.target_flight_level > 0 {
                    let GroundLocation(x, y) = port.location + <CardinalDirection as Into<OrdinalDirection>>::into(port.launch_direction).as_offset();
                    self.location = Location::Flight(AirLocation(x, y, 1));
                }
            }
        }
        self.ticks_active += 1;
    }
    fn flight_level(&self) -> u16 {
        match self.location {
            Location::Airport(_) => 0,
            Location::Flight(AirLocation(_, _, fl)) => fl,
        }
    }
} impl GridRenderable for Plane {
    fn location(&self) -> Option<GroundLocation> {
        match self.location {
            Location::Airport(_) => None,
            Location::Flight(air_location) => Some(air_location.into()),
        }
    }
    fn render(&self, command: &Command) -> String {
        let emphasis = match command {
            Command { plane: Some(callsign), .. } if callsign.to_ascii_lowercase() == self.callsign.to_ascii_lowercase() => COMMAND_TARGET_EMPHASIS,
            _ => "",
        };
        let color = match self.show {
            Visibility::Marked => "\x1b[32m",
            _ => "\x1b[2m",
        };

        format!("{}{}{}{}\x1b[0m", emphasis, color, self.callsign, self.flight_level())
    }
} impl ListRenderable for Plane {
    fn render(&self, command: &Command) -> String {
        let colorize = self.show == Visibility::Marked;
        let emphasis = match command {
            Command { plane: Some(callsign), .. } if callsign.to_ascii_lowercase() == self.callsign.to_ascii_lowercase() => COMMAND_TARGET_EMPHASIS,
            _ => "",
        };
        let color = match self.show {
            Visibility::Marked => "\x1b[32m",
            _ => "\x1b[2m",
        };
        let airport = match self.location {
            Location::Flight(_) => format!("   "),
            Location::Airport(a) => format!("@{}", a.to_display_string(colorize)),
        };
        let command = match (self.show, self.command) {
            (Visibility::Ignored, _) => format!("---"),
            (_, Some(c)) => c.to_short_string(colorize),
            _ => String::new(),
        };
        format!("\x1b[0m{}{}{}{}{COMMAND_TARGET_EMPHASIS_RESET}\x1b[39m{} {}   {}", emphasis, color, self.callsign, self.flight_level(), airport, self.destination.to_display_string(colorize, true), command)
    }
}
