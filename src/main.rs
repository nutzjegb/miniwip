use clap::Parser;
use crossterm::{
    event::{Event, EventStream},
    Result,
};
use futures::StreamExt;
use std::path::PathBuf;
use tokio::{
    io::AsyncReadExt,
    select,
    time::{interval, Duration},
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

mod app;
mod tui;
use app::{App, AppState, TICKS_MS};

// TODO remove enum, use value_parser
// as a lot more baudrate could work (minicom says so?)
// #[derive(Clone, ValueEnum)]
// enum Baudrate {
//     _19200,
//     _38400,
//     _115200,
// }
// impl Baudrate {
//     fn get_value(&self) -> u32 {
//         match self {
//             Baudrate::_19200 => 19200,
//             Baudrate::_38400 => 38400,
//             Baudrate::_115200 => 115200,
//         }
//     }
// }

#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/ttyS0";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short = 'd', long, default_value = DEFAULT_TTY)]
    device: String,

    #[arg(short, long, default_value = "115200")]
    baudrate: u32,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

async fn event_handler(app: &mut App, port: &mut SerialStream) -> Result<()> {
    let mut buf: [u8; 128] = [0; 128];
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

            /* Serial input */
            maybe_event = port.read(&mut buf) => {
                match maybe_event {
                    Ok(read_bytes) => {
                        //if 1 != read_bytes {
                            println!("- got {} bytes -\r", read_bytes);
                        //}
                        let slice = &buf[0..read_bytes];
                        for i in 0..read_bytes {
                            print!("{:02x}", buf[i]);
                        }
                        println!(" (buf {:?} slice {:?} len {})\r", buf.as_ptr(), slice.as_ptr(), slice.len());
                        //app.handle_serial_event(slice)?;
                    },
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return Err(e);
                    },
                }
            }

            /* Crossterm events */
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if let Event::Key(key_event) = event {
                            match app.handle_key_event(port, key_event)? {
                                AppState::Quit => break,
                                AppState::None => (),
                            }
                        }
                    },
                    Some(Err(e)) => {
                        println!("Error: {:?}", e);
                        return Err(e);
                    }
                    // TODO break on None?
                    None => todo!(), //break,
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

    /* Open serial port */
    let mut port = tokio_serial::new(cli.device.clone(), cli.baudrate).open_native_async()?;
    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    //TODO handle errors
    let mut app = App::init(cli)?;
    let result = event_handler(&mut app, &mut port).await;
    app.cleanup()?;

    result
}

#[tokio::main]
async fn main() -> Result<()> {
    main_app().await?;
    Ok(())
}
