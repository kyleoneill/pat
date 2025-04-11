<script setup lang="ts">
import { ref } from 'vue';

import { createReminderCategory } from '@/api/reminders_api';

import useToasterStore from '@/stores/useToasterStore'
const toasterStore = useToasterStore();

const slug = ref('');
const name = ref('');

const loading = ref(false);

function createNewCategory() {
  if (slug.value === '' || name.value === '') {
    toasterStore.error({text: "Must have reminder details filled out"});
    return
  }
  loading.value = true;
  createReminderCategory(name.value, slug.value).then(response => {
    slug.value = '';
    name.value = '';
  }).catch(error => {
    toasterStore.responseError({error: error});
  }).finally(() => {
    loading.value = false;
  })
}
</script>

<template>
  <h2>Create Reminder Category</h2>
  <div class="input-section">
    <p>Slug:</p>
    <input v-model="slug" />
  </div>
  <div class="input-section">
    <p>Name:</p>
    <input v-model="name" />
  </div>
  <button :disabled="loading === true" @click="createNewCategory">Create Category</button>
</template>

<style scoped>
button {
  padding-inline: 1rem;
}
</style>
