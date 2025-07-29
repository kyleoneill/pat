import type { ReturnUser } from "./user_interfaces";

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
  subscribers: Array<ReturnUser>,
  owner_id: String,
  created_at: Number,
  most_recent_message_id: number,
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
  atomic_id: number,
}


// REQUESTS TO SERVER
interface SendMessagePacket {
  channel_id: String,
  contents: String,
  reply_to: String | null,
}

interface RequestMessages {
  message_count: Number,
  atomic_message_id: Number,
  channel_id: String,
}

export enum WebsocketRequestType {
  CreateMessage = "CreateMessage",
  GetChatState = "GetChatState",
}

export interface WebSocketRequest {
  type: WebsocketRequestType,
  data: SendMessagePacket | RequestMessages,
}

export enum WebsocketResponseType {
  SendChatMessage = "SendChatMessage",
  SendChatState = "SendChatState",
  MessageCreated = "MessageCreated",
  SendError = "SendError",
}

export interface WebSocketResponse {
  type: WebsocketResponseType,
  data: ChatMessage | Array<ChatChannel> | WebSocketError | WebsocketMessageCreated,
}

export interface WebsocketMessageCreated {
  atomic_message_id: Number,
  channel_id: String,
}

export interface WebSocketError {
  status_code: Number,
  msg: String,
}
