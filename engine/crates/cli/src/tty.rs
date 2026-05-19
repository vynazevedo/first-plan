#![allow(dead_code)]

use crossterm::style::{Color, Stylize};
use is_terminal::IsTerminal;
use std::io::{self, Write};

pub fn is_tty() -> bool {
    io::stdout().is_terminal()
}

pub fn output_mode(forced_json: bool) -> OutputMode {
    if forced_json {
        OutputMode::Json
    } else if is_tty() {
        OutputMode::Pretty
    } else {
        OutputMode::Json
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Pretty,
    Json,
}

pub fn print_header(title: &str) {
    let bar = "─".repeat(title.len() + 4);
    println!();
    println!("{}", format!("┌{}┐", bar).with(Color::Cyan));
    println!(
        "{}",
        format!("│  {}  │", title.bold().with(Color::White)).with(Color::Cyan)
    );
    println!("{}", format!("└{}┘", bar).with(Color::Cyan));
}

pub fn print_kv(label: &str, value: &str, color: Color) {
    println!("  {} {}", format!("{}:", label).dim(), value.with(color));
}

pub fn print_kv_bold(label: &str, value: &str, color: Color) {
    println!(
        "  {} {}",
        format!("{}:", label).dim(),
        value.bold().with(color)
    );
}

pub fn print_section(title: &str) {
    println!();
    println!("{}", title.bold().underlined().with(Color::Cyan));
}

pub fn print_success(msg: &str) {
    println!("{} {}", "OK".bold().with(Color::Green), msg);
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "WARN".bold().with(Color::Yellow), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "ERR".bold().with(Color::Red), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", "INFO".bold().with(Color::Blue), msg);
}

pub fn flush() {
    let _ = io::stdout().flush();
}

pub fn strength_color(strength: &str) -> Color {
    match strength {
        "strong" => Color::Green,
        "moderate" => Color::Yellow,
        "weak" => Color::DarkGrey,
        _ => Color::White,
    }
}

pub fn score_bar(score: f64, max: f64, width: usize) -> String {
    let normalized = (score / max).clamp(0.0, 1.0);
    let filled = (normalized * width as f64).round() as usize;
    let empty = width - filled;
    format!(
        "{}{}",
        "█".repeat(filled).with(Color::Green),
        "░".repeat(empty).with(Color::DarkGrey)
    )
}
