# ECE1724 Terminal-Based Multi-User Chat Room with Real-Time and Persistent Messaging

## Team Information: 
- Alex Lehner (alex.lehner@mail.utoronto.ca) - 1004947506
- Mohamad Alkahil (m.alkahil@mail.utoronto.ca) - 1005263448
- Mahmoud (mahmoud.anklis@mail.utoronto.ca) - 1005198313

## Video Demo:
https://youtu.be/HwRxdH78rCQ

## Video Slide Presentation:
PUT LINK HERE

## Motivation:
Our motivation for undertaking this project is twofold: to deepen our own proficiency in Rust and systems programming and to create a comprehensive reference application that supports future Rust learners. As Rust fans, we wanted a project that would challenge us across the full spectrum of application development. We intentionally sought a problem that would require us to explore Rust’s versatility in backend development, frontend UI design, asynchronous communication, persistent storage, and secure user authentication.
Rust’s strong guarantees around memory safety and its ownership system make it especially well-suited for applications involving concurrency, which is an essential requirement for a real-time multi-user chat room. By building a terminal-based chat application with persistent messaging, multi-room support, and authenticated users, we gain hands-on experience with key systems programming concepts such as communication protocols, asynchronous concurrency models, real-time I/O, and database-backed state management. This breadth of exposure aligns perfectly with our personal learning goals and allows us to engage deeply with the practical aspects of designing reliable, secure, and concurrent software.
Beyond our own development, we also aim to address a gap in the Rust ecosystem. While several Rust-based chat applications exist, few provide a fully realized, end-to-end example that integrates a terminal UI, persistent user and message storage, authentication, multi-room capabilities, and structured asynchronous design. As Rust continues to grow in popularity, the demand for well-designed, full-stack reference projects increases. Our project is designed to meet that need by serving as a concrete example of a complete, production-style Rust application, one that future Rustaceans can study, extend, and learn from.
Ultimately, this project represents an opportunity for personal growth while contributing meaningfully to the learning resources available within the Rust community. By building a robust, real-time, terminal-based chat system, we aim to both strengthen our own systems programming skills and support others who are beginning their journeys with Rust.

## Objectives: 
### 1. Build a Command Line Interface (CLI) Application for Communication
+ Allow users to create an account to interact with the system
+ Enable users to create and join rooms
+ Broadcast messages through real-time bi-directional WebSocket connections
### 2. Secure Communication
+ Ensure only authorized clients are able to make requests through JSON Web Tokens (JWT)
+ Hash passwords before storage 
### 3. Handle Concurrent Multi-Room Communication
+ Allow multiple users to connect to multiple rooms concurrently and send messages
+ Ensure broadcasted messages are delivered only to the active users in the intended room
### 4. Ensure Data Persistence
+ Store all user accounts and room data in PostgreSQL database
+ Ensure data survives when a server restart occurs
### 5. Provide a User-Friendly Experience
+ Provide CLI that is easy to navigate
+ Ensure the CLI is visually cohesive and polished 


## Features:
### Architecture
+ __Hybrid Communication__:
  + __HTTP__: Handles the one-time requests such as login/logout, room creation/joining/deletion, and listing rooms/users. All HTTP requests except for signing up and logging in require a JSON Web Token (JWT).
  + __Websockets__: Once a client’s request to join a room is authenticated and processed, a request is made to upgrade the connection from HTTP to a persistent bi-directional WebSocket connection for real-time messaging. The connection is used for broadcasting to all clients with a valid connection.
+  __Technology Stack__: The application is built entirely using Rust. Axum is utilized as the web framework for the server API. Both the client and the server utilize the Tokio async runtime, which is used to efficiently handle concurrent communication.
+ __Server-Side State Tracking__: Hash maps are used for in-memory storage. The active users, rooms, and the broadcast channels are tracked. The data is cleared upon client logout, room deletion, user kick, leaving a room, and WebSocket disconnection.
+  __Broadcast Channels__: Each room has its own broadcast channel to ensure only the intended clients receive messages.

### Chat Room Management
+ __Room Creation__: Clients can create a room (using the `/create <room_id>` command) with a unique ID and with a room password 
+ __Room Ownership__: The creator of the room retains ownership of the room and has special privileges. One such privilege is deleting the room through the `/delete <room_id>` command. Upon deletion the associated data is removed from the database and in-memory storage.
+ __Room Joining__: Clients can join a room as long as valid credentials are provided in the request through the `/join <room_id>` command. Once a client joins a room, they will receive the most recent 200 messages of room history from the database.
+ __Room Discovery__:
  + __List All Rooms__: All rooms that are stored in the system can be listed using the `/all_rooms` command.
  + __List Active Rooms__: List all the rooms as well as the number of active users within them using the `/active_rooms` command.
