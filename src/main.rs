use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
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

impl GridParams {
    fn generate_mesh(&self, ctx: &mut ggez::Context) -> ggez::GameResult<graphics::Mesh> {
        let mut builder = graphics::MeshBuilder::new();

        // Generate vertical lines.
        for i in 0..=self.size.0 {
            let x = (i * self.cell_size.0) as f32;

            builder.line(
                &[
                    na::Point2::new(x, 0.0),
                    na::Point2::new(x, (self.size.1 * self.cell_size.1) as f32),
                ],
                self.line_width,
                self.line_color,
            )?;
        }
        // Generate horizontal lines.
        for i in 0..=self.size.1 {
            let y = (i * self.cell_size.1) as f32;

            builder.line(
                &[
                    na::Point2::new(0.0, y),
                    na::Point2::new((self.size.0 * self.cell_size.0) as f32, y),
                ],
                self.line_width,
                self.line_color,
            )?;
        }

        builder.build(ctx)
    }
}

struct GameState {
    grid_params: GridParams,
    grid_mesh: graphics::Mesh,
}

impl GameState {
    fn new(ctx: &mut ggez::Context) -> ggez::GameResult<GameState> {
        let params = GridParams {
            size: (4, 5),
            cell_size: (64, 32),
            cell_color: graphics::BLACK,
            line_width: 2.0,
            line_color: graphics::WHITE,
        };
        let mesh = params.generate_mesh(ctx)?;
        let state = GameState {
            grid_params: params,
            grid_mesh: mesh,
        };
        Ok(state)
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        graphics::draw(ctx, &self.grid_mesh, (na::Point2::new(0.0, 0.0),))?;

        // Print the fps counter to the screen.
        let fps_counter = graphics::Text::new(format!("{}", timer::fps(ctx) as i32));
        graphics::draw(ctx, &fps_counter, (na::Point2::new(0.0, 0.0),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> ggez::GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut GameState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
