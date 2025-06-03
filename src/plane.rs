use std::fmt::Display;

use crate::{command::{Command, CommandTarget, CompleteAltitude, CompleteAnd, CompleteAt, CompleteCommandSegment, CompleteIn, CompleteTurn}, direction::{CardinalDirection, OrdinalDirection}, location::{AirLocation, Destination, GroundLocation, Location}, map::MapStatic, map_objects::{GridRenderable, ListItemPartRenderable, ListRenderable, COMMAND_TARGET_EMPHASIS, COMMAND_TARGET_EMPHASIS_RESET}};

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
    pub command: Option<CompleteCommandSegment>,
} impl Plane {
    pub fn tick(&mut self, map: &MapStatic) {
        if let Some(cmd) = &self.command {
            self.exec(cmd.clone(), map);
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
    pub fn exec(&mut self, mut command: CompleteCommandSegment, map: &MapStatic) -> bool {
        match command {
            CompleteCommandSegment::SetVisibility(v) => self.show = v.into(),
            CompleteCommandSegment::Altitude(CompleteAltitude::To(a)) => self.target_flight_level = a,
            CompleteCommandSegment::Altitude(CompleteAltitude::Plus(a)) => self.target_flight_level += a,
            CompleteCommandSegment::Altitude(CompleteAltitude::Minus(a)) => self.target_flight_level -= a,
            CompleteCommandSegment::Turn(CompleteTurn::ToHeading(h)) => {
                self.target_direction = h;
                if let Some(CompleteCommandSegment::Circle(_)) = self.command {
                    self.command = None;
                }
            },
            CompleteCommandSegment::Circle(dir) => {
                self.target_direction = self.current_direction.rotated_90(dir.into());
                self.command = Some(command);
            },
            CompleteCommandSegment::At(CompleteAt { ref tail, poi }) => {
                if poi.is_satisfied(self, map) {
                    self.command = None;
                    self.exec(*tail.clone(), map);
                } else {
                    self.command = Some(command);
                    return false;
                }
            },
            CompleteCommandSegment::And(CompleteAnd { ref left, ref right }) => {
                if self.exec(*left.clone(), map) {
                    self.exec(*right.clone(), map);
                } else {
                    self.command = Some(command);
                    return false;
                }
            },
            CompleteCommandSegment::In(CompleteIn { ref tail, ref mut time }) => {
                if *time > 0 {
                    *time -= 1;
                    self.command = Some(command);
                } else {
                    self.command = None;
                    self.exec(*tail.clone(), map);
                }
            },
            CompleteCommandSegment::None => {},
            CompleteCommandSegment::Ref(_) => unreachable!("map should have cast this to its inner value"),
        }
        return true;
    }
} impl GridRenderable for Plane {
    fn location(&self) -> Option<GroundLocation> {
        match self.location {
            Location::Airport(_) => None,
            Location::Flight(air_location) => Some(air_location.into()),
        }
    }
    fn render(&self, command: &Command) -> String {
        let emphasis = match command.target {
            CommandTarget::Plane(p) if p.to_ascii_lowercase() == self.callsign.to_ascii_lowercase() => format!("{COMMAND_TARGET_EMPHASIS}"),
            _ => String::new(),
        };
        let color = match self.show {
            Visibility::Marked => "\x1b[32m",
            _ => "\x1b[2m",
        };

        format!("{}{}{}{}{COMMAND_TARGET_EMPHASIS_RESET}\x1b[39m\x1b[22m", emphasis, color, self.callsign, self.flight_level())
    }
} impl ListRenderable for Plane {
    fn render(&self, command: &Command) -> String {
        let colorize = self.show == Visibility::Marked;
        let emphasis = match command.target {
            CommandTarget::Plane(p) if p.to_ascii_lowercase() == self.callsign.to_ascii_lowercase() => format!("{COMMAND_TARGET_EMPHASIS}"),
            _ => String::new(),
        };
        let color = match self.show {
            Visibility::Marked => "\x1b[32m",
            _ => "\x1b[2m",
        };
        let airport = match self.location {
            Location::Flight(_) => format!("   "),
            Location::Airport(a) => format!("@{}", a.to_display_string(colorize)),
        };
        let command = match (self.show, &self.command) {
            (Visibility::Ignored, _) => format!("---"),
            (Visibility::Unmarked, Some(c)) => c.render(false),
            (Visibility::Marked, Some(c)) => c.render(true),
            _ => String::new(),
        };
        format!("\x1b[0m{}{}{}{}{COMMAND_TARGET_EMPHASIS_RESET}\x1b[39m{} {}   {}", emphasis, color, self.callsign, self.flight_level(), airport, self.destination.to_display_string(colorize, true), command)
    }
}
