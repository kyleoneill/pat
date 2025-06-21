import { reactive } from 'vue';

interface State {
  token: string,
  setToken: (token: string) => void,
  websocketConnection: WebSocket | null,
  setWebsocketConnection: (connection: WebSocket) => void,
}

export const globalState = reactive({
  token: '',
  setToken(new_token: string): void {
    this.token = new_token;
  },
  websocketConnection: null,
  setWebsocketConnection(connection: WebSocket): void {
    this.websocketConnection = connection;
  }
} as State)
