import axios from 'axios';

import type { CreateConnectionsGame } from '@/models/games_interfaces';

export async function createConnectionsGame(gameData: CreateConnectionsGame) {
  return await axios.post("/games/connections", gameData);
}

export async function getAllConnectionGamesForOthers() {
  return await axios.get("/games/connections");
}

export async function getAllConnectionGamesForMe() {
  return await axios.get("/games/connections/mine");
}

export async function getConnectionGameToPlay(gameSlug: string) {
  return await axios.get(`/games/connections/play/${gameSlug}`);
}

export async function trySolveConnectionGameRow(gameSlug: string, data: Array<string>) {
  return await axios.put(`/games/connections/play/${gameSlug}/try_solve`, data);
}
