# ECE1724 Terminal-Based Multi-User Chat Room with Real Time and Persistent Messaging  - Project Proposal

## Team Members
* Alex Lehner - 1004947506
* Mahmoud Anklis - 1005198313
* Mohamad Alkahil - 1005263448
  
## Motivation 
As nascent Rust fans, our motivation for this project is two fold; we want to learn and develop our own skills and while doing so create a project that will help future Rust learners upskill more efficiently. We are driven to increase the breadth of our knowledge in Rust, so we looked for a project that would showcase Rust's versatility to be used across all parts of an application (backend, frontend, database integration).  We are all also interested in systems programming and sought to find a project to deepen our understanding of core concepts, such as communication protocols, asynchronous concurrency, persistent storage, user authentication, and building a real time UI. The project we chose is a terminal-based multi-user chat room with real time and persistent messaging as it will provide us with hands-on experience in Rust and systems programming. This course is a perfect fit for this project as Rust's ownership rules prevent data races and ensure memory safety, which are imperative for a chat room application handling concurrent users and rooms. 

## Filling the Gap in the Rust Eco System - Upskilling Future Rustaceans
In addition to the goal of personal development described above, we are also trying to fill a gap in the Rust ecosystem - our project will serve as a completed reference project for future learners. While a quick google search yields a few Rust chat rooms, none provide a complete example including a terminal-based UI, persistent users and messages, user authentication, and multi-room support. Our project will provide future Rust learners with a concrete example of a terminal-based full-stack application with an emphasis on Rust best practices. 

## Objective
We are making a command line application that will allow a user to communicate with other users in various chat rooms. The project aims to provide secure messaging delivered as quickly as possible. The messages, users and rooms are to be persistent. The command line interface would be easy to navigate and understand. The system would be able to handle multiple concurrent chat rooms with multiple users in each room.

The application will consist of server and client components: 
* The server will handle database interactions, message broadcasting, and client requests.
* The client component will be a command line interface that will allow users to chat in rooms with other users. 

## Key Features
Building the application requires implementing a server, backend and frontend which constitutes an amount of work commensurate with a team of 3 people. Specifics of features required are as follows: 
### Account
* Secure user registration with system unique username and password that meets defined password policy (i.e minimum 8 characters, one special character, and one uppercase letter).
* Account login, logout, and deletion functionality.
### Chat Room
* Secure chat room creation with user defined password meeting our defined password policy.
* Ability to join the chat room with valid credentials (room name and password).
* List all the joined rooms for a user to access.
* Room ownership belongs to the chat creator.
* Room owner can kick other users out of chat.
* Room owner can delete the entire chat room.
* Ability to see who is active in a room.
### Messages
* Real-time broadcast of a message to all users in the same chat room.
### Data Persistence 
* SQL database storage of accounts, chat rooms, active users, and messages.
* Deleting a room or user should remove associated data from database.
### User Interface
* Unique UIs for each view: sign up, login, chat rooms, online users etc.
* Display messages in real time.
* Help guide for users.
### Client Logic
* Maintain the list of active users and their current rooms. 
* Parse commands 
  * Examples:
    * /create room_id password
    * /join room_id password
    * /kick username
### Security 
* Client uses HTTPS to communicate with server during registration/login.
* Passwords are hashed before storage.
* After login, server provides client with a token for future requests.
* Client and server use secure Websockets for real-time message broadcasting.

## Tentative Plan

Our team will work in parallel on client, server, and persistence components, with early agreement on message protocol and database schema to ensure smooth integration. Development will proceed in 4 phases: parallel setup, integration, feature completion, and polishing & testing.

### Breakdown of Team Member Responsibilities:
#### Alex – Client Development 
* Implement the command-line interface with support for commands (/create, /join, /kick, etc.).
* Design and implement UI.
* Handle real-time message display and asynchronous updates.
* Manage client-side session state, including authentication tokens and the list of active rooms.
* Work with Mahmoud on authentication and token issuance.
* Collaborate with Mohamad to define message and request/response protocols between client and server.
 
#### Mahmoud – Database and Authentication
* Design and implement SQL database schema for users, rooms, and messages.
* Implement user authentication (registration, login, password hashing, token issuance).
* Manage data persistence logic (storing messages, users, and room details).
* Coordinate with Mohamad to expose persistence and authentication functionality through the server API.

#### Mohamad – Server Development
* Build the server-side application that manages WebSocket connections and routes messages between clients.
* Implement chat room logic (creation, joining, kicking users, room deletion).
* Handle real-time broadcasting of messages to connected users within each room.
* Ensure secure client-server communication using HTTPS for login/registration and WebSockets for messaging.


### Collaboration Strategy and Phases: 
#### 1. Parallel Setup Phase: 
  Each member will develop and test their components individually, using mock clients/servers where necessary.
  
##### 1a. Client (UI & Command Parsing)
  * Build a rough terminal UI with states to navigate between login, rooms, and chat views.
  * Implement command parsing (/create, /join, /kick, etc.) and client-side session handling.
  * Enable async send/receive of messages and display in real time.
  
##### 1b. Persistence & Authentication
  * Set up the SQL database with schemas for users, rooms, and messages.
  * Implement database functions (add_room, login_user, message persistence).
  * Handle user authentication with password hashing and token issuance.
  
##### 1c. Server (Networking & Message Routing)
  * Define the message protocol and set up the network server to handle multiple clients concurrently.
  * Manage WebSocket connections for real-time message broadcasting.
  * Implement basic room management (create/join, ownership tracking).

#### 2. Integration Phase: 
  * Server connected to database: authentication, persistent message storage, and room/user management.
  * Room management and message broadcasting finalized with persistence and concurrency.
  * Client UI connected to server: sending and receiving real-time messages over WebSockets.

#### 3. Core Features Completion Phase: 
  * Concurrency finalized: multiple rooms with multiple users, safe concurrent message handling.
  * Room ownership logic: owners can kick/remove users and delete rooms.
  * Persistent messaging: saving and retrieving chat history.
  * Chat view: polished, real-time updates in the terminal.
  * Command parsing fully tested and integrated.

#### 4. Polishing and Testing Phase: 
  * Comprehensive testing of user flows (sign up, login, room creation/join, messaging, admin actions).
  * Usability improvements in terminal UI.
  * Improving error handling.
  * Ensuring security practices (password policies, token-based auth, secure sockets).

