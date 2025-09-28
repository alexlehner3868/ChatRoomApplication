# ECE1724 Terminal-Based Multi-User Chat Room with Real Time and Persistent Messaging  - Project Proposal

## Motivation 
As nascent Rust fans, our motivation for this project is two fold; we want to learn and develop our own skills and while doing so create a project that will help future Rust learners upskill more efficiently. We are driven to increase the breadth of our knowledge in Rust, so we looked for a project that would showcase Rust's versatility to be used across all parts of an application (server, backend, UI, database integration).  We are all also interested in systems programming and sought to find a project to deepen our understanding of core concepts, such as communication protocols, asynchronous concurrency, persistent storage, user authentication, and building a real time UI. The project we chose is a terminal-based multi-user chat room with real time and persistent messaging as it will provide us with hands-on experience in Rust and systems programming. This course is a perfect fit for this project as Rust's ownership rules prevent data races and ensure memory safety, which are imperative for a chat room application handling concurrent users and rooms. 

In addition to personal development, we are also trying to fill a gap in the Rust ecosystem - our project will serve as a learning oriented reference. While a quick google search yields a few Rust chat rooms, none provide a complete example including a terminal-based UI, persistent users and messages, user authentication, and multi-room support. Our project will provide future Rust learners with a concrete example of a terminal-based full-stack application with an emphasis on Rust best practices. 


## Objective (in Progress)
The Chat Room Application is a Command line Application that will allow a user to communicate with other users. The project is aiming to provide secure messaging delivered as quickly as possible.The messages are to be presistent. The command line interface should be easy to navigate and understand. The system should be able to handle multiple concurrent chat rooms with multiple users in each room.

The application will consist of server and client components: 
* The server will handle database interactions, message broadcasting, and client requests. 
* The client component will be a command line interface that will allow users to chat in rooms with other users. 

## Key Features(in progress)
#### Account
* Secure user registration with system unique username and password that meets defined password policy (i.e minimum 8 characters, one special character, and one uppercase letter).
* Account login,logout, and deletion functionality.
#### Chat Room
* Secure chat room creation with user defined password meeting our defined password policy.
* Ability to join the chat room with valid creditionals (room name and password)
* Room ownership belongs to the chat creator, as such the owner can kick other users out of chat or delete the entire chat.
#### Messages
* Real-time boradcast of a message to all users in the same chat room.
#### Storage
* SQL database storage of accounts,chat rooms, and messages.
* Deleting a chat should remove associated data from database.
#### User Interface
* Command line parsing
  * Examples:
    * /create room_id password
    * /join room_id password
    * /kick username
* message display
* Command Rference to help users.

  ## Plan (TBD)
