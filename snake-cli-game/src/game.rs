use anyhow::{anyhow, Result};
use rand::prelude::*;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub fn area(&self) -> u16 {
        self.width * self.height
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Location {
    pub x: u16,
    pub y: u16
}

impl Location {
    fn new(x: u16, y: u16) -> Location {
        Location {
            x, y
        }
    }

    fn zero() -> Location {
        Self::new(0, 0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Cell {
    Empty,
    Snake(Direction),
    Food,
    Block
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down
}

impl Direction {
    fn get_opposite_direction(&self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up
        }
    }

    fn get_next_location(&self, location: Location, size: Size) -> Location {
        match self {
            Direction::Left => Location::new((location.x + size.width - 1) % size.width, location.y),
            Direction::Right => Location::new((location.x + 1) % size.width, location.y),
            Direction::Up => Location::new(location.x, (location.y + size.height - 1) % size.height),
            Direction::Down => Location::new(location.x, (location.y + 1) % size.height),
        }
    }
}

struct Snake {
    head: Location,
    tail: Location,
}

impl Snake {
    fn head_direction(&self, field: &Field) -> Direction {
        self.cell_direction(self.head, field)
    }

    fn tail_direction(&self, field: &Field) -> Direction {
        self.cell_direction(self.tail, field)
    }

    fn cell_direction(&self, location: Location, field: &Field) -> Direction {
        let head_cell = field.get_location(location);
        match head_cell {
            Cell::Snake(direction) => direction,
            _ => panic!("Snake head is invalid")
        }
    }
}

pub struct Field {
    size: Size,
    cells: Vec<Cell>
}

impl Field {
    fn new(width: u16, height: u16) -> Field {
        let size = width * height;
        let cells: Vec<Cell> = vec![Cell::Empty; size as usize];

        Field {
            size: Size::new(width, height),
            cells
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn get(&self, x: u16, y: u16) -> Cell {
        let index = self.calc_index(x, y);
        self.cells[index]
    }

    fn get_location(&self, location: Location) -> Cell {
        self.get(location.x, location.y)
    }

    fn set(&mut self, x: u16, y: u16, cell: Cell) {
        let index = self.calc_index(x, y);
        self.cells[index] = cell;
    }

    fn set_location(&mut self, location: Location, cell: Cell) {
        self.set(location.x, location.y, cell);
    }

    fn calc_index(&self, x: u16, y: u16) -> usize {
        (y * self.size.width + x) as usize
    }

    fn reset(&mut self) {
        self.cells.fill(Cell::Empty);
    }
}

pub struct Game {
    field: Field,
    snake: Snake,
    score: u16,
    rnd: ThreadRng,
}

impl Game {
    pub fn new(width: u16, height: u16) -> Box<Game> {
        Box::new(Game {
            field: Field::new(width, height),
            snake: Snake {
                head: Location::zero(),
                tail: Location::zero(),
            },
            score: 0,
            rnd: rand::rng(),
        })
    }

    pub fn field(&self) -> &Field {
        &self.field
    }

    pub fn snake_head_location(&self) -> Location {
        self.snake.head
    }

    pub fn score(&self) -> u16 {
        self.score
    }

    pub fn prepare_level(&mut self) {
        self.field.reset();
        self.generate_blocks();
        self.generate_snake();
        self.generate_food();
        self.score = 0;
    }

    pub fn handle_next_step(&mut self, direction: Direction) -> Result<()> {
        self.set_snake_direction(direction);

        let next_head_direction = self.snake.head_direction(self.field());
        let next_head_location = next_head_direction.get_next_location(self.snake.head, self.field.size);

        if !self.is_snake_next_location_allowed(next_head_location) {
            return Err(anyhow!("Next step is not allowed"));
        }

        let cell = self.field.get_location(next_head_location);
        if let Cell::Food = cell {
            self.score += 1;
            self.increase_snake(next_head_location);
            self.generate_food();
        } else {
            self.move_snake(next_head_location);
        }

        Ok(())
    }

    fn is_snake_next_location_allowed(&self, next_location: Location) -> bool {
        let cell = self.field.get_location(next_location);
        match cell {
            Cell::Empty => true,
            Cell::Snake(_) => next_location == self.snake.tail,
            Cell::Food => true,
            Cell::Block => false
        }
    }

    fn set_snake_direction(&mut self, direction: Direction) {
        let current_direction  = self.snake.head_direction(self.field());

        if direction != current_direction &&
            direction != current_direction.get_opposite_direction() {
            self.field.set_location(self.snake.head, Cell::Snake(direction));
        }
    }

    fn move_snake(&mut self, next_location: Location) {
        let head_direction = self.snake.head_direction(self.field());

        let tail_direction = self.snake.tail_direction(self.field());
        let tail_next_location = tail_direction.get_next_location(self.snake.tail, self.field.size);

        self.field.set_location(self.snake.tail, Cell::Empty);
        self.field.set_location(next_location, Cell::Snake(head_direction));

        self.snake.head = next_location;
        self.snake.tail = tail_next_location;
    }

    fn increase_snake(&mut self, next_location: Location) {
        let current_direction  = self.snake.head_direction(self.field());

        self.field.set_location(next_location, Cell::Snake(current_direction));
        self.snake.head = next_location;
    }

    // Generate food by following algorithm (non-optimal):
    // - generate random value (RND) in range from 0 to field WxH;
    // - iterate thought all cells until we found free cell under index RND, we skip occupied cells and do not count them.
    fn generate_food(&mut self) {
        let area = self.field.size.area() as u32;
        let mut rnd = self.rnd.random::<u32>() % area;

        loop {
            for i in 0..self.field.size.height {
                for j in 0..self.field.size.width {
                    if let Cell::Empty = self.field.get(j, i) {
                        rnd -= 1;
                        if rnd <= 0 {
                            self.field.set(j, i, Cell::Food);
                            return
                        }
                    }
                }
            }
        }
    }

    // Simple generation blocks at the edges of the field
    fn generate_blocks(&mut self) {
        if self.rnd.random_bool(0.2) {
            // No borders at all
            return;
        }

        if self.rnd.random_bool(0.5) {
            // Left and right border
            for i in 0..self.field.size.height {
                self.field.set(0, i, Cell::Block);
            }

            for i in 0..self.field.size.height {
                self.field.set(self.field.size.width - 1, i, Cell::Block);
            }
        }

        if self.rnd.random_bool(0.5) {
            // Top and bottom border
            for i in 0..self.field.size.width {
                self.field.set(i, 0, Cell::Block);
            }

            for i in 0..self.field.size.width {
                self.field.set(i, self.field.size.height - 1, Cell::Block);
            }
        }
    }

    // Simple generation of two cell snake in the field center
    fn generate_snake(&mut self) {
        let row = self.field.size.height / 2;
        let head_col = self.field.size.width / 2;
        let tail_col = head_col - 1;

        self.field.set(head_col, row, Cell::Snake(Direction::Right));
        self.field.set(tail_col, row, Cell::Snake(Direction::Right));

        self.snake = Snake {
            head: Location::new(head_col, row),
            tail: Location::new(tail_col, row)
        }
    }
}