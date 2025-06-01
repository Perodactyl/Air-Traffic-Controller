use std::{fmt::Display, io::{self, IsTerminal, Read, Write}, time::{Duration, Instant}};
use rand::{prelude::*, random, random_range, rng};

use anyhow::Result;
use termion::{raw::IntoRawMode, screen::IntoAlternateScreen};

mod direction;
use direction::{CardinalDirection, OrdinalDirection};

mod location;
use location::{Location, Destination, GroundLocation, AirLocation};

mod map_objects;
use map_objects::{Airport, Beacon, Exit, GridRenderable, ListRenderable, RenderGrid, COMMAND_TARGET_EMPHASIS, COMMAND_TARGET_EMPHASIS_RESET};

mod command;
use command::{Command, CompleteAction, CompleteCommand, CompleteRelOrAbsolute};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayState {
    Marked,
    Unmarked,
    Ignored,
} impl Display for DisplayState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayState::Marked => Ok(()),
            DisplayState::Unmarked | DisplayState::Ignored => write!(f, "\x1b[2m"),
        }
    }
}

#[derive(Debug, Clone)]
struct Plane {
    location: Location,
    destination: Destination,
    target_flight_level: u16,
    callsign: char,
    is_jet: bool,
    ticks_active: u32,
    target_direction: OrdinalDirection,
    current_direction: OrdinalDirection,
    show: DisplayState,
    command: Option<CompleteCommand>,
} impl Plane {
    fn accept_cmd(&mut self, cmd: CompleteCommand, is_at_beacon: bool) {
        let CompleteCommand { action, at, .. } = cmd;
        if at.is_none() || is_at_beacon {
            match action {
                CompleteAction::Altitude(CompleteRelOrAbsolute::To(val)) => self.target_flight_level = val,
                CompleteAction::Altitude(CompleteRelOrAbsolute::Plus(val)) => self.target_flight_level += val,
                CompleteAction::Altitude(CompleteRelOrAbsolute::Minus(val)) => self.target_flight_level -= val,
                CompleteAction::Heading(targ) => self.target_direction = targ,
                CompleteAction::SetVisiblity(v) => self.show = v,
            }
            if is_at_beacon {
                if let Some(c) = self.command {
                    if cmd == c {
                        self.command = None;
                    }
                }
            }
        } else if at.is_some() {
            self.command = Some(cmd);
        }

    }
    fn tick(&mut self, is_at_beacon: bool) {
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
            DisplayState::Marked => "\x1b[32m",
            _ => "\x1b[2m",
        };

        format!("{}{}{}{}\x1b[0m", emphasis, color, self.callsign, self.flight_level())
    }
} impl ListRenderable for Plane {
    fn render(&self, command: &Command) -> String {
        let emphasis = match command {
            Command { plane: Some(callsign), .. } if callsign.to_ascii_lowercase() == self.callsign.to_ascii_lowercase() => COMMAND_TARGET_EMPHASIS,
            _ => "",
        };
        let color = match self.show {
            DisplayState::Marked => "\x1b[32m",
            _ => "\x1b[2m",
        };
        let airport = match self.location {
            Location::Flight(_) => format!("   "),
            Location::Airport(a) => format!("@{}", a.to_display_string(self.show == DisplayState::Marked)),
        };
        let command = match (self.show, self.command) {
            (DisplayState::Ignored, _) => format!("---"),
            (_, Some(c)) => c.to_short_string(self.show == DisplayState::Marked),
            _ => String::new(),
        };
        format!("{}{}{}{}{COMMAND_TARGET_EMPHASIS_RESET}\x1b[39m{} {}   {}", emphasis, color, self.callsign, self.flight_level(), airport, self.destination, command)
    }
}

#[derive(Debug, Clone, Copy)]
enum GameStatus {
    PlanesCrashed(char, char),
    PlaneExited(char),
    PlaneFailedLanding(char),
} impl Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::PlanesCrashed(a, b) => write!(f, "Plane {a} crashed into {b}."),
            GameStatus::PlaneExited(p) => write!(f, "Plane {p} exited improperly."),
            GameStatus::PlaneFailedLanding(p) => write!(f, "Plane {p} landed improperly."),
        }
    }
}

#[derive(Debug, Clone)]
struct Map {
    width: u16,
    height: u16,
    current_command: Command,

    exits: Vec<Exit>,
    beacons: Vec<Beacon>,
    path_markers: Vec<GroundLocation>,
    airports: Vec<Airport>,

