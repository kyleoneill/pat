<script setup lang="ts">
import { RouterView } from 'vue-router'
import LoginForm from '@/components/LoginForm.vue'
import { global_state } from './stores/store'

import app_config from '@/../config.json'
import axios from 'axios'
axios.defaults.baseURL = app_config.base_url

import Sidebar from "@/components/MainSidebar.vue"

let maybe_token: string | null = localStorage.getItem("token")
if (maybe_token != null) {
  global_state.set_token(maybe_token)
}
</script>

<template>
  <LoginForm v-if="maybe_token == null"/>
  <div class="app" v-else>
    <Sidebar />
    <div class="content">
      <main>
        <RouterView />
      </main>
    </div>
  </div>
</template>

<style>

.app {
  display: grid;
  grid-template-columns: 0.5fr 1fr;
  align-items: start;
  padding: 0 1rem;
}

main {
  min-width: 50rem;
}

</style>
