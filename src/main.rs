#![windows_subsystem = "windows"]

use ggez;
use ggez::event;
use ggez::graphics;
use ggez::input;
use ggez::mint;
use ggez::timer;
use std::fs::File;
use std::io::prelude::*;
use tinyfiledialogs;

struct GridParams {
    /// The number of cells in each (row, column) of the grid.
    size: (usize, usize),
    /// The size of each cell (width, height) in pixels.
    cell_size: (usize, usize),
    /// The color with which to fill a cell (if needed).
    cell_color: graphics::Color,
    /// The width of the lines that mark the grid in pixels.
    line_width: f32,
    /// The color with which to draw the lines that mark the grid.
    line_color: graphics::Color,
}

/// Generates a `Mesh` for grid lines according to the
/// given `GridParams`.
///
/// This should probably only be called once for each `GridParams`
/// and have the resulting `Mesh` be cached.
fn generate_grid_mesh(
    ctx: &mut ggez::Context,
    params: &GridParams,
) -> ggez::GameResult<graphics::Mesh> {
    let mut builder = graphics::MeshBuilder::new();

    // Generate vertical lines.
    for i in 0..=params.size.0 {
        let x = (i * params.cell_size.0) as f32;

        builder.line(
            &[
                mint::Point2 { x: x, y: 0.0 },
                mint::Point2 {
                    x: x,
                    y: (params.size.1 * params.cell_size.1) as f32,
                },
            ],
            params.line_width,
            params.line_color,
        )?;
    }
    // Generate horizontal lines.
    for i in 0..=params.size.1 {
        let y = (i * params.cell_size.1) as f32;

        builder.line(
            &[
                mint::Point2 { x: 0.0, y: y },
                mint::Point2 {
                    x: (params.size.0 * params.cell_size.0) as f32,
                    y: y,
                },
            ],
            params.line_width,
            params.line_color,
        )?;
    }

    builder.build(ctx)
}

/// Generates a `Mesh` for the grid cells according
/// to the given `GridParams` and grid state.
///
/// `grid_state` **has** to be *horizontally packed*,
/// meaning that the outer `Vec` holds many rows.
///
/// The dimensions of `grid_state` are taken from
/// `params`, and are **not** checked. It's up to
/// the caller to make sure they are synchronized (`grid_state` may be bigger).  
fn generate_grid_cells_mesh(
    ctx: &mut ggez::Context,
    params: &GridParams,
    grid_state: &Vec<Vec<bool>>,
) -> ggez::GameResult<graphics::Mesh> {
    let mut builder = graphics::MeshBuilder::new();
    let mut num_rectangles = 0;

    for row in 0..params.size.1 {
        let y = (row * params.cell_size.1) as f32 + params.line_width * 0.5;
        for column in 0..params.size.0 {
            let x = (column * params.cell_size.0) as f32 + params.line_width * 0.5;

            if grid_state[row][column] {
                builder.rectangle(
                    graphics::DrawMode::fill(),
                    graphics::Rect::new(
                        x,
                        y,
                        params.cell_size.0 as f32 - params.line_width,
                        params.cell_size.1 as f32 - params.line_width,
                    ),
                    params.cell_color,
                );
                num_rectangles += 1;
            }
        }
    }

    // Only build the mesh if it isn't empty, as it
    // currently causes a panic.
    if num_rectangles > 0 {
        let mesh = builder.build(ctx)?;
        Ok(mesh)
    } else {
        Err(ggez::GameError::RenderError(String::from(
            "No vertices in mesh",
        )))
    }
}

/// Calculates the indice of the grid cell under the given point.Result
///
/// If point is **not** on a grid cell, return error.
fn calculate_grid_cell_indices(
    params: &GridParams,
    point: mint::Point2<f32>,
) -> Result<mint::Point2<usize>, ()> {
    let x: usize = (point.x / params.cell_size.0 as f32) as usize;
    let y: usize = (point.y / params.cell_size.1 as f32) as usize;

    if x < params.size.0 && y < params.size.1 {
        Ok(mint::Point2 { x: x, y: y })
    } else {
        Err(())
    }
}

/// Updates the grid state according to the rules of Game of Life.
///
/// Returns a **new** grid_state.
fn update_grid_state(params: &GridParams, grid_state: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
    let mut new_state = grid_state.clone();
    // Update each cell in the grid.
    for row in 0..params.size.1 {
        for column in 0..params.size.0 {
            const INDICE_OFFSETS: [[isize; 2]; 8] = [
                [-1, -1],
                [0, -1],
                [1, -1],
                [-1, 0],
                [1, 0],
                [-1, 1],
                [0, 1],
                [1, 1],
            ];

            // Check neighbors.
            let mut living_neighbors: u32 = 0;
            for indices in &INDICE_OFFSETS {
                let x = column as isize + indices[0];
                let y = row as isize + indices[1];

                if (0..(params.size.0 as isize)).contains(&x)
                    && (0..(params.size.1 as isize)).contains(&y)
                {
                    living_neighbors += if grid_state[y as usize][x as usize] {
                        1
                    } else {
                        0
                    };
                }
            }

            let is_living = grid_state[row][column];
            if is_living {
                // Any live cell with fewer than two live neighbours dies, as if by underpopulation.
                // Any live cell with more than three live neighbours dies, as if by overpopulation.
                if living_neighbors < 2 || living_neighbors > 3 {
                    new_state[row][column] = false;
                }
            }
            // Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
            else if living_neighbors == 3 {
                new_state[row][column] = true;
            }
        }
    }
    new_state
}

