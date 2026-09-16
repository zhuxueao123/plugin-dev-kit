mod cli;
mod config;
mod http;
mod output;

use clap::Parser;

fn main() {
    let app = cli::Cli::parse();
    let command_name = app.command_name();
    output::configure_output(app.output.clone());
    if let Err(error) = cli::run(app) {
        let _ = output::print_error(&command_name, &error.to_string());
        std::process::exit(1);
    }
}
