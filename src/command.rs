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
            PointOfInterest::Beacon(None) => format!("\x1b[33m*\x1b[39m"),
            PointOfInterest::Beacon(Some(n)) => format!("\x1b[33m*{n}\x1b[39m"),
            PointOfInterest::Default(n) => format!("\x1b[33m*{n}\x1b[39m"),
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
pub struct In {
    pub tail: Box<CommandSegment>,
    pub time: Option<u16>,
} impl CommandFragment<CompleteIn> for In {
    fn input(&mut self, letter: char) -> InputHandling {
        match (self.time, letter) {
            (None, '\x7f') => return InputHandling::Back,
            (None, '0'..='9') => self.time = Some(digit_as_num(letter)),
            _ => return InputHandling::Unhandled,
        }

        InputHandling::Handled
    }
    fn as_text(&self) -> String {
        match self.time {
            None => format!("{} in \x1b[36m#\x1b[39m ticks", self.tail.as_text()),
            Some(t) => format!("{} in \x1b[36m#{t}\x1b[39m ticks", self.tail.as_text()),
        }
    }
    fn to_complete(&self) -> Option<CompleteIn> {
        let Some(delay) = self.time else { return None };
        let Some(tail) = self.tail.to_complete() else { return None };
        Some(CompleteIn {
            tail: Box::new(tail),
            time: delay,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CompleteIn {
    pub tail: Box<CompleteCommandSegment>,
    pub time: u16,
} impl ListItemPartRenderable for CompleteIn {
    fn render(&self, colorize: bool) -> String {
        if colorize {
            format!("{}\x1b[36m#{}\x1b[39m", self.tail.render(true), self.time)
        } else {
            format!("{}#{}", self.tail.render(false), self.time)
        }
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
pub struct Ref(Option<u16>);
impl CommandFragment<CompleteRef> for Ref {
    fn input(&mut self, letter: char) -> InputHandling {
        match (self.0, letter) {
            (None, '\x7f') => return InputHandling::Back,
            (Some(_), '\x7f') => self.0 = None,
            (None, '0'..='9') => self.0 = Some(digit_as_num(letter)),
            _ => return InputHandling::Unhandled,
        }

        InputHandling::Handled
    }
    fn as_text(&self) -> String {
        match self.0 {
            None => format!("\x1b[34m%\x1b[39m"),
            Some(n) => format!("\x1b[34m%{n}\x1b[39m"),
        }
    }
    fn to_complete(&self) -> Option<CompleteRef> {
        self.0.map(CompleteRef)
    }
}

#[derive(Debug, Clone)]
pub struct CompleteRef(pub u16);
impl ListItemPartRenderable for CompleteRef {
    fn render(&self, colorize: bool) -> String {
        if colorize {
            format!("\x1b[34m%{}\x1b[39m", self.0)
        } else {
            format!("%{}", self.0)
        }
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
    In(In),
    Ref(Ref),
} impl CommandSegment {
    pub fn current_segment(&self) -> CommandSegment {
        match self {
            CommandSegment::And(And { right, .. }) => right.current_segment(),
            _ => self.clone(),
        }
    }
    pub fn target(&self) -> Option<PointOfInterest> {
        match self.current_segment() {
            CommandSegment::At(At { poi: Some(p), .. }) => Some(p.clone()),
            _ => None,
        }
    }
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
                    '%' => *self = CommandSegment::Ref(Ref::default()),

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
            CommandSegment::And(a) => a.input(letter),
            CommandSegment::In(i) => i.input(letter),
            CommandSegment::Ref(r) => r.input(letter),
        };

        match response {
            InputHandling::Unhandled => {
                match self {
                    CommandSegment::And(a) if a.to_complete().is_none() => InputHandling::Unhandled,
                    CommandSegment::At(a) if a.to_complete().is_none()  => InputHandling::Unhandled,
                    CommandSegment::In(i) if i.to_complete().is_none()  => InputHandling::Unhandled,
                    _ => match letter {
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
                        },
                        '#' | 'i' => {
                            *self = CommandSegment::In(In {
                                tail: Box::new(self.clone()),
                                time: None,
                            });
                            InputHandling::Handled
                        }
                        _ => InputHandling::Unhandled,
                    }
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
                },
                CommandSegment::In(i) => {
                    *self = *i.tail.clone();
                    InputHandling::Handled
                },
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
            CommandSegment::In(i) => i.as_text(),
            CommandSegment::Ref(r) => r.as_text(),
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
            CommandSegment::In(i) => i.to_complete().map(CompleteCommandSegment::In),
            CommandSegment::Ref(r) => r.to_complete().map(CompleteCommandSegment::Ref),
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
    In(CompleteIn),
    Ref(CompleteRef),
    None,
} impl ListItemPartRenderable for CompleteCommandSegment {
    fn render(&self, colorize: bool) -> String {
        match self {
            CompleteCommandSegment::Altitude(a) => a.render(colorize),
            CompleteCommandSegment::Turn(t) => t.render(colorize),
            CompleteCommandSegment::Circle(c) => c.render(colorize),
            CompleteCommandSegment::SetVisibility(v) => v.render(colorize),
            CompleteCommandSegment::At(a) => a.render(colorize),
            CompleteCommandSegment::And(a) => a.render(colorize),
            CompleteCommandSegment::In(i) => i.render(colorize),
            CompleteCommandSegment::Ref(r) => r.render(colorize),
            CompleteCommandSegment::None => if colorize { String::from("\x1b[41m[]\x1b[49m") } else { String::from("[]") },
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum CommandTarget {
    #[default]
    None,
    Plane(char),
    Slot(Option<u16>),
} impl CommandFragment<CompleteCommandTarget> for CommandTarget {
    fn input(&mut self, letter: char) -> InputHandling {
        match (&self, letter) {
            (CommandTarget::None, '\x7f') => return InputHandling::Back,
            (CommandTarget::Plane(_), '\x7f') => *self = CommandTarget::None,
            (CommandTarget::Slot(None), '\x7f') => *self = CommandTarget::None,
            (CommandTarget::Slot(Some(_)), '\x7f') => *self = CommandTarget::Slot(None),

            (CommandTarget::None, 'a'..='z' | 'A'..='Z') => *self = CommandTarget::Plane(letter),
            (CommandTarget::None, '%') => *self = CommandTarget::Slot(None),
            (CommandTarget::Slot(None), '0'..='9') => *self = CommandTarget::Slot(Some(digit_as_num(letter))),
            _ => return InputHandling::Unhandled,
        }

        InputHandling::Handled
    }
    fn as_text(&self) -> String {
        match self {
            CommandTarget::None => String::new(),
            CommandTarget::Plane(c) => format!("\x1b[32m{c}\x1b[39m: "),
            CommandTarget::Slot(None) => format!("\x1b[34m%\x1b[39m"),
            CommandTarget::Slot(Some(n)) => format!("\x1b[34m%{n}\x1b[39m: "),
        }
    }
    fn to_complete(&self) -> Option<CompleteCommandTarget> {
        match self {
            CommandTarget::Plane(c) => Some(CompleteCommandTarget::Plane(*c)),
            CommandTarget::Slot(Some(n)) => Some(CompleteCommandTarget::Slot(*n)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompleteCommandTarget {
    Plane(char),
    Slot(u16),
} impl CompleteCommandTarget {
    pub fn as_text(self) -> String {
        let incomplete: CommandTarget = self.into();
        incomplete.as_text()
    }
} impl Into<CommandTarget> for CompleteCommandTarget {
    fn into(self) -> CommandTarget {
        match self {
            CompleteCommandTarget::Plane(p) => CommandTarget::Plane(p),
            CompleteCommandTarget::Slot(s)  => CommandTarget::Slot(Some(s)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Command {
    pub target: CommandTarget,
    pub head: CommandSegment,
} impl Command {
    pub fn reset(&mut self) {
        *self = Default::default();
    }
    pub fn is_empty(&self) -> bool {
        self.target == CommandTarget::None
    }
    pub fn input(&mut self, letter: char) {
        match self.target.to_complete() {
            None => { self.target.input(letter); },
            Some(_) => match self.head.input(letter) {
                InputHandling::Handled => {},
                InputHandling::Unhandled => {
                    eprintln!("Input {letter:?} returned InputHandling::Unhandled on {:?}", self.head);
                },
                InputHandling::Back => {
                    self.target.input('\x7f');
                },
            },
        }
    }
    pub fn to_complete(&mut self) -> Option<CompleteCommand> {
        let Some(target) = self.target.to_complete() else { return None };
        let Some(command) = self.head.to_complete() else { return None };
        Some(CompleteCommand {
            target, head: command,
        })
    }
    pub fn current_segment(&self) -> CommandSegment {
        self.head.current_segment()
    }
} impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.target.as_text())?;
        write!(f, "{}", self.head.as_text())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CompleteCommand {
    pub target: CompleteCommandTarget,
    pub head: CompleteCommandSegment,
} impl ListItemPartRenderable for CompleteCommand {
    fn render(&self, colorize: bool) -> String {
        self.head.render(colorize)
    }
}
