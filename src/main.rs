#![windows_subsystem = "windows"]

use ggez;
use ggez::event;
use ggez::graphics;
use ggez::input;
use ggez::mint;
use ggez::timer;

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
    Err(())
}

struct GameState {
    grid_params: GridParams,
    grid_mesh: graphics::Mesh,
    grid_state: Vec<Vec<bool>>,
}

impl GameState {
    fn new(ctx: &mut ggez::Context) -> ggez::GameResult<GameState> {
        let params = GridParams {
            size: (4, 5),
            cell_size: (64, 64),
            cell_color: graphics::BLACK,
            line_width: 2.0,
            line_color: graphics::WHITE,
        };
        let mesh = generate_grid_mesh(ctx, &params)?;
        let default_grid = vec![vec![false; params.size.0]; params.size.1];
        let state = GameState {
            grid_params: params,
            grid_mesh: mesh,
            grid_state: default_grid,
        };
        Ok(state)
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        if input::mouse::button_pressed(ctx, input::mouse::MouseButton::Left) {
            let position = input::mouse::position(ctx);
            match calculate_grid_cell_indices(&self.grid_params, position) {
                Err(()) => (),
                Ok(point) => {}
            }
        }

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
