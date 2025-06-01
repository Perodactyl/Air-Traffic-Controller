use serde::Deserialize;

use crate::{direction::OrdinalDirection, map_objects::{Airport, Exit, GridRenderable}};
use std::{fmt::Display, ops::Add};

///Also used to represent a path marker.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct GroundLocation(pub u16, pub u16);
impl From<AirLocation> for GroundLocation {
    fn from(value: AirLocation) -> Self {
        GroundLocation(value.0, value.1)
    }
} impl Add<(i16, i16)> for GroundLocation {
    type Output = GroundLocation;
    fn add(self, rhs: (i16, i16)) -> Self::Output {
        GroundLocation(
            ((self.0 as i16) + rhs.0) as u16,
            ((self.1 as i16) + rhs.1) as u16
        )
    }
} impl GridRenderable for GroundLocation {
    fn location(&self) -> Option<GroundLocation> {
        Some(*self)
    }
    fn render(&self, _command: &crate::command::Command) -> String {
        "+ ".to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct AirLocation(pub u16, pub u16, pub u16);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Location {
    Airport(Airport),
    Flight(AirLocation),
} impl Into<GroundLocation> for Location {
    fn into(self) -> GroundLocation {
        match self {
            Location::Airport(a) => a.location,
            Location::Flight(al) => al.into(),
        }
    }
}

///Also represents a start location
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Destination {
    Airport(Airport),
    Exit(Exit),
} impl Destination {
    pub fn entry(&self) -> Location {
        match self {
            Destination::Airport(a) => Location::Airport(*a),
            Destination::Exit(Exit { entry_location, .. }) => Location::Flight(*entry_location),
        }
    }
    #[allow(dead_code)]
    pub fn exit(&self) -> Location {
        match self {
            Destination::Airport(a) => Location::Airport(*a),
            Destination::Exit(Exit { exit_location, .. }) => Location::Flight(*exit_location),
        }
    }
    pub fn entry_dir(&self) -> OrdinalDirection {
        match self {
            Destination::Airport(Airport { launch_direction, .. }) => (*launch_direction).into(),
            Destination::Exit(Exit { entry_direction, .. }) => *entry_direction,
        }
    }
    #[allow(dead_code)]
    pub fn exit_dir(&self) -> OrdinalDirection {
        match self {
            Destination::Airport(Airport { launch_direction, .. }) => (*launch_direction).into(),
            Destination::Exit(Exit { exit_direction, .. }) => *exit_direction,
        }
    }
    pub fn entry_height(&self) -> u16 {
        match self {
            Destination::Airport(_) => 0,
            Destination::Exit(Exit { entry_location: AirLocation(_, _, height), .. }) => *height,
        }
    }
    #[allow(dead_code)]
    pub fn exit_height(&self) -> u16 {
        match self {
            Destination::Airport(_) => 0,
            Destination::Exit(Exit { exit_location: AirLocation(_, _, height), .. }) => *height,
        }
    }
} impl Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Destination::Airport(Airport { index: no, .. }) => write!(f, "A{no}"),
            Destination::Exit(Exit { index: no, .. }) => write!(f, "E{no}"),
        }
    }
}
