<script setup lang="ts">
import { getAllReminderCategories, createReminder } from '@/api/reminders_api'
import type { Ref } from 'vue'
import { ref } from 'vue'

import type { ReminderCategory } from '@/models/reminder_interfaces'

import { Priority } from '@/models/reminder_interfaces'

import useToasterStore from '@/stores/useToasterStore'
const toasterStore = useToasterStore();

const loading = ref(true)
const categories: Ref<ReminderCategory[]> = ref([])

const name = ref('')
const description = ref('')
const reminderCategories = ref([])
const priority = ref(Priority.Medium)

function createNewReminder() {
  if (name.value === '' || description.value === '') {
    toasterStore.error({text: "Name and description must be filled out"});
    return
  }
  loading.value = true
  createReminder(name.value, description.value, reminderCategories.value, priority.value).then(response => {
    toasterStore.success({text: `Created new reminder with name ${name.value}`});
    name.value = ''
    description.value = ''
    reminderCategories.value = []
    priority.value = Priority.Medium
  }).catch(error => {
    toasterStore.responseError({error: error});
  }).finally(() => {
    loading.value = false
  })
}

function fetch_reminder_categories() {
  getAllReminderCategories().then(response => {
    categories.value = response.data
  }).catch(error => {
    toasterStore.responseError({error: error});
  })
}

fetch_reminder_categories()
loading.value = false
</script>

<template>
  <h2>Create Reminder</h2>
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
</style>
