use colored::*;
use std::io::{self, Write};

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

pub fn user_message(username: &str, message: &str) {
    println!("{}", format!("{}: {}", username, message).blue());
}

pub fn my_message(message: &str) {
    println!("{}", format!("You: {}", message));
}

pub fn system_prompt(text: &str) {
    print!("{}", text.cyan());
    io::stdout().flush().unwrap();
}