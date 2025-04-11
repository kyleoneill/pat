<script setup lang="ts">
import type { Ref } from 'vue'
import { ref } from 'vue'
import { RouterLink } from 'vue-router'
import { getAllReminders, getAllReminderCategories, deleteReminderById } from '@/api/reminders_api'
import ReminderCards from '@/components/reminders/ReminderCards.vue'
import type { Reminder, ReminderCategory } from '@/models/reminder_interfaces'

import useToasterStore from '@/stores/useToasterStore'
const toasterStore = useToasterStore();

const loading = ref(true);
const reminders: Ref<Map<string, Array<Reminder>>> = ref(new Map<string, Array<Reminder>>());
const categories = ref(new Map<string, ReminderCategory>);

const priorities = ["Very High", "High", "Medium", "Low"];

function fetch_reminders() {
  getAllReminders().then(response => {
    for (const i in response.data) {
      const reminder: Reminder = response.data[i];

      // Swap out the ID in reminder.categories with the name of each category
      const categoryIds = reminder.categories;
      reminder.categories = [];
      for (const i in categoryIds) {
        const category = categories.value.get(categoryIds[i]);
        if (category !== undefined) {
          reminder.categories.push(category.name);
        }
      }

      if(reminders.value.has(reminder.priority)) {
        const reminderList = reminders.value.get(reminder.priority);
        if(reminderList === undefined) {
          // TODO: Raise an error here? This shouldn't be a possible state
          console.log("Impossible error state reached");
        }
        else {
          reminderList.push(reminder);
        }
      }
      else {
        reminders.value.set(reminder.priority, [reminder]);
      }
    }
  }).catch(error => {
    toasterStore.responseError({error: error});
  })
}

function fetch_reminder_categories() {
  getAllReminderCategories().then(response => {
    const categoryMap = new Map();
    for (const i in response.data) {
      const category: ReminderCategory = response.data[i]
      categoryMap.set(category._id, category);
    }
    categories.value = categoryMap
  }).catch(error => {
    toasterStore.responseError({error: error});
  })
}

function handleDeleteReminder(priority: string, reminderId: string, reminderName: string) {
  deleteReminderById(reminderId).then(res => {
    const reminderList = reminders.value.get(priority);
    if (reminderList !== undefined) {
      const newReminders = reminderList.filter(reminder => reminder._id !== reminderId)
      if(newReminders.length === 0) {
        reminders.value.delete(priority);
      }
      else {
        reminders.value.set(priority, newReminders);
      }
    }
    toasterStore.success({text: `Deleted reminder ${reminderName}`});
  }).catch(error => {
    toasterStore.responseError({error: error});
  })
}

fetch_reminder_categories();
fetch_reminders();

loading.value = false;

</script>

<template>
  <div class="reminders">
    <div class="section-header">
      <RouterLink class="router-button" to="/reminders/new">Create Reminder</RouterLink>
      <RouterLink class="router-button" to="/reminders/category/new">Create Category</RouterLink>
    </div>
    <div>
      <h1>Reminders</h1>
      <h2 v-if="reminders.size === 0">No Reminders Set</h2>
      <div v-else>
        <div class="reminder-cards" v-for="priority in priorities" :key="priority">
          <ReminderCards
            v-if="reminders.has(priority.replace(' ', ''))"
            :reminders="reminders"
            :priority="priority"
            @delete-reminder="handleDeleteReminder"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>

.reminder-cards {
  margin-bottom: 1rem;
}

@media (min-width: 1024px) {
  .reminders {
    min-height: 100vh;
    align-items: center;
  }
}
</style>
