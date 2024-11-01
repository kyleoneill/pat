import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/reminders',
      name: 'reminders',
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/reminders/RemindersView.vue')
    },
    // TODO: Can I break routing into multiple files and keep the "reminders" routes in the reminders folder?
    {
      path: '/reminders/new',
      name: 'create-reminder',
      component: () => import('../views/reminders/CreateReminderView.vue')
    },
    {
      path: '/reminders/category/new',
      name: 'create-reminder-category',
      component: () => import('../views/reminders/CreateReminderCategoryView.vue')
    }
  ]
})

export default router
