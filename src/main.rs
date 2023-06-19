use clap::Parser;
use crossterm::{
    event::{Event, EventStream},
    Result,
};
use futures::{future::FutureExt, select, StreamExt};
use std::path::PathBuf;

mod app;
mod tui;
use app::{App, AppState};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    name: Option<String>,
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

async fn event_handler(app: &mut App) -> Result<()> {
    let mut reader = EventStream::new();

    loop {
        //let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut input_event = reader.next().fuse();

        select! {
            //_ = delay => { println!(".\r"); },
            maybe_event = input_event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if let Event::Key(key_event) = event {
                            match app.handle_key_event(key_event)? {
                                AppState::Quit => break,
                                AppState::None => (),
                            }
                        }
                    },
                    Some(Err(e)) => {
                        println!("Error: {:?}", e);
                        return Err(e);
                    }
                    // TODO why break on None?
                    None => break,
                }
            }
        };
    }
    Ok(())
}

async fn main_app() -> Result<()> {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    //TODO handle errors
    let mut app = App::init()?;
    let result = event_handler(&mut app).await;
    app.cleanup()?;

    result
}

#[tokio::main]
async fn main() -> Result<()> {
    main_app().await?;
    Ok(())
}
