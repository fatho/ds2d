use cgmath::{Rad, Vector2};
use ds2d::{
    graphics::{self, Color, Quad, Texture2D},
    Context, GameResult,
};
use log::error;

pub struct HelloGame {
    batch: graphics::BatchRender,
    face: Texture2D,
    faces: Vec<(Vector2<f32>, Rad<f32>)>,
}

impl HelloGame {
    pub fn new(ctx: &mut Context) -> GameResult<HelloGame> {
        let batch = graphics::BatchRender::new(ctx)?;
        let face = graphics::Texture2D::from_memory(ctx, include_bytes!("face.png"))?;
        let faces: Vec<_> = std::iter::repeat_with(|| {
            let pos = rand::random::<Vector2<f32>>() * 800f32;
            let angle = rand::random::<Rad<f32>>();
            (pos, angle)
        })
        .take(1000)
        .collect();
        Ok(Self { batch, face, faces })
    }
}

impl ds2d::Game for HelloGame {
    fn draw(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        graphics::clear(ctx, Color::CORNFLOWER_BLUE);
        let hidpi = graphics::scale_factor(ctx) as f32;

        self.batch.draw_triangle(
            [
                Vector2::new(400.0, 200.0),
                Vector2::new(500.0, 400.0),
                Vector2::new(100.0, 400.0),
            ],
            Color::RED,
        );
        self.batch.draw_triangle(
            [
                Vector2::new(100.0, 400.0),
                Vector2::new(500.0, 400.0),
                Vector2::new(400.0, 700.0),
            ],
            Color::GREEN,
        );
        for (pos, angle) in self.faces.iter().copied() {
            self.batch.draw_quad(
                Quad::textured(self.face.clone())
                    .centered_at(pos)
                    .scale(hidpi / 2.0)
                    .with_rotation(angle),
            )
        }
        graphics::draw(ctx, &mut self.batch)?;
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
