use ds2d::{graphics::Color, input::keyboard, GameResult};
use log::error;

pub struct InputGame {
    color: Color,
}

impl InputGame {
    pub fn new() -> Self {
        Self {
            color: Color::CORNFLOWER_BLUE,
        }
    }
}

impl ds2d::Game for InputGame {
    fn draw(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        ds2d::graphics::clear(ctx, self.color);
        Ok(())
    }

    fn update(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        while ds2d::timer::run_fixed_timestep(ctx, 60.0, 5) {
            // Emulate analog axes with binary keyboard input
            let red_axis = keyboard::axis1d(ctx, keyboard::KeyCode::Q, keyboard::KeyCode::W);
            let green_axis = keyboard::axis1d(ctx, keyboard::KeyCode::A, keyboard::KeyCode::S);
            let blue_axis = keyboard::axis1d(ctx, keyboard::KeyCode::Z, keyboard::KeyCode::X);

            self.color.r = (self.color.r + red_axis / 600.0).max(0.0).min(1.0);
            self.color.g = (self.color.g + green_axis / 600.0).max(0.0).min(1.0);
            self.color.b = (self.color.b + blue_axis / 600.0).max(0.0).min(1.0);
        }
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
