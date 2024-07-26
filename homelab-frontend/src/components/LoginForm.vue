<script setup lang="ts">
import { ref } from 'vue'
import { auth_user } from '@/api/user_api'

const username = ref('')
const password = ref('')

const display_username_error_msg = ref(false)
const display_password_error_msg = ref(false)
const response_error = ref('')
const loading = ref(false)

function try_login() {
  // Reset values if they were set from a previous attempt
  display_username_error_msg.value = false
  display_password_error_msg.value = false
  response_error.value = ''


  // Check if the username or password are not set
  if(username.value === '') {
    display_username_error_msg.value = true
    return
  }
  else if(password.value === '') {
    display_password_error_msg.value = true
    return
  }

  // Try to log in
  loading.value = true
  auth_user(username.value, password.value)
    .then(response => {
      loading.value = false
      localStorage.setItem("token", response.data)
      location.reload()
    }).catch(error => {
      // TODO: Verify that `error.response.status === 404`? What do we do if else?
      response_error.value = error.response.data
      loading.value = false
  })
}
</script>

<template>
  <div class="login">
    <h1 class="green">Login</h1>

    <p>Username:</p>
    <input v-model="username">
    <div v-if="display_username_error_msg === true">
      <p class="error-text">Invalid Username</p>
    </div>
    <br v-else />

    <p>Password:</p>
    <input v-model="password" type="password">
    <div v-if="display_password_error_msg === true">
      <p class="error-text">Invalid Password</p>
    </div>
    <br v-else />

    <button :disabled="loading === true" @click="try_login">Login</button>
    <p class="error-text" v-if="response_error !== ''">{{response_error}}</p>
  </div>
</template>

<style scoped>
.login {
  margin-left: 40vw;
}

h1 {
  font-weight: 500;
  font-size: 2.6rem;
  position: relative;
  top: -10px;
}

button {
  margin-top: 1rem;
  padding-inline: 1.25rem;
}

@media (min-width: 1024px) {
  .login h1 {
    text-align: left;
  }
}
</style>
