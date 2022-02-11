// use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{Button, Key, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::PressEvent;
use piston_window::PistonWindow as Window;

use std::collections::VecDeque;

type Color = [f32; 4];

const BODY_COLOR: Color = [0.8, 0.8, 0.8, 1.0];
const BACKGROUND_COLOR: Color = [0.0, 0.0, 0.0, 1.0];
const FOOD_COLOR: Color = [1.0, 0.0, 0.0, 1.0];

#[derive(PartialEq, Debug)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
    None,
}

impl Direction {
    // Returns true if the snake can move in the `other` direction
    // (a.k.a if self and other are perpendicular)
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
    // Warning: The head is at the back
    body: VecDeque<(u8, u8)>,
    grid_size: (u8, u8),
    dir: Direction,
    // The time since the snake moved for the last time.
    snake_last_move: f64,
    // Length to add to the tail
    growing: u32,
    growth_rate: u32,
    tick_rate: f64,
    // If the direction has already changed this tick. Prevents from going
    // in two directions in one tick.
    dir_changed: bool,
    food_position: (u8, u8),
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
                    } else if (x, y) == self.food_position {
                        rectangle(FOOD_COLOR, rec, transform, gl);
                    } else {
                        rectangle(BACKGROUND_COLOR, rec, transform, gl);
                    }
                }
            }
        })
    }
    pub fn update(&mut self, args: &UpdateArgs) -> Result<(), ()> {
        self.snake_last_move += args.dt;
        if self.snake_last_move >= self.tick_rate {
            self.snake_last_move %= self.tick_rate;
            self.dir_changed = false;
            let head = *self.body.back().unwrap();

            let next_pos;
            match self.dir {
                Direction::None => return Ok(()),
                Direction::Up => {
                    if head.1 == 0 {
                        return Err(());
                    }
                    next_pos = (head.0, head.1 - 1);
                }
                Direction::Down => {
                    next_pos = (head.0, head.1 + 1);
                    if next_pos.1 >= self.grid_size.1 {
                        return Err(());
                    }
                }
                Direction::Left => {
                    if head.0 == 0 {
                        return Err(());
                    }
                    next_pos = (head.0 - 1, head.1);
                }
                Direction::Right => {
                    next_pos = (head.0 + 1, head.1);
                    if next_pos.0 == self.grid_size.0 {
                        return Err(());
                    }
                }
            }
            if next_pos == self.food_position {
                self.generate_new_food();
                self.growing += self.growth_rate;
                println!("Score: {}", self.body.len());
            }
            if self.growing == 0 {
                self.body.pop_front();
            } else {
                self.growing -= 1;
            }
            if self.body.contains(&next_pos) {
                return Err(());
            }
            self.body.push_back(next_pos);
        }
        Ok(())
    }

    pub fn new(gl: GlGraphics, grid_size: (u8, u8), tick_rate: f64) -> Self {
        let mut body = VecDeque::new();
        let center = (grid_size.0 / 2, grid_size.1 / 2);
        body.push_back(center);
        let mut food_position = center;
        while food_position == center {
            food_position = (fastrand::u8(0..grid_size.0), fastrand::u8(0..grid_size.1));
        }

        Self {
            gl,
            body,
            dir: Direction::None,
            snake_last_move: 0.0,
            grid_size,
            growing: 4,
            growth_rate: 4,
            tick_rate,
            dir_changed: false,
            food_position,
        }
    }

    pub fn generate_new_food(&mut self) {
        let mut food_pos = *self.body.front().unwrap();
        while self.body.contains(&food_pos) {
            food_pos = (
                fastrand::u8(0..self.grid_size.0),
                fastrand::u8(0..self.grid_size.1),
            );
        }
        self.food_position = food_pos;
    }
}

fn main() {
    let open_gl = OpenGL::V3_3;
    let mut window: Window = WindowSettings::new("Snake game", [400, 400])
        .graphics_api(open_gl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut glyph_cache = GlyphCache::new(
        "assets/FiraSans-Regular.ttf",
        (),
        opengl_graphics::TextureSettings::new(),
    )
    .unwrap();

    let mut should_continue = true;

    while should_continue {
        let mut app = App::new(GlGraphics::new(open_gl), (32, 32), 1.0 / 10.0);
        let mut event = Events::new(EventSettings::new());
        while let Some(e) = event.next(&mut window) {
            if let Some(args) = e.render_args() {
                app.render(&args);
            }
            if let Some(args) = e.update_args() {
                if app.update(&args).is_err() {
                    should_continue = true;
                    break;
                }
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
                        app.growing += 1;
                        app.dir_changed = true;
                    }
                    _ => (),
                }
            }
        }
        if should_continue {
            should_continue = false;
            while let Some(e) = event.next(&mut window) {
                if let Some(args) = e.render_args() {
                    app.gl.draw(args.viewport(), |c, gl| {
                        use graphics::*;

                        clear(BACKGROUND_COLOR, gl);
                        text::Text::new_color(BODY_COLOR, (args.viewport().rect[2] / 10) as u32)
                            .draw(
                                "    Game Over",
                                &mut glyph_cache,
                                &DrawState::default(),
                                c.transform
                                    .trans(10.0, (args.viewport().rect[3] as f32 / 2.2) as f64),
                                gl,
                            )
                            .unwrap();
                        text::Text::new_color(BODY_COLOR, (args.viewport().rect[2] / 23) as u32)
                            .draw(
                                "Press Space to play a new game or esc to exit...",
                                &mut glyph_cache,
                                &DrawState::default(),
                                c.transform
                                    .trans(10.0, (args.viewport().rect[3] as f32 / 1.8) as f64),
                                gl,
                            )
                            .unwrap();
                    });
                }
                if let Some(Button::Keyboard(Key::Space)) = e.press_args() {
                    should_continue = true;
                    break;
                }
            }
        }
    }
}
