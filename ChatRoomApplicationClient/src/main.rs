
use std::io::{self, Write};
use std::net::TcpStream;
use rpassword::read_password;

mod color_formatting;
mod chat_client; 

use color_formatting::*;
use chat_client::ChatClient;


fn sign_up(client: &mut ChatClient) -> bool { 
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
            return false;
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
            return false;
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

    let mut user_created = false;
    match client.create_user(&username, &password) {
        Ok(_) => {
            success(&format!("Account created succesfully for {}", username));
            user_created = true;
        },
        Err(reason) => {
            error(&format!("Failed to create account: {}", reason));
        }
    }
 
    user_created
}

fn login(client: &mut ChatClient) -> bool {
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

    match client.login(username, &password) {
        Ok(_) => {
            client.username = Some(username.to_string());
            success(&format!("Login successful — welcome, {}!", username));
            return true
        },
        Err(reason) => {
            error(&format!("Login failed: {}", reason));
            return false
        }
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

fn logout(client: &mut ChatClient) {
    client.logout();
    success("You have successfully logged out");
}

fn show_all_rooms() {
    header("All Rooms");
    // TODO connect to db and print all rooms 
    //TODO connec to chat client ALEX
}

fn show_active_rooms() {
    header("Active Rooms");
    // TODO connect to database and get list of each room. get number of each person in each room
    //TODO connec to chat client ALEX
}

fn create_room(args: Vec<&str>) {
    // TODO connect to chat cliebnt ALEX
    if args.len() < 3 {
        warning("Usage: /create <room_id> <password>");
        return;
    }

    let room_id = args[1];
    // Todo - cehck that room name doenst already exist. Add into db the room name and password. 
    success(&format!("Creating Room - {}", room_id));
}

fn delete_room(args: Vec<&str>){
     // TODO connect to chat cliebnt ALEX
    if args.len() < 2 {
        warning("Usage: /delete <room_id>");
        return;
    }
    let room_id = args[1];
    // TODO check that room name exists. 
    // TODO check that the current user is the owner of the room 
    // TODO remove "kick" all active users from the room 
    // TODO delete the room from the db 
    success(&format!("Deleting Room - {}", room_id));
}

fn join_room(args: Vec<&str>) {
     // TODO connect to chat cliebnt ALEX
    if args.len() < 3 {
        warning("Usage: /join <room_id> <password>");
        return;
    }

    let room_id = args[1];
    let valid_credentials = true;
    // TODO check that room exists and password matches
    if !valid_credentials {
        error("Invalid Room ID or Password");
        return;
    }
    // todo join the room
    in_chat_room(room_id);
}

fn kick_user(args: Vec<&str>) {
     // TODO connect to chat cliebnt ALEX
    if args.len() < 2 {
        warning("Usage: /kick <username>");
        return;
    }
    // TODO check that the user is the owner 
    // TODO check that the user is an active user
    // TODO remove the user from the room (communicate with server and db)
}


fn leave_room() {

 // TODO connect to chat cliebnt ALEX
     warning(&format!("Leaving Room - ROOM NAME FROM STRUCT"));
    success("Returned to Lobby");
}

fn show_active_users(room_id: &str) {
     // TODO connect to chat cliebnt ALEX
    header(&format!("Active Users in {}", room_id));
    // TODO get list of active users from the database (ALEX)
}

fn in_chat_room(room_id: &str){
     // TODO connect to chat cliebnt ALEX
     // TODO get async messages and send async messages

    success(&format!("Connected to {}", room_id));

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
                leave_room();
                break;
            }
            "/active_users" => show_active_users(room_id),  // TODO connect to chat cliebnt ALEX
            "/kick" => kick_user(args),   // TODO connect to chat cliebnt ALEX
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


fn alex_chat_room_loop(client: &mut ChatClient) {
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
                "/login" => logged_in = login(client),  
                "/sign_up" => logged_in = sign_up(client),
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
                "/all_rooms" => show_all_rooms(),          // TODO connect to chat cliebnt ALEX
                "/active_rooms" => show_active_rooms(),  // TODO connect to chat cliebnt ALEX
                "/create" => create_room(args.clone()),  // TODO connect to chat cliebnt ALEX
                "/delete" => delete_room(args.clone()),  // TODO connect to chat cliebnt ALEX
                "/join" => join_room(args.clone()),  // TODO connect to chat cliebnt ALEX
                "/logout" => {
                    logout(client);
                    logged_in = false;
                }
                _ => error("Unknown Command - get /help"),
            }
        }
    }
}

fn main() {

    let server_address = "127.0.0.1:12345"; // placehodler
    let stream = TcpStream::connect(server_address);

    //let mut client = ChatClient::init(stream); uncomment when server is set up 
    let mut client = ChatClient::init(None);
    alex_chat_room_loop(&mut client);
}
