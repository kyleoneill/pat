<script setup lang="ts">
import { RouterLink } from 'vue-router'
</script>

<template>
  <div class="sidebar">
    <nav>
      <ul>
        <li><RouterLink to="/">Home</RouterLink></li>
        <li><RouterLink to="/reminders">Reminders</RouterLink></li>
        <li><RouterLink to="/games/connections">Connections</RouterLink></li>
        <li><a @click="$emit('logout')">Logout</a></li>
      </ul>
    </nav>
  </div>
</template>

<style scoped>
.sidebar {
  line-height: 1.5;
  margin-right: 2rem;

  padding-right: 1rem;
  border-right-color: var(--vt-c-text-light-2);
  border-right-style: solid;
  border-right-width: thin;
}

nav {
  width: 100%;
  font-size: 20px;
  margin-top: 2rem;
}

nav a.router-link-exact-active {
  color: var(--color-text);
}

nav a.router-link-exact-active:hover {
  background-color: transparent;
}

nav a {
  border-left: 1px solid var(--color-border);
  cursor: pointer;
}

nav a:first-of-type {
  border: 0;
}

nav > ul {
  list-style-type: None;
  padding-left: 0;
  padding-top: 5px;
  padding-bottom: 5px;
}

li {
  padding-bottom: 0.5rem;
}
</style>
