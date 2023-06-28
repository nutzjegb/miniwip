use clap::{Parser, ValueEnum};
use crossterm::{
    event::{Event, EventStream},
    Result,
};
use futures::StreamExt;
use std::path::PathBuf;
use tokio::{
    time::{Duration, interval},
    select,
};

mod app;
mod tui;
use app::{App, AppState, TICKS_MS};

// TODO remove enum, use value_parser
// as a lot more baudrate could work (minicom says so?)
#[derive(Clone, ValueEnum)]
enum Baudrate {
    _19200,
    _38400,
    _115200,
}
impl Baudrate {
    fn get_value(&self) -> u32 {
        match self {
            Baudrate::_19200 => 19200,
            Baudrate::_38400 => 38400,
            Baudrate::_115200 => 115200,
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short = 'd', long, default_value = "/dev/ttyS0")]
    device: String,

    #[arg(short, long, default_value = "115200")]
    baudrate: Baudrate,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

async fn event_handler(app: &mut App) -> Result<()> {
    let mut reader = EventStream::new();
    let mut interval = interval(Duration::from_millis(TICKS_MS));

    loop {
        select! {
            /* Tick */
            _ = interval.tick() => {
                match app.tick() {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return Err(e);
                    }
                }
            }

            /* Crossterm events */
            maybe_event = reader.next() => {
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
    let mut app = App::init(cli)?;
    let result = event_handler(&mut app).await;
    app.cleanup()?;

    result
}

#[tokio::main]
async fn main() -> Result<()> {
    main_app().await?;
    Ok(())
}
