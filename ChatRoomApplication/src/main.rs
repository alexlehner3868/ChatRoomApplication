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
    // TODO ---> set the user as no longer active. will need some struct or something to know who the user is ALEX
}

fn show_all_rooms() {
    // TODO (need to query the Database) ALEX
    println!("[All Rooms]");
    // print list of rooms
}

fn show_active_rooms() {
    // TODO --> need to query list of rooms and active users in each room  ALEX
    println!("[Active Rooms]");
    // print active rooms 
    // - Room_Name (# Active Users)
}

fn create_room(args: Vec<&str>) {
    // TODO --> Need to connect to database?  ALEX
    
    // Too few arguments provided
    if args.len() < 3 {
        println!("[Usage: /create <room_id> <password>]");
        return;
    }

    let room_id= args[1];
    let password = args[2];

    // Check that the room_name is unique and valid 
    // Check that the password is good 

    // Call database (or server) to create the room and set the owner to this user

    println!("[Creating Room - {}]", room_id);
}

fn delete_room(args: Vec<&str>){
    // TODO -- connect to database and server ALEX

    if args.len() < 2 {
        println!("[Usage: /delete <room_id>]");
        return;
    }

    // Check that room exists 
    // Check that this user is the owner 

    // Kick out all users from the current room. loop through all users and call kick function
    // Delete room

}

fn join_room(args: Vec<&str>) {
    // TODO need to conect to backend and database ALEX

    // Too few arguments provided
    if args.len() < 3 {
        println!("[Usage: /join <room_id> <password>]");
        return;
    }
    let room_id = args[1];
    let passowrd = args[2];

    // Check that room exists and passowrd matches
    let valid_credentials = true;

    if !valid_credentials {
        println!("[Invalid Room ID or Password]");
        return;
    }
    // Mark user as active in the database
    // Connect user to chatroom and check that connections was successful. it if was then print message and go to in chat_room
    in_chat_room(room_id);
}

fn kick_user(args: Vec<&str>) {
    // TODO - connect to server and database ALEX

    // Too few arguments provided
    if args.len() < 2 {
        println!("[Usage: /kick <username>]");
        return;
    }
    // Check that the user is the owner of the database (can create a help function)
    // Communicate with the server and the database to remove the user from the room
}

fn leave_room(room_id: &str) {
    // TODO: connect to server and database and disconnect from room ALEX
    // -tell server that the user has left the room
    // Update the databse
    // broascast message to the chatroom 

    println!("[Leaving Room - {}]", room_id);

    // Verify that we've left the room 
    println!("[Returned to Lobby]");
}

fn show_active_users(room_id: &str) {
    // TODO: Connect to db to get list of active users ALEX
    println!("[Active Users in {}]", room_id);

    // print out list of users
}


fn in_chat_room(room_id: &str){
    // TODO --> Need to connect to the server and make async ALEX
    println!("[Connected to {}]", room_id);

    loop {
        print!("{}> ", room_id);
        std::io::stdout().flush().unwrap();

        let mut user_input = String::new();
        if std::io::stdin().read_line(&mut user_input).is_err() {
            continue;
        }

        let input = user_input.trim();
        if input.is_empty() {
            continue;
        }

        let args: Vec<&str> = input.split_whitespace().collect();

        match args[0] {
            "/help" => {
                print_help();
            }
            "/leave" =>{
                leave_room(room_id);
            }
            "/active_users" => {
                show_active_users(room_id);
            }
            "/kick" => {
                kick_user(args);
            }
            "/delete" => {
                delete_room(args);
            }
            "/quit" => {
                println!("[Quitting Program]");
                std::process::exit(1);
            }
            msg => {
                //send message ALEX
            }
        }

    }

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
        println!("");
        print!("[Lobby] > ");
        io::stdout().flush().unwrap();

        let mut user_input = String::new();
        
        match io::stdin().read_line(&mut user_input) {
            Ok(_) => {
                let input = user_input.trim();
                let args: Vec<&str> = input.split_whitespace().collect();

                match args[0] {
                    "/help" => {
                        print_help();
                    }
                    "/quit" => {
                        println!("[Quitting Program]");
                        std::process::exit(1);
                    }
                    "/all_rooms" => {
                        show_all_rooms();
                    }
                    "/active_rooms" => {
                        show_active_rooms();
                    }
                    "/create" => {
                        create_room(args);
                    }
                    "/join" => {
                        join_room(args);
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
