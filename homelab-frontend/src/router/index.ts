import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '../views/HomeView.vue';
import remindersRoutes from './reminders_routes';
import gamesRoutes from './games_routes';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    ...remindersRoutes,
    ...gamesRoutes,
  ]
})

export default router
