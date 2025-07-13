use crossterm::style::{StyledContent, Stylize};
use crate::game_state::Coord;

/// Rotation object
pub enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

impl Rotation {
    /// Get the next rotation clockwise
    pub fn rotate_cw(&self) -> Self {
        match self {
            Rotation::R0 => Rotation::R90,
            Rotation::R90 => Rotation::R180,
            Rotation::R180 => Rotation::R270,
            Rotation::R270 => Rotation::R0,
        }
    }

    /// Get the next rotation counter clockwise
    pub fn rotate_ccw(&self) -> Self {
        match self {
            Rotation::R0 => Rotation::R270,
            Rotation::R90 => Rotation::R0,
            Rotation::R180 => Rotation::R90,
            Rotation::R270 => Rotation::R180,
        }
    }

    /// Gets the current rotation as a string for debug
    #[allow(unused)]
    pub fn get_string(&self) -> String {
        match self {
            Rotation::R0 => "0",
            Rotation::R90 => "90",
            Rotation::R180 => "180",
            Rotation::R270 => "270",
        }.to_string()
    }
}

/// All colors the various shapes can be
#[derive(Clone, Copy)]
pub enum ShapeColor {
    Cyan,
    Blue,
    Orange,
    Yellow,
    Green,
    Purple,
    Red,
    None,
}

impl ShapeColor {
    /// Determines if the color is representative of a block
    pub fn is_block(&self) -> bool {
        match self {
            ShapeColor::None => false,
            _ => true,
        }
    }

    /// Returns a tile styled based on the color
    pub fn color_tile(&self) -> StyledContent<&str> {
        match self {
            ShapeColor::Cyan => "██".cyan(),
            ShapeColor::Blue => "██".blue(),
            ShapeColor::Orange => "██".dark_red(),
            ShapeColor::Yellow => "██".yellow(),
            ShapeColor::Green => "██".green(),
            ShapeColor::Purple => "██".magenta(),
            ShapeColor::Red => "██".red(),
            _ => "██".reset()
        }
    }
}

// All possible shapes
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    I,
    J,
    L,
    O,
    Z,
    T,
    S
}

impl Shape {
    /// Gets the color associated with the shape
    pub fn get_color(&self) -> ShapeColor {
        use ShapeColor::*;
        match self {
            Shape::I => Cyan,
            Shape::J => Blue,
            Shape::L => Orange,
            Shape::O => Yellow,
            Shape::Z => Green,
            Shape::T => Purple,
            Shape::S => Red,
        }
    }

    /// Gets the next shape in order
    #[allow(unused)]
    pub fn get_next_shape_ord(&self) -> Self {
        use Shape::*;
        match self {
            I => J,
            J => L,
            L => O,
            O => Z,
            Z => T,
            T => S,
            S => I,
        }
    }

    /// Returns the spawn offsets (x, y) for each piece
    pub fn get_spawn_offsets(&self) -> Coord {
        match self {
            Shape::I => Coord{x: 3, y: 1},
            Shape::O => Coord{x: 3, y: 1},

            // Other shapes are the same
            Shape::J | Shape::L |
            Shape::Z | Shape::T |
            Shape::S => Coord{x: 3, y: 2},
        }
    }


    /// Returns a random piece
    pub fn random() -> Self {
        use Shape::*;
        match rand::random_range(0..7) {
            0 => I,
            1 => J,
            2 => L,
            3 => O,
            4 => Z,
            5 => T,
            6 => S,
            _ => unreachable!()
        }
    }

    /// Gets the wall kick data for the current shape
    pub fn get_kick_data(&self, from: &Rotation, to: &Rotation) -> [(i16, i16); 5] {
        use Shape::*; 
        use Rotation::*;

        match self {
            J | L | S | T | Z => {
                match (from, to) {
                    // 0 - R
                    (R0, R90) => [(0,0), (-1,0), (-1,1), (0,-2), (-1,-2)],
                    (R90, R0) => [(0,0), (1,0), (1,-1), (0,2), (1,2)],

                    // R - 2
                    (R90, R180) => [(0,0), (1,0), (1,-1), (0,2), (1,2)],
                    (R180, R90) => [(0,0), (-1,0), (-1,1), (0,-2), (-1,-2)],

                    // 2 - L
                    (R180, R270) => [(0,0), (1,0), (1,1), (0,-2), (1,-2)],
                    (R270, R180) => [(0,0), (-1,0), (-1,-1), (0,2), (-1,2)],

                    // L - 0
                    (R270, R0) => [(0,0), (-1,0), (-1,-1), (0,2), (-1,2)],
                    (R0, R270) => [(0,0), (1,0), (1,1), (0,-2), (1,-2)],

                    _ => unreachable!()
                }
            },
            O => [(0,0); 5], // i love you so much O piece please be my wife
            I => match(from, to) {
                    // 0 - R
                    (R0, R90) => [(0,0), (-2,0), (1,0), (-2,-1), (1,2)],
                    (R90, R0) => [(0,0), (2,0), (-1,0), (2,1), (-1,-2)],

                    // R - 2
                    (R90, R180) => [(0,0), (-1,0), (2,0), (-1,2), (2,-1)],
                    (R180, R90) => [(0,0), (1,0), (-2,1), (1,-2), (-2,1)],

                    // 2 - L
                    (R180, R270) => [(0,0), (2,0), (-1,0), (2,1), (-1,-2)],
                    (R270, R180) => [(0,0), (-2,0), (1,0), (-2,-1), (1,2)],

                    // L - 0
                    (R270, R0) => [(0,0), (1,0), (-2,0), (1,-2), (-2,1)],
                    (R0, R270) => [(0,0), (-1,0), (2,0), (-1,2), (2,-1)],

                    _ => unreachable!()
            },
        }
    }

