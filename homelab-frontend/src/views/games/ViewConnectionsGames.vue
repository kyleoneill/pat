<script setup lang="ts">
  import type { Ref } from 'vue';

  import { RouterLink } from 'vue-router';
  import { ref } from 'vue';

  import type { ListedConnectionGame } from '@/models/games_interfaces';

  import { getAllConnectionGamesForOthers } from '@/api/games_api';
  import SimplifiedConnectionGame from '@/components/games/connections/SimplifiedConnectionGame.vue';

  import useToasterStore from '@/stores/useToasterStore';
  const toasterStore = useToasterStore();

  const loading = ref(false);
  const connectionGames: Ref<Array<ListedConnectionGame>> = ref([]);

  function getGames() {
    loading.value = true;
    getAllConnectionGamesForOthers().then(response => {
      connectionGames.value = response.data;
    }).catch(error => {
      toasterStore.responseError({error: error});
    }).finally(() => {
      loading.value = false;
    })
  }

  getGames();
</script>

<template>
  <div class="section-header">
    <RouterLink class="router-button" to="/games/connections/new">Create Connections</RouterLink>
  </div>
  <div>
    <h2>Connection Games</h2>
    <div v-if="!loading">
      <SimplifiedConnectionGame v-for="connectionGame in connectionGames" :key="connectionGame.slug" :connection-game="connectionGame" />
    </div>
  </div>
</template>

<style scoped>

</style>
