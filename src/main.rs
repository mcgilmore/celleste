use clap::{Parser};

use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Canvas, Color, DrawMode, DrawParam, Mesh};
use ggez::{
    input::keyboard::{KeyCode, KeyInput},
    input::mouse::MouseButton,
    Context, ContextBuilder, GameResult,
};

use serde::{Deserialize, Serialize};

use std::collections::{HashSet, HashMap};
use std::fs;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Celleste - A 2D cellular automaton",
    long_about = "Celleste - A 2D cellular automaton\n\n\
The rules can be customized using B<number>/S<number> notation. Default is Conway's Game of Life (B3/S23).\n\n\
Controls:\n\
- Space: Pause/Resume simulation\n\
- Right Click: Add a cell\n\
- S: Save the current state\n\
- L: Load a state from the specified file"
)]
struct Cli {
    /// Path to the save file (default: ./celleste_save.json)
    #[arg(
        short,
        long, 
        default_value_t = get_default_save_file(), 
        help = "Path to save the automaton state."
    )]
    save_file: String,

    /// Rules in B<number>/S<number> format (default: B3/S23)
    #[arg(
        short,
        long,
        default_value = "B3/S23",
        help = "Rules for the automaton in B<number>/S<number> format."
    )]
    rules: String,

    /// Path to load a saved automaton state
    #[arg(
        short = 'l',
        long,
        help = "Path to load a previously saved automaton state."
    )]
    load_file: Option<String>,
}

fn get_default_save_file() -> String {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let default_path = current_dir.join("celleste_save.json");
    default_path
        .to_str()
        .expect("Failed to convert default path to string")
        .to_string()
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize)]
struct Cell(i32, i32);

#[derive(Serialize, Deserialize)]
struct SaveState {
    alive_cells: HashSet<Cell>,
    rules: String,
}

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

struct Automaton {
    alive_cells: HashSet<Cell>,
    cell_size: f32,
    offset_x: f32,
    offset_y: f32,
    dragging: bool,
    drag_start: Option<(f32, f32)>,
    running: bool,
    rules: Rules,
    save_file: String,
}

impl Automaton {
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
            save_file: "./celleste_save.json".to_string(),
        }
    }

    fn set_save_file(&mut self, file_path: String) {
        self.save_file = file_path;
    }

    fn step(&mut self) {
        // Accumulate counts of live neighbors for every cell
        let mut neighbor_counts: HashMap<Cell, usize> = HashMap::new();
        for &cell in &self.alive_cells {
            // For each neighbor of a live cell, increment its count
            for neighbor in self.get_neighbors(cell) {
                *neighbor_counts.entry(neighbor).or_insert(0) += 1;
            }
        }

        let mut new_state = HashSet::new();
        // Evaluate the new state based on neighbor counts
        for (cell, count) in neighbor_counts {
             if self.alive_cells.contains(&cell) {
                 // For live cells, check if they survive
                 if self.rules.survival.contains(&count) {
                      new_state.insert(cell);
                 }
             } else {
                 // For dead cells, check if they are born
                 if self.rules.birth.contains(&count) {
                      new_state.insert(cell);
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

    fn save_to_file(&self, file_path: &str) {
        let save_state = SaveState {
            alive_cells: self.alive_cells.clone(),
            rules: format!("B{}/S{}", 
                self.rules.birth.iter().map(|b| b.to_string()).collect::<String>(),
                self.rules.survival.iter().map(|s| s.to_string()).collect::<String>()
            ),
        };
        match serde_json::to_string(&save_state) {
            Ok(json) => {
                if let Err(err) = fs::write(file_path, json) {
                    eprintln!("Failed to save game state: {}", err);
                } else {
                    println!("Game state saved to {}", file_path);
                }
            }
            Err(err) => eprintln!("Failed to serialize game state: {}", err),
        }
    }

    fn load_from_file(&mut self, file_path: &str) {
        match fs::read_to_string(file_path) {
            Ok(json) => match serde_json::from_str::<SaveState>(&json) {
                Ok(save_state) => {
                    self.alive_cells = save_state.alive_cells;
                    match Rules::from_string(&save_state.rules) {
                        Ok(rules) => self.rules = rules,
                        Err(err) => eprintln!("Failed to parse rules from save state: {}", err),
                    }
                    println!("Game state and rules loaded from {}", file_path);
                }
                Err(err) => eprintln!("Failed to deserialize game state: {}", err),
            },
            Err(err) => eprintln!("Failed to read game state from file: {}", err),
        }
    }
}

impl EventHandler for Automaton {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.running {
            self.step();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        let mut mb = graphics::MeshBuilder::new();

        for &cell in &self.alive_cells {
            let rect = graphics::Rect::new(
                (cell.0 as f32 * self.cell_size) + self.offset_x,
                (cell.1 as f32 * self.cell_size) + self.offset_y,
                self.cell_size,
                self.cell_size,
            );
            mb.rectangle(DrawMode::fill(), rect, Color::WHITE)?;
        }

        let mesh_data = mb.build();
        let mesh = Mesh::from_data(ctx, mesh_data);
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
            match keycode {
                KeyCode::Space => {
                    // Toggle the `running` state
                    self.running = !self.running;
                }
                KeyCode::S => {
                    // Save the current state to a file
                    self.save_to_file(&self.save_file);
                }
                KeyCode::L => {
                    // Clone the save file path to avoid immutable borrow conflicts
                    let save_file = self.save_file.clone();
                    self.load_from_file(&save_file);
                }
                _ => {}
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
            self.dragging = true;
            self.drag_start = Some((x, y));
        } else if button == MouseButton::Right {
            self.toggle_cell(x, y);
        }
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) -> GameResult {
        if button == MouseButton::Left {
            self.dragging = false;
            self.drag_start = None;
        }
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        _x: f32,
        _y: f32,
        dx: f32,
        dy: f32,
    ) -> GameResult {
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
}

fn main() -> GameResult {
    let cli = Cli::parse();

    let rules = Rules::from_string(&cli.rules).unwrap_or_else(|err| {
        eprintln!("Error parsing rules: {}", err);
        std::process::exit(1);
    });

    let cb = ContextBuilder::new("Celleste", "alskdfjsaodjkf")
        .window_setup(ggez::conf::WindowSetup::default().title("Celleste"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(1600.0, 1200.0));
    let (ctx, event_loop) = cb.build()?;

    // Default initial state
    let initial_state = vec![
        Cell(50, 50),
        Cell(51, 50),
        Cell(52, 50),
        Cell(52, 51),
        Cell(51, 52),
    ];

    let mut game = Automaton::new(initial_state.clone(), 10.0, rules);

    // Set the save file from the CLI argument
    game.set_save_file(cli.save_file);

    // Load from the provided file if specified
    if let Some(load_file) = cli.load_file {
        game.load_from_file(&load_file);
    } else {
        println!("No load file provided. Using default");
    }

    event::run(ctx, event_loop, game)
}
