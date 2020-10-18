use cgmath::{Deg, Rad, Vector2};
use ds2d::{Context, GameResult, graphics::{self, Color, Rect}, input::keyboard, timer};
use log::error;

pub struct HelloGame {
    sprite: graphics::Sprite,
}

impl HelloGame {
    pub fn new(ctx: &mut Context) -> GameResult<HelloGame> {
        let tex = graphics::Texture2D::from_memory(ctx, include_bytes!("face.png"))?;
        let sprite = graphics::Sprite::new(
            ctx,
            tex,
            Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 },
            Rect { x: 100.0, y: 100.0, w: 64.0, h: 64.0 },
            Vector2 { x: 0.5, y: 0.5 },
            Deg(45.0).into(),
            //Deg(0.0).into(),
            Color::WHITE
        )?;

        Ok(Self { sprite })
    }
}

impl ds2d::Game for HelloGame {
    fn draw(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        graphics::clear(ctx, Color::CORNFLOWER_BLUE);
        graphics::draw(ctx, &mut self.sprite)?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        let v = keyboard::axis2d(ctx, keyboard::KeyCode::A, keyboard::KeyCode::D, keyboard::KeyCode::W, keyboard::KeyCode::S);
        let pixel_per_second = 200.0;
        let update_rate = 60.0;
        while timer::run_fixed_timestep(ctx, update_rate, 10) {
            let delta = v * pixel_per_second / update_rate as f32;
            let delta_angle = v.x / update_rate as f32;
            let mut dest = self.sprite.destination();
            dest.set_position(dest.position() + delta);
            self.sprite.set_destination(dest);
            self.sprite.set_rotation(self.sprite.rotation() + Rad(delta_angle));
        }
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
