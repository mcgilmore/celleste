use ggez::{Context, ContextBuilder, GameResult, input::keyboard::{KeyCode, KeyInput}};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Canvas, Color, DrawParam, Mesh};
use std::ops::Add;

const WIDTH: usize = 400;
const HEIGHT: usize = 400;
const DIFF_A: f32 = 2.0; // Diffusion rate for chemical A
const DIFF_B: f32 = 1.0; // Diffusion rate for chemical B
const DIFF_C: f32 = 0.6; // Diffusion rate for chemical C
const FEED: f32 = 0.04;
const KILL: f32 = 0.062;

#[derive(Clone, Copy)]
struct Cell {
    a: f32,
    b: f32,
    c: f32,
}

struct BelousovZhabotinsky {
    grid: Vec<Vec<Cell>>,
    next_grid: Vec<Vec<Cell>>,
    running: bool,
    cell_size: f32,
}

impl BelousovZhabotinsky {
    fn new(cell_size: f32) -> Self {
        let mut grid = vec![vec![Cell { a: 1.0, b: 0.0, c: 0.0 }; WIDTH]; HEIGHT];

        // Seed the initial reaction in the center
        for y in (HEIGHT / 2 - 10)..(HEIGHT / 2 + 10) {
            for x in (WIDTH / 2 - 10)..(WIDTH / 2 + 10) {
                grid[y][x] = Cell { a: 0.5, b: 0.25, c: 0.0 };
            }
        }

        Self {
            next_grid: grid.clone(),
            grid,
            running: true,
            cell_size,
        }
    }

    fn step(&mut self) {
        for y in 1..HEIGHT - 1 {
            for x in 1..WIDTH - 1 {
                let cell = self.grid[y][x];

                // Calculate diffusion
                let laplace_a = self.laplace(x, y, |c| c.a);
                let laplace_b = self.laplace(x, y, |c| c.b);
                let laplace_c = self.laplace(x, y, |c| c.c);

                // Reaction-diffusion equations
                let reaction_ab = cell.a * cell.b * cell.b;
                let reaction_bc = cell.b * cell.c * cell.c;

                let new_a = cell.a + (DIFF_A * laplace_a - reaction_ab + FEED * (1.0 - cell.a));
                let new_b = cell.b + (DIFF_B * laplace_b + reaction_ab - reaction_bc - (KILL + FEED) * cell.b);
                let new_c = cell.c + (DIFF_C * laplace_c + reaction_bc - KILL * cell.c);

                self.next_grid[y][x] = Cell {
                    a: new_a.clamp(0.0, 1.0),
                    b: new_b.clamp(0.0, 1.0),
                    c: new_c.clamp(0.0, 1.0),
                };
            }
        }

        // Swap the grids
        std::mem::swap(&mut self.grid, &mut self.next_grid);
    }

    fn laplace<F>(&self, x: usize, y: usize, f: F) -> f32
    where
        F: Fn(Cell) -> f32,
    {
        let mut sum = 0.0;
        sum += f(self.grid[y][x]) * -1.0;
        sum += f(self.grid[y - 1][x]) * 0.2;
        sum += f(self.grid[y + 1][x]) * 0.2;
        sum += f(self.grid[y][x - 1]) * 0.2;
        sum += f(self.grid[y][x + 1]) * 0.2;
        sum += f(self.grid[y - 1][x - 1]) * 0.05;
        sum += f(self.grid[y - 1][x + 1]) * 0.05;
        sum += f(self.grid[y + 1][x - 1]) * 0.05;
        sum += f(self.grid[y + 1][x + 1]) * 0.05;
        sum
    }
}

impl EventHandler for BelousovZhabotinsky {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.running {
            self.step();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let cell = self.grid[y][x];
                let intensity_a = (cell.a.clamp(0.0, 1.0) * 255.0) as u8;
                let intensity_b = (cell.b.clamp(0.0, 1.0) * 255.0) as u8;
                let intensity_c = (cell.c.clamp(0.0, 1.0) * 255.0) as u8;

                let color = Color::from_rgb(intensity_a, intensity_b, intensity_c);

                let rect = graphics::Rect::new(
                    x as f32 * self.cell_size,
                    y as f32 * self.cell_size,
                    self.cell_size,
                    self.cell_size,
                );
                let rectangle = Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, color)?;
                canvas.draw(&rectangle, DrawParam::default());
            }
        }

        canvas.finish(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, key_input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(keycode) = key_input.keycode {
            if keycode == KeyCode::Space {
                self.running = !self.running;
            }
        }
        Ok(())
    }
}

fn main() -> GameResult {
    let cb = ContextBuilder::new("bz_reaction", "Author")
        .window_setup(ggez::conf::WindowSetup::default().title("Belousov-Zhabotinsky Reaction"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 800.0));
    let (ctx, event_loop) = cb.build()?;

    let mut game = BelousovZhabotinsky::new(4.0);
    event::run(ctx, event_loop, game)
}
