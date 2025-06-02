use std::fmt::Display;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CircleDirection {
    #[default]
    Clockwise,
    CounterClockwise
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardinalDirection {
    #[serde(alias = "n")]
    North,
    #[serde(alias = "s")]
    South,
    #[serde(alias = "e")]
    East,
    #[serde(alias = "w")]
    West
} impl Into<OrdinalDirection> for CardinalDirection {
    fn into(self) -> OrdinalDirection {
        match self {
            CardinalDirection::North => OrdinalDirection::North,
            CardinalDirection::South => OrdinalDirection::South,
            CardinalDirection::East  => OrdinalDirection::East,
            CardinalDirection::West  => OrdinalDirection::West,
        }
    }
} impl TryFrom<OrdinalDirection> for CardinalDirection {
    type Error = ();
    fn try_from(value: OrdinalDirection) -> Result<Self, Self::Error> {
        match value {
            OrdinalDirection::North => Ok(CardinalDirection::North),
            OrdinalDirection::South => Ok(CardinalDirection::South),
            OrdinalDirection::East  => Ok(CardinalDirection::East),
            OrdinalDirection::West  => Ok(CardinalDirection::West),
            _ => Err(()),
        }
    }
} impl Display for CardinalDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            CardinalDirection::North => "^",
            CardinalDirection::South => "v",
            CardinalDirection::East  => ">",
            CardinalDirection::West  => "<",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrdinalDirection {
    #[serde(alias = "n")]
    North,
    #[serde(alias = "s")]
    South,
    #[serde(alias = "e")]
    East,
    #[serde(alias = "w")]
    West,
    #[serde(alias = "ne")]
    NorthEast,
    #[serde(alias = "se")]
    SouthEast,
    #[serde(alias = "nw")]
    NorthWest,
    #[serde(alias = "sw")]
    SouthWest,
} impl OrdinalDirection {
    pub fn as_offset(&self) -> (i16, i16) {
        match self {
            OrdinalDirection::North => ( 0, -1),
            OrdinalDirection::South => ( 0,  1),
            OrdinalDirection::East  => ( 1,  0),
            OrdinalDirection::West  => (-1,  0),
            OrdinalDirection::NorthEast => ( 1, -1),
            OrdinalDirection::SouthEast => ( 1,  1),
            OrdinalDirection::NorthWest => (-1, -1),
            OrdinalDirection::SouthWest => (-1,  1),
        }
    }
    pub fn rotate_toward(self, target: OrdinalDirection) -> OrdinalDirection {
        use OrdinalDirection::*;
        match (self, target) { //Yes. I just wrote 64 lines of truth table.
            //Valid cases
            (North, West)          => West,
            (North, NorthWest)     => NorthWest,
            (North, North)         => North,
            (North, NorthEast)     => NorthEast,
            (North, East)          => East,

            (NorthEast, NorthWest) => NorthWest,
            (NorthEast, North)     => North,
            (NorthEast, NorthEast) => NorthEast,
            (NorthEast, East)      => East,
            (NorthEast, SouthEast) => SouthEast,

            (East, North)          => North,
            (East, NorthEast)      => NorthEast,
            (East, East)           => East,
            (East, SouthEast)      => SouthEast,
            (East, South)          => South,

            (SouthEast, NorthEast) => NorthEast,
            (SouthEast, East)      => East,
            (SouthEast, SouthEast) => SouthEast,
            (SouthEast, South)     => South,
            (SouthEast, SouthWest) => SouthWest,

            (South, East)          => East,
            (South, SouthEast)     => SouthEast,
            (South, South)         => South,
            (South, SouthWest)     => SouthWest,
            (South, West)          => West,

            (SouthWest, SouthEast) => SouthEast,
            (SouthWest, South)     => South,
            (SouthWest, SouthWest) => SouthWest,
            (SouthWest, West)      => West,
            (SouthWest, NorthWest) => NorthWest,

            (West, South)          => South,
            (West, SouthWest)      => SouthWest,
            (West, West)           => West,
            (West, NorthWest)      => NorthWest,
            (West, North)          => North,

            (NorthWest, SouthWest) => SouthWest,
            (NorthWest, West)      => West,
            (NorthWest, NorthWest) => NorthWest,
            (NorthWest, North)     => North,
            (NorthWest, NorthEast) => NorthEast,
            
            //Reflex angles
            (North, SouthEast)     => East,
            (North, SouthWest)     => West,
            (NorthEast, South)     => SouthEast,
            (NorthEast, West)      => NorthWest,
            (East, NorthWest)      => North,
            (East, SouthWest)      => South,
            (SouthEast, North)     => NorthEast,
            (SouthEast, West)      => SouthWest,
            (South, NorthEast)     => East,
            (South, NorthWest)     => West,
            (SouthWest, North)     => NorthWest,
            (SouthWest, East)      => SouthEast,
            (West, NorthEast)      => North,
            (West, SouthEast)      => South,
            (NorthWest, East)      => NorthEast,
            (NorthWest, South)     => SouthWest,

            //180s (always go CW)
            (North, South)         => East,
            (NorthEast, SouthWest) => SouthEast,
            (East, West)           => South,
            (SouthEast, NorthWest) => SouthWest,
            (South, North)         => West,
            (SouthWest, NorthEast) => NorthWest,
            (West, East)           => North,
            (NorthWest, SouthEast) => NorthEast,
        }
    }
    pub fn rotated_90(&self, direction: CircleDirection) -> OrdinalDirection {
        use OrdinalDirection::*;
        use CircleDirection::*;
        match (self, direction) {
            (North,     Clockwise)        => East,
            (NorthEast, Clockwise)        => SouthEast,
            (East,      Clockwise)        => South,
            (SouthEast, Clockwise)        => SouthWest,
            (South,     Clockwise)        => West,
            (SouthWest, Clockwise)        => NorthWest,
            (West,      Clockwise)        => North,
            (NorthWest, Clockwise)        => NorthEast,

            (North,     CounterClockwise) => West,
            (NorthWest, CounterClockwise) => SouthWest,
            (West,      CounterClockwise) => South,
            (SouthWest, CounterClockwise) => SouthEast,
            (South,     CounterClockwise) => East,
            (SouthEast, CounterClockwise) => NorthEast,
            (East,      CounterClockwise) => North,
            (NorthEast, CounterClockwise) => NorthWest,
        }
    }
    pub fn to_deg(&self) -> u16 {
        match self {
            OrdinalDirection::North     => 0,
            OrdinalDirection::NorthEast => 45,
            OrdinalDirection::East      => 90,
            OrdinalDirection::SouthEast => 135,
            OrdinalDirection::South     => 180,
            OrdinalDirection::SouthWest => 225,
            OrdinalDirection::West      => 270,
            OrdinalDirection::NorthWest => 315,
        }
    }
}
