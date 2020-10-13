use log::error;

fn main() {
    stderrlog::new().quiet(false).verbosity(5).init().unwrap();

    let (event_loop, context) = match ds2d::ContextBuilder::new().title("Hello World!").build() {
        Ok(ok) => ok,
        Err(err) => {
            error!("Could not create context: {:?}", err);
            std::process::exit(1);
        }
    };

    ds2d::run(event_loop, context)
}
