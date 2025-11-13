use colored::*;
use std::io::{self, Write};
use chrono::{DateTime, Local};

/*
 * color_formatting.rs
 *
 * This file has helper function to standardize formatting for client side printing
 *
 *  - header(text: &str):
 *      Indicates the start of a section (eg Login or sign up)
 *
 *  - success(text: &str):
 *      Used to confirm that a comment went through
 *
 *  - error(text: &str):
 *      Used to report errors 
 *
 *  - warning(text: &str):
 *      Used to draw attention to something that happenend
 *
 *  - info(text: &str):
 *      Used for additonal info on system messages
 *
 *  - user_message(timestamp: &str, username: &str, message: &str):
 *      Prints a chat room message that is recieved
 *
 *  - my_message(message: &str):
 *      Prints a chat room message that you sent
 *
 *  - system_prompt(text: &str):
 *      Prints the system prompt that indicates user input needed
 *
 *  - system_message(message: &str):
 *      Prints a message from the system on a change in state (eg user joined a room)
 */

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
    let width = 80;
    println!("{:>width$} {}", "You:".blue().bold(), message.white(),width = width);
}

pub fn system_prompt(text: &str) {
    print!("{}", text.cyan());
    io::stdout().flush().unwrap();
}

pub fn system_message(message: &str){
    println!("{}", message.dimmed());
}