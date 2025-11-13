use std::io::{self, Write};
use tokio;
use futures_util::TryStreamExt;

mod color_formatting;
mod chat_client; 
mod messages;
mod terminal_erasing;
mod user_commands;

use color_formatting::*;
use terminal_erasing::*;
use chat_client::ChatClient;
use crate::messages::ServerWsMessage;
use user_commands::*;


async fn in_chat_room(client: &mut ChatClient, room_id: &str) {
    success(&format!("[Connected to {}]", room_id));

    // Channel to signal that a user needs to exit the room
    let (exit_tx, exit_rx) = tokio::sync::watch::channel(false);

    // Spawn task for incoming WebSocket messages
    let mut receiver = client.ws_receiver.take().unwrap();

    // Clones used in async spawned task
    let username_clone = client.username.clone(); 
    let exit_tx_clone = exit_tx.clone(); 
    let current_room = room_id.to_string();

    // Spawn task to listen for incoming WebSocket messages
    tokio::spawn(async move {
        while let Ok(Some(msg)) = receiver.try_next().await {
            if let Ok(text) = msg.to_text() {
                if let Ok(parsed) = serde_json::from_str::<ServerWsMessage>(text) {
                    match parsed {
                        // Chat room message from another user
                        ServerWsMessage::MessageBroadcast(chat_msg) => {
                            if chat_msg.user_id == "system" {
                                system_message(&format!("{}: {}", chat_msg.user_id, chat_msg.content));
                            } else if chat_msg.user_id != username_clone.clone().unwrap_or_default() {
                                erase_current_line();
                                user_message(&chat_msg.timestamp, &chat_msg.user_id, &chat_msg.content);
                                system_prompt(&format!("[{}]> ", chat_msg.room_id));
                            }
                        }
                        // If current room was deleted, alert user and signal exit
                        ServerWsMessage::RoomDeleted { room_id: deleted_room } => {
                            if deleted_room == current_room {
                                warning("[Room has been deleted]");
                                let _ = exit_tx_clone.send(true);
                                break;
                            }
                        }
                        // Notify that a new user joined the chat room
                        ServerWsMessage::UserJoined { room_id: joined_room, user_id: joined_user } => {
                            if joined_room == current_room && joined_user != username_clone.clone().unwrap_or_default() {
                                erase_current_line();
                                system_message(&format!("[{} has joined]", joined_user));
                                system_prompt(&format!("[{}]> ", joined_room));
                            }
                        }
                        // Notify that a user left the room
                        ServerWsMessage::UserLeft { room_id: left_room, user_id: left_user } => {
                            if left_room == current_room && left_user != username_clone.clone().unwrap_or_default() {
                                erase_current_line();
                                system_message(&format!("[{} has left]", left_user));
                                system_prompt(&format!("[{}]> ", left_room));
                            }
                        }
                        // Handle user being kicked from chat
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
                        ServerWsMessage::Pong { .. } => {} // TBD
                        // Display error from server
                        ServerWsMessage::Error { error_msg } => {
                            error(&error_msg);
                        }
                    }
                }
            }
        }
    });

    // User input loop
    loop {
        // Check if forced to leave room (on kick or room deletion)
        if *exit_rx.borrow() {
            client.current_room = None;
            success("[Returned to Lobby]");
            break;
        }

        system_prompt(&format!("[{}]> ", room_id));
        io::stdout().flush().unwrap();

        // Get user input
        let mut user_input = String::new();
        if io::stdin().read_line(&mut user_input).is_err() {
            continue;
        }

        let input = user_input.trim();
        if input.is_empty() {
            continue;
        }

        // Remove the prompt line for cleanliness of output (TODO - does this also remove a /kick or a /active_users ??? ALEX sort out)
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


#[tokio::main]
async fn main() {
    // URLs of the server (for HTTP requests and for WebSockets)
    let server_url_ws = "ws://127.0.0.1:3000";
    let server_url = "http://127.0.0.1:3000";

    // Create the ChatClient
    let mut client = ChatClient::init(server_url, server_url_ws);
    
    success("Welcome to the Rust Chat Room!");

    let mut logged_in = false;

    loop {
        // Authentication loop - Keep iterating until user logs in 
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
        
        // Lobby loop 
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
