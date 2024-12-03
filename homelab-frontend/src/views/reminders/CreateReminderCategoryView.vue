<script setup lang="ts">
import { ref } from 'vue'

import { createReminderCategory } from '@/api/reminders_api'

const slug = ref('')
const name = ref('')

const loading = ref(false)

function create_new_category() {
  if (slug.value === '' || name.value === '') {
    // TODO: Error message here when I have a better way to render them
    return
  }
  loading.value = true
  createReminderCategory(name.value, slug.value).then(response => {
    slug.value = ''
    name.value = ''
  }).catch(error => {
    // TODO
  }).finally(() => {
    loading.value = false
  })
}
</script>

<template>
  <h1>Create Reminder Category</h1>
  <div class="input-section">
    <p>Slug:</p>
    <input v-model="slug" />
  </div>
  <div class="input-section">
    <p>Name:</p>
    <input v-model="name" />
  </div>
  <button :disabled="loading === true" @click="create_new_category">Create Category</button>
</template>

<style scoped>
button {
  padding-inline: 1rem;
}

.input-section {
  margin-bottom: 1rem;
}
</style>
