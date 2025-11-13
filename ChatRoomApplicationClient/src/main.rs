use std::io::{self, Write};
use tokio;
use rpassword::read_password;
use futures_util::TryStreamExt;

mod color_formatting;
mod chat_client; 
mod messages;
mod terminal_erasing;

use color_formatting::*;
use terminal_erasing::*;
use chat_client::ChatClient;
use crate::messages::ServerWsMessage;

async fn sign_up(client: &mut ChatClient) { 
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

async fn login(client: &mut ChatClient) -> bool {
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


fn print_help() {
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

async fn delete_room(client: &mut ChatClient, args: Vec<&str>){
    if args.len() < 2 {
        warning("Usage: /delete <room_id>");
        return;
    }
    let room_id = args[1];
    
    client.delete_room(room_id).await;
}

async fn in_chat_room(client: &mut ChatClient, room_id: &str) {
    success(&format!("[Connected to {}]", room_id));
    let username = client.username.clone();
   
    // Channel to signal main loop to exit (e.g., kicked or room deleted)
    let (exit_tx, exit_rx) = tokio::sync::watch::channel(false);

    // Spawn a task to handle incoming WebSocket messages
    let mut receiver = client.ws_receiver.take().unwrap();
    let current_room = room_id.to_string();

    let username_clone = username.clone();
    let exit_tx_clone = exit_tx.clone();

    tokio::spawn(async move {
        while let Ok(Some(msg)) = receiver.try_next().await {
            if let Ok(text) = msg.to_text() {
                if let Ok(parsed) = serde_json::from_str::<ServerWsMessage>(text) {
                    match parsed {
                        ServerWsMessage::MessageBroadcast(chat_msg) => {
                            if chat_msg.user_id == "system" {
                                info(&format!("{}: {}", chat_msg.user_id, chat_msg.content));
                            } else if chat_msg.user_id != username_clone.clone().unwrap_or_default() {
                                erase_current_line();
                                user_message(&chat_msg.timestamp, &chat_msg.user_id, &chat_msg.content);
                                system_prompt(&format!("[{}]> ", chat_msg.room_id));
                            }
                        }
                        ServerWsMessage::RoomDeleted { room_id: deleted_room } => {
                            if deleted_room == current_room {
                                warning("[Room has been deleted]");
                                let _ = exit_tx_clone.send(true);
                                break;
                            }
                        }
                        ServerWsMessage::UserJoined { room_id: joined_room, user_id: joined_user } => {
                            if joined_room == current_room && joined_user != username_clone.clone().unwrap_or_default() {
                                erase_current_line();
                                system_message(&format!("[{} has joined]", joined_user));
                                system_prompt(&format!("[{}]> ", joined_room));
                            }
                        }
                        ServerWsMessage::UserLeft { room_id: left_room, user_id: left_user } => {
                            if left_room == current_room && left_user != username_clone.clone().unwrap_or_default() {
                                erase_current_line();
                                system_message(&format!("[{} has left]", left_user));
                                system_prompt(&format!("[{}]> ", left_room));
                            }
                        }
                        ServerWsMessage::UserKicked { room_id: kicked_room, user_id: kicked_user } => {
                            if kicked_room == current_room {
                                erase_current_line();
                                if kicked_user == username_clone.clone().unwrap_or_default() {
                                    warning("[You have been kicked]");
                                    let _ = exit_tx_clone.send(true);
                                    break;
                                } else {
                                    system_message(&format!("[{} has been kicked]", kicked_user));
                                    system_prompt(&format!("[{}]> ", kicked_room));
                                }
                            }
                        }
                        ServerWsMessage::Pong { .. } => {}
                        ServerWsMessage::Error { error_msg } => {
                            error(&error_msg);
                        }
                    }
                }
            }
        }
    });

    loop {
        // Check if forced to leave room
        if *exit_rx.borrow() {
            client.current_room = None;
            success("[Returned to Lobby]");
            break;
        }

        system_prompt(&format!("[{}]> ", room_id));
        io::stdout().flush().unwrap();

        let mut user_input = String::new();
        if io::stdin().read_line(&mut user_input).is_err() {
            continue;
        }

        let input = user_input.trim();
        if input.is_empty() {
            continue;
        }

        erase_last_line();

        let args: Vec<&str> = input.split_whitespace().collect();
        if args.is_empty() {
            continue;
        }

        match args[0] {
            "/leave" => {
                client.leave_room(&room_id).await;
                success("[Returned to Lobby]");
                break;
            }
            "/help" => print_help(),
            "/active_users" => client.get_active_users().await,
            "/kick" => kick_user(client, args.clone()).await, 
            "/quit" => {
                warning("Quitting Program");
                std::process::exit(1);
            }
            _ => {
                client.chat_message(input).await;
            }
        }
    }
}


async fn join_room(client: &mut ChatClient, args: Vec<&str>) {
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

async fn kick_user(client: &mut ChatClient, args: Vec<&str>) {
    if args.len() < 2 {
        warning("Usage: /kick <username>");
        return;
    }
    
    client.kick_user(args[1]).await;
}


async fn create_room(client: &mut ChatClient, args: Vec<&str>) {
    if args.len() < 3 {
        warning("Usage: /create <room_id> <password>");
        return;
    }

    let room_id = args[1];
    let password = args[2];
    client.create_room(room_id, password).await;
}


#[tokio::main]
async fn main() {
    let server_url_ws = "ws://127.0.0.1:3000";
    let server_url = "http://127.0.0.1:3000";
    let mut client = ChatClient::init(server_url, server_url_ws);
    
     success("Welcome to the Rust Chat Room!");
    let mut logged_in = false;

    loop {
        while !logged_in {
            info("[Please /login or /sign_up or /help]");
            system_prompt(">");
            io::stdout().flush().unwrap();

            let mut input = String::new(); 
            if io::stdin().read_line(&mut input).is_err() {
                continue;
            }

            let user_input = input.trim();
            match user_input {
                "/login" => logged_in = login(&mut client).await,
                "/sign_up" => sign_up(&mut client).await,
                "/help" => print_help(),
                "/quit" => {
                    warning("Quitting Program");
                    std::process::exit(1);
                }
                _ => error("Unknown Command - try /help"),
            }
        }

        success("Connected to Chat Room Lobby");

        while logged_in {
            system_prompt("[Lobby]> ");
            io::stdout().flush().unwrap();

            let mut user_input = String::new();
            if io::stdin().read_line(&mut user_input).is_err() {
                continue;
            }

            let input = user_input.trim();
            let args: Vec<&str> = input.split_whitespace().collect();
            if args.is_empty() {
                continue;
            }

            match args[0] {
                "/join" => join_room(&mut client, args.clone()).await,
                "/all_rooms" => client.show_all_rooms(false).await,    
                "/active_rooms" => client.show_all_rooms(true).await,  
                "/create" => create_room(&mut client, args.clone()).await,
                "/delete" => delete_room(&mut client, args.clone()).await,  
                "/logout" => {
                    client.logout().await;
                    logged_in = false;
                }
                "/quit" => {
                    warning("Quitting Program");
                    std::process::exit(1);
                }
                "/help" => print_help(),
                _ => error("Unknown Command - try /help"),
            }
        }
    }
}