/// Updates the size of the window according to the given grid parameters.ggez
///
/// Calls `graphics::set_mode`.
fn update_window_size(ctx: &mut ggez::Context, params: &GridParams) -> ggez::GameResult {
    graphics::set_mode(
        ctx,
        ggez::conf::WindowMode::default().dimensions(
            (params.size.0 * params.cell_size.0) as f32,
            (params.size.1 * params.cell_size.1) as f32,
        ),
    )
}

struct GameState {
    grid_params: GridParams,
    grid_mesh: graphics::Mesh,
    grid_state: Vec<Vec<bool>>,
    playing: bool,
    last_update: std::time::Duration,
    update_tick: std::time::Duration,
    mouse_button_pressed_last_frame: bool,
    save_key_pressed_last_frame: bool,
    load_key_pressed_last_frame: bool,
    play_key_pressed_last_frame: bool,
}

impl GameState {
    fn new(ctx: &mut ggez::Context) -> ggez::GameResult<GameState> {
        let params = GridParams {
            size: (20, 15),
            cell_size: (20, 20),
            cell_color: graphics::WHITE,
            line_width: 2.0,
            line_color: graphics::BLACK,
        };
        update_window_size(ctx, &params)?;

        let mesh = generate_grid_mesh(ctx, &params)?;
        let default_grid = vec![vec![false; params.size.0]; params.size.1];
        let state = GameState {
            grid_params: params,
            grid_mesh: mesh,
            grid_state: default_grid,
            playing: false,
            last_update: std::time::Duration::default(),
            update_tick: std::time::Duration::from_millis(10),
            mouse_button_pressed_last_frame: false,
            save_key_pressed_last_frame: false,
            load_key_pressed_last_frame: false,
            play_key_pressed_last_frame: false,
        };
        Ok(state)
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        let time = ggez::timer::time_since_start(ctx);
        if !self.playing {
            let pressed = input::mouse::button_pressed(ctx, input::mouse::MouseButton::Left);
            if self.mouse_button_pressed_last_frame && !pressed {
                let position = input::mouse::position(ctx);
                match calculate_grid_cell_indices(&self.grid_params, position) {
                    Err(()) => (),
                    Ok(point) => {
                        let value = self.grid_state[point.y][point.x];
                        self.grid_state[point.y][point.x] = !value;
                    }
                }
            }
            self.mouse_button_pressed_last_frame = pressed;

            let pressed = input::keyboard::is_key_pressed(ctx, input::keyboard::KeyCode::S);
            if !self.save_key_pressed_last_frame && pressed {
                match tinyfiledialogs::save_file_dialog("Save", "./grid-state.json") {
                    None => (),
                    Some(file) => {
                        let serialized = serde_json::to_string(&self.grid_state).unwrap();
                        match File::create(file) {
                            Ok(mut file) => file.write_all(serialized.as_bytes()).unwrap(),
                            _ => (),
                        }
                    }
                }
            }
            self.save_key_pressed_last_frame = pressed;

            let pressed = input::keyboard::is_key_pressed(ctx, input::keyboard::KeyCode::L);
            if !self.load_key_pressed_last_frame && pressed {
                match tinyfiledialogs::open_file_dialog("Open", "./", None) {
                    None => (),
                    Some(file) => match File::open(file) {
                        Ok(mut file) => {
                            let mut file_contents = String::new();
                            file.read_to_string(&mut file_contents).unwrap();

                            match serde_json::from_str(&file_contents) {
                                Ok(deserialized) => {
                                    self.grid_state = deserialized;
                                    let new_size =
                                        (self.grid_state[0].len(), self.grid_state.len());
                                    if self.grid_params.size != new_size {
                                        self.grid_params.size = new_size;
                                        self.grid_mesh =
                                            generate_grid_mesh(ctx, &self.grid_params)?;
                                        update_window_size(ctx, &self.grid_params)?;
                                    }
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    },
                }
            }
            self.load_key_pressed_last_frame = pressed;
        } else if (time - self.last_update) >= self.update_tick {
            self.grid_state = update_grid_state(&self.grid_params, &self.grid_state);
            self.last_update = time;
        }

        let pressed = input::keyboard::is_key_pressed(ctx, input::keyboard::KeyCode::Return);
        if !self.play_key_pressed_last_frame && pressed {
            self.last_update = time - self.update_tick;
            self.playing = !self.playing;
        }
        self.play_key_pressed_last_frame = pressed;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        // draw the grid outline
        graphics::draw(ctx, &self.grid_mesh, (mint::Point2 { x: 0.0, y: 0.0 },))?;
        // draw the grid cells
        let grid_cells_mesh = generate_grid_cells_mesh(ctx, &self.grid_params, &self.grid_state);
        match grid_cells_mesh {
            Ok(mesh) => graphics::draw(ctx, &mesh, (mint::Point2 { x: 0.0, y: 0.0 },))?,
            _ => (),
        }

        // Print the fps counter to the screen.
        let fps_counter = graphics::Text::new(format!("{}", timer::fps(ctx) as i32));
        graphics::draw(ctx, &fps_counter, (mint::Point2 { x: 0.0, y: 0.0 },))?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut ggez::Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height))
            .unwrap();
    }
}

pub fn main() -> ggez::GameResult {
    let cb = ggez::ContextBuilder::new("life-rs", "Eran Cohen").window_setup(
        ggez::conf::WindowSetup::default()
            .title("Game of Life")
            .vsync(true),
    );
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut GameState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
