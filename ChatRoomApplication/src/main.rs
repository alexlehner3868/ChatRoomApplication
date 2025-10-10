use std::io::{self, Write};


fn sign_up() -> bool {
    // Alex TODO 
    let mut signed_up = false;
    // Get username. Check that it doesnt already exist. loop until valid username or quit
    // Get password. Make sure password is vlaid. if not, reask for it 
    // prohib usernames or passwords that start with \ 
    signed_up
}

fn login() -> bool {
    // Alex TODO
    let mut login = false;
    // looop in here. if invalid username or password, give the option to renter credentials, sign up or quit

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
    println!("  /delete            Delete your chat room (owner only) (usage: /delete <room_id>\n");

    println!("Messaging Commands:");
    println!("  <message>          Type and send a message to your current room\n");

    println!("==============================");

}


fn alex_chat_room_loop(){
    println!("[Welcome to the Rust Chat Room Application!]");
    let mut logged_in = false;

    while !logged_in {
        println!("");
        println!(r"[Please /login or /sign_up or get /help]");
        print!("> ");
        io::stdout().flush().unwrap();
        let mut user_input = String::new(); 

        match io::stdin().read_line(&mut user_input){
            Ok(_) => {
                let user_input = user_input.trim(); 
                
                match user_input {
                    "/login" => {
                        logged_in = login();
                    },
                    "/sign_up" => {
                        logged_in = sign_up();
                    },
                    "/help" => { // Want to format help menu so that it has universal commands
                        print_help()
                    },
                    "/quit" =>{
                        println!("[Quitting Program]");
                        std::process::exit(1);
                    },
                    _ => {
                        println!("[Unknown Command");
                    }
                }
            }
            Err(_) => continue,
        }
    }
}

fn main() {
    //alex_chat_room_loop();
}
