<script setup lang="ts">
  import type { Ref } from 'vue';

  import { RouterLink } from 'vue-router';
  import { ref } from 'vue';

  import { global_state } from '@/stores/store';

  import { connectChat } from '@/api/chat_api';

  import useToasterStore from '@/stores/useToasterStore';
  const toasterStore = useToasterStore();

  const loading = ref(false);

</script>

<template>
  <div class="section-header">
    <!-- <RouterLink class="router-button" to="/games/connections/new">Create Connections</RouterLink> -->
  </div>
  <div>
    <h2>Chat</h2>
    <div v-if="!loading">
        <button @click="connectChat(global_state.token)">Debug</button>
    </div>
  </div>
</template>

<style scoped>

</style>
