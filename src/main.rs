extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use rand::Rng;
use std::collections::LinkedList;
use std::f64;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use piston::{Button, ButtonEvent, ButtonState, EventLoop, Key};

pub struct App {
    gl: GlGraphics,
    snake: Snake,
    food: Food,
    item_size: f64,
    width: f64,
    height: f64,
    ended: bool,
}

impl App {
    fn init(opengl: OpenGL, width: f64, height: f64, item_size: f64) -> Self {
        let food_pos = random_pos(width, height, item_size);
        let snake_pos = random_pos(width, height, item_size);

        App {
            gl: GlGraphics::new(opengl),
            snake: Snake::new(15.0, snake_pos),
            food: Food::new(food_pos, item_size),
            item_size,
            width: width as f64,
            height: height as f64,
            ended: false,
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        self.gl.draw(args.viewport(), |_c, gl| {
            graphics::clear(WHITE, gl);
        });

        self.snake.render(&mut self.gl, args);
        self.food.render(&mut self.gl, args)
    }

    fn update(&mut self) {
        match self
            .snake
            .update(&self.food.position, self.width, self.height)
        {
            SnakeMoveResult::Ok => return,
            SnakeMoveResult::Food => {
                self.food
                    .reset(self.width, self.height, self.item_size, &self.snake)
            }
            SnakeMoveResult::End => self.ended = true,
        }
    }

    fn handle_input(&mut self, btn: &Button) {
        let last_direction = self.snake.direction.clone();

        self.snake.direction = match btn {
            &Button::Keyboard(Key::Up) if last_direction != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down) if last_direction != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Left) if last_direction != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right) if last_direction != Direction::Left => Direction::Right,
            _ => last_direction,
        }
    }
}

#[derive(Clone, PartialEq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

struct Position {
    x: f64,
    y: f64,
}

enum SnakeMoveResult {
    Ok,
    Food,
    End,
}

struct Snake {
    body: LinkedList<Position>,
    size: f64,
    color: [f32; 4],
    direction: Direction,
}

impl Snake {
    fn new(size: f64, initial_pos: Position) -> Self {
        Snake {
            body: LinkedList::from([initial_pos]),
            size,
            color: [1.0, 0.0, 0.0, 1.0],
            direction: Direction::Right,
        }
    }

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        let squares: Vec<graphics::types::Rectangle> = self
            .body
            .iter()
            .map(|pos| graphics::rectangle::square(pos.x, pos.y, self.size))
            .collect();

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            squares
                .into_iter()
                .for_each(|square| graphics::rectangle(self.color, square, transform, gl))
        });
    }
    fn update(&mut self, food_pos: &Position, width: f64, height: f64) -> SnakeMoveResult {
        let mut new_x = self.body.front().expect("Snake has no body").x;
        let mut new_y = self.body.front().expect("Snake has no body").y;
        match self.direction {
            Direction::Left => new_x -= self.size,
            Direction::Right => new_x += self.size,
            Direction::Up => new_y -= self.size,
            Direction::Down => new_y += self.size,
        }

        let mut new_pos = Position { x: new_x, y: new_y };

        if new_pos.x >= width {
            new_pos.x = 0.0;
        } else if new_pos.x < 0.0 {
            new_pos.x = width - self.size;
        }

        if new_pos.y >= height {
            new_pos.y = 0.0;
        } else if new_pos.y < 0.0 {
            new_pos.y = height - self.size;
        }

        if self.self_collision(&new_pos) {
            return SnakeMoveResult::End;
        } else {
            self.body.push_front(new_pos);

            if new_x == food_pos.x && new_y == food_pos.y {
                return SnakeMoveResult::Food;
            } else {
                self.body.pop_back().unwrap();
                return SnakeMoveResult::Ok;
            }
        }
    }

    fn self_collision(&self, new_pos: &Position) -> bool {
        self.body
            .iter()
            .any(|pos| pos.x == new_pos.x && pos.y == new_pos.y)
    }
}

struct Food {
    position: Position,
    size: f64,
    color: [f32; 4],
}

impl Food {
    fn new(initial_pos: Position, item_size: f64) -> Self {
        Food {
            position: initial_pos,
            size: item_size,
            color: [0.0, 1.0, 0.0, 1.0],
        }
    }
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        let square = graphics::rectangle::square(self.position.x, self.position.y, self.size);

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            graphics::rectangle(self.color, square, transform, gl)
        });
    }

    fn reset(&mut self, width: f64, height: f64, item_size: f64, snake: &Snake) {
        // TODO Check against snake body
        let new_pos = random_pos(width, height, item_size);
        if snake.self_collision(&new_pos) {
            self.reset(width, height, item_size, snake);
        } else {
            self.position = new_pos;
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    const WIDTH: f64 = 300.0;
    const HEIGHT: f64 = 300.0;
    const ITEM_SIZE: f64 = 15.0;

    // * Creates a window
    let mut window: Window = WindowSettings::new("Snake game", [WIDTH, HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = App::init(opengl, WIDTH, HEIGHT, ITEM_SIZE);
    let mut events = Events::new(EventSettings::new()).ups(10);
    while let Some(e) = events.next(&mut window) {
        if app.ended {
            break;
        }
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(_args) = e.update_args() {
            app.update();
        }
        if let Some(args) = e.button_args() {
            if args.state == ButtonState::Press {
                app.handle_input(&args.button);
            }
        }
    }
}

fn random_pos(width: f64, height: f64, item_size: f64) -> Position {
    let mut rng = rand::thread_rng();
    let grid_size_height = (height / item_size) - 1.0;
    let grid_size_height = grid_size_height.floor() as i64;
    let grid_size_width = (width / item_size) - 1.0;
    let grid_size_width = grid_size_width.floor() as i64;
    let rand_x = rng.gen_range(0..grid_size_width) as f64;
    let rand_y = rng.gen_range(0..grid_size_height) as f64;

    let rand_x = rand_x * item_size;
    let rand_y = rand_y * item_size;

    Position {
        x: rand_x,
        y: rand_y,
    }
}
