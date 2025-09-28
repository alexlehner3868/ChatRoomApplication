# ChatRoomApplication

## Objective (in progress)
The Chat Room Application is a Command line Application that will allow a user to communicate with other users. The project is aiming to provide secure messaging delivered as quickly as possible.The messages are to be presistent. The command line interface should be easy to navigate and understand. The system should be able to handle multiple concurrent chat rooms with multiple users in each room.

The application will consist of server and client components: 
* The server will handle database interactions, message broadcasting, and client requests. 
* The client component will be a command line interface that will allow users to chat in rooms with other users. 

## Key Features(in progress)
#### Account
* Secure user registration with system unique username and password that meets defined password policy(i.e minimum 8 characters, one special character, and one uppercase letter).
* Account login,logout, and deletion functionality.
#### Chat Room
* Secure chat room creation with user defined password meeting our defined password policy.
* Ability to join the chat room with valid creditionals(room name and password)
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
