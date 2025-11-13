use std::io::{self, Write};

use crate::chat_client::ChatClient;
use crate::color_formatting::*;
use crate::in_chat_room;
use rpassword::read_password;

pub fn print_help() {
    println!();
    println!("==============================");
    println!("           HELP MENU          ");
    println!("==============================\n");

    println!("General Commands:");
    println!("  /help              Show this help menu");
    println!("  /quit              Quit the chat room application\n");

    println!("Authentication Commands:");
    println!("  /sign_up           Create a new username and password");
    println!("  /login             Login with your username and password");
    println!("  /logout            Logout of the chatroom application\n");

    println!("Navigation Commands:");
    println!("  /all_rooms         Show all available chat rooms");
    println!("  /active_rooms      Show all active chat rooms");
    println!("  /create            Create a new chat room (usage: /create <room_id> <password>)");
    println!("  /join              Join an existing chat room (usage: /join <room_id> <password>)");
    println!("  /delete            Delete your chat room (owner only) (usage: /delete <room_id>)\n");


    println!("Room Management Commands:");
    println!("  /active_users      Show all active users in the current room");
    println!("  /kick              Remove a user from your room. Need to own chat room (usage: /kick <username>)");
    println!("  /leave             Leave the current chat room\n");

    println!("Messaging Commands:");
    println!("  <message>          Type and send a message to your current room\n");

    println!("==============================");
}

pub async fn delete_room(client: &mut ChatClient, args: Vec<&str>){
    if args.len() < 2 {
        warning("Usage: /delete <room_id>");
        return;
    }
    let room_id = args[1];
    
    client.delete_room(room_id).await;
}


pub async fn join_room(client: &mut ChatClient, args: Vec<&str>) {
    if args.len() < 3 {
        warning("Usage: /join <room_id> <password>");
        return;
    }

    let room_id = args[1];
    let password = args[2];

    if client.join_room(room_id, password).await {
        in_chat_room(client, room_id).await;
    }
}

pub async fn kick_user(client: &mut ChatClient, args: Vec<&str>) {
    if args.len() < 2 {
        warning("Usage: /kick <username>");
        return;
    }
    
    client.kick_user(args[1]).await;
}


pub async fn create_room(client: &mut ChatClient, args: Vec<&str>) {
    if args.len() < 3 {
        warning("Usage: /create <room_id> <password>");
        return;
    }

    let room_id = args[1];
    let password = args[2];

    client.create_room(room_id, password).await;
}

pub async fn sign_up(client: &mut ChatClient) { 
    header("Sign Up");
    info("Please enter a username (type /quit to cancel):");

    let username = loop {
        print!("Username: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let username = input.trim();

        if username == "/quit" {
            warning("Sign up cancelled");
            return;
        }

        if username.is_empty() || username.starts_with('/') {
            error("Invalid username");
            continue;
        }

        break username.to_string();
    };

    println!("");
    info("Please enter a password that meets the criteria:");
    info("- Minimum 8 characters");
    info("- At least one uppercase letter");
    info("- At least one special character");
    info("(Type /quit to cancel)");

    let password = loop {
        print!("Password: ");
        io::stdout().flush().unwrap();

        let input = match read_password() {
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

    client.create_user(&username, &password).await;
}

pub async fn login(client: &mut ChatClient) -> bool {
    header("Login");
    info("Please enter your username and password to log in.");
    info("(Type /quit at any time to cancel)");

    print!("Username: ");
    io::stdout().flush().unwrap();

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
    let password = match read_password() {
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

    if client.login(username, &password).await {
        true
    } else {
        false
    }
}
