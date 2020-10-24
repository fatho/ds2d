use cgmath::{Deg, Rad, Vector2};
use ds2d::{
    graphics::{self, Color, Rect},
    input::keyboard,
    timer, Context, GameResult,
};
use log::error;

pub struct HelloGame {
    sprite: graphics::Sprite,
    player_pos_prev: Vector2<f64>,
    player_rotation_prev: Rad<f64>,
    player_pos: Vector2<f64>,
    player_rotation: Rad<f64>,
}

impl HelloGame {
    pub fn new(ctx: &mut Context) -> GameResult<HelloGame> {
        let tex = graphics::Texture2D::from_memory(ctx, include_bytes!("face.png"))?;
        let sprite = graphics::Sprite::new(
            ctx,
            tex,
            Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
            Rect {
                x: 0.0,
                y: 0.0,
                w: 64.0,
                h: 64.0,
            },
            Vector2 { x: 0.5, y: 0.5 },
            Deg(0.0).into(),
            Color::WHITE,
        )?;

        let player_pos = Vector2::new(100.0, 100.0);
        let player_rotation = Deg(45.0).into();

        // The animation remains smooth even when setting this to e.g 5 ups.
        timer::set_updates_per_second(ctx, 30.0);

        Ok(Self {
            sprite,
            player_pos,
            player_pos_prev: player_pos,
            player_rotation,
            player_rotation_prev: player_rotation,
        })
    }
}

impl ds2d::Game for HelloGame {
    fn draw(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        use cgmath::VectorSpace;

        graphics::clear(ctx, Color::CORNFLOWER_BLUE);

        let factor = timer::interpolation_factor(ctx);

        self.sprite.set_position(self.player_pos_prev.lerp(self.player_pos, factor).map(|x| x as f32));
        self.sprite.set_rotation(Rad(
            ((1.0 - factor) * self.player_rotation_prev.0 + factor * self.player_rotation.0) as f32
        ));
        graphics::draw(ctx, &mut self.sprite)?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        let v = keyboard::axis2d(
            ctx,
            keyboard::KeyCode::A,
            keyboard::KeyCode::D,
            keyboard::KeyCode::W,
            keyboard::KeyCode::S,
        );
        let pixel_per_second = 200.0;
        while timer::run_fixed_timestep(ctx, 10) {
            let delta = v * pixel_per_second * timer::timestep(ctx).as_secs_f64();
            let delta_angle = v.x * timer::timestep(ctx).as_secs_f64();
            self.player_pos_prev = self.player_pos;
            self.player_pos += delta;
            self.player_rotation_prev = self.player_rotation;
            self.player_rotation += Rad(delta_angle);
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