+ __List Active Users__: List all the active users in the current room the client is currently connected to using the `/active_users` command.

### Security 
+ __Registration__: Clients can create an account using the `/sign_up` command. The user must create an account with a unique username with a password meeting the minimum length of 8 characters with lowercase, uppercase, and special characters.
+ __Password Hashing__: The Argon2 hashing algorithm is applied prior to database storage.
+ __Authentication__: Upon successful login into the system (using the `/login` command), the server issues a JWT to the client for all future requests. The token is removed upon logging out using the `/logout` command to prevent unauthorized access.
+ __Secure Websockets__: To establish a WebSocket connection, a valid JWT is required to ensure only an authorized user can broadcast to a room.

### Real-Time Message Broadcasting
+ __Message Delivery__: A client can send any message they want to broadcast to the room without a `/` prefix. The messages are sent to the database to ensure persistence.
+ __User Kick__: The owner can remove users from the room using the `/kick <username>` command. The associated data is removed from the database and in-memory storage.
+ __Leave Room__: A client can leave a room using the room using the `/leave` command. The associated data is removed from the in-memory storage.
+ __System Notifications__:  Clients receive real-time notifications when:
  + Another user joins the room
  + Another user leaves the room
  + A user is kicked from the room
  + The room owner deletes the room

### Persistence
+ __Database__:  All user and room data is stored in a PostgreSQL database, ensuring data survives even if the server restarts.
<img width="1124" height="524" alt="Image" src="https://github.com/user-attachments/assets/adbd802e-d572-42a7-afee-e66bde8424ac" />

### User Experience
+ __User Command Driven__: User provides commands to access varying features and navigate the system. 
+ __Help Info__: A help menu is available at any time through the `/help` command, which breaks down all commands and their actions.
+ __CLI Visual Clarity__: 
  + Errors - red
  + Success - green
  + Warnings - yellow
  + Section headers - magenta
  + System prompts (EX: `[Lobby]>`) - cyan
  + Chat messages are color-coded by user with usernames and timestamps
  + A client's own messages will be aligned to the right side of the terminal.
+  __State-Based CLI__: The available commands are dependent on context:
  + Not logged in: `/sign_up`, `/login`
  + In lobby: `/create`, `/join`, `/all_rooms`, `/active_rooms`, `/delete` (owner only), `/logout`
  + In room: `/leave`, `/active_users`, `/kick` (owner only)
  + Always available: `/help`, `/quit`
+ __Password Masking__: The input is hidden for passwords.

## User’s Guide:
This section described how to set up the chatroom, which includes the server and the client. Ensure the server is running before the client’s code is run. 

### Server Setup 
Run the server by navigating to *ChatRoomApplicationServer* in the project directory and run:
```
cargo run
```


### User Setup
To start the client locally navigate to the *ChatRoomApplicationClient* in the project directory and run:
``` 
cargo run 
```
Upon launch, the client begins in the logged out state and users can register or login

Note: Ensure the server is running before the client is launched. 

#### 1. User Authentication 
**1.1 Registering a New Account**

To create a new user account:
```
/sign_up
```
You will be prompted to provide a unique username and a password that meets a desired criteria. On success, the server will create the account, and the client returns to the logged-out state. 

Note: if `/quit` is entered at any prompt, the registration is cancelled and the program gracefully terminates. 

**1.2 Logging In**

To sign in to an existing account:
```
/login
```
You will be prompted for your username and password. Upon successful authentication, the client will store your authentication token and you will enter the lobby.

#### 2. Lobby Commands
When the user is logged in but not inside of a specific chat room, they have access to the following

**2.1 View All Rooms**

Display a list of every room currently registered on the server:
```
/all_rooms
```

**2.2 View Active Rooms**

Shows the number of users online and currently in each room: 
```
/active_rooms
```

**2.3 Create a Room**

Create a new chat room:
```
/create <room_name>
```
You will then be prompted to input a password. 

Note: The room name needs to be unique

**2.4 Join Room**

To join an existing room:
```
/join <room_name>
```
You will then be prompted to input the password. If the room name and password are correct, a WebSocket connection is opened, and the user will move into the room and transition to the in-room state. 

**2.5 Delete Room**

