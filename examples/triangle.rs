use ds2d::{Context, GameResult, graphics::{self, Color, Drawable}};
use log::error;

pub struct HelloGame {
    tri: graphics::Mesh,
}

impl HelloGame {
    pub fn new(ctx: &mut Context) -> GameResult<HelloGame> {
        let tri = graphics::Mesh::new(ctx, &[
            cgmath::Vector2::new(0.0, 0.5),
            cgmath::Vector2::new(-0.5, -0.5),
            cgmath::Vector2::new(0.5, -0.5),
        ])?;
        Ok(Self {
            tri
        })
    }
}

impl ds2d::Game for HelloGame {
    fn draw(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        ds2d::graphics::clear(ctx, Color::CORNFLOWER_BLUE);
        // TODO: graphics::draw(&self.tri, ...) would be more consistent
        // Or: self.tri.draw(&mut ctx)
        self.tri.draw()?;
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ds2d::Context) -> GameResult<()> {
        Ok(())
    }

    fn exit(&mut self, _ctx: &mut ds2d::Context) -> GameResult<()> {
        Ok(())
    }
}

fn main() {
    stderrlog::new().quiet(false).verbosity(5).init().unwrap();

    let (event_loop, mut context) = match ds2d::ContextBuilder::new()
        .debug(true)
        .title("Hello World!")
        .build()
    {
        Ok(ok) => ok,
        Err(err) => {
            error!("Could not create context: {:?}", err);
            std::process::exit(1);
        }
    };

    let game = HelloGame::new(&mut context).unwrap();

    ds2d::run(event_loop, context, game)
}
