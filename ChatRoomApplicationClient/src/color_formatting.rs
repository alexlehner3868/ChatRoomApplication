use colored::*;
use std::io::{self, Write};
use chrono::{DateTime, Local};

pub fn header(text: &str) {
    println!("");
    println!("{}", format!("[{}]", text).magenta().bold());
}

pub fn success(text: &str) {
    println!("{}", format!("[{}]", text).green());
    println!("");
}

pub fn error(text: &str) {
    println!("{}", format!("[{}]", text).red());
    println!("");
}

pub fn warning(text: &str) {
    println!("{}", format!("[{}]", text).yellow());
    println!("");
}

pub fn info(text: &str) {
    println!("{}", text);
}

pub fn user_message(timestamp: &str, username: &str, message: &str) {

    let short_time = DateTime::parse_from_rfc3339(timestamp).map(|dt| dt.with_timezone(&Local).format("%m-%d %H:%M").to_string()).unwrap_or_else(|_| timestamp.to_string());
    println!("[{}] {}: {}", short_time.dimmed(), username.green().bold(),message.white());
}

pub fn my_message(message: &str) {
    let width = 80; // adjust to terminal width
    println!("{:>width$} {}", "You:".blue().bold(), message.white(),width = width);
}

pub fn system_prompt(text: &str) {
    print!("{}", text.cyan());
    io::stdout().flush().unwrap();
}

pub fn system_message(message: &str){
    println!("{}", message.dimmed());
}