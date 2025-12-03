use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{broadcast, Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod message;
use message::{
    ChatMessage, ClientWsMessage, CreateRoomResponse, ErrorResponse, JoinRoomResponse,
    ServerWsMessage, LogoutRequest, SuccessResponse,
};

mod db;
mod models;
mod routes;
mod state;

use crate::db::init_db_from_env;
use crate::routes::auth::{login_handler, register_handler, authenticate_request};
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
        .route("/ws", get(websocket_handler))
        /*
        TODO: IMPLEMENT ROUTES
                .route("/leave_room", post(leave_room_handler))
                .route("/delete_room", post(delete_room_handler))
                .route("/all_rooms", post(list_all_rooms_handler))
                .route("/list_room_users", post(list_room_users_handler))
                 */
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
    Json(req): Json<LogoutRequest>,
) -> impl IntoResponse {
    tracing::info!("Logout request received");
    // authenticate jwt and get user_id
    let user_id = match authenticate_request(&headers).await{
        Ok(id) =>  id,
        Err(e) => {
            tracing::warn!("Authentication failed for logout: {}", e);
            let response = ErrorResponse::AuthenticationFailed {
                message: String::from("Authentication invalid for logging out"),
            };

             return (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
    };

    // clean up user from whatever room they are in
    let room_id = {
        let mut user_rooms = state.user_rooms.lock().await;
        user_rooms.remove(&user_id)
    };

    match room_id{
        Some(id) => {
            let mut rooms = state.rooms.lock().await;
            match rooms.get_mut(&id){
                Some(room) => {
                    room.members.remove(&user_id);
                }
                None => tracing::info!("{} room was already gone at logout", id),
            }

        },
        None => tracing::info!("{} was not in a room at logout", user_id),
    }

    tracing::info!("{} Logged out", user_id);

    // send success message
    let response = SuccessResponse {
        message: String::from("Successfully logged out"),
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

#[derive(Deserialize, Debug)]
struct CreateRoomRequestDemo {
    room_id: String,
    room_password: String,
    user_id: String, // TEMPORARY: Remove when JWT auth is implemented
}

/*
Brief Explanation: creates a new room from request body.

Parameters:
    state: Arc<AppState> - the shared app state that contains the pool Hashmaps for the rooms
    req: Json<CreateRoomRequestDemo> - deseralize the json request body into CreateRoomRequestDemo
Returns:
    Response - seralized response stating success or failure
*/
async fn create_room_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRoomRequestDemo>,
) -> impl IntoResponse {
    tracing::info!("Create room request: {:?}", req);
    // get the locks
    let mut rooms = state.rooms.lock().await;
    let mut room_channels = state.room_channels.lock().await;
    let mut user_rooms = state.user_rooms.lock().await;

    // Check if room already exists
    if rooms.contains_key(&req.room_id) {
        let error = ErrorResponse::RoomAlreadyExists {
            room_id: req.room_id.clone(),
        };
        return (StatusCode::CONFLICT, Json(error)).into_response();
    }

    // TODO: Validate room_id format and password policy
    // TODO: Extract user_id from JWT token in Authorization header (remove user_id from body)

    // Create room
    let room = Room {
        room_id: req.room_id.clone(),
        room_password: req.room_password.clone(),
        owner: req.user_id.clone(),
        members: HashSet::new(),
    };

    rooms.insert(req.room_id.clone(), room);

    // Create broadcast channel for this room
    let (tx, _rx) = broadcast::channel(100);

    // set the transmitter for the room
    room_channels.insert(req.room_id.clone(), tx);

    // set the creator of the room
    user_rooms.insert(req.user_id.clone(), req.room_id.clone());

    // TODO: Save room to database
    let response = CreateRoomResponse {
        room_id: req.room_id,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

// TEMPORARY: For demo purposes, we'll accept user_id in the request body
// In production, this should be extracted from JWT token
#[derive(Deserialize, Debug)]
struct JoinRoomRequestDemo {
    room_id: String,
    room_password: String,
    user_id: String, // TEMPORARY: Remove when JWT auth is implemented
}

/*
Brief Explanation: Joins a room based on request body info.

Parameters:
    state: Arc<AppState> - the shared app state that contains the pool Hashmaps for the rooms
    req: Json<JoinRoomRequestDemo> - deseralize the json request body into JoinRoomRequestDemo
Returns:
    Response - seralized response stating success or failure
*/
async fn join_room_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<JoinRoomRequestDemo>,
) -> impl IntoResponse {
    tracing::info!("Join room request: {:?}", req);

    // get the locks
    let mut rooms = state.rooms.lock().await;
    let mut user_rooms = state.user_rooms.lock().await;

    // Check if room exists
    let room = match rooms.get_mut(&req.room_id) {
        Some(r) => r,
        None => {
            let error = ErrorResponse::RoomNotFound {
                room_id: req.room_id.clone(),
            };
            return (StatusCode::NOT_FOUND, Json(error)).into_response();
        }
    };

    // TO DO: Hash passowrd and check if hashed passwords are accurate
    // Verify password
    if room.room_password != req.room_password {
        let error = ErrorResponse::InvalidPassword {
            message: "Incorrect room password".to_string(),
        };
        return (StatusCode::UNAUTHORIZED, Json(error)).into_response();
    }
    // Add user to room
    room.members.insert(req.user_id.clone());

    // Add user to user_rooms mapping
    user_rooms.insert(req.user_id.clone(), req.room_id.clone());

    // TODO: Load chat history from database
    let chat_history = Vec::new(); // Empty for now

    // TODO: Save user room membership to database

    let response = JoinRoomResponse {
        room_id: req.room_id,
        chat_history,
    };

    (StatusCode::OK, Json(response)).into_response()
}

// TEMPORARY: For demo purposes, we'll accept user_id in the request body later should use JWT
#[derive(Deserialize)]
struct WsQuery {
    user_id: String,
}

/*
Brief Explanation: Upgrades http to websocket connection.

Parameters:
    state: Arc<AppState> - the shared app state that contains the pool Hashmaps for the rooms
    req: Query<WsQuery> - deseralize the parameters
Returns:
    Response - the upgraded websocket connection
*/
async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::info!("WebSocket connection request from user: {}", query.user_id);

    // TODO: Validate JWT token from query params or headers
    // For now, we just accept the user_id

    ws.on_upgrade(move |socket| handle_websocket(socket, query.user_id, state))
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

    // Clone tx to be used by recv_task
    let tx = {
        let channels = state.room_channels.lock().await;
        match channels.get(&room_id) {
            Some(tx) => tx.clone(),
            None => {
                tracing::error!("No broadcast channel for room {}", room_id);
                return;
            }
        }
    };
    let recv_user_id = user_id.clone();
    let recv_room_id = room_id.clone();
    let recv_state = state.clone();

    // run task in the background and wait for the user to try to send a new message to the channel
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // process the message but if there is an error log it and dont send anything out
            if let Err(e) =
                handle_client_message(&text, &recv_user_id, &recv_room_id, &tx, &recv_state).await
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
    tx: &broadcast::Sender<String> - the broadcase channel transmitter used in the associated room,
    state: &Arc<AppState> - The shared app state that contains the pool Hashmaps for the rooms
Returns:
    Response: Result<(), String> - Returns Ok(()) when message is successfuly processed and Err(String) otherwise
*/
async fn handle_client_message(
    text: &str,
    user_id: &str,
    room_id: &str,
    tx: &broadcast::Sender<String>,
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

            // TODO: Save message to database

            let broadcast_msg = ServerWsMessage::MessageBroadcast(chat_msg);
            // serialize message
            let json = match serde_json::to_string(&broadcast_msg) {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::error!("Failed to serialize message for {}: {}", user_id, e);
                    return Err(format!("Failed to serialize message: {}", e));
                }
            };

            // send serialized message to channel
            let _ = tx.send(json);
        }

        ClientWsMessage::LeaveRoom {
            room_id: leave_room_id,
        } => {
            if leave_room_id != room_id {
                return Err("Cannot leave a room you're not in".to_string());
            }
            // Disconnect will be handled by the WebSocket close
            tracing::info!("User {} leaving room {}", user_id, room_id);
        }

        ClientWsMessage::KickUser {
            room_id: kick_room_id,
            user_id: kick_user_id,
        } => {
            // TODO: Verify that the requesting user is the room owner
            // For now, we'll allow anyone to kick (not secure!)

            let kicked_msg = ServerWsMessage::UserKicked {
                room_id: kick_room_id.clone(),
                user_id: kick_user_id.clone(),
            };
            broadcast_to_room(state, &kick_room_id, &kicked_msg).await;

            // TODO: Actually disconnect the kicked user
        }

        ClientWsMessage::Ping { timestamp } => {
            let pong = ServerWsMessage::Pong { timestamp };
            // serialize message
            let json = match serde_json::to_string(&pong) {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::error!("Failed to serialize pong for {}: {}", user_id, e);
                    return Err(format!("Failed to serialize pong: {}", e));
                }
            };

            // send serialized message to channel
            let _ = tx.send(json);
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
