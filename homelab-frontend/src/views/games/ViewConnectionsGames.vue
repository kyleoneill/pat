<script setup lang="ts">
  import { RouterLink } from 'vue-router'

  import Toaster from "@/components/ToasterComponent.vue"
</script>

<template>
  <Toaster />
  <div class="section-header">
    <RouterLink class="router-button" to="/games/connections/new">Create Connections</RouterLink>
  </div>
  <div>
    <div>Connections will go here</div>
  </div>
</template>

<style scoped>

</style>
