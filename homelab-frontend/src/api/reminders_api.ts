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
