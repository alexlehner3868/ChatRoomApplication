# ECE1724 Terminal-Based Multi-User Chat Room with Real Time and Persistent Messaging  - Project Proposal

## Team Members
* Alex Lehner - 1004947506
* Mahmoud Anklis - 
* Mohamad Alkahil - 1005263448
  
## Motivation 
As nascent Rust fans, our motivation for this project is two fold; we want to learn and develop our own skills and while doing so create a project that will help future Rust learners upskill more efficiently. We are driven to increase the breadth of our knowledge in Rust, so we looked for a project that would showcase Rust's versatility to be used across all parts of an application (server, backend, UI, database integration).  We are all also interested in systems programming and sought to find a project to deepen our understanding of core concepts, such as communication protocols, asynchronous concurrency, persistent storage, user authentication, and building a real time UI. The project we chose is a terminal-based multi-user chat room with real time and persistent messaging as it will provide us with hands-on experience in Rust and systems programming. This course is a perfect fit for this project as Rust's ownership rules prevent data races and ensure memory safety, which are imperative for a chat room application handling concurrent users and rooms. 

## Filling the Gap in The Rust Eco System - Upskilling Future Rustaceans
In addition to the goal of personal development described above, we are also trying to fill a gap in the Rust ecosystem - our project will serve as a learning oriented reference. While a quick google search yields a few Rust chat rooms, none provide a complete example including a terminal-based UI, persistent users and messages, user authentication, and multi-room support. Our project will provide future Rust learners with a concrete example of a terminal-based full-stack application with an emphasis on Rust best practices. 

## Objective
We are making a command line application that will allow a user to communicate with other users in various chat rooms. The project is aiming to provide secure messaging delivered as quickly as possible. The messages, users and rooms are to be persistent. The command line interface should be easy to navigate and understand. The system should be able to handle multiple concurrent chat rooms with multiple users in each room.

The application will consist of server and client components: 
* The server will handle database interactions, message broadcasting, and client requests.
* The client component will be a command line interface that will allow users to chat in rooms with other users. 

## Key Features
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
* After login, Server provides client with a token for future requests.
* Client and Server use secure Websockets for real-time message broadcasting.

## Work For Team Members
### Alex Lehner
* The client component will focus on command parsing, async message display, and sending requests to server.
### Mahmoud Anklis 
* Responsible for designing and implmenting the SQL schema, and authentacation system.
### Mohamad Alkahil
* The server component will manage websocket connections, message routing, and database transactions.


## Tentative Plan (TBD)
