import axios from 'axios';

import type { CreateConnectionsGame } from '@/models/games_interfaces';

export async function createConnectionsGame(game_data: CreateConnectionsGame) {
  return await axios.post("/games/connections", game_data)
}
