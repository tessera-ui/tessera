use std::time::Duration;

use console::style;
use indicatif::HumanDuration;

const STATUS_WIDTH: usize = 12;

pub fn status(label: &str, message: impl AsRef<str>) {
    eprintln!(
        "{:>width$} {}",
        style(label).green().bold(),
        message.as_ref(),
        width = STATUS_WIDTH
    );
}

pub fn warn(message: impl AsRef<str>) {
    eprintln!("{}: {}", style("warning").yellow().bold(), message.as_ref());
}

pub fn error(message: impl AsRef<str>) {
    eprintln!("{}: {}", style("error").red().bold(), message.as_ref());
}

pub fn note(message: impl AsRef<str>) {
    eprintln!("{}: {}", style("note").blue().bold(), message.as_ref());
}

pub fn step(message: impl AsRef<str>) {
    eprintln!("    {}", message.as_ref());
}

pub fn format_duration(duration: Duration) -> String {
    HumanDuration(duration).to_string()
}
