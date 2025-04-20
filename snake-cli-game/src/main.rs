mod game;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::io::{stdout, Stdout, Write};
use std::time::{Duration, Instant};

use crate::game::{Cell, Direction, Game};

fn main() {
    let (term_width, term_height) = terminal::size().expect("Failed to get terminal size");

    let mut game = Game::new(term_width, term_height - 1);
    let mut direction = Direction::Right;

    let stdout = stdout();
    let mut renderer = GameRenderer::new(&stdout);

    game.prepare_level();
    renderer.prepare();

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

            renderer.render(&game);

            last_update = Instant::now();
        }
    }
}

pub struct GameRenderer<'a> {
    output: String,
    stdout: &'a Stdout,
}

impl<'a> GameRenderer<'a> {
    pub fn new(stdout: &'a Stdout) -> GameRenderer<'a> {
        GameRenderer {
            output: String::new(),
            stdout,
        }
    }

    pub fn prepare(&mut self) {
        execute!(self.stdout, EnterAlternateScreen, Hide).unwrap();
        terminal::enable_raw_mode().unwrap();
    }

    pub fn render(&mut self, game: &Game) {
        execute!(self.stdout, MoveTo(0, 0)).unwrap();

        let game_size = game.field().size();

        let capacity = (game_size.area() + 2 * game_size.height + 100) as usize;
        if self.output.capacity() < capacity {
            self.output.reserve(capacity - self.output.capacity());
        }

        let head = game.snake_head_location();

        self.output.clear();
        for y in 0..game_size.height {
            for x in 0..game_size.width {
                let cell = game.field().get(x, y);
                match cell {
                    Cell::Empty => self.output.push(' '),
                    Cell::Snake(_) => {
                        let ch = if x == head.x && y == head.y { '☻' } else { '☺' };
                        self.output.push(ch)
                    },
                    Cell::Block => self.output.push('█'),
                    Cell::Food => self.output.push('♥'),
                }
            }
            self.output.push_str("\r\n");
        }
        self.output.push_str(&format!("Score: {}", game.score()));

        self.stdout.write_all(self.output.as_bytes()).unwrap();
        self.stdout.flush().unwrap();
    }
}

impl<'a> Drop for GameRenderer<'a> {
    fn drop(&mut self) {
        execute!(self.stdout, Show, LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap();
    }
}