To delete a room:
```
/delete <room_name>
```
All users in the room are disconnected and notified, and the room is removed from the server. 
Note: This only works if the user is the room’s owner (i.e., they created it)

**2.6 Logging out**

Once logged in, use this to log out and return to a logged-out state:
```
/logout
```

#### 3. In-Room Commands 
Once joining a room, the previous messages from that room will be shown (up to the 200th most recent) and the user then has access to messaging functionality and room-level interactions

**3.1 Sending Messages**

Any input that does not begin with `/` is treated as a chat message, for example:
```
Hello y’all!
```
Messages are serialized to JSON and broadcast through the server to all connected users.

**3.2 Active Users**

List of all users currently active in the chat room:
```
/active_users
```

**3.3 Kick a User**

To remove a user from the chat room:
```
/kick <username>
```
The user is removed from the chat room and is returned to the lobby. This is not a permanent removal.

Note: Only the room owner can kick users

**3.3 Leave Room**

To leave the room and return to the lobby: 
```
/leave
```
This will close your WebSocket connection to the room.

#### 4. Common Commands
These are commands that are available at any location 

**4.1 Get Help**

To see all available commands:
```
/help
```
The help menu is color-coded and shows available commands for the user based on their state in the program. 

**4.2 Exit the Application**

To quit the program:
```
/quit
```


## Reproducibility Guide: 

This section describes how to set up the chatroom, which includes the server and the client. 

### Prerequisites
*Note* The application runs on Linux, MacOS, or Windows WSL. 
Ensure that the following prerequisites are installed:
Rust
Git
OpenSSL (usually already pre-installed on Linux, Mac and WSL)

### Cloning the Repo

    git clone https://github.com/alexlehner3868/ChatRoomApplication.git
    cd ChatRoomApplication

### Server Setup

Before running the server, ensure that there is a `.env` file located inside the root of the *ChatRoomApplicationServer* folder. 
<img width="231" height="343" alt="Image" src="https://github.com/user-attachments/assets/5f984a54-b936-4c59-86e0-4ad42ea721a9" />

The `.env` file contains two important environment variables:

 -  `DATABASE_URL=`
	 - This is the **environment variable that stores the connection string to the database**.
	 - It tells the application **where the database is and how to connect to it**. The database is hosted on Supabase.
 - `JWT_SECRET=`
	 - This is the **secret key used to sign and verify JWT authentication tokens**.

The `.env` file was provided directly to the Professor and TA team via email (sent by Mahmoud Anklis, mahmoud.anklis@mail.utoronto.ca), as it cannot be committed to GitHub without compromising application security. Exposing these secrets would allow unauthorized parties to generate valid tokens.

Run the server by navigating to *ChatRoomApplicationServer* in the project directory and run:
```
cd ChatRoomApplicationServer
cargo run
```
This will launch the server on port 3000 and will start listening for the client to connect to it and send data.

### Client Setup

In the case a different port number should be selected when launching the backend, ensure that the client and server URLs are matching:
<img width="1086" height="291" alt="Image" src="https://github.com/user-attachments/assets/84c612c7-7e2b-42f7-a098-fe4026c58ce7" />

<img width="914" height="225" alt="Image" src="https://github.com/user-attachments/assets/1124eacd-17ca-4739-b710-3171160c5690" />

To start the client locally, navigate to the *ChatRoomApplicationClient* in the project directory and run:
```
cd ChatRoomApplicationClient
cargo run
```
Upon launch, the client begins in the logged-out state, and users can register or log in.
**Note:** Ensure the server is running before the client is launched.


## Lessons Learned and Concluding Remarks:
Throughout this project, we learned valuable lessons in project management and technical development. At the outset, we intentionally divided the project into three distinct components (client, server, and database), allowing us to work in parallel without blocking each other. This really helped us make expedient progress and highlighted to us the importance of clearly partitioning responsibilities. However, we also learned that dividing work without clear communication can lead to rework. Because the client and the server were initially defined in isolation, the messaging protocols diverged, and we later had to reconcile these differences by rewriting parts of both the client and the server. This was an important lesson on balancing the desire for parallel work with the need for alignment on shared interfaces and increased communication.  

We also learned a lot about asynchronous programming by completing this project. We implemented concurrent message handling, non-blocking I/O, and async workflows prior to being taught it in class. This challenged us as we shunned our reliance on LLMs (even enjoying a *raw* coding experience on planes during reading week) and improved our ability to use documentation, written resources, and the compiler. Since we integrated asynchronicity in our code prior to the lecture on it, we were better able to learn during the class. Coming into the lecture with a foundational understanding of the topic enabled us to leave the class with a deeper understanding of the topic. This highlighted to us the benefit of coming to lectures prepared, having reviewed concepts ahead of time to maximize learning. 