    planes: Vec<Plane>,
    exit_state: Option<GameStatus>,
    tick_no: u32,
    planes_landed: u32,
} impl Map {
    fn tick(&mut self) {
        if self.exit_state.is_some() { return; }

        let mut planes_to_remove = vec![];
        for (i, plane) in self.planes.iter_mut().enumerate() {
            let mut is_at_beacon = false;
            if let Some(CompleteCommand { at: Some(at), .. }) = plane.command {
                for beacon in &self.beacons {
                    if beacon.index == at && beacon.location == plane.location.into() {
                        is_at_beacon = true;
                        if plane.show == DisplayState::Unmarked {
                            plane.show = DisplayState::Marked;
                        }
                        break
                    };
                }
            }
            plane.tick(is_at_beacon);
            if let Location::Flight(loc) = plane.location {
                let AirLocation(x, y, level) = loc;
                if level == 0 {
                    let mut success = false;
                    for airport in &self.airports {
                        if airport.location == GroundLocation(x, y) {
                            if <CardinalDirection as Into<OrdinalDirection>>::into(airport.launch_direction) == plane.current_direction {
                                success = true;
                                break;
                            }
                        }
                    }
                    if success {
                        planes_to_remove.push(i);
                    } else {
                        self.exit_state = Some(GameStatus::PlaneFailedLanding(plane.callsign));
                    }
                } else {
                    let mut exited_correctly = false;
                    for exit in &self.exits {
                        if exit.exit_location == loc && exit.exit_direction == plane.current_direction {
                            planes_to_remove.push(i);
                            exited_correctly = true;
                            break;
                        }
                    }
                    if !exited_correctly && (x == 0 || x == self.width-1 || y == 0 || y == self.height-1) {
                        self.exit_state = Some(GameStatus::PlaneExited(plane.callsign));
                    }
                }
            }
        }
        'check_collision: for plane_a in &self.planes {
            for plane_b in &self.planes {
                if !std::ptr::eq(plane_a, plane_b) {
                    match (plane_a.location, plane_b.location) {
                        (Location::Flight(AirLocation(ax, ay, az)), Location::Flight(AirLocation(bx, by, bz))) => {
                            let dx = bx.abs_diff(ax);
                            let dy = by.abs_diff(ay);
                            let dz = bz.abs_diff(az);
                            if dx <= 1 && dy <= 1 && dz <= 1 {
                                self.exit_state = Some(GameStatus::PlanesCrashed(plane_a.callsign, plane_b.callsign));
                                break 'check_collision;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        for (j, plane) in planes_to_remove.into_iter().enumerate() {
            self.planes.remove(plane - j);
            self.planes_landed += 1;
        }
        if self.tick_no % 30 == 0 {
            self.generate_plane();
        }
        self.tick_no += 1;
    }
    fn generate_plane(&mut self) {
        if self.planes.len() == 26 {
            return;
        }
        let start = self.generate_location(None);
        let finish = self.generate_location(Some(start));
        let is_jet = random();
        let callsign = 'generate: loop {
            let c = random_range(if is_jet { b'a' ..= b'z' } else { b'A' ..= b'Z' }) as char;
            for plane in &self.planes {
                if plane.callsign.to_ascii_lowercase() == c.to_ascii_lowercase() {
                    continue 'generate;
                }
            }
            break c;
        };
        self.planes.push(Plane {
            location: start.entry(),
            destination: finish,
            target_flight_level: start.entry_height(),
            callsign,
            is_jet,
            ticks_active: 0,
            current_direction: start.entry_dir(),
            target_direction: start.entry_dir(),
            show: DisplayState::Marked,
            command: Default::default(),
        });
    }
    fn generate_location(&self, exclude: Option<Destination>) -> Destination {
        let mut pool = vec![];
        for exit in &self.exits {
            let candidate = Destination::Exit(*exit);
            if let Some(exclude) = exclude {
                if candidate == exclude {
                    continue;
                }
            }
            pool.push(candidate);
        }
        for airport in &self.airports {
            let candidate = Destination::Airport(*airport);
            if let Some(exclude) = exclude {
                if candidate == exclude {
                    continue;
                }
            }
            pool.push(candidate);
        }

        *pool.choose(&mut rng()).expect("location pool to be non-empty")
    }
    fn render(&self, output: &mut impl Write) -> Result<()> {
        let mut grid = RenderGrid::new(self.width, self.height, self.current_command);
        for mark in &self.path_markers {
            grid.add(mark);
        }
        for exit in &self.exits {
            grid.add(exit);
        }
        for beacon in &self.beacons {
            grid.add(beacon);
        }
        for airport in &self.airports {
            grid.add(airport);
        }
        for plane in &self.planes {
            grid.add(plane);
        }

        write!(output, "{}{}", termion::cursor::Goto(1, 1), termion::clear::All)?;
        write!(output, "{}", grid.render())?;
        let table_left = self.width * 2 + 2;
        let mut table_top = 3;
        write!(output, "{}Time: {:<4} Score: {:<4}", termion::cursor::Goto(table_left, 1), self.tick_no, self.planes_landed)?;
        write!(output, "{}\x1b[1mplane dest cmd\x1b[0m", termion::cursor::Goto(table_left, 2))?;
        for plane in &self.planes {
            write!(output, "{}{}", termion::cursor::Goto(table_left, table_top), <Plane as ListRenderable>::render(plane, &self.current_command))?;
            table_top += 1;
        }
        match self.exit_state {
            None => write!(output, "{}\x1b[0m{}", termion::cursor::Goto(1, self.height + 2), self.current_command)?,
            Some(msg) => write!(output, "{}\x1b[0m{}", termion::cursor::Goto(1, self.height + 2), msg)?,
        }

        output.flush()?;

        Ok(())
    }
}

macro_rules! path_markers {
    {$($x:expr,$y:expr),*} => {
        vec![
            $(GroundLocation($x, $y),)+
        ]
    };
}

fn main() -> Result<()> {
    if !io::stdout().is_terminal() {
        panic!("Not an interactive terminal.");
    }
    let mut stdout = io::stdout().into_raw_mode()?.into_alternate_screen()?;
    write!(stdout, "{}", termion::cursor::Hide)?;
    stdout.flush()?;
    let mut input = termion::async_stdin();

    let mut map = Map {
        width: 21,
        height: 21,
        current_command: Default::default(),
        exits: vec![
            Exit {
                index: 0,
                entry_location: AirLocation(10, 0, 7),
                entry_direction: OrdinalDirection::South,
                exit_location: AirLocation(10, 0, 9),
                exit_direction: OrdinalDirection::North,
            },
            Exit {
                index: 1,
                entry_location: AirLocation(20, 10, 7),
                entry_direction: OrdinalDirection::West,
                exit_location: AirLocation(20, 10, 9),
                exit_direction: OrdinalDirection::East,
            },
            Exit {
                index: 2,
                entry_location: AirLocation(10, 20, 7),
                entry_direction: OrdinalDirection::North,
                exit_location: AirLocation(10, 20, 9),
                exit_direction: OrdinalDirection::South,
            },
            Exit {
                index: 3,
                entry_location: AirLocation(0, 10, 7),
                entry_direction: OrdinalDirection::East,
                exit_location: AirLocation(0, 10, 9),
                exit_direction: OrdinalDirection::West,
            }
        ],
        beacons: vec![
            Beacon {
                index: 0,
                location: GroundLocation(4, 10),
            },
            Beacon {
                index: 1,
                location: GroundLocation(16, 10),
            },
            Beacon {
                index: 2,
                location: GroundLocation(10, 10),
            }
        ],
        path_markers: path_markers![
            10, 1, 10, 2, 10, 3, 10, 4, 10, 5, 10, 6, 10, 7, 10, 8, 10, 9,
            10, 11, 10, 12, 10, 13, 10, 14, 10, 15, 10, 16, 10, 17, 10, 18, 10, 19, 10, 20,
            1, 10, 2, 10, 3, 10, 5, 10, 6, 10, 7, 10, 8, 10, 9, 10,
            11, 10, 12, 10, 13, 10, 14, 10, 15, 10, 17, 10, 18, 10, 19, 10, 20, 10,
            4, 9, 4, 8, 4, 7, 4, 6, 4, 5,
            16, 10, 16, 11, 16, 12, 16, 13, 16, 14, 16, 15
        ],
        airports: vec![
            Airport {
                index: 0,
                location: GroundLocation(4, 4),
                launch_direction: CardinalDirection::South,
            },
            Airport {
                index: 1,
                location: GroundLocation(16, 16),
                launch_direction: CardinalDirection::North,
            }
        ],
        planes: vec![],

        exit_state: None,
        tick_no: 0,
        planes_landed: 0,
    };
    map.render(&mut stdout)?;

    let mut char_buf = [0u8];
    let mut last_tick = Instant::now();
    let mut is_dirty = true;
    
    'game: loop {
        if let Ok(count) = input.read(&mut char_buf) {
            if count > 0 {
                is_dirty = true;
                let ch = char_buf[0] as char;
                if ch == '\x03' {
                    break 'game;
                } else if ch == '\x1b' {
                    map.current_command.reset();
                } else if ch == '\n' || ch == '\r' {
                    if let Some(cmd) = map.current_command.try_complete() {
                        for plane in &mut map.planes {
                            if plane.callsign.to_ascii_lowercase() == cmd.plane.to_ascii_lowercase() {
                                plane.accept_cmd(cmd, false);
                                break;
                            }
                        }
                        map.current_command.reset();
                    }
                    if map.current_command.is_empty() {
                        last_tick = Instant::now();
                        map.tick();
                        is_dirty = true;
                    }
                } else {
                    map.current_command.input(ch);
                }
            }
        }
        
        if Instant::now().duration_since(last_tick) >= Duration::from_secs(1) {
            last_tick = Instant::now();
            map.tick();
            is_dirty = true;
        }
        
        if is_dirty {
            map.render(&mut stdout)?;
            is_dirty = false;
        }
    }

    drop(stdout);
    drop(input);
    print!("{}", termion::cursor::Show);

    Ok(())
}