    /// Gets the current shape array based on rotation
    pub fn get_shape(&self, rot: &Rotation) -> [[bool; 4]; 4] {
        use Shape::*;
        use Rotation::*;

        match self {
            // I peice
            I => match rot {
                R0 => [
                    [false, false, false, false],
                    [true , true , true , true ],
                    [false, false, false, false],
                    [false, false, false, false],
                ],
                R90 => [
                    [false, false, true, false],
                    [false, false, true, false],
                    [false, false, true, false],
                    [false, false, true, false],
                ],
                R180 => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [true , true , true , true ],
                    [false, false, false, false],
                ],
                R270 => [
                    [false, true, false, false],
                    [false, true, false, false],
                    [false, true, false, false],
                    [false, true, false, false],
                ],
            },
            J => match rot {
                R0 => [
                    [true , false, false, false],
                    [true , true , true , false],
                    [false, false, false, false],
                    [false, false, false, false],
                ],
                R90 => [
                    [false, true , true , false],
                    [false, true , false, false],
                    [false, true , false, false],
                    [false, false, false, false],
                ],
                R180 => [
                    [false, false, false, false],
                    [true , true , true , false],
                    [false, false, true , false],
                    [false, false, false, false],
                ],
                R270 => [
                    [false, true , false, false],
                    [false, true , false, false],
                    [true , true , false, false],
                    [false, false, false, false],
                ],
            },
            L => match rot {
                R0 => [
                    [false, false, true , false],
                    [true , true , true , false],
                    [false, false, false, false],
                    [false, false, false, false],
                ],
                R90 => [
                    [false, true , false, false],
                    [false, true , false, false],
                    [false, true , true , false],
                    [false, false, false, false],
                ],
                R180 => [
                    [false, false, false, false],
                    [true , true , true , false],
                    [true , false, false, false],
                    [false, false, false, false],
                ],
                R270 => [
                    [true , true , false, false],
                    [false, true , false, false],
                    [false, true , false, false],
                    [false, false, false, false],
                ],
            },
            O => { // i <3 u square shape
                [
                    [false, false, false, false],
                    [false, true , true , false],
                    [false, true , true , false],
                    [false, false, false, false],
                ]
            },
            S => match rot {
                R0 => [
                    [false, true , true , false],
                    [true , true , false, false],
                    [false, false, false, false],
                    [false, false, false, false],
                ],
                R90 => [
                    [false, true , false, false],
                    [false, true , true , false],
                    [false, false, true , false],
                    [false, false, false, false],
                ],
                R180 => [
                    [false, false, false, false],
                    [false, true , true , false],
                    [true , true , false, false],
                    [false, false, false, false],
                ],
                R270 => [
                    [true , false, false, false],
                    [true , true , false, false],
                    [false, true , false, false],
                    [false, false, false, false],
                ],
            },
            Z => match rot {
                R0 => [
                    [true , true , false, false],
                    [false, true , true , false],
                    [false, false, false, false],
                    [false, false, false, false],
                ],
                R90 => [
                    [false, true , false, false],
                    [true , true , false, false],
                    [true , false, false, false],
                    [false, false, false, false],
                ],
                R180 => [
                    [false, false, false, false],
                    [true , true , false, false],
                    [false, true , true , false],
                    [false, false, false, false],
                ],
                R270 => [
                    [false, false, true , false],
                    [false, true , true , false],
                    [false, true , false, false],
                    [false, false, false, false],
                ],
            },
            T => match rot {
                R0 => [
                    [false, true , false, false],
                    [true , true , true , false],
                    [false, false, false, false],
                    [false, false, false, false],
                ],
                R90 => [
                    [false, true , false, false],
                    [false, true , true , false],
                    [false, true , false, false],
                    [false, false, false, false],
                ],
                R180 => [
                    [false, false, false, false],
                    [true , true , true , false],
                    [false, true , false , false],
                    [false, false, false, false],
                ],
                R270 => [
                    [false, true , false, false],
                    [true , true , false, false],
                    [false, true , false, false],
                    [false, false, false, false],
                ],
            },
        }
    }
}
