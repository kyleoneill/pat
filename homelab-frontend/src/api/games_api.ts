import axios from 'axios';

import type { CreateConnectionsGame } from '@/models/games_interfaces';
import type { RouteParamValue } from 'vue-router'

export async function createConnectionsGame(gameData: CreateConnectionsGame) {
  return await axios.post("/games/connections", gameData)
}

export async function getAllConnectionGamesForOthers() {
  return await axios.get("/games/connections")
}

export async function getAllConnectionGamesForMe() {
  return await axios.get("/games/connections/mine")
}

export async function getConnectionGameToPlay(gameSlug: string) {
  return await axios.get(`/games/connections/play/${gameSlug}`)
}
