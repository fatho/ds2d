use cgmath::Vector2;
use ds2d::{
    graphics::{self, Color},
    Context, GameResult,
};
use log::error;

pub struct HelloGame {
    sprite: graphics::Sprite,
}

impl HelloGame {
    pub fn new(ctx: &mut Context) -> GameResult<HelloGame> {
        let tex = graphics::Texture2D::from_file(ctx, "examples/face.png")?;
        let sprite = graphics::Sprite::new(ctx, tex, Vector2::new(100.0, 100.0), Color::WHITE)?;

        Ok(Self { sprite })
    }
}

impl ds2d::Game for HelloGame {
    fn draw(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        graphics::clear(ctx, Color::CORNFLOWER_BLUE);
        graphics::draw(ctx, &mut self.sprite)?;
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
