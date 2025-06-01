use std::{fmt::Display, io::{self, IsTerminal, Read, Write}, time::{Duration, Instant}};
use clap::Parser;
use serde::Deserialize;
use rand::{prelude::*, random, random_range, rng};

use anyhow::Result;
use termion::{raw::IntoRawMode, screen::IntoAlternateScreen};

mod direction;
use direction::{CardinalDirection, OrdinalDirection};

mod location;
use location::{Location, Destination, GroundLocation, AirLocation};

mod map_objects;
use map_objects::{Airport, Beacon, Exit, ListRenderable, RenderGrid };

mod command;
use command::{Command, CompleteCommand };

mod plane;
use plane::{Plane, Visibility};

mod map;
use map::Map;

#[derive(Debug, Clone, Copy)]
pub enum GameStatus {
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



#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    list: bool,
    #[arg(short, long, default_value_t = String::from("crossing"))]
    map: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    eprintln!("{args:?}");
    if args.list {
        use std::fs::{read_dir, read};
        let maps = read_dir("maps")?.map(|f| -> Result<Map> {
            let file = f?;
            let contents = read(file.path())?;
            Ok(serde_json::de::from_slice(&contents)?)
        }).filter_map(Result::ok).collect::<Vec<_>>();

        println!("{}", tabled::Table::new(maps).with(tabled::settings::Style::blank()));
        return Ok(());
    }

    if !io::stdout().is_terminal() {
        panic!("Not an interactive terminal.");
    }
    use std::fs::{exists, read};
    let map_file = if exists(&args.map)? { format!("{}", args.map) }
    else if exists(&format!("{}.json", args.map))? { format!("{}.json", args.map) }
    else { format!("maps/{}.json", args.map) };
    eprintln!("{map_file:?}");
    let map_data = read(&map_file)?;
    let mut map: Map = serde_json::de::from_slice(&map_data)?;

    let mut stdout = io::stdout().into_raw_mode()?.into_alternate_screen()?;
    write!(stdout, "{}", termion::cursor::Hide)?;
    stdout.flush()?;
    let mut input = termion::async_stdin();

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
