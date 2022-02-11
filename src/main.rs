use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{Button, Key, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::PressEvent;

use std::collections::VecDeque;

type Color = [f32; 4];

const BODY_COLOR: Color = [0.8, 0.8, 0.8, 1.0];
const BACKGROUND_COLOR: Color = [0.0, 0.0, 0.0, 1.0];
// const FOOD_COLOR: Color = [1.0, 0.0, 0.0, 1.0];

#[derive(PartialEq, Debug)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
    None,
}

impl Direction {
    fn can_move(&self, other: &Direction) -> bool {
        match self {
            Direction::Up | Direction::Down
                if *other == Direction::Up || *other == Direction::Down =>
            {
                false
            }
            Direction::Left | Direction::Right
                if *other == Direction::Left || *other == Direction::Right =>
            {
                false
            }
            Direction::None => true,
            _ => true,
        }
    }
}

struct App {
    gl: GlGraphics,
    body: VecDeque<(u8, u8)>,
    grid_size: (u8, u8),
    dir: Direction,
    // The time since the snake moved for the last time.
    snake_last_move: f64,
    // TODO: change that to an interger
    growing: bool,
    snake_update_time: f64,
    dir_changed: bool,
}

impl App {
    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;
        let height = args.window_size[0] / self.grid_size.0 as f64;
        let width = args.window_size[1] / self.grid_size.1 as f64;
        let rec = rectangle::rectangle_by_corners(0.0, 0.0, height, width);

        self.gl.draw(args.viewport(), |c, gl| {
            for x in 0..self.grid_size.0 {
                for y in 0..self.grid_size.1 {
                    let transform = c.transform.trans(x as f64 * height, y as f64 * width);
                    if self.body.contains(&(x, y)) {
                        rectangle(BODY_COLOR, rec, transform, gl);
                    } else {
                        rectangle(BACKGROUND_COLOR, rec, transform, gl);
                    }
                }
            }
        })
    }
    pub fn update(&mut self, args: &UpdateArgs) {
        self.snake_last_move += args.dt;
        if self.snake_last_move >= self.snake_update_time {
            self.snake_last_move %= self.snake_update_time;
            self.dir_changed = false;
            let head = *self.body.back().unwrap();
            if let Direction::None = self.dir {
                return;
            }
            let next_pos;
            match self.dir {
                Direction::Up => {
                    next_pos = (head.0, head.1 - 1);
                    if head.1 == 0 {
                        panic!("Cannot move up more");
                    }
                }
                Direction::Down => {
                    next_pos = (head.0, head.1 + 1);
                    if next_pos.1 >= self.grid_size.1 {
                        panic!("Cannot go down more");
                    }
                }
                Direction::Left => {
                    next_pos = (head.0 - 1, head.1);
                    if head.0 == 0 {
                        panic!("Cannot go left more");
                    }
                }
                Direction::Right => {
                    next_pos = (head.0 + 1, head.1);
                    if next_pos.0 == self.grid_size.0 {
                        panic!("Cannot go right more");
                    }
                }
                #[allow(unreachable_code)]
                Direction::None => next_pos = unreachable!(),
            }
            if !self.growing {
                self.body.pop_front();
            } else {
                self.growing = false;
            }
            if self.body.contains(&next_pos) {
                panic!("Ran into own tail");
            }
            self.body.push_back(next_pos);
        }
    }
    pub fn new(gl: GlGraphics, grid_size: (u8, u8)) -> Self {
        Self {
            gl,
            body: {
                let mut a = VecDeque::new();
                a.push_back((grid_size.0 / 2, grid_size.1 / 2));
                a
            },
            dir: Direction::None,
            snake_last_move: 0.0,
            grid_size,
            growing: false,
            snake_update_time: 0.4,
            dir_changed: false,
        }
    }
}

fn main() {
    let open_gl = OpenGL::V3_3;
    let mut window: Window = WindowSettings::new("Snake game", [400, 400])
        .graphics_api(open_gl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = App::new(GlGraphics::new(open_gl), (16, 16));

    let mut event = Events::new(EventSettings::new());
    while let Some(e) = event.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }
        if let Some(args) = e.update_args() {
            app.update(&args);
        }
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Up | Key::Z if app.dir.can_move(&Direction::Up) => {
                    if !app.dir_changed {
                        app.dir = Direction::Up;
                    }
                    app.dir_changed = true;
                }
                Key::Down | Key::S if app.dir.can_move(&Direction::Down) => {
                    if !app.dir_changed {
                        app.dir = Direction::Down;
                    }
                    app.dir_changed = true;
                }
                Key::Left | Key::Q if app.dir.can_move(&Direction::Left) => {
                    if !app.dir_changed {
                        app.dir = Direction::Left;
                    }
                    app.dir_changed = true;
                }
                Key::Right | Key::D if app.dir.can_move(&Direction::Right) => {
                    if !app.dir_changed {
                        app.dir = Direction::Right;
                    }
                    app.dir_changed = true;
                }
                #[cfg(debug_assertions)]
                Key::Space => {
                    app.growing = true;
                    app.dir_changed = true;
                }
                _ => (),
            }
        }
    }
}
