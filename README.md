# Jordtris
A simple recreation of Tetris in a terminal, made with rust🦀!

https://github.com/user-attachments/assets/c420b748-d0f3-4c4d-9161-9fd1bacbc217

## Features

- Rotation, movement and gravity
- Line Clearing
- Game Over Detection
- 7-bag piece queue
- SRS Rotation system

## Controls
| Key         | Action                     |
|-------------|----------------------------|
| ← / →       | Move piece left / right    |
| ↓           | Soft drop (faster fall)    |
| Space       | Hard drop                  |
| X / ↑       | Rotate Clockwise           |
| Z           | Rotate Counter Clockwise   |
| Ctrl + C    | Quit game                  |

## Upcoming Features

- Advanced scoring system
- Saved scores/scoreboard
- Pause menu

## Usage

### Release

1. Go to the [Releases Page](https://github.com/JordanJonThomas/jordtris/releases).
2. Download the binary (Available for windows only)
3. Run the executable! 

### Build it yourself

Make sure you have [Rust installed](https://www.rust-lang.org/tools/install).

```bash
git clone https://github.com/JordanJonThomas/jordtris
cd jordtris
cargo run
