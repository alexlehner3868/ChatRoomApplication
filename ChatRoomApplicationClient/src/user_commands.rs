use std::io::{self, Write};
use std::process::{Command, Stdio};

use crate::chat_client::ChatClient;
use crate::color_formatting::*;
use crate::color_formatting::{BOLD, BRIGHT_GREEN, RESET, YELLOW};
use crate::in_chat_room;
use crate::utils::*;

pub fn print_help() {
    let help_text = format!(
        r#"
{title}=============================={reset}
           HELP MENU          
{title}=============================={reset}

{title}General Commands:{reset}
  {b}{cmd}/help{r}              Show this help menu
  {b}{cmd}/quit{r}              Quit the chat room application

{title}Authentication Commands:{reset}
  {b}{cmd}/sign_up{r}           Create a new username and password
  {b}{cmd}/login{r}             Login with your username and password
  {b}{cmd}/logout{r}            Logout of the chatroom application

{title}Navigation Commands:{reset}
  {b}{cmd}/all_rooms{r}         Show all available chat rooms
  {b}{cmd}/active_rooms{r}      Show all active chat rooms
  {b}{cmd}/create{r}            Create a new chat room (usage: /create <room_id>)
  {b}{cmd}/join{r}              Join an existing chat room (usage: /join <room_id>)
  {b}{cmd}/delete{r}            Delete your chat room (owner only) (usage: /delete <room_id>)

{title}Room Management Commands:{reset}
  {b}{cmd}/active_users{r}      Show all active users in the current room
  {b}{cmd}/kick{r}              Remove a user from your current room (owner only)(usage: /kick <username>)
  {b}{cmd}/leave{r}             Leave the current chat room

{title}Messaging Commands:{reset}
  {b}{cmd}<message>{r}          Send a message to your current room

{title}(Press 'q' to exit help){reset}
==============================
"#,
        title = BRIGHT_GREEN,
        cmd = YELLOW,
        b = BOLD,
        r = RESET,
        reset = RESET
    );

    let mut window = Command::new("less")
        .arg("-R")
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to launch help menu");

    // Write the help menu into the window
    if let Some(stdin) = window.stdin.as_mut() {
        stdin.write_all(help_text.as_bytes()).unwrap();
    }

    // Wait for the user to exit menu
    window.wait().unwrap();
}

pub async fn delete_room(client: &mut ChatClient, args: Vec<&str>) {
    if args.len() < 2 {
        warning("Usage: /delete <room_id>");
        return;
    }
    let room_id = args[1];

    // Send command to server
    client.delete_room(room_id).await;
}

pub async fn join_room(client: &mut ChatClient, args: Vec<&str>) {
    if args.len() < 2 {
        warning("Usage: /join <room_id>");
        return;
    }

    let room_id = args[1];

    print!("Password: ");
    io::stdout().flush().unwrap();

    let password = match read_sensitive_information() {
        Ok(p) => p,
        Err(_) => {
            error("Failed reading password");
            return;
        }
    };

    // Check if server connects user to roon and if so go to chatroom loop
    if client.join_room(room_id, password.trim()).await {
        in_chat_room(client, room_id).await;
    }
}

pub async fn kick_user(client: &mut ChatClient, args: Vec<&str>) {
    if args.len() < 2 {
        warning("Usage: /kick <username>");
        return;
    }

    // Send kick request to server
    client.kick_user(args[1]).await;
}

pub async fn create_room(client: &mut ChatClient, args: Vec<&str>) {
    if args.len() < 2 {
        warning("Usage: /create <room_id>");
        return;
    }

    // Extract room id from the command
    let room_id = args[1];

    // Get password from user
    print!("Password: ");
    io::stdout().flush().unwrap();

    let password = match read_sensitive_information() {
        Ok(p) => p,
        Err(_) => {
            error("Failed reading password");
            return;
        }
    };

    // Send create room request to server
    client.create_room(room_id, password.trim()).await;
}

pub async fn sign_up(client: &mut ChatClient) {
    header("Sign Up");
    info("Please enter a username (type /quit to cancel):");

    // Get username
    let username = loop {
        print!("Username: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let username = input.trim();

        // Exit if /quit was given
        if username == "/quit" {
            warning("Sign up cancelled");
            return;
        }

        // Usernmae checks
        if username.is_empty() || username.starts_with('/') {
            error("Invalid username");
            continue;
        }

        break username.to_string();
    };

    info("\nPlease enter a password that meets the criteria:");
    info("- Minimum 8 characters");
    info("- At least one uppercase letter");
    info("- At least one special character");
    info("(Type /quit to cancel)");

    // Loop to get password
    let password = loop {
        print!("Password: ");
        io::stdout().flush().unwrap();

        let input = match read_sensitive_information() {
            Ok(p) => p,
            Err(_) => {
                error("Failed to read password");
                continue;
            }
        };

        let password = input.trim();

        if password == "/quit" {
            warning("Sign up cancelled");
            return;
        }

        let password_valid = password.len() >= 8
            && password.chars().any(|c| c.is_uppercase())
            && password.chars().any(|c| !c.is_alphanumeric());

        if !password_valid {
            error("Password does not meet policy requirements");
            continue;
        }

        break password.to_string();
    };

    // Send request to server to create account
    client.create_user(&username, &password).await;
}

pub async fn login(client: &mut ChatClient) -> bool {
    header("Login");
    info("Please enter your username and password to log in.");
    info("(Type /quit at any time to cancel)");

    print!("Username: ");
    io::stdout().flush().unwrap();

    // Get username
    let mut username = String::new();
    if io::stdin().read_line(&mut username).is_err() {
        error("Error reading username");
        return false;
    }
    let username = username.trim();

    if username == "/quit" {
        warning("Login cancelled");
        return false;
    }

    print!("Password: ");
    io::stdout().flush().unwrap();
    // Get password
    let password = match read_sensitive_information() {
        Ok(pw) => pw.trim().to_string(),
        Err(_) => {
            error("Error reading password");
            return false;
        }
    };

    if password == "/quit" {
        warning("Login cancelled");
        return false;
    }

    // Try to establish connection with server for user
    client.login(username, &password).await
}
