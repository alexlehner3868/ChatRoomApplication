use serde::{Serialize, Deserialize};
// This file has all the messages and asscoiated datastructure to be sent between the server and client
// for both HTTPS and Websocket requests/responses.


// The following are associated with the HTTPS Account/Authentication requests

// derive used to avoid having to implment our our own implmentations
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct RegisterRequest{
// MUST IMPLEMENT NO CONFLICT VALIDATION
    pub user_id: String,
// MUST IMPLEMENT POLICY VALIDATION(even if client already has validation)
    pub password: String,

}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct LoginRequest{
    pub user_id: String,
    pub password: String,
}

// The request will already have the token in the header which contains user_id so nothing is needed
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct LogoutRequest{}

// The request will already have the token in the header which contains user_id so nothing is needed
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct DeleteAccountRequest{}

// The following are associated with the HTTPS Authentication responses 
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct AuthSuccessResponse{
    pub token: String,
    pub user_id:String,
}

// The following are associated with the HTTPS room management requests
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct CreateRoomRequest{
// MUST IMPLEMENT NO CONFLICT VALIDATION(even if client already has validation)
    pub room_id: String,
// MUST IMPLEMENT POLICY VALIDATION(even if client already has validation)
    pub room_password: String,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct JoinRoomRequest{
    pub room_id: String,
    pub room_password: String,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct DeleteRoomRequest{
    pub room_id: String,
}

// even though JoinRoomRequest should get a deafault amount of chat history this request is necessary
// if a client wants to load in even more history
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct GetChatHistoryRequest{
    pub room_id: String,
    // for pagination if we are loading in a set amount
    pub limit: Option<usize>,
    // where to grab the next messages of size limit from, probably based on timestamp
    pub before_timestamp: Option<String>,
}

// doesnt need body as it will pull the user_id from the token attached to http request
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct ListRoomsRequest{
    pub only_active: bool,
}

// even though JoinRoomRequest should get a deafault amount of chat history this request 
// is potentially useful for reconection issues
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct ListRoomUsersRequest{
    pub room_id: String,
}

// The following are associated with the HTTPS room management requests
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct CreateRoomResponse{
    pub room_id: String,
    pub created_at: String,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct JoinRoomResponse{
    pub room_id: String,
    pub chat_history: Vec<ChatMessage>,
    pub active_users: Vec<String>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct GetChatHistoryResponse{
    pub room_id: String,
    pub chat_history: Vec<ChatMessage>,
    // useful to stop clients from making unecessary requests if no more messages are available
    pub more_messages: bool,
}


#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct ListRoomsResponse{
    pub rooms: Vec<RoomInfo>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct ListRoomUsersResponse{
    pub room_id: String,
    pub active_users: Vec<String>,
}

// this is a generic response used for LogoutRequest, DeleteAccountRequest, and DeleteRoomRequest
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct SuccessResponse{
    pub message: String,
}

// The following are associated with the websocket real-time messages
#[derive(Serialize,Deserialize,Debug,Clone)]
//so we can easily tell which option from the enum was sent
#[serde(tag="type")]
pub enum ClientWsMessage{
    LeaveRoom{room_id: String},
    KickUser{room_id: String, user_id: String},
    SendMessage{room_id: String, content: String},
    // to be used for health checks
    Ping{timestamp: String},
}


#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(tag="type")]
pub enum ServerWsMessage{
    RoomDeleted{room_id: String},
    UserJoined{room_id: String, user_id: String},
    UserLeft{room_id: String, user_id: String},
    UserKicked{room_id: String, user_id: String},
    MessageBroadcast(ChatMessage),
    // to be used for health checks
    Pong{timestamp: String},
    Error{error_msg:String},
}

// The following are the data structures used in the messages
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct ChatMessage{
    // potential primary key
    pub room_id: String,
    pub user_id: String,
    // potential primary key
    pub message_id: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct RoomInfo{
    pub room_id: String,
    pub owner: String,
    pub users_count: usize,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(tag="error_type")]
pub enum ErrorResponse{
    AuthenticationFailed{message: String},
    UserAlreadyExists{user_id: String},
    UserNotFound{user_id: String},
    InvalidPassword{message: String},
    RoomNotFound{room_id: String},
    RoomAlreadyExists{room_id: String},
    NotInRoom{room_id: String},
    ServerError{message: String},
}


