use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{broadcast, Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod message;
use message::{
    ChatMessage, ClientWsMessage, CreateRoomRequest, CreateRoomResponse, DeleteRoomRequest,
    ErrorResponse, JoinRoomRequest, JoinRoomResponse, ListRoomUsersRequest, ListRoomUsersResponse,
    ListRoomsRequest, ListRoomsResponse, LogoutRequest, RoomInfo, ServerWsMessage, SuccessResponse,
};

mod db;
mod models;
mod routes;
mod state;

use crate::db::init_db_from_env;
use crate::routes::auth::{authenticate_request, login_handler, register_handler};
use sqlx::PgPool;

use dotenvy::dotenv;

// the struct represents the room used by users.
#[derive(Clone)]
struct Room {
    room_id: String,
    room_password: String,
    // user_id of the creator of the room
    owner: String,
    //  a set of user_ids connected to room via websockets.
    members: HashSet<String>,
}

// the struct represents the state of the system to be shared across requests and connections.
struct AppState {
    // maps room_id to Room
    rooms: Mutex<HashMap<String, Room>>,
    // maps room_id to channel
    room_channels: Mutex<HashMap<String, broadcast::Sender<String>>>,
    // maps user_id to the room_id that each user is in
    user_rooms: Mutex<HashMap<String, String>>,
    // database pool connection
    db_pool: PgPool,
}

#[tokio::main]
async fn main() {
    // set up the logger
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // load environment variables
    dotenv().ok();

    // the database connection pool
    let pool = init_db_from_env().await;

    let state = Arc::new(AppState {
        rooms: Mutex::new(HashMap::new()),
        room_channels: Mutex::new(HashMap::new()),
        user_rooms: Mutex::new(HashMap::new()),
        db_pool: pool,
    });

    let app = Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/create_room", post(create_room_handler))
        .route("/join_room", post(join_room_handler))
        .route("/delete_room", post(delete_room_handler))
        .route("/all_rooms", post(list_all_rooms_handler))
        .route("/list_room_users", post(list_room_active_users_handler))
        .route("/ws", get(websocket_handler))
        .with_state(state);

    // listen for any requests
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:3000").await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind to 127.0.0.1:3000: {}", e);
            std::process::exit(1);
        }
    };

    let addr = match listener.local_addr() {
        Ok(addr) => addr,
        Err(e) => {
            tracing::error!("Failed to get address: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Server listening on {}", addr);

    // respond to request with one of the defined routes
    match axum::serve(listener, app).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Server error: {}", e);
            std::process::exit(1);
        }
    }
}

