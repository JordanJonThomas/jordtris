use core::time;
use std::{io::{self, stdout, Stdout, Write}, process::exit, thread::sleep, time::{Duration, Instant}};
use crossterm::{cursor::{self, MoveDown, MoveTo}, event::{poll, read, Event, KeyCode, KeyModifiers}, execute, style::{self, Print}, terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType}, QueueableCommand};
use game_state::{Coord, GamePhase, GameState};
use shapes::{Rotation, Shape};

// All shape logic
mod shapes;
mod game_state;

const INFO_WIDTH: usize = 16;

/// A direction
#[allow(unused)]
enum Direction {
    Left,
    Right,
    Up, 
    Down,
}

impl Direction {
    // Returns a value of 1 or -1 based on the direction
    pub fn to_value(&self) -> i16 {
        match self {
            Direction::Left => -1,
            Direction::Right => 1,
            Direction::Up => -1,
            Direction::Down => 1,
        }
    }
}

/// Cleans the program and exits
fn clean() {
    execute!( stdout(),
        cursor::Show,
        terminal::LeaveAlternateScreen,
        style::ResetColor,
    ).unwrap();
    let _ = disable_raw_mode();
    std::process::exit(0);
}

/// Setup program 
fn setup() -> GameState {
    let _ = enable_raw_mode().unwrap(); // Disable buffering

    // Prepare terminal
    execute!( stdout(),
        cursor::Hide,
        terminal::EnterAlternateScreen,
    ).unwrap();

    GameState::new() // Return new gamestate
}
/// Updates the game state based on player keypresses
fn update(game: &mut GameState) -> Result<(), io::Error> {
    // Get fall duration
    let fall_interval = Duration::from_millis(500); // TODO: Adjust for difficulty
    let lock_interval = Duration::from_millis(500); // 500ms lock delay 

    // Check if fall is requred
    if game.last_fall.elapsed() >= fall_interval {
        // Fall piece and update last fall
        game.last_fall = Instant::now();
        let fell = game.fall_player();

        // If piece could not fall, check lock delay
        if !fell && game.last_input.elapsed() >= lock_interval {
            game.place_and_reset();
        }
    }

    // Screen Event poll
    while poll(time::Duration::from_secs(0))? {
        // read event
        match read()? {
            // Keypress
            Event::Key(evt) => {
                // Move right
                if evt.code == KeyCode::Right && !evt.kind.is_release() {
                    // Try to move player
                    game.move_player_horizontal(Direction::Right);
                    game.last_input = Instant::now();
                }

                // Move left
                if evt.code == KeyCode::Left && !evt.kind.is_release() {
                    game.move_player_horizontal(Direction::Left);
                    game.last_input = Instant::now();
                }

                // Rotate cw (up)
                if (evt.code == KeyCode::Up || evt.code == KeyCode::Char('x')) && !evt.kind.is_release() {
                    game.rotate_player(Direction::Up);
                    game.last_input = Instant::now();
                }

                // Move player down
                if evt.code == KeyCode::Down && !evt.kind.is_release() {
                    let _ = game.fall_player();
                }

                // Rotate ccw (down)
                if evt.code == KeyCode::Char('z') && !evt.kind.is_release() {
                    game.rotate_player(Direction::Down);
                    game.last_input = Instant::now();
                }

                // Hard drop
                if evt.code == KeyCode::Char(' ') && !evt.kind.is_release() {
                    game.hard_drop();
                    game.last_input = Instant::now();
                }

                // Hold piece
                if evt.code == KeyCode::Char('c') && !evt.kind.is_release() {
                    game.hold();
                }

                // Control + c
                if evt.code == KeyCode::Char('c') 
                    && evt.modifiers.contains(KeyModifiers::CONTROL)
                {
                    clean(); // Clean and exit game
                }
            },

            // Ignore other events
            _ => {},
        }
    }

    Ok(())
}

/// Determines if the tile in an area overlaps with a player tile
fn is_player_tile(x: i16, y: i16, px: i16, py: i16, shape: &[[bool;4];4]) -> bool {
    if x >= px && x < px + 4 && y >= py && y < py + 4 {
        let local_x = x - px;
        let local_y = y - py;
        shape[local_y as usize][local_x as usize]
    } else {
        false
    }
}

