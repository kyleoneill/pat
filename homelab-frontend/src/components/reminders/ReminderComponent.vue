<script setup lang="ts">
import type { Reminder } from '@/models/reminder_interfaces'

const props = defineProps<{
  reminder: Reminder
}>()

defineEmits(['delete-reminder'])

</script>

<template>
  <div class="reminder-card">
    <div class="card-header">
      <h2 class="card-name">{{ props.reminder.name }}</h2>
    </div>
    <div class="card-body">
      <p class="card-description">{{ props.reminder.description }}</p>
    </div>
    <div class="card-footer">
      <div class="card-details">
        <p class="card-categories" v-if="props.reminder.categories.length > 0">Categories: {{ props.reminder.categories.join(', ') }}</p>
      </div>
      <button @click="$emit('delete-reminder')" class="delete-button">Delete</button>
    </div>
  </div>
</template>

<style scoped>

.reminder-card {
  border: 1px solid var(--color-border-light);
  border-radius: 0.5rem;
  padding: 1rem;
  min-width: 15rem;
  width: max-content;
  max-width: 30rem;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.card-header {
  margin-bottom: 0.5rem;
}

.card-name {
  font-size: var(--text-size);
  margin: 0;
  font-weight: bold;
}

.card-body {
  flex-grow: 1;
  margin-bottom: 1.5rem;
}

.card-details {
  font-size: var(--small-text-size);
  color: var(--vt-c-text-dark-2);
  margin-right: 2.5rem;
}

.card-footer {
  display: flex;
}

.delete-button {
  margin-left: auto;
  background-color: #ff4d4d;
  color: white;
  border: none;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
}

.delete-button:hover {
  background-color: #ff3333;
}

</style>
