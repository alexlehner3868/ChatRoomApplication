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
    // TODO --> Format help so it has all commands possible and seperates it by section (general(help, quit), intial( login, sign up), room manamgment etc)
    println!("");
    println!("[Help Menu]");
    println!(r"/login                        Login with known username and passowrd");
    println!(r"/sign_up                      Create a username and password");
    println!(r"/quit                         Quit the application");
    println!(r"/help                         See list of available commands");

    println!()
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
    alex_chat_room_loop();
}
