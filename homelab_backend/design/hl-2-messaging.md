# Messaging

## Requirements
Users should have the ability to send messages to each other. Messages should be stored in the database and must be
able to be retrieved in a paginated format.

Support should be added for features common to modern chat applications, including
- Italics
- Bolding
- Spoiler tags
- Render blocks marked with triple backticks, ideally with code syntax highlighting
- Single backtick blocks
- Emojis
- Editing an owned message
- Pinning messages within a chat
- Reacting to a message with emojis
- Embedding links
  - Render gifs if the link directs to a gif
- Attachments
  - gifs, images, videos
- Searching a conversation for text
- Read receipts
- Message Deletion
- Reply to a message

A message will be sent by a user and sent to a channel. Users "subscribe" to a channel to receive messages sent to it.
A group chat may be 5 users subscribed to a channel. A direct message between two users will be a channel with two
recipients.

## Backend
Support must be added for websocket connections, allowing for bidirectional communication with a client. When the server
receives a message from a client it should store the message in the database and push the message to all online recipients
of the associated channel. A message payload should include the message text and metadata about the message. A message
struct might look like:

```rust
// Message
struct EmojiDetails {
  id: String,
  name: String,
}

struct Reactions {
  count: i64,
  emoji: EmojiDetails,
}

struct Attachment {
  // This is attached media to a message, like an image or video
}

struct Message {
  id: String,
  channel_id: String, // id of the channel the message was sent to
  author: ReturnUser,
  timestamp: i64,
  edited_timestamp: i64,
  contents: String,
  reply_to: Option<String>,
  reactions: Vec<Reactions>,
  pinned: Bool, // Is this message pinned in the current channel?
  attachments: Vec<Attachment>, // If this message has attached media, like an image or video
}

// Channel
enum ChannelType {
  DirectMessage,
  Group,
  Server, // Stub type to support a more featured group chat, like a Discord server
}

struct Channel {
  id: String,
  channel_type: ChannelType,
  name: Option<String>,
  message_count: i64,
  pinned_messages: Vec<String>,
  recipients: Vec<String>, // Vec of user IDs
}
```

For a direct message, a channel must be created when one user messages another for the first time.

For group chats, a user must be able to create a new group and then invite recipients to it.

When a channel is deleted, a task should be generated that deletes all messages associated with it.

### Attachments
Attachments can be uploaded when sending a message. Attached media should be deleted from the database when its
associated message is deleted.

### Websocket Connections
The server must maintain a connection with a client once it is opened. Data about this connection must be saved, such
as the ID of the user who opened it. This connection should be closed after a timeout period of inactivity.

There should be some way for the server to bundle and store all open connections with associated data in state and
keep track of how many are open, there should be a maximum number of connections allowed before the server begins
rejecting new requests.

### Websocket Packets
The server will send and receive packets over websocket connections. Clients will send a packet to create a message, the
server will send packets to clients when there are new messages for them to receive.

```rust
enum PacketType {
  ChatMessage,
}

// Should packets have binary data? Should the enum determine that is in its content?
struct ReceivePacket {
  packet_type: PacketType,
  packet_content: String,
  destination: String, // ID of the channel this message is meant for
}

struct SendPacket {
  packet_status: StatusCode,
  packet_type: PacketType,
  packet_content: String,
  destination: String, // ID of the channel this message is meant for
}
```

### Broadcasting
Messages will need to be broadcast to all relevant clients, an example flow would look like this
```text
- User 1 sends a message to channel 1
  - Channel 1 has 3 subscribers, users 1, 2, and 3
- The server receives a packet from user 1
    - Check that user 1 is in channel 1
    - Create a message in the db
- The server checks the subscriber list for channel 1
    - Check if it has an active connection with users 2 and 3
    - Sends the message to each user with an active connection
```

## Frontend
A messaging section must be added to the application. This section will allow a user to search for other users who they
can message. The landing page for messages should show a list of channels that the user is currently in. The landing
page should allow the user to create a new channel and invite other users to it.

When opening a channel, messages in that channel should be retrieved in batches. The most recent X messages should be loaded,
with more being loaded in batches as the user scrolls up through the channel.

A channel should contain a search area so a user can search the channel for given text.

### Websocket Connections
The client must handle dealing with a connection closing, possibly making a new connection when the user begins using the
page again. While in active use, maybe the client should send a "heartbeat" or a ping to keep the connection alive.
Might be worth seeing how other chat applications handle this problem.

### Rendering a Message
Text within a message may contain tags, emojis, code blocks, etc. which need special rendering when displaying a message.
A list of these includes:
- Tags
  - Italics - \<i>foo\</i>
  - Bolding - \<b>foo\</b>
  - Spoiler \<spoil>foo\</spoil>
- Render blocks marked with triple backticks, ideally with code syntax highlighting
- Emojis
- Embedded links
  - Render a gif or image if the link directs to one
- Attachments
  - gifs, images, videos
- Single backticks
  - Text enclosed between single backticks should ignore all other render styling and be displayed like an inline block
  - e.g., `this _text_` will be rendered with underscores rather than the word "text" being italicized

### Interacting with a message
All messages should allow for the following interactions:
- Reply
- Pin
- React

Messages owned by the current user should allow for the following interactions:
- Edit
- Delete
