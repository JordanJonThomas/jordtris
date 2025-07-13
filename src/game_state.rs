use std::time::Instant;

use crate::{shapes::{Rotation, Shape, ShapeColor}, Direction};

/// A position on the screen
#[derive(Clone)]
pub struct Coord {
    pub x: i16,
    pub y: i16,
}

/// Represent the current phase of the game the player is in
pub enum GamePhase {
    Playing,
    GameOver,
    Help,
    Score,
}

/// Represents the current game state
pub struct GameState {
    pub player_pos: Coord,
    pub current_shape: Shape,
    pub rotation: Rotation,
    pub board: [[ShapeColor; 10]; 22],
    pub last_fall: Instant,
    pub last_input: Instant,
    pub score: i32,
    pub held: Option<Shape>,
    pub shape_queue: Vec<Shape>,
    pub just_held: bool,
    pub game_phase: GamePhase,
}

impl GameState {
    /// Creates a new game
    pub fn new() -> Self {
        let shape = Shape::random(); // Get starting shape
        GameState {
            player_pos: shape.get_spawn_offsets(),
            current_shape: shape,
            rotation: Rotation::R0,
            board: [[ShapeColor::None; 10]; 22],
            last_fall: Instant::now(),
            last_input: Instant::now(),
            score: 0,
            held: None,
            shape_queue: create_new_7_bag().to_vec(),
            just_held: false,
            game_phase: GamePhase::Playing,
        }
    }

    /// Determines if a piece can be placed at a position
    pub fn can_place(&self, shape: &Shape, rot: &Rotation, at: &Coord) -> bool {
        let shape = shape.get_shape(rot);
        //let mut right_most = 0;

        // Iterate shape
        for dx in 0..4 {
            for dy in 0..4 {
                if shape[dy][dx] { // Shape tile found
                    // Get new coords
                    let x = at.x + dx as i16;
                    let y = at.y + dy as i16;

                    // Bounds checks
                    if x < 0 || y < 0 {
                        return false;
                    }

                    if x >= 10 || y >= 22 {
                        return false;
                    }

                    // Check for existing tile
                    if y >= 0 && self.board[y as usize][x as usize].is_block() { // Safe cast, bound check
                        return false;
                    }
                }
            }
        }

        // All cases passed
        true
    }

    /// Places the player onto the board, triggers game over
    /// if the attempted placement cannot be completed
    pub fn place_player(&mut self) {
        // get current shape
        let shape = self.current_shape;

        // Determine if player can be placed
        if !self.can_place(&shape, &self.rotation, &self.player_pos) {
            self.game_phase = GamePhase::GameOver; // Piece cant be placed, game over
        }

        // Get shape array
        let shape = shape.get_shape(&self.rotation);

        // Iterate player
        for dx in 0..4 {
            for dy in 0..4 {
                // Get tiles in shape
                if shape[dy][dx] {
                    // Get board positions
                    let x = (self.player_pos.x + dx as i16) as usize;
                    let y = (self.player_pos.y + dy as i16) as usize;

                    // Place piece
                    self.board[y][x] = self.current_shape.get_color(); 
                }
            }
        }

        // Allow hold again
        self.just_held = false;

        // Attempt to clear any lines
        self.clear_lines();
    }

    /// Spawns a random new piece at the top of the board 
    fn reset_player_piece(&mut self) {
        // New piece and reset position
        self.current_shape = self.get_next_shape();
        self.player_to_top();
    }

    /// Places and resets the player
    pub fn place_and_reset(&mut self) {
        self.place_player();
        self.reset_player_piece();

        // Determine if moving player to top is game loss
        if !self.can_place(&self.current_shape, &self.rotation, &self.player_pos) {
            self.game_phase = GamePhase::GameOver;
        }
    }

