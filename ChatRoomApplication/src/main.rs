use std::io::{self, Write};

fn sign_up() -> bool {
    // Alex TODO 
    let mut signed_up = false;
    // Get username. Check that it doesn't already exist. Loop until valid username or quit
    // Get password. Make sure password is valid. If not, reask for it 
    // Prohibit usernames or passwords that start with '\'
    // Likely need a struct with the current state of the user (name, current room, etc)
    signed_up
}

fn login() -> bool {
    // Alex TODO
    let mut login = false;
    // Loop in here. If invalid username or password, give the option to re-enter credentials, sign up, or quit
    // Likely need a struct with the current state of the user (name, current room, etc)
    login = true;
    println!("[Login Successful]\n");
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
    // TODO ---> set the user as no longer active. will need some struct or something to know who the user is
}

fn alex_chat_room_loop() {
    println!("[Welcome to the Rust Chat Room Application!]");
    let mut logged_in = false;

    // Loop until user is logged in either through login or signup
    while !logged_in {
        println!("");
        println!(r"[Please /login or /sign_up or get /help]");
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new(); 

        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let user_input = input.trim();

                match user_input {
                    "/login" => {
                        logged_in = login();
                    }
                    "/sign_up" => {
                        logged_in = sign_up();
                    }
                    "/help" => {
                        print_help();
                    }
                    "/quit" => {
                        println!("[Quitting Program]");
                        std::process::exit(1);
                    }
                    _ => {
                        println!("[Unknown Command - get /help]");
                    }
                }
            }
            Err(_) => continue,
        }
    }

    println!("[Connected to Chat Room Lobby]");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut user_input = String::new();

        match io::stdin().read_line(&mut user_input) {
            Ok(_) => {
                let user_input = user_input.trim();

                match user_input {
                    "/help" => {
                        print_help();
                    }
                    "/quit" => {
                        println!("[Quitting Program]");
                        std::process::exit(1);
                    }
                    _ => {
                        println!("[Unknown Command - get /help]");
                    }
                }
            }
            Err(_) => continue,
        }
    }
}

fn main() {
    alex_chat_room_loop();
}
