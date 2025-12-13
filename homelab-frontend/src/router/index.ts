import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '../views/HomeView.vue';
import remindersRoutes from './reminders_routes';
import gamesRoutes from './games_routes';
import chatRoutes from './chat_routes';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/my_account',
      name: 'my-account',
      component: () => import('../views/account/AccountView.vue')
    }, 
    ...remindersRoutes,
    ...gamesRoutes,
    ...chatRoutes,
  ]
})

export default router
