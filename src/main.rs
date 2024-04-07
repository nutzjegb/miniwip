use anyhow::{Error, Result};
use clap::builder::TypedValueParser;
use clap::Parser;
use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use std::path::PathBuf;
use tokio::{
    io::AsyncReadExt,
    select,
    time::{interval, Duration},
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
#[cfg(windows)]
use tokio::signal::windows::ctrl_close;

mod app;
mod tui;
use app::{App, AppResults, TICKS_MS};

#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/ttyS0";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short = 'D', long, default_value = DEFAULT_TTY)]
    device: String,

    #[arg(short, long, default_value_t = 115200)]
    baud_rate: u32,

    #[arg(short = 'B', long, default_value = "8",
        value_parser = clap::builder::PossibleValuesParser::new(["5", "6", "7", "8"])
            .map(|s| match s.as_str() {
            "5" => tokio_serial::DataBits::Five,
            "6" => tokio_serial::DataBits::Six,
            "7" => tokio_serial::DataBits::Seven,
            "8" => tokio_serial::DataBits::Eight,
            _ => unreachable!(),
        }))]
    data_bits: tokio_serial::DataBits,

    #[arg(short, long, default_value = "none",
        value_parser = clap::builder::PossibleValuesParser::new(["none", "odd", "even"])
            .map(|s| match s.as_str() {
            "none" => tokio_serial::Parity::None,
            "odd" => tokio_serial::Parity::Odd,
            "even" => tokio_serial::Parity::Even,
            _ => unreachable!(),
        }))]
    parity: tokio_serial::Parity,

    #[arg(short, long, default_value = "1",
        value_parser = clap::builder::PossibleValuesParser::new(["1", "2"])
            .map(|s| match s.as_str() {
            "1" => tokio_serial::StopBits::One,
            "2" => tokio_serial::StopBits::Two,
            _ => unreachable!(),
        }))]
    stop_bits: tokio_serial::StopBits,

    #[arg(short, long, default_value = "none",
        value_parser = clap::builder::PossibleValuesParser::new(["none", "software", "hardware"])
            .map(|s| match s.as_str() {
            "none" => tokio_serial::FlowControl::None,
            "software" => tokio_serial::FlowControl::Software,
            "hardware" => tokio_serial::FlowControl::Hardware,
            _ => unreachable!(),
        }))]
    flow_control: tokio_serial::FlowControl,
}

async fn event_handler(app: &mut App, port: &mut SerialStream) -> Result<()> {
    let mut buf: [u8; 128] = [0; 128];
    let mut reader = EventStream::new();
    let mut interval = interval(Duration::from_millis(TICKS_MS));

    /* No idea if this works on windows... */
    #[cfg(unix)]
    let mut sig_term = signal(SignalKind::terminate())?;
    #[cfg(windows)]
    let mut sig_term = ctrl_close()?;

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
                        let slice = &buf[0..read_bytes];
                        app.handle_serial_event(slice)?;
                    },
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return Err(e.into());
                    },
                }
            }

            /* Crossterm events */
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if let Event::Key(key_event) = event {
                            match app.handle_key_event(port, key_event)? {
                                AppResults::Quit => break,
                                AppResults::None => (),
                            }
                        }
                        // TODO handle other events (like resize)?
                    },
                    Some(Err(e)) => {
                        println!("Error: {:?}", e);
                        return Err(e.into());
                    }
                    // TODO break on None?
                    None => todo!(), //break,
                }
            }

            /* Exit when needed */
            _ = sig_term.recv() => {
                // TODO parse the result?
                break;
            }
        };
    }
    Ok(())
}

async fn main_app() -> Result<()> {
    let cli = Cli::parse();

    // TODO actually use config file
    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    /* Set default options */
    let builder = tokio_serial::new(cli.device.clone(), cli.baud_rate)
        .data_bits(cli.data_bits)
        .parity(cli.parity)
        .stop_bits(cli.stop_bits)
        .flow_control(cli.flow_control);
    /* Open serial port */
    let mut port = builder
        .open_native_async()
        .map_err(|e| Error::msg(format!("Could not open {} ({})", cli.device, e)))?;

    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    let mut app = App::init(cli)?;
    let result = event_handler(&mut app, &mut port).await;
    app.cleanup()?;

    result
}

// TODO?
// fn init_panic_hook() {
//     let original_hook = take_hook();
//     set_hook(Box::new(move |panic_info| {
//         // intentionally ignore errors here since we're already in a panic
//         let _ = execute!(stdout(), LeaveAlternateScreen);
//         let _ = disable_raw_mode();
//         original_hook(panic_info);
//     }));
// }

#[tokio::main]
async fn main() -> Result<()> {
    main_app().await?;
    Ok(())
}
