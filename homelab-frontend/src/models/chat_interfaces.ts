export interface CreateChatChannelData {
  name?: String,
  channel_type: Number,
  slug: String,
}

export interface ListChatChannelsParams {
  my_channels?: boolean,
  all_channels?: boolean,
  subscribed?: boolean,
}

export interface ChatChannelSubscribeData {
  channel_id: String,
}

export interface ChatChannel {
  _id: String,
  slug: String,
  channel_type: String,
  name?: String,
  pinned_messages: Array<String>,
  subscribers: Array<String>,
  owner_id: String,
  created_at: Number,
}

interface EmojiDetails {
  id: String,
  name: String,
}

interface Reactions {
  count: number,
  emoji: EmojiDetails,
}

export interface ChatMessage {
  _id: String,
  channel_id: String,
  author_id: String,
  contents: String,
  reply_to: String | null,
  reactions: Array<Reactions>,
  pinned: boolean,
  created_at: number,
  updated_at: number,
}

interface SendMessagePacket {
  channel_id: String,
  contents: String,
  reply_to: String | null,
}

export interface WebSocketRequest {
  type: String,
  data: SendMessagePacket,
}

export interface WebSocketResponse {
  type: String,
  data: ChatMessage,
}
