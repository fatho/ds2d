pub(crate) mod context;
pub use context::{run, Context, ContextBuilder};

// expose the public interface of the various subsystems
pub mod graphics;
pub mod input;

#[derive(Debug)]
pub enum GameError {
    /// There was an error in the graphics subsystem.
    Graphics(String),
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::Graphics(err) => writeln!(f, "graphics error: {}", err),
        }
    }
}

pub type GameResult<T> = Result<T, GameError>;

/// Implemented by the struct holding the game state.
pub trait Game {
    /// Called every frame when the game should render its state.
    /// Returning an error will cause the game to exit.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()>;

    /// Called every frame when the game should update its state.
    /// Returning an error will cause the game to exit.
    fn update(&mut self, ctx: &mut Context) -> GameResult<()>;

    /// Called immediately before the game stops.
    /// Should free any remaining resources acquired from the context here.
    /// The game state will be dropped immediately afterwards.
    // TODO: should the game be able to delay exit?
    fn exit(&mut self, ctx: &mut Context) -> GameResult<()>;
}
