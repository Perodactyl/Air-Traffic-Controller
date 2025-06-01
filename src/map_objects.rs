use serde::Deserialize;

use crate::{command::{Command, Setting}, direction::{CardinalDirection, OrdinalDirection}, location::{AirLocation, GroundLocation}};

pub const COMMAND_TARGET_EMPHASIS: &str = "\x1b[4m";
pub const COMMAND_TARGET_EMPHASIS_RESET: &str = "\x1b[24m";

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Airport {
    pub location: GroundLocation,
    pub launch_direction: CardinalDirection,
    pub index: u16,
} impl Airport {
    pub fn to_display_string(&self, colorize: bool) -> String {
        format!("{}{}{}{}", if colorize { "\x1b[34m" } else { "" }, self.launch_direction, self.index, if colorize { "\x1b[39m" } else { "" })
    }
} impl GridRenderable for Airport {
    fn location(&self) -> Option<GroundLocation> {
        Some(self.location)
    }
    fn render(&self, _command: &Command) -> String {
        self.to_display_string(true)
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Beacon {
    pub index: u16,
    pub location: GroundLocation,
} impl Beacon {
    pub fn to_display_string(&self, colorize: bool) -> String {
        format!("{}*{}{}", if colorize { "\x1b[33m" } else { "" }, self.index, if colorize { "\x1b[39m" } else { "" })
    }
} impl GridRenderable for Beacon {
    fn location(&self) -> Option<GroundLocation> {
        Some(self.location)
    }
    fn render(&self, command: &Command) -> String {
        let emphasis = match command {
            Command { at: Setting::Set(index), .. } if *index == self.index => COMMAND_TARGET_EMPHASIS,
            _ => "",
        };
        format!("{}{}{COMMAND_TARGET_EMPHASIS_RESET}", emphasis, self.to_display_string(true))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Exit {
    pub index: u16,
    pub entry_location: AirLocation,
    pub entry_direction: OrdinalDirection,
    pub exit_location: AirLocation,
    pub exit_direction: OrdinalDirection,
} impl Exit {
    pub fn to_display_string(&self, colorize: bool, show_char: bool) -> String {
        match (colorize, show_char) {
            (false, false) => format!("{} ", self.index),
            (false, true)  => format!("E{}", self.index),
            (true, false)  => format!("\x1b[31m{} \x1b[0m", self.index),
            (true, true)   => format!("\x1b[31mE{}\x1b[0m", self.index),
        }
    }
} impl GridRenderable for Exit {
    fn location(&self) -> Option<GroundLocation> {
        Some(self.entry_location.into())
    }
    fn render(&self, _command: &Command) -> String {
        self.to_display_string(true, false)
    }
}

pub struct RenderGrid {
    pub width: u16,
    pub height: u16,
    command: Command,
    tiles: Vec<String>,
} impl RenderGrid {
    pub fn new(width: u16, height: u16, command: Command) -> Self {
        RenderGrid {
            width, height, command,
            tiles: vec!["\x1b[2m. \x1b[0m".to_string(); (width*height) as usize],
        }
    }
    pub fn add(&mut self, obj: &impl GridRenderable) {
        if let Some(GroundLocation(x, y)) = obj.location() {
            let result = obj.render(&self.command);
            let loc = self.index_of(x, y);
            self.tiles[loc] = result;
        }
    }
    fn index_of(&self, x: u16, y: u16) -> usize {
        eprintln!("{x} {y}");
        ((y as usize) * (self.width as usize)) + (x as usize)
    }
    fn get(&self, x: u16, y: u16) -> &str {
        &self.tiles[self.index_of(x, y)]
    }
} impl RenderGrid {
    pub fn render(&self) -> String {
        let mut out = String::with_capacity((self.width * self.height * 2) as usize);
        for y in 0..self.height {
            for x in 0..self.width {
                out.push_str(self.get(x, y));
            }
            out.push_str(&format!("\x1b[{}D\x1b[B", self.width * 2));
        }
        out
    }
}

pub trait GridRenderable {
    fn location(&self) -> Option<GroundLocation>;
    fn render(&self, command: &Command) -> String;
}

pub trait ListRenderable {
    fn render(&self, command: &Command) -> String;
}
