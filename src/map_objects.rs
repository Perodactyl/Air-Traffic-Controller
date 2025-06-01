use crate::{command::{Command, Setting}, direction::{CardinalDirection, OrdinalDirection}, location::{AirLocation, GroundLocation}, DisplayState};
use std::{fmt::Display, io::Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Airport {
    pub location: GroundLocation,
    pub launch_direction: CardinalDirection,
    pub index: u16,
} impl Into<RenderCell> for &Airport {
    fn into(self) -> RenderCell {
        RenderCell::Airport(self.index, self.launch_direction)
    }
} impl Into<GroundLocation> for &Airport {
    fn into(self) -> GroundLocation {
        self.location
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Beacon {
    pub location: GroundLocation,
    pub index: u16,
} impl Into<RenderCell> for &Beacon {
    fn into(self) -> RenderCell {
        RenderCell::Beacon(self.index)
    }
} impl Into<GroundLocation> for &Beacon {
    fn into(self) -> GroundLocation {
        GroundLocation::from(self.location)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Exit {
    pub index: u16,
    pub entry_location: AirLocation,
    pub entry_direction: OrdinalDirection,
    pub exit_location: AirLocation,
    pub exit_direction: OrdinalDirection,
} impl Into<RenderCell> for &Exit {
    fn into(self) -> RenderCell {
        RenderCell::Exit(self.index)
    }
} impl Into<GroundLocation> for &Exit {
    fn into(self) -> GroundLocation {
        GroundLocation::from(self.entry_location)
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum RenderCell {
    #[default]
    Blank,
    PathMark,
    Beacon(u16),
    Airport(u16, CardinalDirection),
    Exit(u16),
    Airplane(char, u16, DisplayState),
} impl Display for RenderCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderCell::Blank => write!(f, "\x1b[2m. \x1b[0m"),
            RenderCell::PathMark => write!(f, "+ "),
            RenderCell::Beacon(no) => write!(f, "\x1b[33m*{no}\x1b[0m"),
            RenderCell::Airport(no, dir) => write!(f, "\x1b[34m{dir}{no}\x1b[0m"),
            RenderCell::Exit(no) => write!(f, "\x1b[31m{no} \x1b[0m"),
            RenderCell::Airplane(callsign, fl, state) => write!(f, "{state}{callsign}{fl}\x1b[0m"),
        }
    }
}

pub struct RenderGrid {
    pub width: u16,
    pub height: u16,
    tiles: Vec<RenderCell>,
} impl RenderGrid {
    pub fn new(width: u16, height: u16) -> Self {
        RenderGrid {
            width, height,
            tiles: vec![RenderCell::Blank; (width*height) as usize],
        }
    }
    fn index_of(&self, x: u16, y: u16) -> usize {
        (y * self.width + x) as usize
    }
    fn set(&mut self, x: u16, y: u16, cell: RenderCell) {
        let i = self.index_of(x, y);
        self.tiles[i] = cell;
    }
    pub fn set_loc(&mut self, loc: GroundLocation, cell: RenderCell) {
        self.set(loc.0, loc.1, cell);
    }
    fn get(&self, x: u16, y: u16) -> RenderCell {
        self.tiles[self.index_of(x, y)]
    }
} impl RenderGrid {
    pub fn render(&self, output: &mut impl Write, command: &Command) -> std::io::Result<()> {
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.get(x, y);
                match (cell, command) {
                    (
                        RenderCell::Airplane(callsign, ..),
                        Command { plane: Some(target), .. }
                    ) if callsign.to_ascii_lowercase() == target.to_ascii_lowercase() => {
                        write!(output, "\x1b[1m")?;
                    },
                    (
                        RenderCell::Beacon(index),
                        Command { at: Setting::Set(target), .. }
                    ) if index == *target => {
                        write!(output, "\x1b[1m")?;
                    },
                    _ => {},
                }
                write!(output, "{}\x1b[0m", cell)?;
            }
            write!(output, "\n\x1b[G")?;
        }
        Ok(())
    }
}
