use cgmath::Vector2;
use ds2d::{
    graphics::{self, Color},
    Context, GameResult,
};
use log::error;

pub struct HelloGame {
    rasterizer: graphics::text::Rasterizer,
    font: graphics::text::Font,
}

impl HelloGame {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let font_data: Vec<_> = include_bytes!("RobotoSlab-Regular.ttf").as_ref().to_owned();
        let mut rasterizer = graphics::text::Rasterizer::new(ctx)?;
        let font = rasterizer.create_font(font_data)?;
        Ok(Self { rasterizer, font })
    }
}

impl ds2d::Game for HelloGame {
    fn draw(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        graphics::clear(ctx, Color::CORNFLOWER_BLUE);

        let font_scale = graphics::scale_factor(ctx) as f32;

        let style1 = graphics::text::Style {
            font: self.font.clone(),
            size: 20.0 * font_scale,
            color: Color::WHITE,
        };
        let style2 = graphics::text::Style {
            font: self.font.clone(),
            size: 30.0 * font_scale,
            color: Color::from_rgba(1.0, 1.0, 0.0, 1.0),
        };

        let mut text = graphics::text::TextBuffer::new();
        text.add(
            &style1,
            Vector2::new(20.0, 20.0),
            "Hello you wonderful world!",
        );
        text.add(
            &style2,
            Vector2::new(200.0, 200.0),
            "Yellow text on a blue background, splendid!",
        );
        let fps = 1.0 / ds2d::timer::average_delta(ctx).as_secs_f64();
        text.add(
            &style1,
            Vector2::new(20.0, 80.0),
            format!("We're running at a steady {:.2} fps.", fps),
        );

        graphics::draw(ctx, &mut self.rasterizer.rasterize(&text))?;

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
        // no vsync for more impressive (but useless) FPS
        .vsync(false)
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
