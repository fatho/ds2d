use cgmath::Vector2;
use ds2d::{
    graphics::{self, Color, Text},
    Context, GameResult,
};
use log::error;

pub struct HelloGame {
    rasterizer: graphics::Rasterizer,
    font: graphics::Font,
}

impl HelloGame {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let font_data: Vec<_> = include_bytes!("RobotoSlab-Regular.ttf").as_ref().to_owned();
        let mut rasterizer = graphics::Rasterizer::new(ctx)?;
        let font = rasterizer.create_font(font_data)?;
        Ok(Self { rasterizer, font })
    }
}

impl ds2d::Game for HelloGame {
    fn draw(&mut self, ctx: &mut ds2d::Context) -> GameResult<()> {
        graphics::clear(ctx, Color::CORNFLOWER_BLUE);

        let mut text = Text::new();
        text.add(
            &self.font,
            20.0 * graphics::scale_factor(ctx) as f32,
            Vector2::new(20.0, 20.0),
            "Hello you wonderful world!",
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
