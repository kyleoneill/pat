import type { CreateChatChannelData, ListChatChannelsParams, ChatChannelSubscribeData, ChatMessage, WebSocketError } from '@/models/chat_interfaces';

import axios from 'axios';
import { websocket_base_url } from '@/../config.json';
import { globalState } from '@/stores/store';

export async function connectChat(token: String) {
  const socket = new WebSocket(`${websocket_base_url}/chat/ws?auth_token=${token}`);
  socket.onmessage = (event) => {
    const websocketResponse = JSON.parse(event.data);
    if (websocketResponse.type === 'SendChatMessage') {
      const chatMessage: ChatMessage = websocketResponse.data;
      if (!globalState.chatMessages.has(chatMessage.channel_id)) {
        globalState.chatMessages.set(chatMessage.channel_id, []);
      }
      globalState.chatMessages.get(chatMessage.channel_id)?.push(chatMessage);
    }
    else if (websocketResponse.type === 'SendChatState') {
      const chatMessages: Array<ChatMessage> = websocketResponse.data;
      chatMessages.forEach(message => {
        if (!globalState.chatMessages.has(message.channel_id)) {
          globalState.chatMessages.set(message.channel_id, []);
        }
        globalState.chatMessages.get(message.channel_id)?.push(message);
      });
    }
    else if (websocketResponse.type === 'SendError') {
      const errorResponse: WebSocketError = websocketResponse.data;
      console.error(`DEBUG: Websocket error ${errorResponse.status_code}: ${errorResponse.msg}`)
      // TODO: Actual error handling
    }
  };
  socket.onclose = (event) => {
    // How do I know if I initiated this, or if the server did? If the server initiated, we want to try to re-start it again
    globalState.setWebsocketConnection(null);
  };
  globalState.setWebsocketConnection(socket);
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