/// Determines if a tile overlaps with a ghost preview
fn is_ghost_tile(x: usize, y: usize, gx: i16, py: i16, shape: &[[bool;4];4]) -> bool {
    for dy in 0..4 {
        for dx in 0..4 {
            if shape[dy][dx] {
                let gx = gx + dx as i16;
                let gy = py + dy as i16;

                if gx == x as i16 && gy == y as i16 {
                    return true;
                }
            }
        }
    }
    false
}

/// Appends the line at an index with a padding line for an info section
fn info_padding_line(frames: &mut Vec<String>, idx: usize) {
    if let Some(line) = frames.get_mut(idx) {
        *line = format!( 
            "{}  │{}│",
            line,
            " ".repeat(INFO_WIDTH),
        );
    }
}

/// Draws a frame of the game
fn draw(out: &mut Stdout, game: &GameState, previous_frame: &mut Vec<String>) -> Result<(), io::Error> {
    // Terminal size
    let size = terminal::size().expect("Could not get terminal");

    // Create game frame
    let mut frames: Vec<String> = vec![String::new(); 23];

    // Draw top line
    if let Some(line) = frames.get_mut(1) {
        *line = format!(
            "{}{}{}",
            "┌",
            "─".repeat(20),
            "┐"
        );
    }

    // Assemble frame
    let shape = game.current_shape.get_shape(&game.rotation);
    for y in 2..22 { // only render visible area
        let frame = frames.get_mut(y).unwrap();
        frame.push_str("│"); // Edge 

        // Render board pieces
        for x in 0..10 {
            if game.board[y][x].is_block() {
                *frame = format!("{}{}", frame, game.board[y][x].color_tile())
                //frame.push_str("██"); 
            } else if is_player_tile(x as i16, y as i16,
                game.player_pos.x, 
                game.player_pos.y, 
                &shape) {
                *frame = format!(
                    "{}{}",
                    frame,
                    game.current_shape.get_color().color_tile()
                );
                //frame.push_str("██");
            } else if is_ghost_tile(
                x, y,
                game.player_pos.x,
                game.get_drop_position(&shape), &shape){
                frame.push_str("░░");
            } else { // Empty space
                frame.push_str("  ");
            }
        }


        frame.push_str("│"); // edge
        //frames.push(frame);
    }

    // Bottom line
    if let Some(line) = frames.get_mut(22) {
        *line = format!( 
            "└{}┘",
            "─".repeat(20),
        );
    }

    // Draw score box
    if let Some(line) = frames.get_mut(1) {
        *line = format!( 
            "{}  ┌{}{}{}┐",
            line,
            "─".repeat(4),
            " POINTS ",
            "─".repeat(4),
        )
    }
    if let Some(line) = frames.get_mut(2) {
        let score = game.score.to_string();
        let total_pad = INFO_WIDTH - score.len();
        let left_pad = total_pad / 2;
        let right_pad = total_pad - left_pad;

        *line = format!( 
            "{}  │{}{}{}│",
            line,
            " ".repeat(left_pad),
            score,
            " ".repeat(right_pad),
        );
    }
    if let Some(line) = frames.get_mut(3) {
        *line = format!( 
            "{}  └{}┘",
            line,
            "─".repeat(INFO_WIDTH),
        );
    }

    // Held shape
    if let Some(line) = frames.get_mut(4) {
        *line = format!( 
            "{}  ┌{}{}{}┐",
            line,
            "─".repeat(5),
            " HOLD ",
            "─".repeat(5),
        )
    }
    info_padding_line(&mut frames, 5);
    let mut current_line = 6;

    // Draw held shape
    let shape = game.held;
    for x in 0..2 {
        if let Some(shape) = shape {
            // Offset for o shape
            let y = if let Shape::O = shape {1}else{0};

            // Get line
            let line = shape.get_shape(&Rotation::R0)[0+x+y];
            let color = shape.get_color();

            // Convert line to str
            let mut tile_str = String::new();
            for tile in line {
                if tile {
                    tile_str = format!("{}{}", tile_str, color.color_tile());
                }else {
                    tile_str = format!("{}  ", tile_str);
                }
            }

            if let Some(line) = frames.get_mut(current_line) {
                *line = format!(
                    "{}  │    {}    │",
                    line,
                    tile_str
                )
            }
        } else {
            info_padding_line(&mut frames, current_line);
        }
        current_line += 1;
    }

    // Finish held box
    info_padding_line(&mut frames, current_line);
    current_line+=1;
    if let Some(line) = frames.get_mut(current_line) {
        *line = format!( 
            "{}  └{}┘",
            line,
            "─".repeat(INFO_WIDTH),
        )
    }

    // Draw shape queue
    current_line +=1;
    if let Some(line) = frames.get_mut(current_line) {
        *line = format!( 
            "{}  ┌{}{}{}┐",
            line,
            "─".repeat(5),
            " NEXT ",
            "─".repeat(5),
        )
    }
    current_line +=1;
    info_padding_line(&mut frames, current_line);
    current_line +=1;
    for shape_idx in 0..3 { // Iterate shape queue
        let shape = game.shape_queue[shape_idx];
        for x in 0..2 {
            // Offset for o shape
            let y = if let Shape::O = shape {1}else{0};

            // Get line
            let line = shape.get_shape(&Rotation::R0)[0+x+y];
            let color = shape.get_color();

            // Convert line to str
            let mut tile_str = String::new();
            for tile in line {
                if tile {
                    tile_str = format!("{}{}", tile_str, color.color_tile());
                }else {
                    tile_str = format!("{}  ", tile_str);
                }
            }

            if let Some(line) = frames.get_mut(current_line) {
                *line = format!(
                    "{}  │    {}    │",
                    line,
                    tile_str
                )
            }
            current_line += 1;
        }
        info_padding_line(&mut frames, current_line);
        current_line += 1;
    }
    // Finish queue box
    if let Some(line) = frames.get_mut(current_line) {
        *line = format!( 
            "{}  └{}┘",
            line,
            "─".repeat(INFO_WIDTH),
        )
    }

    // Get size of play area
    let play_size = 20;

    // Draw frame lines
    for (y, frame) in frames.iter().enumerate() {
        // Only draw different lines
        if previous_frame.get(y) == Some(frame) {
            continue; 
        }

        // Draw
        out.queue(cursor::MoveTo(
            (size.0 / 2) - play_size as u16,
            (y as u16) + (size.1/2) - 15
        ))?;
        out.queue(style::Print(frame))?;

        // Update previous
        previous_frame[y] = frame.clone()
    }

    // flush term
    out.flush()
}

