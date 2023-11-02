mod cli;

use clap::{Arg, ArgMatches};
use log::{debug, error, info, trace, warn, LevelFilter};

use crate::cli::{ArgHandler, DefaultHandler, EnvHandler, FileHandler, Handler};

/// Sets up logging based on the specified verbosity level.
///
/// This function initializes the logging framework using `env_logger` crate.
/// The verbosity level determines the amount of log output that will be displayed.
///
/// # Examples
///
/// ```
/// use crate::setup_logging;
///
/// setup_logging("debug");
/// ```
///
/// # Arguments
///
/// * `verbose` - A string slice representing the desired verbosity level.
///   Valid values are "off", "error", "warn", "info", "debug", and "trace".
///   If an invalid value is provided, the default level will be set to "info".
///
/// # Dependencies
///
/// This function depends on the following crates:
///
/// - `env_logger` - For setting up logging.
/// - `log` - For defining log levels.
///
/// # Panics
///
/// This function will panic if the `verbose` string cannot be parsed into a `LevelFilter`.
///
/// # Notes
///
/// It is recommended to call this function early in the program to set up logging
/// before any log messages are generated.
///
fn setup_logging(verbose: &str) {
    env_logger::builder()
        .filter(None, verbose.parse().unwrap_or(LevelFilter::Info))
        .init();

    error!("log level enabled: error");
    warn!("log level enabled: warn");
    info!("log level enabled: info");
    debug!("log level enabled: debug");
    trace!("log level enabled: trace");
}

fn fixme1(matches: &ArgMatches) {
    println!("Running fixme1: {:?}", matches);

    let verbosity_handler = ArgHandler::new(matches).next(
        EnvHandler::new()
            .prefix("FIXME_")
            .next(
                FileHandler::new("~/.config/fixme/verbosity")
                    .next(DefaultHandler::new("info").into())
                    .into(),
            )
            .into(),
    );
    if let Some(verbosity) = verbosity_handler.handle_request("verbosity") {
        println!("Verbosity: {}", verbosity);
    }
}

fn fixme2(matches: &ArgMatches) {
    println!("Running fixme2: {:?}", matches);
}

struct App {
    args: clap::Command,
}

impl App {
    pub fn new() -> Self {
        App {
            args: clap::Command::new("FIXME")
                .version("v1.0.0")
                .author("Your Name <your.email@example.com>")
                .about("FIXME")
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .value_name("VERBOSE")
                        // .default_value(Settings::default().verbose)
                        .help("Set the logging verbosity level.")
                        .long_help("Choices: [off, error, warn, info, debug, trace]"),
                )
                .infer_subcommands(true)
                .arg_required_else_help(true)
                .subcommand(
                    clap::Command::new("fixme1")
                        .about("Executes the fixme1 function")
                        .arg(
                            Arg::new("input")
                                .help("Input for the fixme1 function")
                                .required(false)
                                .index(1),
                        ),
                )
                .subcommand(
                    clap::Command::new("fixme2")
                        .about("Executes the fixme2 function")
                        .arg(
                            Arg::new("input")
                                .help("Input for the fixme2 function")
                                .required(true)
                                .index(1),
                        ),
                ),
        }
    }

    pub fn run_with_args<I, T>(&mut self, args: I) -> Result<(), Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let matches = self.args.clone().get_matches_from(args);

        if let Some(verbosity) = matches.get_one::<String>("verbose") {
            setup_logging(verbosity);
        }

        match matches.subcommand() {
            Some(("fixme1", sub_m)) => fixme1(sub_m),
            Some(("fixme2", sub_m)) => fixme2(sub_m),
            _ => eprintln!("Invalid subcommand!"),
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.run_with_args(std::env::args().into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_with_args() {
        assert_eq!(
            Some(()),
            App::new()
                .run_with_args(&vec!["fixme.exe", "fixme1", "0"])
                .ok()
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new().run()
}
