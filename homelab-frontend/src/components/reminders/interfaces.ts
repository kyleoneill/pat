export enum Priority {
  Low = "Low",
  Medium = "Medium",
  High = "High",
  VeryHigh = "VeryHigh"
}

export interface ReminderCategory {
  _id: string,
  slug: string,
  name: string,
  user_id: string
}

export interface Reminder {
  _id: string,
  categories: Array<string>,
  date_time: number,
  description: string,
  name: string,
  priority: string,
  user_id: string
}