/// Waits for player input to determine next action
fn game_over_update(game: &mut GameState, out: &mut Stdout) -> Result<(), io::Error> {
    // Draw game over box
    // ┌───┐
    // │   │
    // └───┘
    let mut frames: Vec<String> = vec![];
    frames.push(format!("┌{}┐", "─".repeat(26)));
    frames.push("│        Gameover          │".to_string());
    frames.push("│   Press Ctrl+C to exit   │".to_string());
    frames.push("│ Any other key to restart │".to_string());
    frames.push(format!("└{}┘", "─".repeat(26)));

    let size = terminal::size().unwrap();
    let x = size.0/2 - 13;
    let y = size.1/2 - 2;
    out.queue(MoveTo(x,y))?;
    for (i, frame) in frames.iter().enumerate() {
        out.queue(MoveTo(x,y+i as u16))?;
        out.queue(Print(frame))?;
    }


    // Input
    while poll(time::Duration::from_secs(0))? {
        match read()? {
            Event::Key(evt) => {
                // Control + c
                if evt.code == KeyCode::Char('c') 
                    && evt.modifiers.contains(KeyModifiers::CONTROL)
                {
                    clean(); // Clean and exit game
                }

                // Any other key restarts
                *game = GameState::new();
                out.queue(Clear(ClearType::All))?;
            },

            // Ignore other events
            _ => ()
        }
    }
    Ok(())
}

/// Program entry point
fn main() -> Result<(), io::Error> {
    let mut state = setup(); // Set up game
    let frame_time = Duration::from_secs_f64(1.0 / 24.0);
    let mut out = stdout();

    let mut previous_frame: Vec<String> = vec![String::new(); 23];

    // Enter game loop
    loop {
        if let GamePhase::Playing = state.game_phase { // Playing
            // Get current time
            let start = Instant::now();

            // Game logic
            draw(&mut out, &state, &mut previous_frame)?; // Draw game
            update(&mut state)?; // Update game

            // Wait for frame
            let elapsed = start.elapsed();
            if elapsed < frame_time {
                sleep(frame_time - elapsed);
            }
        } else if let GamePhase::GameOver = state.game_phase { // Game over
            // Game over screen
            previous_frame = vec![String::new(); 23]; // Reset frames to avoid printing
            // bug
            game_over_update(&mut state, &mut out)?; // Game over update screen
            // TODO: Score screen?
        }
    }
}