/*
Brief Explanation: logs out a user.

Parameters:
    state: Arc<AppState> - the shared app state that contains the pool Hashmaps for the rooms
    headers: HeaderMap - the header containng the jwt
    req: Json<LogoutRequest> - deseralize the json request body into LogoutRequest. Currently its an empty struct as user_id should be in jwt.
Returns:
    Response - seralized response stating success or failure
*/
async fn logout_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(_req): Json<LogoutRequest>,
) -> impl IntoResponse {
    tracing::info!("Logout request received");
    // authenticate jwt and get user_id
    let user_id = match authenticate_request(&headers).await {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("Authentication failed for logout: {:?}", e);
            let response = ErrorResponse::AuthenticationFailed {
                message: String::from("Authentication invalid for logging out"),
            };

            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    // clean up user from whatever room they are in
    let room_id = {
        let mut user_rooms = state.user_rooms.lock().await;
        user_rooms.remove(&user_id)
    };

    match room_id {
        Some(id) => {
            let mut rooms = state.rooms.lock().await;
            match rooms.get_mut(&id) {
                Some(room) => {
                    room.members.remove(&user_id);
                }
                None => tracing::info!("{} room was already gone at logout", id),
            }
        }
        None => tracing::info!("{} was not in a room at logout", user_id),
    }

    tracing::info!("{} Logged out", user_id);

    // send success message
    let response = SuccessResponse {
        message: String::from("Successfully logged out"),
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

/*
Brief Explanation: creates a new room from request body.

Parameters:
    state: Arc<AppState> - the shared app state that contains the pool Hashmaps for the rooms
    headers: HeaderMap - the header containng the jwt
    req: Json<CreateRoomRequestDemo> - deseralize the json request body into CreateRoomRequestDemo
Returns:
    Response - seralized response stating success or failure
*/
async fn create_room_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateRoomRequest>,
) -> impl IntoResponse {
    tracing::info!("Create room request: {:?}", req);
    // authenticate jwt and get user_id
    let user_id = match authenticate_request(&headers).await {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("Authentication failed for creating room: {:?}", e);
            let response = ErrorResponse::AuthenticationFailed {
                message: String::from("Authentication invalid for creating room"),
            };

            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    // TODO: Mahmoud Validate room_id and password and if they are good add them to database (make sure to hash password)
    // if it fails then use match to send the appropriate error response to caller
    // please change the instance below to go from  room_password: req.room_password.clone() to using the hashed passowrd

    // get the locks
    let mut rooms = state.rooms.lock().await;
    let mut room_channels = state.room_channels.lock().await;

    // Check if room already exists
    if rooms.contains_key(&req.room_id) {
        let error = ErrorResponse::RoomAlreadyExists {
            room_id: req.room_id.clone(),
        };
        return (StatusCode::CONFLICT, Json(error)).into_response();
    }

    // Create room
    let room = Room {
        room_id: req.room_id.clone(),
        room_password: req.room_password.clone(),
        owner: user_id.clone(),
        members: HashSet::new(),
    };

    rooms.insert(req.room_id.clone(), room);

    // Create broadcast channel for this room
    let (tx, _rx) = broadcast::channel(100);

    // set the transmitter for the room
    room_channels.insert(req.room_id.clone(), tx);

    // send successful response
    let response = CreateRoomResponse {
        room_id: req.room_id,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

/*
Brief Explanation: Joins a room based on request body info, it essentially is just verifying the user can join and get previous history. A seperate request should be made to upgrade to websocket.

Parameters:
    state: Arc<AppState> - the shared app state that contains the pool Hashmaps for the rooms
    headers: HeaderMap - the header containng the jwt
    req: Json<JoinRoomRequest> - deseralize the json request body into JoinRoomRequest
Returns:
    Response - seralized response stating success or failure
*/
async fn join_room_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<JoinRoomRequest>,
) -> impl IntoResponse {
    tracing::info!("Join room request: {:?}", req);
    // authenticate jwt and get user_id
    let user_id = match authenticate_request(&headers).await {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("Authentication failed for joining room: {:?}", e);
            let response = ErrorResponse::AuthenticationFailed {
                message: String::from("Authentication invalid for joining room"),
            };

            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    // TO DO: Mahmoud verify hashed room_password against the password from the request password
    // if room doesnt exist in database or verififcation fails return the the appropraite ErrorResponse and return to caller
    // Load chat history from database and add user to room in database

    // if room.room_password != req.room_password {
    //     let error = ErrorResponse::InvalidPassword {
    //         message: "Incorrect room password".to_string(),
    //     };
    //     return (StatusCode::UNAUTHORIZED, Json(error)).into_response();
    // }
    let chat_history = Vec::new(); // Temporary Empty for now

    // get the lock
    let mut user_rooms = state.user_rooms.lock().await;

    // Add user to user_rooms mapping
    user_rooms.insert(user_id.clone(), req.room_id.clone());

    let response = JoinRoomResponse {
        room_id: req.room_id,
        chat_history,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/*
Brief Explanation: delete room.

Parameters:
    state: Arc<AppState> - the shared app state that contains the pool Hashmaps for the rooms
    headers: HeaderMap - the header containng the jwt
    req: Json<DeleteRoomRequest> - deseralize the json request body into DeleteRoomRequest
Returns:
    Response - seralized response stating success or failure
*/
async fn delete_room_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DeleteRoomRequest>,
) -> impl IntoResponse {
    tracing::info!("Delete room request received for room: {}", &req.room_id);
    // authenticate jwt and get user_id
    let user_id = match authenticate_request(&headers).await {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("Authentication failed for deleting room: {:?}", e);
            let response = ErrorResponse::AuthenticationFailed {
                message: String::from("Authentication invalid for deleting room"),
            };

            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    // get the owner of the room
    let room_owner = {
        let rooms = state.rooms.lock().await;
        match rooms.get(&req.room_id) {
            Some(room) => room.owner.clone(),
            None => {
                tracing::warn!(
                    "Deleting failed due to issue finding room: {}",
                    &req.room_id
                );
                let response = ErrorResponse::RoomNotFound {
                    room_id: req.room_id.clone(),
                };

                return (StatusCode::NOT_FOUND, Json(response)).into_response();
            }
        }
    };

    // check if owner of the room matches the user making the request
    if room_owner != user_id {
        tracing::warn!("Only owner can delete room: {}", &req.room_id);
        let response = ErrorResponse::InvalidPermissions {
            message: format!("Failed to delete room: {}", &req.room_id),
        };

        return (StatusCode::FORBIDDEN, Json(response)).into_response();
    }

    // let all active users in room know that room is deleted
    {
        let room_deleted_msg = ServerWsMessage::RoomDeleted {
            room_id: req.room_id.clone(),
        };
        broadcast_to_room(&state, &req.room_id, &room_deleted_msg).await;
        // pause for a little bit to ensure users recived message
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // clean up

    // delete the room
    {
        let mut rooms = state.rooms.lock().await;
        rooms.remove(&req.room_id);
    }

    // remove boradcast channel for room
    {
        let mut room_channels = state.room_channels.lock().await;
        room_channels.remove(&req.room_id);
        tracing::info!("Removed broadcast channel for room: {}", &req.room_id);
    }

    // remove users attached to the room
    {
        let mut user_rooms = state.user_rooms.lock().await;
        let room_id_to_delete = &req.room_id;
        // only keep pairs where room_id is not the same as the room being deleted
        user_rooms.retain(|_user_id, room_value_id| room_value_id != room_id_to_delete);
        tracing::info!("Removed users for room: {}", room_id_to_delete);
    }

    tracing::info!("Room {} Deleted", &req.room_id);

    // TODO: Mahmoud database data deletion idk if you wana have a sepertae function in another file to do this

    // send success message
    let response = SuccessResponse {
        message: format!("Successfully deleted room: {}", &req.room_id),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/*
Brief Explanation: list all rooms that a user has ever joined that they have not permanentaly left.

Parameters:
    state: Arc<AppState> - the shared app state that contains the pool Hashmaps for the rooms
    headers: HeaderMap - the header containng the jwt
    req: Json<ListRoomsRequest> - deseralize the json request body into ListRoomsRequest
Returns:
    Response - seralized response stating success or failure
*/
async fn list_all_rooms_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(_req): Json<ListRoomsRequest>,
) -> impl IntoResponse {
    tracing::info!("List all rooms request received");
    // authenticate jwt and get user_id
    let _user_id = match authenticate_request(&headers).await {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("Authentication failed for listing all rooms: {:?}", e);
            let response = ErrorResponse::AuthenticationFailed {
                message: String::from("Authentication invalid for listing all room"),
            };

            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    // TODO: Mahmoud database data pull so we want to get all the rooms that a user was previously joined
    // the database should a vector of rooms where each element contains the room_id, the owner of the room, and default value of zero set for users_count
    // we set it to a default of 0 as the in-memory state will actually know who is connected to the room rn
    //  currently rooms_db is just an empty vector but can you please fill it with the data needed
    let rooms_db: Vec<RoomInfo> = Vec::new();

    // get the lock for rooms to get the number of users currently in the room
    let rooms = state.rooms.lock().await;
    let mut final_rooms: Vec<RoomInfo> = Vec::new();

    // go through each room that the caller was previously in and updated how many users are currently in the rooms to be sent to the caller
    for mut room_info in rooms_db {
        // check if there is anyone in the room
        if let Some(active_room) = rooms.get(&room_info.room_id) {
            // update the default count to represent how many users are actually in the room
            room_info.users_count = active_room.members.len();
        }
        // add the room info to be vector to be sent to caller
        final_rooms.push(room_info);
    }

    // send success message
    let response = ListRoomsResponse { rooms: final_rooms };

    (StatusCode::OK, Json(response)).into_response()
}

/*
Brief Explanation: list all active users in the room of interest

Parameters:
    state: Arc<AppState> - the shared app state that contains the pool Hashmaps for the rooms
    headers: HeaderMap - the header containng the jwt
    req: Json<ListRoomUsersRequest> - deseralize the json request body into ListRoomUsersRequest
Returns:
    Response - seralized response stating success or failure
*/
async fn list_room_active_users_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ListRoomUsersRequest>,
) -> impl IntoResponse {
    tracing::info!("List all users in room {} request received", &req.room_id);
    // authenticate jwt
    let user_id = match authenticate_request(&headers).await {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!(
                "Authentication failed for listing all users in room: {:?}",
                e
            );
            let response = ErrorResponse::AuthenticationFailed {
                message: String::from("Authentication invalid for listing all users in room"),
            };

            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    //get the room of interest
    let rooms = state.rooms.lock().await;
    let room = match rooms.get(&req.room_id) {
        Some(r) => r,
        None => {
            tracing::error!(
                "{} room not in memory when trying to list all users.",
                &req.room_id
            );
            let response = ErrorResponse::RoomNotFound {
                room_id: req.room_id.clone(),
            };
            return (StatusCode::NOT_FOUND, Json(response)).into_response();
        }
    };

    // go through all active members and add to vector to be sent to caller
    let mut room_users: Vec<String> = Vec::new();
    for user in room.members.iter() {
        room_users.push(user.clone())
    }

    tracing::info!(
        "User {} requestd {:?} from room: {}",
        &user_id,
        &room_users,
        &req.room_id
    );
    // send success message
    let response = ListRoomUsersResponse {
        room_id: req.room_id.clone(),
        active_users: room_users,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/*
Brief Explanation: Upgrades http to websocket connection.

Parameters:
    state: Arc<AppState> - the shared app state that contains the pool Hashmaps for the rooms
    headers: HeaderMap - the header containng the jwt
Returns:
    Response - the upgraded websocket connection
*/
async fn websocket_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // authenticate jwt
    let user_id = match authenticate_request(&headers).await {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!(
                "Authentication failed for listing all users in room: {:?}",
                e
            );
            let response = ErrorResponse::AuthenticationFailed {
                message: String::from("Authentication invalid for websocket connection"),
            };

            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };
    tracing::info!("WebSocket connection request from user: {}", user_id);

    ws.on_upgrade(move |socket| handle_websocket(socket, user_id, state))
}

/*
Brief Explanation: Handles reciveing websocket messages and broadcasting those messages to users.

Parameters:
    socket: WebSocket - The websocket used for real time communication
    user_id: String - The user who sent the message(TEMPORARY)
    state: Arc<AppState> - the shared app state that contains the pool Hashmaps for the rooms
Returns:
    N/A
*/
async fn handle_websocket(socket: WebSocket, user_id: String, state: Arc<AppState>) {
    // split the sender and receiver to send  and recive at the same time
    let (mut sender, mut receiver) = socket.split();

    // find the room the user is in
    let room_id = {
        let user_rooms = state.user_rooms.lock().await;
        user_rooms.get(&user_id).cloned()
    };

    // check that the user is in a room
    let room_id = match room_id {
        Some(id) => id,
        None => {
            tracing::warn!("User {} connected without joining a room", user_id);
            let error = ServerWsMessage::Error {
                error_msg: "You must join a room before connecting to WebSocket".to_string(),
            };
            match serde_json::to_string(&error) {
                Ok(seralized_error) => {
                    let _ = sender.send(Message::Text(seralized_error)).await;
                }
                Err(e) => {
                    tracing::error!("Failed to serialize error for {}: {}", user_id, e);
                }
            }

            // close the connection
            return;
        }
    };

    // adds the user to room
    {
        let mut rooms = state.rooms.lock().await;
        match rooms.get_mut(&room_id) {
            Some(room) => {
                room.members.insert(user_id.clone());
            }
            None => {
                tracing::error!("Error inserting {} into {}", &user_id, &room_id);
            }
        }
    }

    // subscribe to to room broadcast channel to receive messages
    let mut rx = {
        let channels = state.room_channels.lock().await;
        match channels.get(&room_id) {
            Some(tx) => tx.subscribe(),
            None => {
                tracing::error!("No broadcast channel for room {}", room_id);
                return;
            }
        }
    };

    // Notify room that user joined
    let join_msg = ServerWsMessage::UserJoined {
        room_id: room_id.clone(),
        user_id: user_id.clone(),
    };
    broadcast_to_room(&state, &room_id, &join_msg).await;

    // run task in the background and wait for any new messages to arrive from channel for the user
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // when a message comes to the channel send it out to the user with the websocket connection
            // if the message fails to send then end the task
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let recv_user_id = user_id.clone();
    let recv_room_id = room_id.clone();
    let recv_state = state.clone();

    // run task in the background and wait for the user to try to send a new message to the channel
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // process the message but if there is an error log it and dont send anything out
            if let Err(e) =
                handle_client_message(&text, &recv_user_id, &recv_room_id, &recv_state).await
            {
                tracing::error!("Error handling message: {}", e);
            }
        }
    });

    // Wait for one task to finish and stop the other
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Cleanup
    {
        let mut rooms = state.rooms.lock().await;
        let mut user_rooms = state.user_rooms.lock().await;
        match rooms.get_mut(&room_id) {
            Some(room) => {
                room.members.remove(&user_id);
            }
            None => {
                tracing::error!("Error cleaning up {} from {}", &user_id, &room_id);
            }
        }
        user_rooms.remove(&user_id);
    }

    // Notify room that user left
    let leave_msg = ServerWsMessage::UserLeft {
        room_id: room_id.clone(),
        user_id: user_id.clone(),
    };
    broadcast_to_room(&state, &room_id, &leave_msg).await;

    tracing::info!("User {} disconnected from room {}", user_id, room_id);
}

/*
Brief Explanation: Processes a websocket message such as sending a chat message or kicking a user.

Parameters:
    text: &str - The string received from a user,
    user_id: &str - The ID of the user who sent the message,
    room_id: &str The ID of the room message was sent from,
    state: &Arc<AppState> - The shared app state that contains the pool Hashmaps for the rooms
Returns:
    Response: Result<(), String> - Returns Ok(()) when message is successfuly processed and Err(String) otherwise
*/
async fn handle_client_message(
    text: &str,
    user_id: &str,
    room_id: &str,
    state: &Arc<AppState>,
) -> Result<(), String> {
    // deserialize message sent from user
    let msg: ClientWsMessage = match serde_json::from_str(text) {
        Ok(msg) => msg,
        Err(e) => {
            tracing::error!("Failed to deserialize error for {}: {}", user_id, e);
            return Err(format!("Failed to deserialize message: {}", e));
        }
    };

    // decide what to do based on message type
    match msg {
        ClientWsMessage::SendMessage {
            room_id: msg_room_id,
            content,
        } => {
            // Verify user is in the room they're trying to send to
            if msg_room_id != room_id {
                return Err("Cannot send to a room you're not in".to_string());
            }

            // create message object to be sent to all users in room
            let chat_msg = ChatMessage {
                room_id: room_id.to_string(),
                user_id: user_id.to_string(),
                message_id: uuid::Uuid::new_v4().to_string(),
                content,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            // TODO: Mahmoud save message to database

            // send message to everyone in room
            let broadcast_msg = ServerWsMessage::MessageBroadcast(chat_msg);
            broadcast_to_room(state, room_id, &broadcast_msg).await;

            tracing::info!("User {} sent message {:?}", user_id, &broadcast_msg);
        }

        ClientWsMessage::LeaveRoom {
            room_id: leave_room_id,
        } => {
            if leave_room_id != room_id {
                return Err("Cannot leave a room you're not in".to_string());
            }
            // clean up
            {
                let mut user_rooms = state.user_rooms.lock().await;
                user_rooms.remove(user_id);
            }
            {
                let mut rooms = state.rooms.lock().await;
                match rooms.get_mut(room_id) {
                    Some(room) => {
                        room.members.remove(user_id);
                    }
                    None => tracing::info!("room {} missing when user tried to leave", &room_id),
                }
            }

            // left others in the room know that user left
            let left_msg = ServerWsMessage::UserLeft {
                room_id: leave_room_id.clone(),
                user_id: user_id.to_string(),
            };
            broadcast_to_room(state, &leave_room_id, &left_msg).await;

            tracing::info!("User {} leaving room {}", user_id, room_id);
        }

        ClientWsMessage::KickUser {
            room_id: kick_room_id,
            user_id: kick_user_id,
        } => {
            // get the owner of the room
            let room_owner = {
                let rooms = state.rooms.lock().await;
                match rooms.get(&kick_room_id) {
                    Some(room) => room.owner.clone(),
                    None => {
                        tracing::warn!(
                            "kicking user failed due to issue finding room: {}",
                            kick_room_id
                        );

                        return Err("kicking user failed due to issue finding room".to_string());
                    }
                }
            };

            // check if owner of the room matches the user making the request
            if room_owner != user_id {
                tracing::warn!("Only owner can kick users from room: {}", kick_room_id);
                return Err("Only owner can kick users from room".to_string());
            }
            // check if owner is trying to kick themselves
            if kick_user_id == user_id {
                tracing::warn!("Owner cannot kick themselves from room: {}", kick_room_id);
                return Err("Owner cannot kick themselves from room".to_string());
            }
            // TODO: Mahmoud update the database to remove the kicked user and if they were never in the room return error

            // clean up
            {
                let mut user_rooms = state.user_rooms.lock().await;
                user_rooms.remove(&kick_user_id);
            }
            {
                let mut rooms = state.rooms.lock().await;
                match rooms.get_mut(&kick_room_id) {
                    Some(room) => {
                        room.members.remove(&kick_user_id);
                    }
                    None => {
                        tracing::info!("room {} missing when trying to kick user", kick_room_id)
                    }
                }
            }

            let kicked_msg = ServerWsMessage::UserKicked {
                room_id: kick_room_id.clone(),
                user_id: kick_user_id.clone(),
            };
            broadcast_to_room(state, &kick_room_id, &kicked_msg).await;

            tracing::info!("User {} kicked from room: {}", &kick_user_id, &kick_room_id);
        }
    }

    Ok(())
}

/*
Brief Explanation: Sends message through room channel
Parameters:
    msg: &ServerWsMessage - The message to broadcast to the room
    room_id: &str The ID of the room message was sent from,
    state: &Arc<AppState> - The shared app state that contains the pool Hashmaps for the rooms
Returns:
   N/A
*/
async fn broadcast_to_room(state: &Arc<AppState>, room_id: &str, msg: &ServerWsMessage) {
    // seralize the message to be sent
    let json = match serde_json::to_string(msg) {
        Ok(j) => j,
        Err(e) => {
            tracing::error!("Failed to serialize broadcast message: {}", e);
            return;
        }
    };

    // get the the broadcast transmitter and send message to all listeners
    let channels = state.room_channels.lock().await;
    if let Some(tx) = channels.get(room_id) {
        let _ = tx.send(json);
    }
}
