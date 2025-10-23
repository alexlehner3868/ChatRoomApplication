use reqwest::Client;
use serde::{Serialize, Deserialize};

use crate::color_formatting::*;
use crate::messages::*;

pub struct ChatClient {
    pub server_url: String,
    pub http: Client,
    pub auth_token: Option<String>,
    pub username: Option<String>,
    pub current_room: Option<String>,
}

impl ChatClient {

    pub fn init(server_url: &str) -> Self {
        ChatClient {
            server_url: server_url.to_string(),
            http: Client::new(),
            auth_token: None,
            username: None,
            current_room: None,
        }
    }

    pub async fn send_json_to_server<T: Serialize>( &self, endpoint: &str, msg: &T,) -> Result<String, reqwest::Error> {
        let mut request = self.http.post(format!("{}/{}", self.server_url, endpoint)).json(msg);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?.text().await?;
        Ok(response)
    }

    pub async fn create_user(&mut self, username: &str, password: &str) -> bool {
        let req = RegisterRequest {
            user_id: username.to_string(),
            password: password.to_string(),
        };

        match self.send_json_to_server("create_user", &req).await {
            Ok(resp_str) => {
                if let Ok(resp) = serde_json::from_str::<AuthSuccessResponse>(&resp_str) {
                    success(&format!("User '{}' created successfully!", resp.user_id));
                    self.auth_token = Some(resp.token);
                    return true;
                } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&resp_str) {
                    match err {
                        ErrorResponse::AuthenticationFailed { message } => {
                            error(&format!("Error: Authentication failed: {}", message));
                        }
                        ErrorResponse::ServerError { message } => {
                            error(&format!("Server error: {}", message));
                        }
                        ErrorResponse::UserAlreadyExists { user_id } => {
                            error(&format!("Error: User {} already exists", user_id));
                        }
                        _ => {
                            error(&format!("Error: {:?}", err));
                        }
                    }
                } else {
                    error("Unexpected server response");
                }
            }
            Err(e) => {
                error(&format!("Connection error: {}", e));
            }
        }

