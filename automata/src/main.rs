use ggez::{Context, ContextBuilder, GameResult, input::keyboard::{KeyCode, KeyInput}, input::mouse::MouseButton};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Canvas, Color, DrawMode, DrawParam, Mesh};
use ggez::GameError;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::env;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize)]
struct Cell(i32, i32);

struct Rules {
    birth: Vec<usize>,
    survival: Vec<usize>,
}

impl Rules {
    fn from_string(rule_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = rule_str.split('/').collect();
        if parts.len() != 2 || !parts[0].starts_with('B') || !parts[1].starts_with('S') {
            return Err("Invalid rule format. Expected 'B<number>/S<number>'.".to_string());
        }

        let birth = parts[0][1..]
            .chars()
            .filter_map(|c| c.to_digit(10))
            .map(|d| d as usize)
            .collect();

        let survival = parts[1][1..]
            .chars()
            .filter_map(|c| c.to_digit(10))
            .map(|d| d as usize)
            .collect();

        Ok(Self { birth, survival })
    }
}

struct Automata {
    alive_cells: HashSet<Cell>,
    cell_size: f32,
    offset_x: f32,
    offset_y: f32,
    dragging: bool,
    drag_start: Option<(f32, f32)>,
    running: bool,
    rules: Rules,
}

impl Automata {
    fn new(initial_state: Vec<Cell>, cell_size: f32, rules: Rules) -> Self {
        let alive_cells = initial_state.into_iter().collect();
        Self {
            alive_cells,
            cell_size,
            offset_x: 0.0,
            offset_y: 0.0,
            dragging: false,
            drag_start: None,
            running: true,
            rules,
        }
    }

    fn step(&mut self) {
        let mut new_state = HashSet::new();
        let mut neighbor_counts = HashSet::new();

        for &cell in &self.alive_cells {
            let neighbors = self.get_neighbors(cell);
            let live_count = neighbors.iter().filter(|&&n| self.alive_cells.contains(&n)).count();

            if self.rules.survival.contains(&live_count) {
                new_state.insert(cell);
            }

            for &neighbor in &neighbors {
                neighbor_counts.insert(neighbor);
            }
        }

        for neighbor in neighbor_counts {
            if !self.alive_cells.contains(&neighbor) {
                let live_count = self.get_neighbors(neighbor)
                    .iter()
                    .filter(|&&n| self.alive_cells.contains(&n))
                    .count();
                if self.rules.birth.contains(&live_count) {
                    new_state.insert(neighbor);
                }
            }
        }

        self.alive_cells = new_state;
    }

    fn get_neighbors(&self, cell: Cell) -> Vec<Cell> {
        let mut neighbors = Vec::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx != 0 || dy != 0 {
                    neighbors.push(Cell(cell.0 + dx, cell.1 + dy));
                }
            }
        }
        neighbors
    }

    fn toggle_cell(&mut self, x: f32, y: f32) {
        let grid_x = ((x - self.offset_x) / self.cell_size).floor() as i32;
        let grid_y = ((y - self.offset_y) / self.cell_size).floor() as i32;
        let cell = Cell(grid_x, grid_y);
        if self.alive_cells.contains(&cell) {
            self.alive_cells.remove(&cell);
        } else {
            self.alive_cells.insert(cell);
        }
    }
}

impl EventHandler for Automata {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.running {
            self.step();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);

        for &cell in &self.alive_cells {
            let rect = graphics::Rect::new(
                (cell.0 as f32 * self.cell_size) + self.offset_x,
                (cell.1 as f32 * self.cell_size) + self.offset_y,
                self.cell_size,
                self.cell_size,
            );
            let rectangle = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::WHITE)?;
            canvas.draw(&rectangle, DrawParam::default());
        }

        canvas.finish(ctx)
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) -> GameResult {
        if button == MouseButton::Left {
            self.dragging = true;
            self.drag_start = Some((x, y));
        } else if button == MouseButton::Right {
            self.toggle_cell(x, y);
        }
        Ok(())
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) -> GameResult {
        if button == MouseButton::Left {
            self.dragging = false;
            self.drag_start = None;
        }
        Ok(())
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, dx: f32, dy: f32) -> GameResult {
        if self.dragging {
            self.offset_x += dx;
            self.offset_y += dy;
        }
        Ok(())
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) -> GameResult {
        let zoom_factor = 0.1;
        if y > 0.0 {
            self.cell_size *= 1.0 + zoom_factor;
        } else if y < 0.0 {
            self.cell_size *= 1.0 - zoom_factor;
        }
        Ok(())
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
    let args: Vec<String> = env::args().collect();
    let default_rule = "B3/S23".to_string(); // Create a binding for the default rule
    let rule_str = args.get(1).unwrap_or(&default_rule); // Use the binding here

    let rules = Rules::from_string(rule_str).unwrap_or_else(|err| {
        eprintln!("Error parsing rules: {}", err);
        std::process::exit(1);
    });

    let cb = ContextBuilder::new("automata", "alskdfjsaodjkf")
        .window_setup(ggez::conf::WindowSetup::default().title("Automata"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(1600.0, 1200.0));
    let (ctx, event_loop) = cb.build()?;

    let initial_state = vec![
        Cell(50, 50), Cell(51, 50), Cell(52, 50),
        Cell(52, 51), Cell(51, 52),
    ];

    let game = Automata::new(initial_state, 10.0, rules);
    event::run(ctx, event_loop, game)
}