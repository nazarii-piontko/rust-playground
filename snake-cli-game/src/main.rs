mod game;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::io::{stdout, Write};
use std::time::{Duration, Instant};

use crate::game::{Cell, Direction, Game};

fn main() {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide).unwrap();
    terminal::enable_raw_mode().unwrap();

    let (term_width, term_height) = terminal::size().expect("Failed to get terminal size");

    let mut game = Game::new(term_width, term_height - 1);
    game.prepare_level();

    let mut direction = Direction::Right;

    let game_size = game.field().size();

    let mut output = String::with_capacity((game_size.area() + 2 * game_size.height + 100) as usize);

    let mut step_interval = Duration::from_millis(150);
    let mut last_update = Instant::now();

    loop {
        if event::poll(Duration::from_millis(50)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Up => direction = Direction::Up,
                    KeyCode::Down => direction = Direction::Down,
                    KeyCode::Left => direction = Direction::Left,
                    KeyCode::Right => direction = Direction::Right,
                    KeyCode::Char('+') => if step_interval > Duration::from_millis(50) { step_interval -= Duration::from_millis(50) },
                    KeyCode::Char('-') => step_interval += Duration::from_millis(50),
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }

        if last_update.elapsed() >= step_interval {
            if game.handle_next_step(direction).is_err() {
                game.prepare_level();
                direction = Direction::Right;
            }

            execute!(stdout, MoveTo(0, 0)).unwrap();

            // Render
            output.clear();
            let head = game.snake_head_location();
            for y in 0..game_size.height {
                for x in 0..game_size.width {
                    let cell = game.field().get(x, y);
                    match cell {
                        Cell::Empty => output.push(' '),
                        Cell::Snake(_) => {
                            let ch = if x == head.x && y == head.y { '☻' } else { '☺' };
                            output.push(ch)
                        },
                        Cell::Block => output.push('█'),
                        Cell::Food => output.push('♥'),
                    }
                }
                output.push_str("\r\n");
            }
            output.push_str(&format!("Score: {}", game.score()));

            stdout.write_all(output.as_bytes()).unwrap();
            stdout.flush().unwrap();

            last_update = Instant::now();
        }
    }

    execute!(stdout, Show, LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
}
