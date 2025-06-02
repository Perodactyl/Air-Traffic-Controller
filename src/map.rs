use crate::{command::{Command, CompleteCommand}, direction::{CardinalDirection, OrdinalDirection}, location::{AirLocation, Destination, GroundLocation, Location}, map_objects::{Airport, Beacon, Exit, ListRenderable, RenderGrid}, plane::{Plane, Visibility}, GameStatus};
use anyhow::Result;
use std::io::Write;
use serde::Deserialize;
use tabled::Tabled;
use rand::{random, random_range, rng, prelude::*};

#[derive(Debug, Clone, Deserialize)]
pub struct MapStatic {
    pub width: u16,
    pub height: u16,
    pub exits: Vec<Exit>,
    pub beacons: Vec<Beacon>,
    pub airports: Vec<Airport>,
    pub path_markers: Vec<GroundLocation>,
}

#[derive(Debug, Clone, Deserialize, Tabled)]
pub struct Map {
    #[tabled(rename = "Map")]
    name: String,
    #[tabled(rename = "Author")]
    author: String,

    #[tabled(skip)]
    #[serde(flatten)]
    info: MapStatic,
    #[serde(skip)]
    pub current_command: Command,

    #[tabled(skip)]
    #[serde(skip)]
    pub planes: Vec<Plane>,
    #[tabled(skip)]
    #[serde(skip)]
    exit_state: Option<GameStatus>,
    #[tabled(skip)]
    #[serde(skip)]
    tick_no: u32,
    #[tabled(skip)]
    #[serde(skip)]
    planes_landed: u32,
} impl Map {
    pub fn tick(&mut self) {
        if self.exit_state.is_some() { return; }

        let mut planes_to_remove = vec![];
        for (i, plane) in self.planes.iter_mut().enumerate() {
            plane.tick(&self.info);
            if let Location::Flight(loc) = plane.location {
                let AirLocation(x, y, level) = loc;
                if level == 0 {
                    let mut success = false;
                    for airport in &self.info.airports {
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
                    for exit in &self.info.exits {
                        if exit.exit_location == loc && exit.exit_direction == plane.current_direction {
                            planes_to_remove.push(i);
                            exited_correctly = true;
                            break;
                        }
                    }
                    if !exited_correctly && (x == 0 || x == self.info.width-1 || y == 0 || y == self.info.height-1) {
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
            show: Visibility::Marked,
            command: None,
        });
    }
    fn generate_location(&self, exclude: Option<Destination>) -> Destination {
        let mut pool = vec![];
        for exit in &self.info.exits {
            let candidate = Destination::Exit(*exit);
            if let Some(exclude) = exclude {
                if candidate == exclude {
                    continue;
                }
            }
            pool.push(candidate);
        }
        for airport in &self.info.airports {
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
    pub fn exec(&mut self, command: CompleteCommand) {
        eprintln!("{command:?}");
        for plane in &mut self.planes {
            if plane.callsign.to_ascii_lowercase() == command.plane.to_ascii_lowercase() {
                plane.exec(command.head, &self.info);
                return;
            }
        }
        eprintln!("Plane {} not found.", command.plane);
    }
    pub fn render(&self, output: &mut impl Write) -> Result<()> {
        let mut grid = RenderGrid::new(self.info.width, self.info.height, &self.current_command);
        for mark in &self.info.path_markers {
            grid.add(mark);
        }
        for exit in &self.info.exits {
            grid.add(exit);
        }
        for beacon in &self.info.beacons {
            grid.add(beacon);
        }
        for airport in &self.info.airports {
            grid.add(airport);
        }
        for plane in &self.planes {
            grid.add(plane);
        }

        write!(output, "{}{}", termion::cursor::Goto(1, 1), termion::clear::All)?;
        write!(output, "{}", grid.render())?;
        let table_left = self.info.width * 2 + 2;
        let mut table_top = 3;
        write!(output, "{}Time: {:<4} Score: {:<4}", termion::cursor::Goto(table_left, 1), self.tick_no, self.planes_landed)?;
        write!(output, "{}\x1b[1mplane dest cmd\x1b[0m", termion::cursor::Goto(table_left, 2))?;
        for plane in &self.planes {
            write!(output, "{}{}", termion::cursor::Goto(table_left, table_top), <Plane as ListRenderable>::render(plane, &self.current_command))?;
            table_top += 1;
        }
        match self.exit_state {
            None => write!(output, "{}\x1b[0m{}", termion::cursor::Goto(1, self.info.height + 2), self.current_command)?,
            Some(msg) => write!(output, "{}\x1b[0m{}", termion::cursor::Goto(1, self.info.height + 2), msg)?,
        }

        output.flush()?;

        Ok(())
    }
}
