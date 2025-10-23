
use std::io::{self, Write};
use tokio;
use rpassword::read_password;

mod color_formatting;
mod chat_client; 
mod messages;

use color_formatting::*;
use chat_client::ChatClient;

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

        if username.is_empty() {
            error("Invalid username — cannot be empty");
            continue;
        } else if username.starts_with('/') {
            error("Invalid username — cannot start with /");
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

    let password = loop { // Change to use read_password for security (ALEX todo)
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

    client.login(username, &password).await

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

async fn create_room(client: &mut ChatClient, args: Vec<&str>) {
    if args.len() < 3 {
        warning("Usage: /create <room_id> <password>");
        return;
    }

    let room_id = args[1];
    let password = args[2];
    client.create_room(room_id, password).await;
}

async fn delete_room(client: &mut ChatClient, args: Vec<&str>){
    if args.len() < 2 {
        warning("Usage: /delete <room_id>");
        return;
    }
    let room_id = args[1];
    
    client.delete_room(room_id).await;
}

async fn join_room(client: &mut ChatClient, args: Vec<&str>) {
     // TODO connect to chat cliebnt ALEX
    if args.len() < 3 {
        warning("Usage: /join <room_id> <password>");
        return;
    }

    let room_id = args[1];
    let password = args[2];

    if client.join_room(room_id, password).await{
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

async fn in_chat_room(client: &mut ChatClient, room_id: &str){
     // TODO connect to chat cliebnt ALEX
     // TODO get async messages and send async messages

    success(&format!("[Connected to {}]", room_id));

    loop {
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

        let args: Vec<&str> = input.split_whitespace().collect();

        match args[0] {
            "/help" => print_help(),
            "/leave" => {
                client.leave_room(room_id).await;
                success("[Returned to Lobby]");
                break;
            }
            "/active_users" => client.get_active_users().await,
            "/kick" => kick_user(client, args).await,   // TODO connect to chat cliebnt ALEX
            "/quit" => {
                warning("Quitting Program");
                std::process::exit(1);
            }
            _msg => {
                 // How to send and recieve async messages ALEX
            }
        }
    }
}


async fn alex_chat_room_loop(client: &mut ChatClient) {
    success("Welcome to the Rust Chat Room Application!");
    let mut logged_in = false;

    loop {
        while !logged_in {
            info("[Please /login or /sign_up or get /help]");
            system_prompt(">");
            io::stdout().flush().unwrap();

            let mut input = String::new(); 
            if io::stdin().read_line(&mut input).is_err() {
                continue;
            }

            let user_input = input.trim();
            match user_input {
                "/login" => logged_in = login(client).await,  
                "/sign_up" => sign_up(client).await,
                "/help" => print_help(),
                "/quit" => {
                    warning("Quitting Program");
                    std::process::exit(1);
                }
                _ => error("Unknown Command - get /help"),
            }
        }

        success("Connected to Chat Room Lobby");

        while logged_in {
            system_prompt("[Lobby] > ");
            io::stdout().flush().unwrap();

            let mut user_input = String::new();
            if io::stdin().read_line(&mut user_input).is_err() {
                continue;
            }

            let input = user_input.trim();
            let args: Vec<&str> = input.split_whitespace().collect();
            if args.is_empty() { continue; }

            match args[0] {
                "/help" => print_help(),
                "/quit" => {
                    warning("Quitting Program");
                    std::process::exit(1);
                }
                "/all_rooms" => client.show_all_rooms(false).await,    
                "/active_rooms" => client.show_all_rooms(true).await,  // TODO connect to chat cliebnt ALEX
                "/create" => create_room(client, args.clone()).await,  // TODO connect to chat cliebnt ALEX
                "/delete" => delete_room(client, args.clone()).await,  // TODO connect to chat cliebnt ALEX
                "/join" => join_room(client, args.clone()).await,  // TODO connect to chat cliebnt ALEX
                "/logout" => {
                    client.logout().await;
                    logged_in = false;
                }
                _ => error("Unknown Command - get /help"),
            }
        }
    }
}

#[tokio::main]
async fn main() {

    // Configure the client 
    let server_url = "http://127.0.0.1:8000"; 
    let mut client = ChatClient::init(server_url);

    alex_chat_room_loop(&mut client).await;
}