        false
    }


    pub async fn login(&mut self, username: &str, password: &str) -> bool {
        let req = LoginRequest {
            user_id: username.to_string(),
            password: password.to_string()
        };

        match self.send_json_to_server("login", &req).await {
            Ok(resp_str) => {
                if let Ok(resp) = serde_json::from_str::<AuthSuccessResponse>(&resp_str) {
                    success(&format!("Welcome {}!", resp.user_id));
                    self.auth_token = Some(resp.token);
                    self.username = Some(resp.user_id);
                    true
                } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&resp_str) {
                    match err {
                        ErrorResponse::AuthenticationFailed { message } => {
                            error(&format!("Error: Authentication failed: {}", message));
                        }
                        ErrorResponse::InvalidPassword {..} | ErrorResponse::UserNotFound {..} => {
                            error("Error: Invalid username or password");
                        }
                        ErrorResponse::ServerError { message } => {
                            error(&format!("Server error: {}", message));
                        }
                        _ => {
                            error(&format!("Error: {:?}", err));
                        }
                    }
                    false
                } else {
                    error("Unexpected server response");
                    false
                }
            }
            Err(e) => {
                error(&format!("Connection error: {}", e));
                false
            }
        }
    }


    pub async fn join_room(&mut self, room_id: &str, password: &str) -> bool {
        let req = JoinRoomRequest {
            room_id: room_id.to_string(),
            room_password: password.to_string(),
        };

        match self.send_json_to_server("join_room", &req).await {
            Ok(resp_str) => {
                if let Ok(resp) = serde_json::from_str::<JoinRoomResponse>(&resp_str) {
                    success(&format!("[Connected to '{}']", resp.room_id));
                    self.current_room = Some(resp.room_id.clone());
                
                    if !resp.chat_history.is_empty() {
                        header("Chat History");
                        for msg in resp.chat_history {
                            if msg.user_id == self.username.clone().unwrap_or_default() {
                                my_message(&msg.timestamp, &msg.content);
                            }else{
                                user_message(&msg.timestamp, &msg.user_id, &msg.content);
                            }
                        }
                    }

                    true
                } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&resp_str) {
                    match err {
                        ErrorResponse::AuthenticationFailed { message } => {
                            error(&format!("Error: Authentication failed: {}", message));
                        }
                        ErrorResponse::InvalidPassword { .. } => {
                            error("Error: Invalid password");
                        }
                        ErrorResponse::RoomNotFound { room_id } => {
                            error(&format!("Error: Room {} not found", room_id));
                        }
                        _ => {
                            error(&format!("Error: {:?}", err));
                        }
                    }
                    false
                } else {
                    error("Unexpected server response");
                    false
                }
            }
            Err(e) => {
                error(&format!("Connection error: {}", e));
                false
            }
        }
    }


    pub async fn leave_room(&mut self, room_id: &str) {
        let msg = ClientWsMessage::LeaveRoom {
            room_id: room_id.to_string(),
        };

        let response = match self.send_json_to_server("leave_room", &msg).await {
            Ok(resp) => resp,
            Err(e) => {
                error(&format!("Connection error: {}", e));
                return; // exit the function on error
            }
        };

        if let Ok(err_resp) = serde_json::from_str::<ErrorResponse>(&response) {
            match err_resp {
                ErrorResponse::AuthenticationFailed { message } => {
                    error(&format!("Authentication failed: {}", message));
                    return;
                }

                ErrorResponse::ServerError { message } => {
                    error(&format!("Server error: {}", message));
                    return;
                }
                _ => {}
            };
        }

        self.current_room = None;
        success(&format!("Successfully left {}", room_id));
    }


    pub async fn show_all_rooms(&mut self, active_room_only: bool) {

        let req = ListRoomsRequest {
            only_active: active_room_only,
        };

        let response = match self.send_json_to_server("all_rooms", &req).await {
            Ok(resp) => resp,
            Err(e) => {
                error(&format!("Connection error: {}", e));
                return;
            }
        };

        header("[All Rooms]");
        let parsed: Result<ListRoomsResponse, _> = serde_json::from_str(&response);

        match parsed {
            Ok(list_resp) => {
                if list_resp.rooms.is_empty() {
                    info(" - No chat rooms exist");
                } else {
                    for room in list_resp.rooms {
                        if active_room_only {
                            info(&format!( " - {} [{} users]", room.room_id, room.users_count));
                        }else{
                            info(&format!( " - {}", room.room_id));
                        }
                    }
                }
            }
            Err(_) => error("Failed to parse server response"),
        }
    }
    
    pub async fn create_room(&mut self, room_id: &str, password: &str){

      let req = CreateRoomRequest {
        room_id: room_id.to_string(),
        room_password: password.to_string(),
        };

        let response = match self.send_json_to_server("create_room", &req).await {
            Ok(resp) => resp,
            Err(e) => {
                error(&format!("Connection error: {}", e));
                return;
            }
        };

        if let Ok(resp) = serde_json::from_str::<CreateRoomResponse>(&response) {
            success(&format!("Room Created - {}", resp.room_id));
        }else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&response) {
            match err {
                ErrorResponse::RoomAlreadyExists { room_id } => {
                    error(&format!("Error: Room '{}' already exists", room_id));
                }
                ErrorResponse::AuthenticationFailed { message } => {
                    error(&format!("Error: Authentication failed: {}", message));
                }
                ErrorResponse::ServerError { message } => {
                    error(&format!("Server error: {}", message));
                }
                _ => {
                    error(&format!("Error: {:?}", err));
                }
            }
        }else {
            error(&format!("Unexpected server response: {}", response));
        }
    }

    pub async fn delete_room(&mut self, room_id: &str){
        let req = DeleteRoomRequest {
            room_id: room_id.to_string(),
        };

        let response = match self.send_json_to_server("delete_room", &req).await {
            Ok(resp) => resp,
            Err(e) => {
                error(&format!("Connection error: {}", e));
                return;
            }
        };

        if let Ok(resp) = serde_json::from_str::<SuccessResponse>(&response) {
                success(&format!("{}", resp.message));

        } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&response) {
            match err {
                ErrorResponse::RoomNotFound { room_id } => {
                    error(&format!("Error: Room '{}' does not exist", room_id));
                }
                ErrorResponse::InvalidPermissions { .. } => {
                    error(&format!("Error:  You are not the owner of '{}'", room_id));
                }
                ErrorResponse::ServerError { message } => {
                    error(&format!("Server error: {}", message));
                }
                _ => {
                    error(&format!("Error: {:?}", err));
                }
            }
        } else {
            error(&format!("Unexpected server response: {}", response));
        }

    }

    pub async fn kick_user(&mut self, username: &str){
        let current_room = self.current_room.clone().unwrap_or_else(|| "unknown_room".to_string());
        let req = ClientWsMessage::KickUser {
            room_id: current_room.clone(),
            user_id: username.to_string()
        };

        let response = match self.send_json_to_server("kick_user", &req).await {
            Ok(resp) => resp,
            Err(e) => {
                error(&format!("Connection error: {}", e));
                return;
            }
        };

        if let Ok(resp) = serde_json::from_str::<SuccessResponse>(&response) {
            success(&format!("User '{}' has been kicked from room", username));

        } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&response) {
            match err {
                ErrorResponse::AuthenticationFailed { message } => {
                    error(&format!("Error: Authentication failed: {}", message));
                }
                ErrorResponse::NotInRoom {room_id } => {
                    error(&format!("Error: User '{}' not in room {}", username, room_id));
                }
                ErrorResponse::ServerError { message } => {
                    error(&format!("Error: Server error: {}", message));
                }
                ErrorResponse::InvalidPermissions { ..} => {
                    error(&format!("Error: You dont own room {}; cannot kick {}", current_room, username));
                } 
                _ => {
                    error(&format!("Error: {:?}", err));
                }
            }

        }else {
            error(&format!("Unexpected server response: {}", response));
        }
    }

  
    pub async fn get_active_users(&mut self) {
        let room = match &self.current_room {
            Some(current_room) => current_room,
            None => return, 
        };

        let req = ListRoomUsersRequest {
            room_id: room.to_string(),
        };

        let response = match self.send_json_to_server("list_room_users", &req).await {
            Ok(resp) => resp,
            Err(e) => {
                error(&format!("Connection error: {}", e));
                return;
            }
        };

        if let Ok(users_resp) = serde_json::from_str::<ListRoomUsersResponse>(&response) {
            header(&format!("Active Users in '{}'", room));
            if users_resp.active_users.is_empty() {
                info(" - No active users");
            } else {
                for user in users_resp.active_users {
                    println!(" - {}", user);
                }
            }
        } else if let Ok(err_resp) = serde_json::from_str::<ErrorResponse>(&response) {
            match err_resp {
                ErrorResponse::ServerError { message } => {
                    error(&format!("Server error: {}", message));
                }
                _ => error(&format!("Unexpected error: {:?}", err_resp)),
            }
        } else {
            error(&format!("Unexpected server response: {}", response));
        }
    }

    pub async fn logout(&mut self){

        if let Some(username) = &self.username {
            let req = LogoutRequest {}; 

            match self.send_json_to_server("logout", &req).await {

                Ok(resp_str) => {
                    if let Ok(_resp) = serde_json::from_str::<SuccessResponse>(&resp_str) {
                        success(&format!("User '{}' logged out successfully", username));
                        self.username = None;
                        self.current_room = None;
                    } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&resp_str) {
                        match err {
                            ErrorResponse::AuthenticationFailed { message } => {
                                error(&format!("Logout failed: {}", message));
                            }
                            _ => {
                                error(&format!("Logout failed: {:?}", err));
                            }
                        }
                    } else {
                        error(&format!("Unexpected server response: {}", resp_str));
                    }
                }
                Err(e) => error(&format!("Connection error during logout: {}", e)),
            }
        }
    }

}