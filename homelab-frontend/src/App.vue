<script setup lang="ts">
import { RouterView } from 'vue-router'
import LoginForm from '@/components/LoginForm.vue'
import { global_state } from './stores/store'

import app_config from '@/../config.json'
import axios from 'axios'
axios.defaults.baseURL = app_config.base_url

import { getUserMe } from '@/api/user_api'

import Sidebar from "@/components/MainSidebar.vue"

function logout(): void {
  localStorage.removeItem("token")
  location.reload()
}

let maybe_token: string | null = localStorage.getItem("token")
if (maybe_token != null) {
  global_state.set_token(maybe_token)
  axios.defaults.headers.common["Authorization"] = maybe_token

  // Verify that the token is still valid
  getUserMe()
    .then(
      // We don't care about the response data, just that the response succeeded
    ).catch(error => {
      if(error.response.status_code !== 401) {
        // We got an unexpected error
      }
      // Our token is not valid if we got a 401, so we log out
      logout()
  })
}
</script>

<template>
  <LoginForm v-if="maybe_token == null"/>
  <div class="app" v-else>
    <Sidebar @logout="logout" />
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

.router-button {
  background: var(--vt-c-text-light-1);
  padding: 5px;
  text-align: center;
  border-radius: 5px;
  color: var(--color-text);
}

main {
  min-width: 50rem;
  margin-top: 2rem;
}

</style>
