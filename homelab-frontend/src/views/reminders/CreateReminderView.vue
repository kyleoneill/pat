<script setup lang="ts">
// name
// description
// categories (list, will need a dropdown or picker or something)
//    - Will have to get categories and list them here
//    - List by name, set by ID
// priority (again, list with a picker or something)
import { getAllReminderCategories, createReminder } from '@/api/reminders_api'
import type { Ref } from 'vue'
import { ref } from 'vue'

import type { ReminderCategory } from '@/components/reminders/interfaces'

import { Priority } from '@/components/reminders/interfaces'

const loading = ref(true)
const categories: Ref<ReminderCategory[]> = ref([])

const name = ref('')
const description = ref('')
const reminderCategories = ref([])
const priority = ref(Priority.Medium)

function createNewReminder() {
  if (name.value === '' || description.value === '') {
    // TODO: Error message here when I have a better way to render them
    return
  }
  loading.value = true
  createReminder(name.value, description.value, reminderCategories.value, priority.value).then(response => {
    name.value = ''
    description.value = ''
    reminderCategories.value = []
    priority.value = Priority.Medium
    // TODO: Make a toast or something
  }).catch(error => {
    // TODO
  }).finally(() => {
    loading.value = false
  })
}

function fetch_reminder_categories() {
  getAllReminderCategories().then(response => {
    categories.value = response.data
  }).catch(error => {
    // TODO: Should hook into an error handler set up in App.vue
  })
}

fetch_reminder_categories()
loading.value = false
</script>

<template>
  <h1>Create Reminder</h1>
  <div class="input-section">
    <p>Name:</p>
    <input v-model="name" />
  </div>
  <div class="input-section">
    <p>Description:</p>
    <textarea v-model="description"></textarea>
  </div>
  <div class="input-section">
    <p>Categories:</p>
    <div class="wrapper">
      <div class="cell" v-for="category in categories" :key="category._id">
        <input type="checkbox" :value="category._id" v-model="reminderCategories" />
        <label>{{ category.name }}</label>
      </div>
    </div>
  </div>
  <div class="input-section">
    <p>Priority:</p>
    <select name="priority" v-model="priority">
      <option value="Low">Low</option>
      <option selected value="Medium">Medium</option>
      <option value="High">High</option>
      <option value="VeryHigh">Very High</option>
    </select>
  </div>
  <button :disabled="loading === true" @click="createNewReminder">Create Reminder</button>
</template>

<style scoped>
.wrapper {
  margin-left: 2rem;
  display: grid;
  grid-template-columns: repeat(3, min-content);
}

.cell {
  display: flex;
  margin-right: 2rem;
}

.cell > label {
  margin-left: 0.5rem;
}

button {
  padding-inline: 1rem;
}

.input-section {
  margin-bottom: 1rem;
}
</style>
