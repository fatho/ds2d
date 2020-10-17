use ds2d::{
    graphics::{self, Color},
    Context, GameResult,
};
use log::error;

pub struct HelloGame {
    tri: graphics::Mesh,
}

impl HelloGame {
    pub fn new(ctx: &mut Context) -> GameResult<HelloGame> {
        let _tex = graphics::Texture2D::from_file(ctx, "examples/face.png")?;

        // let tri = graphics::Mesh::new(ctx, &[
        //     cgmath::Vector2::new(0.0, 0.5),
        //     cgmath::Vector2::new(-0.75, 0.0),
        //     cgmath::Vector2::new(0.5, 0.0),
        //     cgmath::Vector2::new(0.0, -0.75),
        // ], &[0, 1, 2, 1, 2, 3])?;
        let tri = graphics::Mesh::new(
            ctx,
            &[
                cgmath::Vector2::new(400.0, 200.0),
                cgmath::Vector2::new(100.0, 400.0),
                cgmath::Vector2::new(500.0, 400.0),
                cgmath::Vector2::new(400.0, 700.0),
                //], &[0, 1, 2, 1, 2, 3])?;
            ],
            &[0, 2, 1, 1, 2, 3],
        )?;
        Ok(Self { tri })
    }
}

impl ds2d::Game for HelloGame {
    fn draw(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        graphics::clear(ctx, Color::CORNFLOWER_BLUE);
        graphics::draw(ctx, &self.tri)?;
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
