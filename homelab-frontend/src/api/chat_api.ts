import axios from 'axios';

import { websocket_base_url } from '@/../config.json';

import type { CreateChatChannelData, ListChatChannelsParams, ChatChannelSubscribeData } from '@/models/chat_interfaces';

export async function connectChat(token: String) {
  const socket = new WebSocket(`${websocket_base_url}/chat/ws?auth_token=${token}`);
  return socket;
}

export async function createChatChannel(channelData: CreateChatChannelData) {
  return await axios.post("/chat/channels", channelData);
}

export async function listChatChannels(channelListParams?: ListChatChannelsParams) {
  return await axios.get("/chat/channels", {params: channelListParams});
}

export async function chatChannelSubscribe(subscribeData: ChatChannelSubscribeData) {
  return await axios.put("/chat/channels/subscribe", subscribeData);
}

export async function chatChannelUnsubscribe(subscribeData: ChatChannelSubscribeData) {
  return await axios.put("/chat/channels/unsubscribe", subscribeData);
}
