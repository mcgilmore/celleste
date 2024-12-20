use ggez::{Context, ContextBuilder, GameResult, input::keyboard::{KeyCode, KeyInput}, input::mouse::MouseButton};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Canvas, Color, DrawMode, DrawParam, Mesh};
use std::collections::HashSet;

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
struct Cell(i32, i32);

struct GameOfLife {
    alive_cells: HashSet<Cell>,
    cell_size: f32,
    offset_x: f32,
    offset_y: f32,
    dragging: bool,
    drag_start: Option<(f32, f32)>,
    running: bool,
}

impl GameOfLife {
    fn new(initial_state: Vec<Cell>, cell_size: f32) -> Self {
        let alive_cells = initial_state.into_iter().collect();
        Self {
            alive_cells,
            cell_size,
            offset_x: 0.0,
            offset_y: 0.0,
            dragging: false,
            drag_start: None,
            running: true,
        }
    }

    fn step(&mut self) {
        let mut new_state = HashSet::new();
        let mut neighbor_counts = HashSet::new();

        for &cell in &self.alive_cells {
            let neighbors = self.get_neighbors(cell);
            let live_count = neighbors.iter().filter(|&&n| self.alive_cells.contains(&n)).count();

            if live_count == 2 || live_count == 3 {
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
                if live_count == 3 {
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

impl EventHandler for GameOfLife {
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
    let cb = ContextBuilder::new("game_of_life", "alskdfjsaodjkf")
        .window_setup(ggez::conf::WindowSetup::default().title("Conway's Game of Life"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0));
    let (ctx, event_loop) = cb.build()?;

    let initial_state = vec![
        Cell(0, 0), Cell(1, 0), Cell(2, 0),
        Cell(2, 1), Cell(1, 2),
    ];

    let game = GameOfLife::new(initial_state, 10.0);
    event::run(ctx, event_loop, game)
}
