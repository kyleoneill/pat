import axios, { type AxiosResponse } from 'axios'

interface ReminderCategory {
  name: string,
  slug: string
}

interface Reminder {
  name: string,
  description: string,
  categories: string[],
  priority: string
}

export async function getAllReminders() {
  return await axios.get("/reminders")
}

export async function getAllReminderCategories() {
  return await axios.get("/reminders/category")
}

export async function deleteReminderById(reminder_id: string) {
  return await axios.delete(`/reminders/${reminder_id}`)
}

export async function createReminderCategory(name: string, slug: string) {
  const data: ReminderCategory = { name: name, slug: slug }
  return await axios.post("/reminders/category", data)
}

export async function createReminder(name: string, description: string, categories: string[], priority: string) {
  const data: Reminder = { name: name, description: description, categories: categories, priority: priority }
  return await axios.post("/reminders", data)
}

// export async function auth_user(username: string, password: string): Promise<AxiosResponse<any, any>> {
//   const data: UserCredentials = { username: username, password: password }
//   return await axios.post("/users/auth", data)
// }

// pub fn reminder_routes() -> Router<AppState> {
//   Router::<AppState>::new()
//     // Reminders
//     .route("/reminders/:reminder_id", put(update_reminder))
//     .route("/reminders/:reminder_id", delete(delete_reminder))
//     // Categories
//     .route("/reminders/category/:category_id", delete(delete_category))
// }
