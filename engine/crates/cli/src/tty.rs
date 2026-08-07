#![allow(dead_code)]

use crossterm::style::{Color, Stylize};
use indicatif::{ProgressBar, ProgressStyle};
use is_terminal::IsTerminal;
use std::io::{self, Write};
use std::time::Duration;

pub fn is_tty() -> bool {
    io::stdout().is_terminal()
}

pub fn stderr_is_tty() -> bool {
    io::stderr().is_terminal()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Pretty,
    Json,
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
pub enum Severity {
    Ok,
    Info,
    Warn,
    Bad,
    Muted,
    Neutral,
}

pub fn severity_color(s: Severity) -> Color {
    match s {
        Severity::Ok => Color::Green,
        Severity::Info => Color::Cyan,
        Severity::Warn => Color::Yellow,
        Severity::Bad => Color::Red,
        Severity::Muted => Color::DarkGrey,
        Severity::Neutral => Color::White,
    }
}

pub fn badge(sev: Severity, label: &str) -> String {
    format!("[{}]", label)
        .bold()
        .with(severity_color(sev))
        .to_string()
}

pub fn dot(sev: Severity) -> String {
    "●".with(severity_color(sev)).to_string()
}

pub fn header(title: &str) {
    let width = title.chars().count() + 4;
    let top = format!("╭{}╮", "─".repeat(width));
    let bot = format!("╰{}╯", "─".repeat(width));
    println!();
    println!("{}", top.with(Color::Cyan));
    println!(
        "{}  {}  {}",
        "│".with(Color::Cyan),
        title.bold().white(),
        "│".with(Color::Cyan)
    );
    println!("{}", bot.with(Color::Cyan));
}

pub fn section(title: &str) {
    println!();
    println!("{} {}", "▎".with(Color::Cyan), title.bold());
}

pub fn subsection(title: &str) {
    println!();
    println!("  {}", title.bold().dim());
}

pub fn rule() {
    let w = terminal_width().unwrap_or(60).min(80);
    println!("{}", "─".repeat(w).with(Color::DarkGrey));
}

pub fn terminal_width() -> Option<usize> {
    crossterm::terminal::size().ok().map(|(w, _)| w as usize)
}

pub fn kv(label: &str, value: &str) {
    println!("  {}  {}", format!("{:>14}", label).dim(), value);
}

pub fn kv_colored(label: &str, value: &str, sev: Severity) {
    println!(
        "  {}  {}",
        format!("{:>14}", label).dim(),
        value.with(severity_color(sev)).bold()
    );
}

pub fn bullet(text: &str) {
    println!("  {} {}", "•".with(Color::DarkGrey), text);
}

pub fn arrow(text: &str) {
    println!("  {} {}", "▸".with(Color::Cyan), text);
}

pub fn table(headers: &[&str], rows: &[Vec<String>]) {
    if headers.is_empty() {
        return;
    }
    let cols = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| display_len(h)).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate().take(cols) {
            let w = display_len(cell);
            if w > widths[i] {
                widths[i] = w;
            }
        }
    }

    let header_line: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| pad_right(h, widths[i]).bold().to_string())
        .collect();
    println!("  {}", header_line.join("  "));

    let rule_line: Vec<String> = widths.iter().map(|w| "─".repeat(*w)).collect();
    println!("  {}", rule_line.join("  ").with(Color::DarkGrey));

    for row in rows {
        let cells: Vec<String> = row
            .iter()
            .enumerate()
            .take(cols)
            .map(|(i, c)| pad_right(c, widths[i]))
            .collect();
        println!("  {}", cells.join("  "));
    }
}

fn pad_right(s: &str, width: usize) -> String {
    let visible = display_len(s);
    if visible >= width {
        return s.to_string();
    }
    let mut out = String::from(s);
    out.push_str(&" ".repeat(width - visible));
    out
}

fn display_len(s: &str) -> usize {
    let mut count = 0usize;
    let mut in_esc = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_esc = true;
            continue;
        }
        if in_esc {
            if ch == 'm' {
                in_esc = false;
            }
            continue;
        }
        count += 1;
    }
    count
}

pub fn print_success(msg: &str) {
    println!("{} {}", "✓".bold().green(), msg);
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "⚠".bold().yellow(), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "✗".bold().red(), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", "•".bold().blue(), msg);
}

pub fn humanize_ms(ms: u128) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let s = ms / 1000;
        format!("{}m{}s", s / 60, s % 60)
    }
}

pub fn humanize_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b < KB {
        format!("{}B", bytes)
    } else if b < MB {
        format!("{:.1}KB", b / KB)
    } else if b < GB {
        format!("{:.1}MB", b / MB)
    } else {
        format!("{:.1}GB", b / GB)
    }
}

pub fn humanize_age_from_iso(iso: &str) -> Option<String> {
    let dt = chrono::DateTime::parse_from_rfc3339(iso).ok()?;
    let now = chrono::Utc::now();
    let dur = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
    let secs = dur.num_seconds();
    if secs < 0 {
        return None;
    }
    if secs < 60 {
        return Some(format!("{}s ago", secs));
    }
    if secs < 3600 {
        return Some(format!("{}m ago", secs / 60));
    }
    if secs < 86400 {
        return Some(format!("{}h ago", secs / 3600));
    }
    Some(format!("{}d ago", secs / 86400))
}

pub fn spinner(msg: &str) -> ProgressBar {
    if !stderr_is_tty() {
        return ProgressBar::hidden();
    }
    let pb = ProgressBar::new_spinner();
    let style = ProgressStyle::default_spinner()
        .template("  {spinner:.cyan} {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_spinner())
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ");
    pb.set_style(style);
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message(msg.to_string());
    pb
}

pub fn multi_spinner(total: usize, msg: &str) -> ProgressBar {
    if !stderr_is_tty() {
        return ProgressBar::hidden();
    }
    let pb = ProgressBar::new(total as u64);
    let style = ProgressStyle::default_bar()
        .template("  {spinner:.cyan} [{pos}/{len}] {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ");
    pb.set_style(style);
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message(msg.to_string());
    pb
}

pub fn flush() {
    let _ = io::stdout().flush();
}

pub fn print_header(title: &str) {
    header(title);
}

pub fn print_section(title: &str) {
    section(title);
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
