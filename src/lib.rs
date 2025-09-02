//! A minimal and pretty logger for the [`log`] crate.
//!
//! To initialize it, call the [`init()`] function.
//!
//! ```
//! use log::LevelFilter;
//!
//! pretty_logging::init(LevelFilter::Info, []);
//!
//! log::trace!("Hello pretty logger!");
//! log::debug!("Hello pretty logger!");
//! log::info!("Hello pretty logger!");
//! log::warn!("Hello pretty logger!");
//! log::error!("Hello pretty logger!");
//! panic!("Hello pretty logger!");
//! ```
//!
//! The [`init()`] function spawns a thread which reads all incoming log messages and writes them
//! to the standard/error output. It holds a lock on the standard output to ensure that log
//! messages are printed in the order they are received, so once the logger is initialized, you
//! must avoid using [`println!`] and [`eprintln!`].
//!
//! You should note that when using this logger, the [`init()`] function will set a custom panic
//! hook, which will override any previous panic hooks set. If you use custom panic hooks, make
//! sure to set them after [`init()`] is called.

use std::{io::Write, panic, sync::mpsc::Sender, thread};

use colored::Colorize;
use log::{Level, LevelFilter};
use time::{OffsetDateTime, macros::format_description};

#[derive(Clone)]
struct Logger(Vec<String>, Sender<(OutputChannel, String)>);

enum OutputChannel {
    Standard,
    Error,
}

impl From<Level> for OutputChannel {
    fn from(level: Level) -> Self {
        match level {
            Level::Error => OutputChannel::Error,
            _ => OutputChannel::Standard,
        }
    }
}

impl Logger {
    fn new(modules: Vec<String>) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        thread::spawn(move || {
            let mut std_lock = std::io::stdout().lock();
            let mut err_lock = std::io::stderr().lock();

            for (output, line) in receiver {
                match output {
                    OutputChannel::Standard => {
                        writeln!(std_lock, "{line}").ok();
                        std_lock.flush().ok();
                    }
                    OutputChannel::Error => {
                        writeln!(err_lock, "{line}").ok();
                        err_lock.flush().ok();
                    }
                }
            }
        });

        Self(modules, sender)
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        if self.0.is_empty() {
            return true;
        }

        for module in &self.0 {
            if metadata.target() == *module || metadata.target().starts_with(&format!("{module}::"))
            {
                return true;
            }
        }

        false
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.1
                .send((
                    record.level().into(),
                    format!(
                        "{} {} {}",
                        get_formatted_timestamp(),
                        get_formatted_level(record.level().as_str()),
                        record.args(),
                    ),
                ))
                .ok();
        }
    }

    fn flush(&self) {}
}

use std::sync::OnceLock;

static LOGGER: OnceLock<Logger> = OnceLock::new();

/// Initializes the logger. This function spawns a thread to read log messages and write them to
/// the appropriate output without blocking the current task.
///
/// This function also sets a custom panic hook to log panics. If you need to set a custom panic
/// hook, set it after this function is called to prevent your custom hook from being overriden.
///
/// Once this function is called, you must avoid calling [`println!`] and [`eprintln!`].
///
/// Arguments:
/// * `filter` - The level filter for the logger.
/// * `modules` - A list of root module names which to log. An empty array will log all modules.
///   You may want to set this to your crate's name, like `["my_crate_name"]`, to only display logs
///   from your crate's modules.
/// 
/// Example:
/// ```
/// use log::LevelFilter;
/// 
/// // Displays all logs from all crates.
/// pretty_logging::init(LevelFilter::Trace, []);
/// ```
pub fn init(filter: LevelFilter, modules: impl IntoIterator<Item = impl ToString>) {
    LOGGER
        .set(Logger::new(
            modules.into_iter().map(|m| m.to_string()).collect(),
        ))
        .ok();

    log::set_logger(LOGGER.get().unwrap())
        .map(|()| log::set_max_level(filter))
        .unwrap();

    panic::set_hook(Box::new(move |panic_info| {
        if filter == LevelFilter::Off {
            return;
        }

        let line = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            format!(
                "{} {} {}",
                get_formatted_timestamp(),
                get_formatted_level("PANIC"),
                s,
            )
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            format!(
                "{} {} {}",
                get_formatted_timestamp(),
                get_formatted_level("PANIC"),
                s,
            )
        } else {
            format!(
                "{} {} A panic occurred! Exitting...",
                get_formatted_timestamp(),
                get_formatted_level("PANIC"),
            )
        };

        LOGGER
            .get()
            .unwrap()
            .1
            .send((OutputChannel::Error, line))
            .ok();
    }));
}

fn get_formatted_timestamp() -> String {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());

    let format = format_description!(
        "[day]/[month]/[year] at [hour]:[minute]:[second].[subsecond digits:2]"
    );

    now.format(&format).unwrap().dimmed().to_string()
}

fn get_formatted_level(level: &str) -> String {
    let string = format!("[{level}]");
    let string = format!("{string:<7}");

    match level {
        "TRACE" => string.dimmed().to_string(),
        "DEBUG" => string.white().to_string(),
        "INFO" => string.blue().to_string(),
        "WARN" => string.yellow().to_string(),
        "ERROR" | "PANIC" => string.red().bold().to_string(),
        _ => string.red().bold().to_string(),
    }
}
