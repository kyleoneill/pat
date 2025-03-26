<script setup lang="ts">

import type { Reminder } from '@/models/reminder_interfaces';
import ReminderComponent from '@/components/reminders/ReminderComponent.vue';

const props = defineProps<{
  priority: string,
  reminders: Map<string, Array<Reminder>>
}>();
const slugifiedPriority = props.priority.replace(" ", "");

defineEmits(['delete-reminder'])

</script>

<template>
  <h2> {{props.priority}} </h2>
  <div class="card-grid">
    <ReminderComponent v-for="reminder in props.reminders.get(slugifiedPriority)" :key="reminder._id" :reminder="reminder" @delete-reminder="$emit('delete-reminder', reminder.priority, reminder._id, reminder.name)" />
  </div>
</template>

<style scoped>

.card-grid {
  display: grid;
  grid-template-areas: "a a a a";
  gap: 1rem;
  grid-template-columns: max-content;
}

</style>
