import type { ReturnUser } from '@/models/user_interfaces';
import type { ChatMessage } from '@/models/chat_interfaces';

import { reactive } from 'vue';

interface State {
  token: string,
  setToken: (token: string) => void,
  currentUser: ReturnUser | null,
  setCurrentUser: (user: ReturnUser) => void,
  websocketConnection: WebSocket | null,
  setWebsocketConnection: (connection: WebSocket | null) => void,
  chatMessages: Map<String, Array<ChatMessage>>,
}

export const globalState = reactive({
  token: '',
  setToken(new_token: string): void {
    this.token = new_token;
  },
  currentUser: null,
  setCurrentUser(user: ReturnUser): void {
    this.currentUser = user;
  },
  websocketConnection: null,
  setWebsocketConnection(connection: WebSocket | null): void {
    this.websocketConnection = connection;
  },
  chatMessages: new Map<String, Array<ChatMessage>>(),
} as State)
