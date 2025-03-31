<script setup lang="ts">
  import type { Ref } from 'vue';

  import { RouterLink } from 'vue-router';
  import { ref } from 'vue';

  import type { ListedConnectionGame } from '@/models/games_interfaces';

  import Toaster from "@/components/ToasterComponent.vue";

  import { getAllConnectionGamesForOthers } from '@/api/games_api';
  import SimplifiedConnectionGame from '@/components/games/connections/SimplifiedConnectionGame.vue';

  const connectionGames: Ref<Array<ListedConnectionGame>> = ref([]);

  function getGames() {
    getAllConnectionGamesForOthers().then(response => {
      connectionGames.value = response.data;
    }).catch(error => {
      // TODO
    }).finally(() => {
      // ?
    })
  }

  getGames();
</script>

<template>
  <Toaster />
  <div class="section-header">
    <RouterLink class="router-button" to="/games/connections/new">Create Connections</RouterLink>
  </div>
  <div>
    <h2>Connection Games</h2>
    <div>
      <SimplifiedConnectionGame v-for="connectionGame in connectionGames" :key="connectionGame.slug" :connection-game="connectionGame" />
    </div>
  </div>
</template>

<style scoped>

</style>
