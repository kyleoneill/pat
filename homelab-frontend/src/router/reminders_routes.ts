export default [
  {
    path: '/reminders',
    name: 'reminders',
    // route level code-splitting
    // which is lazy-loaded when the route is visited.
    component: () => import('../views/reminders/RemindersView.vue')
  },
  {
    path: '/reminders/new',
    name: 'create-reminder',
    component: () => import('../views/reminders/CreateReminderView.vue')
  },
  {
    path: '/reminders/category/new',
    name: 'create-reminder-category',
    component: () => import('../views/reminders/CreateReminderCategoryView.vue')
  },
]
