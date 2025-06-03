use std::{fmt::Display, io::{self, IsTerminal, Read, Write}, time::{Duration, Instant}};
use clap::Parser;

use anyhow::Result;
use termion::{raw::IntoRawMode, screen::IntoAlternateScreen};

mod direction;
mod location;
mod map_objects;
mod command;
mod plane;
mod map;

use map::{Map, MapStatic};

#[derive(Debug, Clone, Copy)]
pub enum GameStatus {
    PlanesCrashed(char, char),
    PlaneExited(char),
    PlaneFailedLanding(char),
} impl Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::PlanesCrashed(a, b) => write!(f, "Plane {a} crashed into plane {b}."),
            GameStatus::PlaneExited(p) => write!(f, "Plane {p} exited improperly."),
            GameStatus::PlaneFailedLanding(p) => write!(f, "Plane {p} landed improperly."),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GameSettings {
    ///In ticks per spawn
    plane_spawn_rate: u32,
    ///In (unit of time) per tick
    tick_rate: Duration,
    allow_landing: bool,
}

#[derive(Debug, Clone, Parser)]
#[command(version, about)]
struct Args {
    ///Lists maps
    #[arg(short, long)]
    list: bool,
    ///Select which map to play on
    #[arg(short, long, default_value_t = String::from("crossing"))]
    map: String,
    ///Set number of ticks between plane spawns
    #[arg(short, long, default_value_t = 30)]
    plane_spawn_rate: u32,
    ///Set delay between ticks in seconds, decimals allowed
    #[arg(short, long, default_value_t = 1.0)]
    tick_rate: f32,
    ///If present, planes' destinations will always be airports
    #[arg(short = 'L', long = "disallow-landing", default_value_t = true, action = clap::ArgAction::SetFalse)]
    allow_landing: bool,
    ///Enter a sequence of keypresses to be entered before the game starts. Use ":" to finish a
    ///command entry.
    #[arg(short = 'i', long = "initialize", default_value_t = String::new())]
    initialize: String,
} impl Into<GameSettings> for Args {
    fn into(self) -> GameSettings {
        GameSettings {
            plane_spawn_rate: self.plane_spawn_rate,
            tick_rate: Duration::from_secs_f32(self.tick_rate),
            allow_landing: self.allow_landing,
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.list {
        use std::fs::{read_dir, read};
        let maps = read_dir("maps")?.map(|f| -> Result<MapStatic> {
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

    let map_text = read(&map_file)?;
    let map_data: MapStatic = serde_json::de::from_slice(&map_text)?;
    let settings = args.clone().into();
    let mut map = Map::new(settings, map_data);

    let mut stdout = io::stdout().into_raw_mode()?.into_alternate_screen()?;
    write!(stdout, "{}", termion::cursor::Hide)?;
    stdout.flush()?;
    let mut input = termion::async_stdin();

    for ch in args.initialize.chars() {
        if ch == ':' {
            if let Some(c) = map.current_command.to_complete() {
                map.exec(c);
                map.current_command.reset();
            }
        } else {
            map.current_command.input(ch);
        }
    }

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
                    if map.current_command.is_empty() {
                        last_tick = Instant::now();
                        map.tick();
                        is_dirty = true;
                    } else if let Some(c) = map.current_command.to_complete() {
                        map.exec(c);
                        map.current_command.reset();
                    }
                } else {
                    map.current_command.input(ch);
                }
            }
        }
        
        if Instant::now().duration_since(last_tick) >= settings.tick_rate {
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