    /// Checks and clears any lines the player has created
    fn clear_lines(&mut self) {
        // Iterate vertically
        let mut y = 21;
        'outer: while y >= 1 {
            // Iterate across line
            for x in 0..10 {
                // If any tiles are not blocks
                if !self.board[y][x].is_block() {
                    y -= 1; // Decrease y
                    continue 'outer;
                }
            }

            // Clear line/move board down
            for row in (1..=y).rev() {
                self.board[row] = self.board[row-1];
            }

            // Increase score
            self.score += 100; // TODO: Proper scoring system
        }
    }

    /// Moves the player to the left or right
    pub fn move_player_horizontal(&mut self, dir: Direction) {
        // Horizontal only
        if let Direction::Up | Direction::Down = dir {
            return;
        }

        // Get new position
        let mut new_pos = self.player_pos.clone();
        new_pos.x += dir.to_value();

        // Determine if piece can be moved
        if !self.can_place(
            &self.current_shape,
            &self.rotation,
            &new_pos,
        ) {
            return;
        }

        // Move piece
        self.player_pos.x += dir.to_value();
    }

    /// Drops the player onto the ghost block
    pub fn hard_drop(&mut self) {
        // get new drop height
        let shape = self.current_shape.get_shape(&self.rotation);
        let drop_y = self.get_drop_position(&shape);

        // Change player position and place
        self.player_pos.y = drop_y;
        self.place_and_reset();
    }

    /// Determines the drop height of the current shape
    pub fn get_drop_position(&self, shape: &[[bool;4];4]) -> i16 {
        let mut ghost_y = self.player_pos.y; // Starting y

        'drop: loop {
            for dx in 0..4 {
                for dy in 0..4 {
                    if shape[dy][dx] { // Shape tile found
                        let x = self.player_pos.x + dx as i16;
                        let y = ghost_y + dy as i16;

                        // Stop if tile would collide with bottom or tile
                        if y + 1 >= 22 ||
                        (y+1 >= 0 && self.board[(y+1) as usize][x as usize].is_block()) {
                            break 'drop;
                        }
                    }
                }
            }

            ghost_y += 1;
        }

        ghost_y
    }

    /// Moves the player to starting position
    pub fn player_to_top(&mut self) {
        // Set new positions
        let new_pos = self.current_shape.get_spawn_offsets();
        let new_rot = Rotation::R0;

        self.player_pos = new_pos;
        self.rotation = new_rot;
    }

    /// Attempts to move the player down one tile, returns false on fail
    pub fn fall_player(&mut self) -> bool {
        // get new position
        let mut new_pos = self.player_pos.clone();
        new_pos.y += 1;

        // Try to move down
        if !self.can_place(
            &self.current_shape,
            &self.rotation,
            &new_pos
        ) {
            return false;
        }

        // Move player
        self.player_pos.y +=1;
        true
    }

    /// Attempts to rotate the player
    /// 
    /// Up and down are the only valid directions and will
    /// be interpreted as cw and ccw respectively.
    pub fn rotate_player(&mut self, dir: Direction) {
        // Get new rotation and offsets
        let new_rot = match dir {
            Direction::Up => self.rotation.rotate_cw(),
            Direction::Down => self.rotation.rotate_ccw(),
            _ => unreachable!() 
        };
        let offsets = self.current_shape.get_kick_data(&self.rotation, &new_rot);

        // Iterate each offset 
        for (dx, dy) in offsets {
            // Calculate new position
            let new_pos = Coord { x: self.player_pos.x + dx, y: self.player_pos.y + dy };

            // Attempt placement
            if self.can_place(
                &self.current_shape, 
                &new_rot, 
                &new_pos
            ) {
                // Placement possible! 
                self.player_pos = new_pos;
                self.rotation = new_rot;
                break;
            }
        }
    }

    /// Swaps the player with the held piece
    pub fn hold(&mut self) {
        // No double hold
        if self.just_held {
            return;
        }

        // Swap shape and held
        let temp = self.current_shape;
        self.current_shape = self.held.unwrap_or(self.get_next_shape());
        self.held = Some(temp);
        self.player_to_top();

        // Set held
        self.just_held = true;
    }

    /// Gets the next shape from the queue and extends if neccesary
    pub fn get_next_shape(&mut self) -> Shape {
        // Assign next player shape
        let new_shape = self.shape_queue[0];

        // Adjust queue
        for i in 0..self.shape_queue.len()-1 {
            self.shape_queue[i] = self.shape_queue[i + 1];
        }

        // Remove last item in queue
        let _ = self.shape_queue.pop();

        // If queue is less then 7, add a new 7bag
        if self.shape_queue.len() < 7 {
            let mut new_bag = create_new_7_bag().to_vec();
            self.shape_queue.append(&mut new_bag);
        }

        // Debug print queue colors
        //stdout().queue(MoveTo(0,0)).unwrap();
        //stdout().queue(terminal::Clear(terminal::ClearType::CurrentLine)).unwrap();
        //for x in 0..self.shape_queue.len() {
        //    stdout().queue(MoveTo(2*x as u16,0)).unwrap();
        //    stdout().queue(style::Print(self.shape_queue[x].get_color().color_tile())).unwrap();
        //}

        new_shape
    }
}

/// Creates a new 7 bag array
pub fn create_new_7_bag() -> [Shape;7]{
    let mut new_queue: [Option<Shape>;7] = [None;7];

    // Assign each shape
    for x in 0..7 {
        // Loop until available shape found
        'new_shape: loop {
            // Get new shape
            let new_shape = Shape::random();

            // Check if shape exists in queue
            for shape in new_queue {
                if let Some(shape) = shape {
                    if shape == new_shape { // Shape already in queue
                        continue 'new_shape;
                    }
                }
            }

            // Shape not found in queue
            new_queue[x] = Some(new_shape);
            break 'new_shape;
        }
    }

    // Return queue
    new_queue.map(|shape| shape.unwrap())
}

