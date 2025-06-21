import type { ChatMessage } from '@/models/chat_interfaces';

import { reactive } from 'vue';

interface State {
  token: string,
  setToken: (token: string) => void,
  websocketConnection: WebSocket | null,
  setWebsocketConnection: (connection: WebSocket) => void,
  chatMessages: Array<ChatMessage>,
}

export const globalState = reactive({
  token: '',
  setToken(new_token: string): void {
    this.token = new_token;
  },
  websocketConnection: null,
  setWebsocketConnection(connection: WebSocket): void {
    this.websocketConnection = connection;
  },
  chatMessages: [],
} as State)
