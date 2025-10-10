use std::io::{self, Write};

// Struct to represent the state of the current user
struct CurrentUser {
    username: String,
    current_room: String,
}


fn alex_chat_room_loop(){
    println!("[Welcome to the Rust Chat Room Application!]");
    let mut logged_in = false;

    while !logged_in {
        println!("");
        println!(r"[Please /login or /sign_up]");
        print!("> ");
        io::stdout().flush().unwrap();
        let mut user_input = String::new(); 

        match io::stdin().read_line(&mut user_input){
            Ok(_) => {
                let user_input = user_input.trim(); 
                
                match user_input {
                    "/login" => {
                        // TODO ALEX
                    },
                    "/sign_up" => {
                        // TODO ALEX
                    },
                    "/help" => {
                        println!("");
                        println!("[Help Menu]");
                        println!(r"/login                        Login with known username and passowrd");
                        println!(r"/sign_up                      Create a username and password");
                        println!(r"/quit                         Quit the application");
                        println!(r"/help                         See list of available commands");

                        println!()
                    },
                    "/quit" =>{
                        println!("[Exiting Program]");
                        std::process::exit(1);
                    },
                    _ => {
                        println!("[Unknown Command - Enter a valid command or use /help]]");
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
