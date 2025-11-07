use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
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

// Import your message protocol types
mod message;
use message::{
    ChatMessage, ClientWsMessage, CreateRoomResponse, JoinRoomResponse,
    ServerWsMessage, ErrorResponse,
};

#[derive(Clone)]
struct Room {
    room_id: String,
    room_password: String,
    owner: String,
    // Set of user_ids currently in this room
    members: HashSet<String>,
}

struct AppState {
    // room_id -> Room
    rooms: Mutex<HashMap<String, Room>>,
    // user_id -> broadcast sender for that user's room
    // Each room has its own broadcast channel
    room_channels: Mutex<HashMap<String, broadcast::Sender<String>>>,
    // user_id -> room_id (tracks which room each user is in)
    user_rooms: Mutex<HashMap<String, String>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = Arc::new(AppState {
        rooms: Mutex::new(HashMap::new()),
        room_channels: Mutex::new(HashMap::new()),
        user_rooms: Mutex::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/create_room", post(create_room_handler))
        .route("/join_room", post(join_room_handler))
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}


// TEMPORARY: For demo purposes, we'll accept user_id in the request body later should use JWT
#[derive(Deserialize,Debug)]
struct CreateRoomRequestDemo {
    room_id: String,
    room_password: String,
    user_id: String, // TEMPORARY: Remove when JWT auth is implemented
}

async fn create_room_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRoomRequestDemo>,
) -> impl IntoResponse {
    tracing::info!("Create room request: {:?}", req);

    let mut rooms = state.rooms.lock().await;
    
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

    // Create broadcast channel for this room
    let (tx, _rx) = broadcast::channel(100);
    state.room_channels.lock().await.insert(req.room_id.clone(), tx);

    rooms.insert(req.room_id.clone(), room);

    // Add creator to user_rooms mapping (they automatically join their created room)
    state.user_rooms.lock().await.insert(req.user_id.clone(), req.room_id.clone());

    // TODO: Save room to database
    // db::save_room(&req.room_id, &req.room_password, &req.user_id).await;

    let response = CreateRoomResponse {
        room_id: req.room_id,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

// TEMPORARY: For demo purposes, we'll accept user_id in the request body
// In production, this should be extracted from JWT token
#[derive(Deserialize,Debug)]
struct JoinRoomRequestDemo {
    room_id: String,
    room_password: String,
    user_id: String, // TEMPORARY: Remove when JWT auth is implemented
}

async fn join_room_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<JoinRoomRequestDemo>,
) -> impl IntoResponse {
    tracing::info!("Join room request: {:?}", req);

    let rooms = state.rooms.lock().await;
    
    // Check if room exists
    let room = match rooms.get(&req.room_id) {
        Some(r) => r,
        None => {
            let error = ErrorResponse::RoomNotFound {
                room_id: req.room_id.clone(),
            };
            return (StatusCode::NOT_FOUND, Json(error)).into_response();
        }
    };

    // Verify password
    if room.room_password != req.room_password {
        let error = ErrorResponse::InvalidPassword {
            message: "Incorrect room password".to_string(),
        };
        return (StatusCode::UNAUTHORIZED, Json(error)).into_response();
    }

    // Add user to user_rooms mapping
    state.user_rooms.lock().await.insert(req.user_id.clone(), req.room_id.clone());

    // TODO: Load chat history from database
    // let chat_history = db::get_chat_history(&req.room_id, 50).await;
    let chat_history = Vec::new(); // Empty for now

    // TODO: Save user room membership to database
    // db::add_user_to_room(&req.user_id, &req.room_id).await;

    let response = JoinRoomResponse {
        room_id: req.room_id,
        chat_history,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[derive(Deserialize)]
struct WsQuery {
    user_id: String,
}

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

async fn handle_websocket(socket: WebSocket, user_id: String, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Determine which room this user is in
    let room_id = {
        let user_rooms = state.user_rooms.lock().await;
        user_rooms.get(&user_id).cloned()
    };

    let room_id = match room_id {
        Some(id) => id,
        None => {
            // User hasn't joined a room yet
            tracing::warn!("User {} connected without joining a room", user_id);
            let error = ServerWsMessage::Error {
                error_msg: "You must join a room before connecting to WebSocket".to_string(),
            };
            let _ = sender.send(Message::Text(serde_json::to_string(&error).unwrap().into())).await;
            return;
        }
    };

    // Add user to room members
    {
        let mut rooms = state.rooms.lock().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            room.members.insert(user_id.clone());
        }
    }

    // Get broadcast receiver for this room
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

    // Spawn task to send broadcast messages to this user
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Clone for recv task
    let tx = {
        let channels = state.room_channels.lock().await;
        channels.get(&room_id).unwrap().clone()
    };
    let recv_user_id = user_id.clone();
    let recv_room_id = room_id.clone();
    let recv_state = state.clone();

    // Spawn task to receive messages from this user
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Err(e) = handle_client_message(
                &text,
                &recv_user_id,
                &recv_room_id,
                &tx,
                &recv_state,
            )
            .await
            {
                tracing::error!("Error handling message: {}", e);
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Cleanup: remove user from room
    {
        let mut rooms = state.rooms.lock().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            room.members.remove(&user_id);
        }
    }

    // Remove from user_rooms mapping
    state.user_rooms.lock().await.remove(&user_id);

    // Notify room that user left
    let leave_msg = ServerWsMessage::UserLeft {
        room_id: room_id.clone(),
        user_id: user_id.clone(),
    };
    broadcast_to_room(&state, &room_id, &leave_msg).await;

    tracing::info!("User {} disconnected from room {}", user_id, room_id);
}

async fn handle_client_message(
    text: &str,
    user_id: &str,
    room_id: &str,
    tx: &broadcast::Sender<String>,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let msg: ClientWsMessage = serde_json::from_str(text)
        .map_err(|e| format!("Failed to parse message: {}", e))?;

    match msg {
        ClientWsMessage::SendMessage { room_id: msg_room_id, content } => {
            // Verify user is in the room they're trying to send to
            if msg_room_id != room_id {
                return Err("Cannot send to a room you're not in".to_string());
            }

            let chat_msg = ChatMessage {
                room_id: room_id.to_string(),
                user_id: user_id.to_string(),
                message_id: uuid::Uuid::new_v4().to_string(),
                content,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            // TODO: Save message to database
            // db::save_message(&chat_msg).await;

            let broadcast_msg = ServerWsMessage::MessageBroadcast(chat_msg);
            let json = serde_json::to_string(&broadcast_msg)
                .map_err(|e| format!("Failed to serialize message: {}", e))?;
            
            let _ = tx.send(json);
        }

        ClientWsMessage::LeaveRoom { room_id: leave_room_id } => {
            if leave_room_id != room_id {
                return Err("Cannot leave a room you're not in".to_string());
            }
            // Disconnect will be handled by the WebSocket close
            tracing::info!("User {} leaving room {}", user_id, room_id);
        }

        ClientWsMessage::KickUser { room_id: kick_room_id, user_id: kick_user_id } => {
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
            let json = serde_json::to_string(&pong)
                .map_err(|e| format!("Failed to serialize pong: {}", e))?;
            let _ = tx.send(json);
        }
    }

    Ok(())
}

async fn broadcast_to_room(state: &Arc<AppState>, room_id: &str, msg: &ServerWsMessage) {
    let json = match serde_json::to_string(msg) {
        Ok(j) => j,
        Err(e) => {
            tracing::error!("Failed to serialize broadcast message: {}", e);
            return;
        }
    };

    let channels = state.room_channels.lock().await;
    if let Some(tx) = channels.get(room_id) {
        let _ = tx.send(json);
    }
}