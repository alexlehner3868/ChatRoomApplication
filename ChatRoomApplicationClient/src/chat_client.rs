use std::io::{self, Write};
use std::net::TcpStream;

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

    fn send_to_server(&mut self, message: &str) -> std::io::Result<()> {
        if let Some(stream) = &mut self.stream {
            stream.write_all(message.as_bytes())?;
            stream.flush()?;
        }else{
            println!("(Testing) would have sent {}", message);
        }
        Ok(())

        // TODO need to add something in here for errors?? 
    }

    pub fn create_user(&mut self, username: &str, password: &str) -> bool {
        self.send_to_server(&format!("/create_user {} {}", username, password));

        // TODO get server response and check if error or not ???
        true
    }

    pub fn join_room(&mut self, room_id: &str, password: &str){
        self.send_to_server(&format!("/join {} {}", room_id, password));
        self.current_room = Some(room_id.to_string());
    }

    pub fn leave_room(&mut self, room_id: &str){
        self.send_to_server("/leave");    
        self.current_room = None; 
    }

    pub fn show_all_rooms(&mut self){
        self.send_to_server("/all_rooms");
    }

    pub fn show_active_rooms(&mut self){
        self.send_to_server("/active_rooms");
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
    }

    pub fn get_active_users(&mut self, room_id: &str){
        self.send_to_server(&format!("/active_users {}", room_id));
    }
}