import type { CreateChatChannelData, ListChatChannelsParams, ChatChannelSubscribeData, ChatMessage } from '@/models/chat_interfaces';

import axios from 'axios';
import { websocket_base_url } from '@/../config.json';
import { globalState } from '@/stores/store';

export async function connectChat(token: String) {
  const socket = new WebSocket(`${websocket_base_url}/chat/ws?auth_token=${token}`);
  socket.onmessage = (event) => {
    const websocketResponse = JSON.parse(event.data);
    if (websocketResponse.type === 'SendChatMessage') {
      const responseData: ChatMessage = websocketResponse.data;
      globalState.chatMessages.push(responseData);
    }
    else if (websocketResponse.type === 'SendError') {
      // TODO: Error handling
    }
  };
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
