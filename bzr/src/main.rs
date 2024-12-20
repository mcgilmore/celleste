use clap::Arg;
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Canvas, Color, DrawParam, Mesh};
use ggez::{
    input::keyboard::{KeyCode, KeyInput},
    input::mouse::MouseButton,
    Context, ContextBuilder, GameResult,
};

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
    width: usize,
    height: usize,
    diff_a: f32,
    diff_b: f32,
    diff_c: f32,
    feed: f32,
    kill: f32,
}

impl BelousovZhabotinsky {
    fn new(
        width: usize,
        height: usize,
        cell_size: f32,
        diff_a: f32,
        diff_b: f32,
        diff_c: f32,
        feed: f32,
        kill: f32,
    ) -> Self {
        let mut grid = vec![
            vec![
                Cell {
                    a: 1.0,
                    b: 0.0,
                    c: 0.0
                };
                width
            ];
            height
        ];

        // Seed the initial reaction in the center
        let center_y = height / 2;
        let center_x = width / 2;

        for y in (center_y - 5)..(center_y + 5) {
            for x in (center_x - 5)..(center_x + 5) {
                if y < height && x < width {
                    grid[y][x] = Cell {
                        a: 0.5,
                        b: 0.25,
                        c: 0.0,
                    };
                }
            }
        }

        Self {
            next_grid: grid.clone(),
            grid,
            running: true,
            cell_size,
            width,
            height,
            diff_a,
            diff_b,
            diff_c,
            feed,
            kill,
        }
    }

    fn step(&mut self) {
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let cell = self.grid[y][x];

                // Calculate diffusion
                let laplace_a = self.laplace(x, y, |c| c.a);
                let laplace_b = self.laplace(x, y, |c| c.b);
                let laplace_c = self.laplace(x, y, |c| c.c);

                // Reaction-diffusion equations
                let reaction_ab = cell.a * cell.b * cell.b;
                let reaction_bc = cell.b * cell.c * cell.c;

                let new_a =
                    cell.a + (self.diff_a * laplace_a - reaction_ab + self.feed * (1.0 - cell.a));
                let new_b = cell.b
                    + (self.diff_b * laplace_b + reaction_ab
                        - reaction_bc
                        - (self.kill + self.feed) * cell.b);
                let new_c = cell.c + (self.diff_c * laplace_c + reaction_bc - self.kill * cell.c);

                self.next_grid[y][x] = Cell {
                    a: new_a.clamp(0.0, 1.0),
                    b: new_b.clamp(0.0, 1.0),
                    c: new_c.clamp(0.0, 1.0),
                };
            }
        }
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

    fn seed_reaction(&mut self, x: f32, y: f32) {
        let grid_x = (x / self.cell_size).floor() as usize;
        let grid_y = (y / self.cell_size).floor() as usize;

        if grid_x < self.width && grid_y < self.height {
            for dy in -3..=3 {
                for dx in -3..=3 {
                    let nx = grid_x as isize + dx;
                    let ny = grid_y as isize + dy;
                    if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                        self.grid[ny as usize][nx as usize] = Cell {
                            a: 0.5,
                            b: 0.25,
                            c: 0.0,
                        };
                    }
                }
            }
        }
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

        let mut mesh_builder = graphics::MeshBuilder::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.grid[y][x];
                let intensity_a = (cell.a.clamp(0.0, 1.0) * 160.0) as u8;
                let intensity_b = (cell.b.clamp(0.0, 1.0) * 255.0) as u8;
                let intensity_c = (cell.c.clamp(0.0, 1.0) * 255.0) as u8;

                let color = Color::from_rgb(intensity_a, intensity_b, intensity_c);

                let rect = graphics::Rect::new(
                    x as f32 * self.cell_size,
                    y as f32 * self.cell_size,
                    self.cell_size,
                    self.cell_size,
                );

                mesh_builder.rectangle(graphics::DrawMode::fill(), rect, color);
            }
        }

        let mesh_data = mesh_builder.build(); // Create MeshData
        let mesh = Mesh::from_data(ctx, mesh_data); // Convert MeshData to Mesh
        canvas.draw(&mesh, DrawParam::default());
        canvas.finish(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        key_input: KeyInput,
        _repeat: bool,
    ) -> GameResult {
        if let Some(keycode) = key_input.keycode {
            if keycode == KeyCode::Space {
                self.running = !self.running;
            }
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if button == MouseButton::Left {
            self.seed_reaction(x, y);
        }
        Ok(())
    }
}

fn main() -> GameResult {
    let matches = clap::Command::new("Belousov-Zhabotinsky Reaction")
        .arg(
            Arg::new("width")
                .long("width")
                .value_parser(clap::value_parser!(usize))
                .default_value("400")
                .help("Set the grid width"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .value_parser(clap::value_parser!(usize))
                .default_value("400")
                .help("Set the grid height"),
        )
        .arg(
            Arg::new("diff_a")
                .long("diff_a")
                .value_parser(clap::value_parser!(f32))
                .default_value("1.0")
                .help("Set the diffusion rate for chemical A"),
        )
        .arg(
            Arg::new("diff_b")
                .long("diff_b")
                .value_parser(clap::value_parser!(f32))
                .default_value("0.5")
                .help("Set the diffusion rate for chemical B"),
        )
        .arg(
            Arg::new("diff_c")
                .long("diff_c")
                .value_parser(clap::value_parser!(f32))
                .default_value("0.3")
                .help("Set the diffusion rate for chemical C"),
        )
        .arg(
            Arg::new("feed")
                .long("feed")
                .value_parser(clap::value_parser!(f32))
                .default_value("0.055")
                .help("Set the feed rate"),
        )
        .arg(
            Arg::new("kill")
                .long("kill")
                .value_parser(clap::value_parser!(f32))
                .default_value("0.062")
                .help("Set the kill rate"),
        )
        .get_matches();

    let width = *matches.get_one::<usize>("width").unwrap();
    let height = *matches.get_one::<usize>("height").unwrap();
    let diff_a = *matches.get_one::<f32>("diff_a").unwrap();
    let diff_b = *matches.get_one::<f32>("diff_b").unwrap();
    let diff_c = *matches.get_one::<f32>("diff_c").unwrap();
    let feed = *matches.get_one::<f32>("feed").unwrap();
    let kill = *matches.get_one::<f32>("kill").unwrap();

    let screen_width = 800.0; // Screen dimensions
    let screen_height = 800.0;

    let cell_size = (screen_width / width as f32).min(screen_height / height as f32);

    let cb = ContextBuilder::new("bzr", "Author")
        .window_setup(ggez::conf::WindowSetup::default().title("bzr"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(screen_width, screen_height));
    let (ctx, event_loop) = cb.build()?;

    let game =
        BelousovZhabotinsky::new(width, height, cell_size, diff_a, diff_b, diff_c, feed, kill);
    event::run(ctx, event_loop, game)
}
