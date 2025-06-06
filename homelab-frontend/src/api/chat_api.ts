import { websocket_base_url } from '@/../config.json';

export async function connectChat(token: String) {
  const socket = new WebSocket(`${websocket_base_url}/chat/ws?auth_token=${token}`);
  return socket;

//   return await axios.get("/chat/ws", {
//     headers: {
//         "Connection": "Upgrade",
//         "Upgrade": "websocket",
//     }
//   });
}
