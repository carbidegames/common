extern crate ggez;
#[macro_use] extern crate slog;
extern crate sloggers;

use {
    std::path,

    ggez::{
        Context, GameResult, GameError,
        conf::{Conf, WindowMode, WindowSetup},
        event::{self, EventHandler},
    },
    slog::{Logger},
    sloggers::{Build, terminal::{TerminalLoggerBuilder}, types::{Severity}},
};

pub fn run_game<F, S>(
    game_id: &'static str, author: &'static str, window_title: &str, init: F
) -> GameResult<()> where
    F: FnOnce(&mut Context, Logger) -> GameResult<S>,
    S: EventHandler
{
    // Set up logging
    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    let log = builder.build().unwrap();

    // Set up the ggez context
    let mut c = Conf::new();
    c.window_mode = WindowMode {
        width: 1280,
        height: 720,
        .. Default::default()
    };
    c.window_setup = WindowSetup {
        title: window_title.into(),
        .. Default::default()
    };
    let ctx = &mut Context::load_from_conf(game_id, author, c).unwrap();

    // Just add the local resources directory
    let path = path::PathBuf::from("./resources");
    ctx.filesystem.mount(&path, true);

    // Initialize and run the game
    let result = init(ctx, log.clone())
        .and_then(|mut s| event::run(ctx, &mut s));

    // If we do get an error, we need to log it before returning it
    if let Err(ref e) = result {
        match e {
            GameError::UnknownError(text) => error!(log, "Fatal:\n{}", text),
            e => error!(log, "Fatal: {}", e)
        }
    }

    result
}
