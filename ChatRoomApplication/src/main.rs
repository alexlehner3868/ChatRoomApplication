use std::io::{self, Write};
use colored::*;

// --------------------
// Helper functions
// --------------------
fn header(text: &str) {
    println!("{}", format!("[{}]", text).magenta().bold());
}

fn success(text: &str) {
    println!("{}", format!("[{}]", text).green());
}

fn error(text: &str) {
    println!("{}", format!("[{}]", text).red());
}

fn warning(text: &str) {
    println!("{}", format!("[{}]", text).yellow());
}

fn info(text: &str) {
    println!("{}", text);
}

fn user_message(username: &str, message: &str) {
    println!("{}", format!("{}: {}", username, message).blue());
}

fn my_message(message: &str) {
    println!("{}", format!("You: {}", message));
}

fn sign_up() -> bool {
    let mut signed_up = false;
    println!("");
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

        // Placeholder: check if username exists
        let username_exists = false;
        if username_exists {
            error("Username already exists");
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

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

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
            println!("");
            continue;
        }

        break password.to_string();
    };

    // Placeholder: create account in database
    let account_created = true;

    if account_created {
        success(&format!("Account created successfully for '{}'", username));
        signed_up = true;
    } else {
        error("Error creating account — please try again later");
    }

    println!("");
    signed_up
}

fn login() -> bool {
    let mut login = false;
    login = true;
    success("Login Successful");
    login
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
    println!("  /leave             Leave the current chat room\n");

    println!("Room Management Commands:");
    println!("  /active_users      Show all active users in the current room");
    println!("  /kick              Remove a user from your room. Need to own chat room (usage: /kick <username>)");
    println!("  /delete            Delete your chat room (owner only) (usage: /delete <room_id>)\n");

    println!("Messaging Commands:");
    println!("  <message>          Type and send a message to your current room\n");

    println!("==============================");
}

fn logout() {
    // TODO: mark user as inactive
}

fn show_all_rooms() {
    header("All Rooms");
}

fn show_active_rooms() {
    header("Active Rooms");
}

fn create_room(args: Vec<&str>) {
    if args.len() < 3 {
        warning("Usage: /create <room_id> <password>");
        return;
    }

    let room_id = args[1];
    success(&format!("Creating Room - {}", room_id));
}

fn delete_room(args: Vec<&str>){
    if args.len() < 2 {
        warning("Usage: /delete <room_id>");
        return;
    }
}

fn join_room(args: Vec<&str>) {
    if args.len() < 3 {
        warning("Usage: /join <room_id> <password>");
        return;
    }

    let room_id = args[1];
    let valid_credentials = true;

    if !valid_credentials {
        error("Invalid Room ID or Password");
        return;
    }

    in_chat_room(room_id);
}

fn kick_user(args: Vec<&str>) {
    if args.len() < 2 {
        warning("Usage: /kick <username>");
        return;
    }
}

// --------------------
// Room Functions
// --------------------
fn leave_room(room_id: &str) {
    warning(&format!("Leaving Room - {}", room_id));
    success("Returned to Lobby");
}

fn show_active_users(room_id: &str) {
    header(&format!("Active Users in {}", room_id));
}

fn in_chat_room(room_id: &str){
    success(&format!("Connected to {}", room_id));

    loop {
        print!("{}", format!("[{}]> ", room_id).cyan());
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
            "/leave" => leave_room(room_id),
            "/active_users" => show_active_users(room_id),
            "/kick" => kick_user(args),
            "/delete" => delete_room(args),
            "/quit" => {
                warning("Quitting Program");
                std::process::exit(1);
            }
            _msg => {
                // send message placeholder
            }
        }
    }
}

// --------------------
// Main Loop
// --------------------
fn alex_chat_room_loop() {
    success("Welcome to the Rust Chat Room Application!");
    let mut logged_in = false;

    while !logged_in {
        print!("{}", "[Please /login or /sign_up or get /help] > ".cyan());
        io::stdout().flush().unwrap();

        let mut input = String::new(); 
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let user_input = input.trim();
        match user_input {
            "/login" => logged_in = login(),
            "/sign_up" => logged_in = sign_up(),
            "/help" => print_help(),
            "/quit" => {
                warning("Quitting Program");
                std::process::exit(1);
            }
            _ => error("Unknown Command - get /help"),
        }
    }

    success("Connected to Chat Room Lobby");

    loop {
        print!("{}", "[Lobby] > ".cyan());
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
            "/all_rooms" => show_all_rooms(),
            "/active_rooms" => show_active_rooms(),
            "/create" => create_room(args.clone()),
            "/join" => join_room(args.clone()),
            _ => error("Unknown Command - get /help"),
        }
    }
}

fn main() {
    alex_chat_room_loop();
}
