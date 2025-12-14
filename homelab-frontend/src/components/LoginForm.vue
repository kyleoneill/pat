<script setup lang="ts">
import { ref } from 'vue';
import { authUser, createUser } from '@/api/user_api';

import useToasterStore from '@/stores/useToasterStore';
const toasterStore = useToasterStore();

const username = ref('');
const password = ref('');

// This should be an enum probably
const pageState = ref('login');

const loading = ref(false);

function try_login() {
  // Check if the username or password are not set
  if(username.value === '') {
    toasterStore.error({text: "Must enter a username"});
    return
  }
  else if(password.value === '') {
    toasterStore.error({text: "Must enter a password"});
    return
  }

  // Try to log in
  loading.value = true
  authUser(username.value, password.value)
    .then(response => {
      localStorage.setItem("token", response.data);
      location.reload();
    }).catch(error => {
      toasterStore.responseError({error: error});
    }).finally(() => {
      loading.value = false;
    })
}

function try_create_user() {
  // Check if the username or password are not set
  if(username.value === '') {
    toasterStore.error({text: "Must enter a username"});
    return
  }
  else if(password.value === '') {
    toasterStore.error({text: "Must enter a password"});
    return
  }

  // Try to create a user
  loading.value = true
  createUser(username.value, password.value)
    .then(response => {
      localStorage.setItem("token", response.data);
      location.reload();
    }).catch(error => {
      toasterStore.responseError({error: error});
    }).finally(() => {
      loading.value = false;
    })
}
</script>

<template>
  <div v-if="pageState == 'login'" class="login">
    <h1 class="green loginTitle">Login</h1>

    <p>Username:</p>
    <input v-model="username">

    <p>Password:</p>
    <input v-model="password" type="password">

    <button :disabled="loading === true" @click="try_login">Login</button>
    <button :disabled="loading === true" @click="pageState='create_user'">Create an Account</button>
  </div>
  <div v-if="pageState == 'create_user'" class="login">
    <h1 class="green loginTitle">Create User</h1>

    <p>Username:</p>
    <input v-model="username">

    <p>Password:</p>
    <input v-model="password" type="password">

    <button :disabled="loading === true" @click="try_create_user">Create User</button>
    <button :disabled="loading === true" @click="pageState='login'">Log In</button>
  </div>
</template>

<style scoped>
.login {
  display: grid;
  place-items: center;

  margin-left: 40vw;
  margin-top: 35vh;
}

.loginTitle {
  text-align: center;
  width: 20vw;
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

input {
  margin-bottom: 0.75rem;
}
</style>
