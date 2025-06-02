use std::fmt::Display;

use crate::{direction::{CircleDirection, OrdinalDirection}, map::MapStatic, map_objects::{GridRenderable, ListItemPartRenderable}, plane::{Plane, Visibility}};

enum InputHandling {
    Handled,
    Unhandled,
    Back,
}

trait CommandFragment<T>: Clone {
    ///Mutates the fragment based on an input.
    fn input(&mut self, letter: char) -> InputHandling;
    fn as_text(&self) -> String;
    fn to_complete(&self) -> Option<T>;
}

fn digit_as_num(digit: char) -> u16 {
    if !('0'..='9').contains(&digit) {
        panic!("Digit out of range: {digit}");
    }
    (digit as u16) - '0' as u16
}

//Could derive Copy, but implicit copy leads to bugginess with *self.
#[derive(Debug, Clone, Default)]
pub enum Altitude {
    #[default]
    Undefined,
    Plus(Option<u16>),
    Minus(Option<u16>),
    To(u16),
}
impl CommandFragment<CompleteAltitude> for Altitude {
    fn input(&mut self, letter: char) -> InputHandling {
        match (&self, letter) {
            (Altitude::Undefined, '\x7f') => { return InputHandling::Back },
            (Altitude::To(_) | Altitude::Plus(None) | Altitude::Minus(None), '\x7f') => *self = Altitude::Undefined,
            (Altitude::Plus(Some(_)), '\x7f') => *self = Altitude::Plus(None),
            (Altitude::Minus(Some(_)), '\x7f') => *self = Altitude::Minus(None),

            (Altitude::Undefined, '0'..='9') => *self = Altitude::To(digit_as_num(letter)),
            (Altitude::Undefined, 'c' | '+' | '=') => *self = Altitude::Plus(None),
            (Altitude::Undefined, 'd' | '-' | '_') => *self = Altitude::Minus(None),

            (Altitude::Plus(None), '0'..='9') => *self = Altitude::Plus(Some(digit_as_num(letter))),
            (Altitude::Minus(None), '0'..='9') => *self = Altitude::Minus(Some(digit_as_num(letter))),
            _ => return InputHandling::Unhandled,
        }

        InputHandling::Handled
    }
    fn as_text(&self) -> String {
        match self {
            Altitude::Undefined => format!("altitude:"),
            Altitude::To(val) => format!("altitude: {val}000ft"),
            Altitude::Plus(None) => format!("altitude: climb"),
            Altitude::Minus(None) => format!("altitude: descend"),
            Altitude::Plus(Some(val)) => format!("altitude: climb {val}000ft"),
            Altitude::Minus(Some(val)) => format!("altitude: descend {val}000ft"),
        }
    }
    fn to_complete(&self) -> Option<CompleteAltitude> {
        match self {
            Altitude::To(v)    => Some(CompleteAltitude::To(*v)),
            Altitude::Plus(Some(v))  => Some(CompleteAltitude::Plus(*v)),
            Altitude::Minus(Some(v)) => Some(CompleteAltitude::Minus(*v)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompleteAltitude {
    Plus(u16),
    Minus(u16),
    To(u16),
} impl ListItemPartRenderable for CompleteAltitude {
    fn render(&self, _colorize: bool) -> String {
        match self {
            CompleteAltitude::To(v) => format!("fl={v}"),
            CompleteAltitude::Plus(v) => format!("fl+{v}"),
            CompleteAltitude::Minus(v) => format!("fl-{v}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Turn {
    #[default]
    None,
    ToHeading(OrdinalDirection),
} impl CommandFragment<CompleteTurn> for Turn {
    fn input(&mut self, letter: char) -> InputHandling {
        match (&self, letter) {
            (Turn::None, '\x7f') => return InputHandling::Back,
            (Turn::None, 'w' | '8') => *self = Turn::ToHeading(OrdinalDirection::North),
            (Turn::None, 'e' | '9') => *self = Turn::ToHeading(OrdinalDirection::NorthEast),
            (Turn::None, 'd' | '6') => *self = Turn::ToHeading(OrdinalDirection::East),
            (Turn::None, 'c' | '3') => *self = Turn::ToHeading(OrdinalDirection::SouthEast),
            (Turn::None, 'x' | '2') => *self = Turn::ToHeading(OrdinalDirection::South),
            (Turn::None, 'z' | '1') => *self = Turn::ToHeading(OrdinalDirection::SouthWest),
            (Turn::None, 'a' | '4') => *self = Turn::ToHeading(OrdinalDirection::West),
            (Turn::None, 'q' | '7') => *self = Turn::ToHeading(OrdinalDirection::NorthWest),
            (Turn::ToHeading(_), '\x7f') => *self = Turn::None,
            _ => return InputHandling::Unhandled,
        }
        
        InputHandling::Handled
    }
    fn as_text(&self) -> String {
        match self {
            Turn::None => format!("turn"),
            Turn::ToHeading(h) => format!("turn to {}", h.to_deg()),
        }
    }
    fn to_complete(&self) -> Option<CompleteTurn> {
        match self {
            Turn::ToHeading(dir) => Some(CompleteTurn::ToHeading(*dir)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompleteTurn {
    ToHeading(OrdinalDirection),
} impl ListItemPartRenderable for CompleteTurn {
    fn render(&self, _colorize: bool) -> String {
        match self {
            CompleteTurn::ToHeading(dir) => format!("{}", dir.to_deg()),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Circle {
    #[default]
    None,
    Clockwise,
    CounterClockwise,
} impl CommandFragment<CompleteCircle> for Circle {
    fn input(&mut self, letter: char) -> InputHandling {
        match (&self, letter) {
            (Circle::None, '\x7f') => return InputHandling::Back,
            (Circle::None, 'q') => *self = Circle::CounterClockwise,
            (Circle::None, 'e') => *self = Circle::Clockwise,
            (Circle::Clockwise | Circle::CounterClockwise, '\x7f') => *self = Circle::None,
            _ => return InputHandling::Unhandled,
        }

        InputHandling::Handled
    }
    fn as_text(&self) -> String {
        String::from(match self {
            Circle::None => "circle",
            Circle::Clockwise => "circle clockwise",
            Circle::CounterClockwise => "circle counter-clockwise",
        })
    }
    fn to_complete(&self) -> Option<CompleteCircle> {
        match self {
            Circle::Clockwise => Some(CompleteCircle::Clockwise),
            Circle::CounterClockwise => Some(CompleteCircle::CounterClockwise),
            _ => Some(CompleteCircle::Clockwise),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompleteCircle {
    Clockwise,
    CounterClockwise,
} impl ListItemPartRenderable for CompleteCircle {
    fn render(&self, _colorize: bool) -> String {
        match self {
            CompleteCircle::Clockwise => format!("circle CW"),
            CompleteCircle::CounterClockwise => format!("circle CCW"),
        }
    }
} impl Into<CircleDirection> for CompleteCircle {
    fn into(self) -> CircleDirection {
        match self {
            CompleteCircle::Clockwise        => CircleDirection::Clockwise,
            CompleteCircle::CounterClockwise => CircleDirection::CounterClockwise,
        }
    }
}

//This enum is always complete.
#[derive(Debug, Clone, Copy)]
pub enum SetVisibility {
    Mark,
    Unmark,
    Ignore,
} impl CommandFragment<SetVisibility> for SetVisibility {
    fn input(&mut self, letter: char) -> InputHandling {
        if letter == '\x7f' { return InputHandling::Back }
        InputHandling::Unhandled
    }
    fn as_text(&self) -> String {
        String::from(match self {
            SetVisibility::Mark   => "mark",
            SetVisibility::Unmark => "unmark",
            SetVisibility::Ignore => "ignore",
        })
    }
    fn to_complete(&self) -> Option<SetVisibility> {
        Some(*self)
    }
} impl Into<Visibility> for SetVisibility {
    fn into(self) -> Visibility {
        match self {
            SetVisibility::Mark   => Visibility::Marked,
            SetVisibility::Unmark => Visibility::Unmarked,
            SetVisibility::Ignore => Visibility::Ignored,
        }
    }
} impl ListItemPartRenderable for SetVisibility {
    fn render(&self, _colorize: bool) -> String {
        match self {
            SetVisibility::Mark   => format!("mark"),
            SetVisibility::Unmark => format!("unmark"),
            SetVisibility::Ignore => format!("ignore"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PointOfInterest {
    Default(u16),
    Beacon(Option<u16>),
} impl CommandFragment<CompletePointOfInterest> for PointOfInterest {
    fn input(&mut self, letter: char) -> InputHandling {
        match (&self, letter) {
            (PointOfInterest::Beacon(None), '\x7f') => return InputHandling::Back,
            (PointOfInterest::Default(_), '\x7f') => return InputHandling::Back,
            (PointOfInterest::Beacon(None), '0'..='9') => *self = PointOfInterest::Beacon(Some(digit_as_num(letter))),
            (PointOfInterest::Beacon(Some(_)), '\x7f') => *self = PointOfInterest::Beacon(None),
            _ => return InputHandling::Unhandled,
        }

        InputHandling::Handled
    }
    fn as_text(&self) -> String {
        match self {
            PointOfInterest::Beacon(None) => format!("beacon"),
            PointOfInterest::Beacon(Some(n)) => format!("beacon {n}"),
            PointOfInterest::Default(n) => format!("beacon {n}"),
        }
    }
    fn to_complete(&self) -> Option<CompletePointOfInterest> {
        match self {
            PointOfInterest::Default(n) | PointOfInterest::Beacon(Some(n)) => Some(CompletePointOfInterest::Beacon(*n)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompletePointOfInterest {
    Beacon(u16),
} impl ListItemPartRenderable for CompletePointOfInterest {
    fn render(&self, colorize: bool) -> String {
        match (self, colorize) {
            (CompletePointOfInterest::Beacon(n), false) => format!("*{n}"),
            (CompletePointOfInterest::Beacon(n), true)  => format!("\x1b[33m*{n}\x1b[39m"),
        }
    }
} impl CompletePointOfInterest {
    pub fn is_satisfied(&self, plane: &Plane, map: &MapStatic) -> bool {
        match self {
            CompletePointOfInterest::Beacon(n) => {
                for beacon in &map.beacons {
                    if beacon.location() == plane.location() && beacon.index == *n {
                        return true;
                    }
                }

                false
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct At {
    tail: Box<CommandSegment>,
    poi: Option<PointOfInterest>,
} impl CommandFragment<CompleteAt> for At {
    fn input(&mut self, letter: char) -> InputHandling {
        match (&mut self.poi, letter) {
            (None, '\x7f') => return InputHandling::Back,
            (None, 'b' | '*') => self.poi = Some(PointOfInterest::Beacon(None)),
            (None, '0'..='9') => self.poi = Some(PointOfInterest::Default(digit_as_num(letter))),
            (Some(ref mut poi), _) => {
                return match poi.input(letter) {
                    InputHandling::Handled => InputHandling::Handled,
                    InputHandling::Unhandled => InputHandling::Unhandled,
                    InputHandling::Back => {
                        self.poi = None;
                        InputHandling::Handled
                    }
                }
            },
            _ => return InputHandling::Unhandled,
        }

        InputHandling::Handled
    }
    fn as_text(&self) -> String {
        format!("{} at {}", self.tail.as_text(), match &self.poi {
            None => String::new(),
            Some(poi) => poi.as_text(),
        })
    }
    fn to_complete(&self) -> Option<CompleteAt> {
        let Some(tail) = self.tail.to_complete() else { return None };
        let Some(ref poi) = self.poi else { return None };
        let Some(complete_poi) = poi.to_complete() else { return None };

        Some(CompleteAt {
            tail: Box::new(tail),
            poi: complete_poi,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CompleteAt {
    pub tail: Box<CompleteCommandSegment>,
    pub poi: CompletePointOfInterest,
} impl ListItemPartRenderable for CompleteAt {
    fn render(&self, colorize: bool) -> String {
        format!("{}@{}", self.tail.render(colorize), self.poi.render(colorize))
    }
}

#[derive(Debug, Clone)]
pub struct And {
    left: Box<CommandSegment>,
    right: Box<CommandSegment>,
} impl CommandFragment<CompleteAnd> for And {
    fn input(&mut self, letter: char) -> InputHandling {
        match (&mut *self.right, letter) {
            (CommandSegment::None, '\x7f') => InputHandling::Back,
            (r, l) => r.input(l)
        }
    }
    fn as_text(&self) -> String {
        format!("{} & {}", self.left.as_text(), self.right.as_text())
    }
    fn to_complete(&self) -> Option<CompleteAnd> {
        let Some(left) = self.left.to_complete() else { return None };
        let Some(right) = self.right.to_complete() else { return None };

        Some(CompleteAnd { left: Box::new(left), right: Box::new(right) })
    }
}

#[derive(Debug, Clone)]
pub struct CompleteAnd {
    pub left: Box<CompleteCommandSegment>,
    pub right: Box<CompleteCommandSegment>,
} impl ListItemPartRenderable for CompleteAnd {
    fn render(&self, colorize: bool) -> String {
        format!("{};{}", self.left.render(colorize), self.right.render(colorize))
    }
}

#[derive(Debug, Clone, Default)]
pub enum CommandSegment {
    #[default]
    None,
    Altitude(Altitude),
    Turn(Turn),
    Circle(Circle),
    SetVisibility(SetVisibility),
    At(At),
    And(And),
} impl Display for CommandSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_text())
    }
} impl CommandFragment<CompleteCommandSegment> for CommandSegment {
    fn input(&mut self, letter: char) -> InputHandling {
        let response = match self {
            CommandSegment::None => {
                match letter {
                    '\x7f' => return InputHandling::Back,
                    'a' => *self = CommandSegment::Altitude(Altitude::default()),
                    't' => *self = CommandSegment::Turn(Turn::default()),
                    'c' => *self = CommandSegment::Circle(Circle::default()),

                    'm' => *self = CommandSegment::SetVisibility(SetVisibility::Mark),
                    'u' => *self = CommandSegment::SetVisibility(SetVisibility::Unmark),
                    'i' => *self = CommandSegment::SetVisibility(SetVisibility::Ignore),
                    _ => return InputHandling::Unhandled,
                }

                return InputHandling::Handled;
            },
            CommandSegment::Altitude(a) => a.input(letter),
            CommandSegment::Turn(t) => t.input(letter),
            CommandSegment::Circle(c) => c.input(letter),
            CommandSegment::SetVisibility(v) => v.input(letter),
            CommandSegment::At(a) => a.input(letter),
            CommandSegment::And(a) => a.input(letter)
        };

        match response {
            InputHandling::Unhandled => {
                match letter {
                    'a' | '@' => {
                        *self = CommandSegment::At(At {
                            tail: Box::new(self.clone()),
                            poi: None,
                        });
                        InputHandling::Handled
                    },
                    '&' | ';' => {
                        *self = CommandSegment::And(And {
                            left: Box::new(self.clone()),
                            right: Box::new(CommandSegment::None),
                        });
                        InputHandling::Handled
                    }
                    _ => InputHandling::Unhandled,
                }
            },
            InputHandling::Handled   => InputHandling::Handled,
            InputHandling::Back => match self {
                CommandSegment::At(a) => {
                    *self = *a.tail.clone();
                    InputHandling::Handled
                },
                CommandSegment::And(a) => {
                    *self = *a.left.clone();
                    InputHandling::Handled
                }
                _ => {
                    *self = CommandSegment::None;
                    InputHandling::Handled
                }
            }
        }
    }
    fn as_text(&self) -> String {
        match self {
            CommandSegment::None => String::new(),
            CommandSegment::Altitude(a) => a.as_text(),
            CommandSegment::Turn(t) => t.as_text(),
            CommandSegment::Circle(c) => c.as_text(),
            CommandSegment::SetVisibility(v) => v.as_text(),
            CommandSegment::At(a) => a.as_text(),
            CommandSegment::And(a) => a.as_text(),
        }
    }
    fn to_complete(&self) -> Option<CompleteCommandSegment> {
        match self {
            CommandSegment::Altitude(a) => a.to_complete().map(CompleteCommandSegment::Altitude),
            CommandSegment::Turn(t) => t.to_complete().map(CompleteCommandSegment::Turn),
            CommandSegment::Circle(c) => c.to_complete().map(CompleteCommandSegment::Circle),
            CommandSegment::SetVisibility(v) => Some(CompleteCommandSegment::SetVisibility(*v)),
            CommandSegment::At(a) => a.to_complete().map(CompleteCommandSegment::At),
            CommandSegment::And(a) => a.to_complete().map(CompleteCommandSegment::And),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CompleteCommandSegment {
    Altitude(CompleteAltitude),
    Turn(CompleteTurn),
    Circle(CompleteCircle),
    SetVisibility(SetVisibility),
    At(CompleteAt),
    And(CompleteAnd),
} impl ListItemPartRenderable for CompleteCommandSegment {
    fn render(&self, colorize: bool) -> String {
        match self {
            CompleteCommandSegment::Altitude(a) => a.render(colorize),
            CompleteCommandSegment::Turn(t) => t.render(colorize),
            CompleteCommandSegment::Circle(c) => c.render(colorize),
            CompleteCommandSegment::SetVisibility(v) => v.render(colorize),
            CompleteCommandSegment::At(a) => a.render(colorize),
            CompleteCommandSegment::And(a) => a.render(colorize),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Command {
    pub plane: Option<char>,
    pub head: CommandSegment,
} impl Command {
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
            match self.head.input(letter) {
                InputHandling::Handled => {},
                InputHandling::Unhandled => {
                    eprintln!("Input {letter:?} returned InputHandling::Unhandled on {:?}", self.head);
                },
                InputHandling::Back => {
                    self.plane = None;
                }
            }
        }
    }
    pub fn to_complete(&mut self) -> Option<CompleteCommand> {
        match self {
            Command { plane: Some(plane), head } => head.to_complete().map(|head| CompleteCommand {
                plane: *plane, head
            }),
            _ => None
        }
    }
} impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(p) = self.plane {
            write!(f, "{}: ", p)?;
            write!(f, "{}", self.head)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CompleteCommand {
    pub plane: char,
    pub head: CompleteCommandSegment,
} impl ListItemPartRenderable for CompleteCommand {
    fn render(&self, colorize: bool) -> String {
        self.head.render(colorize)
    }
}
