use std::fmt::Display;

use crate::{direction::OrdinalDirection, map_objects::Beacon, DisplayState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelOrAbsolute {
    Plus(Option<u16>),
    Minus(Option<u16>),
    To(u16),
    Undefined,
} impl TryInto<CompleteRelOrAbsolute> for &RelOrAbsolute {
    type Error = ();
    fn try_into(self) -> Result<CompleteRelOrAbsolute, ()> {
        match self {
            RelOrAbsolute::Plus(Some(v))  => Ok(CompleteRelOrAbsolute::Plus(*v)),
            RelOrAbsolute::Minus(Some(v)) => Ok(CompleteRelOrAbsolute::Minus(*v)),
            RelOrAbsolute::To(v)    => Ok(CompleteRelOrAbsolute::To(*v)),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompleteRelOrAbsolute {
    Plus(u16),
    Minus(u16),
    To(u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompleteAction {
    Altitude(CompleteRelOrAbsolute),
    Heading(OrdinalDirection),
    SetVisiblity(DisplayState),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Altitude(RelOrAbsolute),
    Heading(Option<OrdinalDirection>),
    SetVisibility(DisplayState),
} impl Action {
    pub fn try_complete(&self) -> Option<CompleteAction> {
        match self {
            Self::Altitude(a) => a.try_into().ok().map(CompleteAction::Altitude),
            Self::Heading(h) => h.map(CompleteAction::Heading),
            Self::SetVisibility(v) => Some(CompleteAction::SetVisiblity(*v)),
        }
    }
} impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Altitude(val) => match val {
                RelOrAbsolute::Undefined => write!(f, "altitude:"),
                RelOrAbsolute::To(v) => write!(f, "altitude: {v}000ft"),
                RelOrAbsolute::Plus(None) => write!(f, "altitude: climb"),
                RelOrAbsolute::Plus(Some(v)) => write!(f, "altitude: climb {v}000ft"),
                RelOrAbsolute::Minus(Some(v)) => write!(f, "altitude: descend {v}000ft"),
                RelOrAbsolute::Minus(None) => write!(f, "altitude: descend"),
            },
            Action::Heading(val) => match val {
                Some(dir) => write!(f, "turn to {}", dir.to_deg()),
                None => write!(f, "turn to"),
            },
            Action::SetVisibility(DisplayState::Marked) => write!(f, "mark"),
            Action::SetVisibility(DisplayState::Unmarked) => write!(f, "unmark"),
            Action::SetVisibility(DisplayState::Ignored) => write!(f, "ignore"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompleteCommand {
    pub plane: char,
    pub action: CompleteAction,
    pub at: Option<u16>
} impl CompleteCommand {
    pub fn to_short_string(&self, colorize: bool) -> String {
        format!("{}{}",
            match self.action {
                CompleteAction::Altitude(CompleteRelOrAbsolute::To(val)) => format!("alt: {val}"),
                CompleteAction::Altitude(CompleteRelOrAbsolute::Plus(val)) => format!("alt: +{val}"),
                CompleteAction::Altitude(CompleteRelOrAbsolute::Minus(val)) => format!("alt: -{val}"),
                CompleteAction::Heading(dir) => format!("hdg: {}", dir.to_deg()),
                CompleteAction::SetVisiblity(DisplayState::Marked) => format!("mark"),
                CompleteAction::SetVisiblity(DisplayState::Unmarked) => format!("unmark"),
                CompleteAction::SetVisiblity(DisplayState::Ignored) => format!("ignore"),
            },
            match self.at {
                Some(b) => format!(" {}", Beacon::to_display_string(&Beacon {
                    location: crate::location::GroundLocation(0, 0),
                    index: b,
                }, colorize)),
                None => String::new()
            }
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Setting<T> {
    #[default]
    ///Value is not currently being input by user.
    Unset,
    ///Value is currently being input by user.
    Setting,
    ///Value has been fully input by user.
    Set(T),
} impl<T> Setting<T> {
    fn is_non_null(&self) -> bool {
        match self {
            Setting::Unset => false,
            Setting::Setting => true,
            Setting::Set(_) => true,
        }
    }
} impl<T> Into<Option<T>> for Setting<T> {
    fn into(self) -> Option<T> {
        match self {
            Setting::Unset => None,
            Setting::Setting => None,
            Setting::Set(val) => Some(val),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Command {
    pub plane: Option<char>,
    pub action: Option<Action>,
    pub at: Setting<u16>,
} impl Command {
    pub fn try_complete(&self) -> Option<CompleteCommand> {
        match (self.plane, self.action?.try_complete(), self.at) {
            (Some(plane), Some(action), beacon) => Some(CompleteCommand {
                plane, action,
                at: beacon.into(),
            }),
            _ => None,
        }
    }
    pub fn reset(&mut self) {
        *self = Default::default();
    }
    pub fn is_empty(&self) -> bool {
        self.plane.is_none()
    }
    pub fn input(&mut self, letter: char) {
        if self.plane.is_none() {
            if ('a'..='z').contains(&letter) || ('A'..='Z').contains(&letter) {
                self.plane = Some(letter);
            }
        } else {
            match self.action {
                _ if letter == '\x7f' && self.at.is_non_null() => {
                    match self.at {
                        Setting::Set(_) => self.at = Setting::Setting,
                        Setting::Setting => self.at = Setting::Unset,
                        _ => {}
                    }
                },
                None => match letter.to_ascii_lowercase() {
                    'a' => self.action = Some(Action::Altitude(RelOrAbsolute::Undefined)),
                    't' | 'h' => self.action = Some(Action::Heading(None)),
                    'i' => self.action = Some(Action::SetVisibility(DisplayState::Ignored)),
                    'u' => self.action = Some(Action::SetVisibility(DisplayState::Unmarked)),
                    'm' => self.action = Some(Action::SetVisibility(DisplayState::Marked)),
                    '\x7f' => self.plane = None,
                    _ => {},
                },
                Some(Action::SetVisibility(_)) if letter == '\x7f' => self.action = None,
                Some(Action::Altitude(RelOrAbsolute::Undefined)) => match letter {
                    '-' | '_' => self.action = Some(Action::Altitude(RelOrAbsolute::Minus(None))),
                    '+' | '=' => self.action = Some(Action::Altitude(RelOrAbsolute::Plus(None))),
                    '0'..='9'  => self.action = Some(Action::Altitude(RelOrAbsolute::To(((letter as u8) - b'0') as u16))),
                    '\x7f' => self.action = None,
                    _ => {},
                },
                Some(Action::Altitude(RelOrAbsolute::Plus(None))) => match letter {
                    '0' ..= '9' => self.action = Some(Action::Altitude(RelOrAbsolute::Plus(Some( ((letter as u8) - b'0') as u16)))),
                    '\x7f' => self.action = Some(Action::Altitude(RelOrAbsolute::Undefined)),
                    _ => {},
                },
                Some(Action::Altitude(RelOrAbsolute::Minus(None))) => match letter {
                    '0' ..= '9' => self.action = Some(Action::Altitude(RelOrAbsolute::Minus(Some( ((letter as u8) - b'0') as u16)))),
                    '\x7f' => self.action = Some(Action::Altitude(RelOrAbsolute::Undefined)),
                    _ => {},
                },
                /*
                 * Keybinds:
                 *  qwe  789  yki 
                 *  a d  4 6  h l
                 *  zxc  123  ujo
                 *
                 *  WASD kpad vim
                 *  Vim bindings: HJKL for cardinals; letters above are CW 45deg.
                 */
                Some(Action::Heading(None)) => match letter.to_ascii_lowercase() {
                    '\x7f' => self.action = None,
                    'w' | 'k' | '8' => self.action = Some(Action::Heading(Some(OrdinalDirection::North))),
                    'e' | 'i' | '9' => self.action = Some(Action::Heading(Some(OrdinalDirection::NorthEast))),
                    'd' | 'l' | '6' => self.action = Some(Action::Heading(Some(OrdinalDirection::East))),
                    'c' | 'o' | '3' => self.action = Some(Action::Heading(Some(OrdinalDirection::SouthEast))),
                    'x' | 'j' | '2' => self.action = Some(Action::Heading(Some(OrdinalDirection::South))),
                    'z' | 'u' | '1' => self.action = Some(Action::Heading(Some(OrdinalDirection::SouthWest))),
                    'a' | 'h' | '4' => self.action = Some(Action::Heading(Some(OrdinalDirection::West))),
                    'q' | 'y' | '7' => self.action = Some(Action::Heading(Some(OrdinalDirection::NorthWest))),
                    _ => {},
                },
                Some(Action::Heading(ref mut dir)) if letter == '\x7f' => *dir = None,
                Some(Action::Altitude(RelOrAbsolute::Plus(ref mut v) | RelOrAbsolute::Minus(ref mut v))) if letter == '\x7f' => *v = None,
                
                _ if self.at == Setting::Unset => {
                    if letter == 'a' || letter == '@' {
                        self.at = Setting::Setting;
                    }
                },
                _ if self.at == Setting::Setting => {
                    match letter {
                        '0' ..= '9' => self.at = Setting::Set(((letter as u8) - b'0') as u16),
                        '\x7f' => self.at = Setting::Unset,
                        _ => {},
                    }
                },
                _ => {},
            }
        }
    }
} impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(p) = self.plane {
            write!(f, "{}: ", p)?;
        }
        if let Some(a) = self.action {
            write!(f, "{a}")?;
        }
        match self.at {
            Setting::Unset => {},
            Setting::Setting => write!(f, " at")?,
            Setting::Set(b) => write!(f, " at \x1b[33m*{b}\x1b[0m")?,
        }
        Ok(())
    }
}
