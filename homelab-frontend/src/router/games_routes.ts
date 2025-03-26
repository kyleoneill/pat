export default [
  {
    path: '/games/connections',
    name: 'view-connections-games',
    component: () => import('../views/games/ViewConnectionsGames.vue')
  },
  {
    path: '/games/connections/new',
    name: 'create-connections-games',
    component: () => import('../views/games/CreateConnectionsGame.vue')
  },
]
