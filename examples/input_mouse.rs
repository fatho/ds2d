use ds2d::{graphics, graphics::Color, input::mouse, GameResult};
use log::error;

pub struct InputGame {
    color: Color,
}

impl InputGame {
    pub fn new() -> Self {
        Self {
            color: Color::BLACK,
        }
    }
}

impl ds2d::Game for InputGame {
    fn draw(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        graphics::clear(ctx, self.color);
        Ok(())
    }

    fn update(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        let pos = mouse::position(ctx);
        let size = graphics::screen_size(ctx);
        self.color.r = (pos.x / size.width as f64) as f32;
        self.color.g = (pos.y / size.height as f64) as f32;
        Ok(())
    }

    fn exit(&mut self, _ctx: &mut ds2d::Context) -> GameResult<()> {
        Ok(())
    }
}

fn main() {
    stderrlog::new().quiet(false).verbosity(5).init().unwrap();

    let (event_loop, context) = match ds2d::ContextBuilder::new()
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

    let game = InputGame::new();

    ds2d::run(event_loop, context, game)
}
