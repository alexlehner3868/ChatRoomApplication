use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{WebSocketStream, connect_async, tungstenite::Message};

use crate::color_formatting::*;
use crate::messages::*;

// Struct to store info for the client-server connection
pub struct ChatClient {
    pub server_url: String,
    pub server_url_ws: String,
    pub http: Client,
    pub auth_token: Option<String>,
    pub username: Option<String>,
    pub current_room: Option<String>,
    pub ws_sender: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    pub ws_receiver: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}

impl ChatClient {
    pub fn init(server_url: &str, ws_url: &str) -> Self {
        ChatClient {
            server_url: server_url.to_string(),
            server_url_ws: ws_url.to_string(),
            http: Client::new(),
            auth_token: None,
            username: None,
            current_room: None,
            ws_sender: None,
            ws_receiver: None,
        }
    }

    // Send chat message to the server to send to a room
    pub async fn chat_message(&mut self, content: &str) {
        if let Some(sender) = &mut self.ws_sender {
            // Construct the message to send
            let msg = ClientWsMessage::SendMessage {
                room_id: self.current_room.clone().unwrap_or_default(), // current room
                content: content.to_string(),                           // chat message
            };

            // Convert to JSON
            let serialized = serde_json::to_string(&msg).unwrap();

            // Send to server through websocket
            if sender.send(Message::Text(serialized.into())).await.is_ok() {
                my_message(content);
            } else {
                error("Failed to send message through WebSocket");
            }
        } else {
            error("Not connected to WebSocket server");
        }
    }

