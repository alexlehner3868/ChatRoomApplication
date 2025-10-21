use std::io::{self, Write};
use std::net::TcpStream;
use std::io::Read;

use crate::color_formatting::*;


// TODO ALEX : all these functions need to get a response from the server to validate that they worked and then return that or the return value to the user

pub struct ChatClient {
    //pub stream: TcpStream, UNCOMMENT WHEN SERVER IS SET UP
    pub stream: Option<TcpStream>, // delete when server is set yp
    pub username: Option<String>,
    pub current_room: Option<String>
}

impl ChatClient {
    //pub fn init(stream: TcpStream) -> Self { USE THIS WHEN SERVER IS SET UP
    pub fn init(stream: Option<TcpStream>) -> Self {
        ChatClient {
            stream,
            username: None,
            current_room: None,
        }
    }

    fn send_to_server(&mut self, message: &str) -> io::Result<String> {
        if let Some(stream) = &mut self.stream {
            stream.write_all(message.as_bytes())?;
            stream.flush()?;


            self.read_from_server()
        }else{
            Ok(String::from("/Success (Testing mode)"))
        }
   
    }

    fn read_from_server(&mut self) -> io::Result<String> { 
        if let Some(stream) = &mut self.stream {
            let mut buffer = [0; 512];
            let bytes_read = stream.read(&mut buffer)?;
            let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
            Ok(response)
        } else {
            Ok(String::from("/Success (Testing mode — no server)"))
        }
    }

    pub fn create_user(&mut self, username: &str, password: &str) -> Result<(), String> {
        match self.send_to_server(&format!("/create_user {} {}", username, password)) {
            Ok(response) => {
                if response.starts_with("/Success") {
                    Ok(())
                } else if response.starts_with("/Error") {
                    // TODO: Server needs to check that no other account has the same username
                    let reason = response.trim_start_matches("/Error").trim().to_string();
                    Err(reason)
                } else {
                    Err("Unexpected server response".to_string())
                }
            }
            Err(e) => Err(format!("Connection error: {}", e)),
        }
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<(), String> {
        match self.send_to_server(&format!("/login {} {}", username, password)) {
            Ok(response) => {
                if response.starts_with("/Success") {
                    Ok(())
                } else if response.starts_with("/Error") {
                    let reason = response.trim_start_matches("/Error").trim().to_string();
                    Err(reason)
                } else {
                    Err("Unexpected server response".to_string())
                }
            }
            Err(e) => Err(format!("Connection error: {}", e)),
        }
    }

    pub fn join_room(&mut self, room_id: &str, password: &str){
        self.send_to_server(&format!("/join {} {}", room_id, password));
        self.current_room = Some(room_id.to_string());
    }

    pub fn leave_room(&mut self, room_id: &str){
        self.send_to_server("/leave");    
        self.current_room = None; 
    }

   pub fn show_all_rooms(&mut self) {
        match self.send_to_server("/all_rooms") {
            Ok(response) => {
                header("All Rooms");

                if response.trim().is_empty() {
                    info("No chat rooms available.");
                } else {
                    let rooms: Vec<&str> = response.trim().lines().collect();
                    println!("Available Chat Rooms:");
                    for room in rooms {
                        println!("  - {}", room);
                    }
                }
            }
            Err(e) => {
                error(&format!("Failed to get rooms: {}", e));
            }
        }
    }

    pub fn show_active_rooms(&mut self){
        self.send_to_server("/active_rooms");
        // self.read_from_server() uncomment when set up TODO parse out input to return 
    }

    pub fn create_room(&mut self, room_id: &str, password: &str){
        self.send_to_server(&format!("/create {} {}", room_id, password));
    }

    pub fn delete_room(&mut self, room_id: &str){
        self.send_to_server(&format!("/delete {}", room_id));
    }

    pub fn kick_user(&mut self, username: &str){
        self.send_to_server(&format!("/kick {}", username));
    }

    pub fn send_message(&mut self, message: &str){
        self.send_to_server(message);
    }

    pub fn get_room_owner(&mut self, room_id: &str){
        self.send_to_server(&format!("/room_owner {}", room_id));
        // self.read_from_server() uncomment when set up and TODO parse out input to return 
    }

    pub fn get_active_users(&mut self, room_id: &str){
        self.send_to_server(&format!("/active_users {}", room_id));
    }
    
    pub fn logout(&mut self) -> io::Result<()> {
        if let Some(username) = &self.username {
            self.send_to_server(&format!("/logout {}", username))?;
        }

        self.username = None;
        self.current_room = None;

        Ok(())
    }
}