In conclusion, while building a project that we are all very proud of, we strengthened both our technical skills and capacity to collaborate effectively on a real distributed system. We are also all leaving this course as Rustaceans. We completed this project *raw,* relying on documentation, course lecture material, and the compiler, and while the learning curve was steep, it was also incredibly fun and rewarding. We learned to think critically and carefully about ownership, state management, and concurrency. We are leaving this course with better strategies for team coordination and an excitement to continue building in and learning Rust. After this course, we are going to continue working on this project; we plan on hosting the server on the cloud (DigitalOcean) so it can be continuously run and truly operate as a persistent system rather than require local deployment. This was outside the scope of this course as hosting a containerized environment requires other technologies, such as Docker, which were prohibited as the entire project needed to be developed with Rust. Since the server would be readily available to users on the internet, we also plan on upgrading our networking from HTTP to HTTPs to meet modern security standards. 


## Individual Contributions:
**Alex: Responsible for implementing the entire client for the chatroom, including user commands, UI and chatroom workflows.**
  - Built robust command-handling system to parse all client commands (`/login`, `/sign_up`, `/join`, `/kick`, `/quit`, `/create`, `/active_users`, `/all_rooms`, `/active_rooms`, `/help`, `/leave`, '`/logout') 
  - Implemented the client’s async architecture to enable non-blocking input, concurrent message handling and responsive UI
  - Built the async client architecture using tokio, including non-blocking input, concurrent messaging and a responsive UI 
  - Implemented WebSockets on the client-side for real-time messaging, including  connection setup, sender/reciever stream management, async messaging, and system events (kicks and room deletions)
  - Built all HTTP interactions on the client-side for login, sign up, room creation, joining rooms, and requesting active users
  - Implemented client workflows and application states (logged-out, in lobby and in chatroom) 
  - Implemented server response handling via HTTP and WebSockets to format and display all messages, updates and errors from the server.
  - Added UI improvements such as standardized colored output and a pager-enabled help menu with dynamic color changes based on the user’s current state (logged out, in lobby, in chatroom)
  -Added secure input handling by masking password fields with asterisks to protect passwords

__Mohamad__: Responsible for designing and implementing the following server logic:
The message protocol to be shared across the server and client
The concurrent in-memory state to keep track of active session data using `Arc<Mutex<>>`:
Users and the room they are connected to
Rooms with the owner and current live members
Isolated tokio broadcast channel per room to ensure all intended users with live WebSocket connections to a room receive messages
The HTTP API routes/logic for:
`/logout`: Clear in-memory session data
`/create_room`: Create new room
`/join_room`: Join room
`/delete_room`: Remove room (Owner only)
`/all_rooms`: List all rooms (In-memory portion)
`/list_room_users`: Get active users
The  HTTP API route/logic to upgrade the HTTP to a WebSocket connection: `/ws`
Processing concurrent WebSocket messages:
Sending text messages
Owner kicking a user from a room
Leaving a room (Owner only)
+ Broadcasting messages received through WebSocket connection to broadcast channel subscribers


**Mahmoud: Responsible for designing and implementing the database layer and authentication of the application**
- Designed and implemented the relational database schema, including **rooms**, **users**, **room_memberships**, and **messages** tables.

- Implemented the full data access layer for data persistence using **SQLx** with typed queries and compile-time checking.
-  Built reusable DB helper functions such as:
    -   `get_user_by_user_id`, `get_user_by_email`
        
    -   `create_room`, `get_room_by_room_id`
        
    -   `add_user_to_room`, `delete_room_by_room_id`, `list_all_rooms`
        
    -   `save_message`, `get_messages_for_room`, `remove_user_from_room`
- Ensured DB interactions follow **source-of-truth rules**, eliminating in-memory state inconsistencies.
- Designed and implemented the structs/models required to match the SQL database tables.
-   Implemented secure user authentication using **Argon2 password hashing** (salted, never stored in plaintext).
-   Developed the **signup and login flows**: validated input, hashed passwords, and stored user records securely.
-   Built a **JWT-based authentication system**: generated tokens on login, validated them on protected REST endpoints like `logout`, `create`, `join`, `delete` room, etc. and the protected WebSocket endpoint for bidirectional communication.
-   Added middleware/guards to attach authenticated user info to requests, ensuring only authorized access to resources.