    // Function to send JSON. to the server through HTTP post request
    pub async fn send_json_to_server<T: Serialize>(
        &self,
        endpoint: &str,
        msg: &T,
    ) -> Result<String, reqwest::Error> {
        let mut request = self
            .http
            .post(format!("{}/{}", self.server_url, endpoint))
            .json(msg);

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

        match self.send_json_to_server("register", &req).await {
            Ok(resp_str) => {
                if let Ok(resp) = serde_json::from_str::<AuthSuccessResponse>(&resp_str) {
                    // User created successfully
                    success(&format!("User '{}' created successfully!", resp.user_id));
                    self.auth_token = Some(resp.token);
                    return true;
                } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&resp_str) {
                    // Failed to create user
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
            password: password.to_string(),
        };
        // self.username = Some(username.to_string()); // un comment for testing
        //return true; // uncomment for testing

        // Send login request to server
        match self.send_json_to_server("login", &req).await {
            Ok(resp_str) => {
                if let Ok(resp) = serde_json::from_str::<AuthSuccessResponse>(&resp_str) {
                    // User was successfully logged in
                    success(&format!("Welcome {}!", resp.user_id));

                    // Save info to chat client
                    self.auth_token = Some(resp.token);
                    self.username = Some(resp.user_id);
                    true
                } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&resp_str) {
                    // Failed to log in
                    match err {
                        ErrorResponse::AuthenticationFailed { message } => {
                            error(&format!("Error: Authentication failed: {}", message));
                        }
                        ErrorResponse::InvalidPassword { .. }
                        | ErrorResponse::UserNotFound { .. } => {
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

        // Send join request to server
        match self.send_json_to_server("join_room", &req).await {
            Ok(resp_str) => {
                if let Ok(resp) = serde_json::from_str::<JoinRoomResponse>(&resp_str) {
                    // Successfully joined
                    // Save room to client
                    self.current_room = Some(resp.room_id.clone());

                    // Connect WebSocket
                    if !self.connect_ws_for_room().await {
                        return false;
                    }

                    // Print out chat history
                    if !resp.chat_history.is_empty() {
                        header("Chat History");
                        for msg in resp.chat_history {
                            if msg.user_id == self.username.clone().unwrap_or_default() {
                                my_message(&msg.content);
                            } else {
                                user_message(&msg.timestamp, &msg.user_id, &msg.content);
                            }
                        }
                    }

                    true
                } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&resp_str) {
                    // Failed to join room
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
        let req = ClientWsMessage::LeaveRoom {
            room_id: room_id.to_string(),
        };

        // Convert to JSON
        let serialized = match serde_json::to_string(&req) {
            Ok(s) => s,
            Err(e) => {
                error(&format!("Failed to serialize LeaveRoom request: {}", e));
                return;
            }
        };

        // Send request to server via websocket
        if let Some(sender) = &mut self.ws_sender {
            if let Err(e) = sender.send(Message::Text(serialized.into())).await {
                error(&format!(
                    "Error: Failed to send LeaveRoom WS message: {}",
                    e
                ));
            }
        } else {
            error("Error: No WebSocket connection to send LeaveRoom request");
        }

        // Close WS
        if let Some(mut sender) = self.ws_sender.take() {
            if let Err(e) = sender.close().await {
                error(&format!("Failed to close WebSocket: {}", e));
            }
        }

        // Update client state
        self.ws_receiver = None;
        self.current_room = None;

        system_message(&format!("[Left {}]", room_id));
    }

    pub async fn show_all_rooms(&mut self, active_room_only: bool) {
        let req = ListRoomsRequest {
            only_active: active_room_only,
        };

        // Send request to server
        let response = match self.send_json_to_server("all_rooms", &req).await {
            Ok(resp) => resp,
            Err(e) => {
                error(&format!("Connection error: {}", e));
                return;
            }
        };

        // Parse response from server response
        let parsed: Result<ListRoomsResponse, _> = serde_json::from_str(&response);

        header("All Rooms");
        match parsed {
            Ok(list_resp) => {
                if list_resp.rooms.is_empty() {
                    info(" - No chat rooms exist");
                } else {
                    // Print out all the rooms and the active user counts
                    for room in list_resp.rooms {
                        if active_room_only {
                            info(&format!(" - {} [{} users]", room.room_id, room.users_count));
                        } else {
                            info(&format!(" - {}", room.room_id));
                        }
                    }
                }
            }
            Err(_) => error("Failed to parse server response"),
        }
    }

    pub async fn create_room(&mut self, room_id: &str, password: &str) {
        let req = CreateRoomRequest {
            room_id: room_id.to_string(),
            room_password: password.to_string(),
            user_id: self.username.clone().unwrap_or_default(),
        };

        // Send response to the server
        let response = match self.send_json_to_server("create_room", &req).await {
            Ok(resp) => resp,
            Err(e) => {
                error(&format!("Connection error: {}", e));
                return;
            }
        };

        // Parse response from the system
        if let Ok(resp) = serde_json::from_str::<CreateRoomResponse>(&response) {
            // Room succesfully created
            success(&format!("Room Created - {}", resp.room_id));
        } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&response) {
            // Server could not create the room
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
        } else {
            error(&format!("Unexpected server response: {}", response));
        }
    }

    pub async fn delete_room(&mut self, room_id: &str) {
        let req = DeleteRoomRequest {
            room_id: room_id.to_string(),
        };

        // Send delete room request to the server
        let response = match self.send_json_to_server("delete_room", &req).await {
            Ok(resp) => resp,
            Err(e) => {
                error(&format!("Connection error: {}", e));
                return;
            }
        };

        // Parse response from the server
        if let Ok(resp) = serde_json::from_str::<SuccessResponse>(&response) {
            // Room deleted
            success(&resp.message.to_string());
        } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&response) {
            // Room unable to be deleted
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

    pub async fn kick_user(&mut self, username: &str) {
        let room_id = match &self.current_room {
            Some(id) => id.clone(),
            None => {
                error("You are not in a room");
                return;
            }
        };

        // Build WS message
        let msg = ClientWsMessage::KickUser {
            room_id: room_id.clone(),
            user_id: username.to_string(),
        };

        let serialized = serde_json::to_string(&msg).unwrap();

        // Send through WebSocket
        if let Some(sender) = &mut self.ws_sender {
            if let Err(e) = sender.send(Message::Text(serialized.into())).await {
                error(&format!("Failed to send kick message: {}", e));
                return;
            }
        } else {
            error("WebSocket not connected");
            return;
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

        // Send request to the server
        let response = match self.send_json_to_server("list_room_users", &req).await {
            Ok(resp) => resp,
            Err(e) => {
                error(&format!("Connection error: {}", e));
                return;
            }
        };

        // Parse response
        if let Ok(users_resp) = serde_json::from_str::<ListRoomUsersResponse>(&response) {
            let current_user = self.username.clone().unwrap_or_default();

            header(&format!("Active Users in '{}'", room));
            if users_resp.active_users.is_empty() {
                info(" - No active users");
            } else {
                for user in users_resp.active_users {
                    if user == current_user {
                        println!(" - {}You{}", YELLOW, RESET);
                    } else {
                        println!(" - {}", user);
                    }
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

    pub async fn logout(&mut self) {
        // Confirm that user is currently logged in
        if let Some(username) = &self.username {
            let req = LogoutRequest {};

            // Send logout request to server
            match self.send_json_to_server("logout", &req).await {
                Ok(resp_str) => {
                    if let Ok(_resp) = serde_json::from_str::<SuccessResponse>(&resp_str) {
                        // User loggout successfully
                        success(&format!("User '{}' logged out successfully", username));

                        // Update client struct
                        self.username = None;
                        self.current_room = None;
                    } else if let Ok(err) = serde_json::from_str::<ErrorResponse>(&resp_str) {
                        // Unable to logout
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

    pub async fn connect_ws_for_room(&mut self) -> bool {
        let ws_url = format!("{}/ws", self.server_url_ws);

        // Copy token
        let token = match &self.auth_token {
            Some(t) => t.clone(),
            None => {
                error("No auth token available — cannot open websocket.");
                return false;
            }
        };

        let mut request = match ws_url.into_client_request() {
            Ok(r) => r,
            Err(e) => {
                error(&format!("Invalid WS URL: {}", e));
                return false;
            }
        };

        // Attach JWT token into header
        request.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );

        // Connect to Websocket
        match connect_async(request).await {
            Ok((ws_stream, _)) => {
                let (sender, receiver) = ws_stream.split();
                self.ws_sender = Some(sender);
                self.ws_receiver = Some(receiver);
                true
            }
            Err(e) => {
                error(&format!("WebSocket connection failed: {}", e));
                false
            }
        }
    }
}
