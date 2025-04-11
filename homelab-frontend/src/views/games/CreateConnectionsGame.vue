<script setup lang="ts">
  import { RouterLink } from 'vue-router';
  import type { Ref } from 'vue'
  import { ref } from 'vue';

  import { createConnectionsGame } from '@/api/games_api';
  import { CreateConnectionsGame } from '@/models/games_interfaces';
  import ConnectionCategoryForm from '@/components/games/connections/ConnectionCategoryForm.vue';

  import useToasterStore from '@/stores/useToasterStore';
  const toasterStore = useToasterStore();

  const loading = ref(false);

  const puzzle_data: Ref<CreateConnectionsGame> = ref(new CreateConnectionsGame());

  function createGame() {
    if (!puzzle_data.value.is_set()) {
      toasterStore.error({text: "Connection game data is not properly filled out"});
      return
    }

    loading.value = true;

    createConnectionsGame(puzzle_data.value).then(response => {
      toasterStore.success({text: `Created new connections game with name ${puzzle_data.value.puzzle_name}`});
      puzzle_data.value = new CreateConnectionsGame();
    }).catch(error => {
      toasterStore.responseError({error: error});
    }).finally(() => {
      loading.value = false;
    })
  }
</script>

<template>
  <div class="section-header">
    <RouterLink class="router-button" to="/games/connections">Back</RouterLink>
  </div>
  <div>
    <!-- TODO: This is pretty ugly :) Should make it look nicer -->
    <h2>Create Connections Game</h2>
    <div class="input-section">
      <label>Name: </label>
      <input v-model="puzzle_data.puzzle_name" />
    </div>
    <ConnectionCategoryForm
      v-for="(row, index) in puzzle_data.connection_categories"
      :key="index"
      :game-row="row"
    />
  </div>
  <button :disabled="loading === true" @click="createGame">Create Connection Game</button>
</template>

<style scoped>

</style>
