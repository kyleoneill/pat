<script setup lang="ts">
  import type { Ref } from 'vue';
  import { ref } from 'vue';
  import { RouterLink, useRoute } from 'vue-router';

  import type { ScrambledGame } from "@/models/games_interfaces";

  import Toaster from "@/components/ToasterComponent.vue";
  import { getConnectionGameToPlay } from "@/api/games_api";

  const route = useRoute();

  const loading = ref(false);

  const currentGame: Ref<ScrambledGame | undefined> = ref();
  const groups: Ref<Array<Array<string>>> = ref([]);

  function getGameToPlay() {
    loading.value = true;
    getConnectionGameToPlay(<string>route.params.gameSlug).then(response => {
      let index: number = 0;
      for (let i: number = 0; i < 4; i++) {
        let currentGroup: Array<string> = [];
        for (let j: number = 0; j < 4; j++) {
          currentGroup.push(response.data.scrambled_clues[index]);
          index++;
        }
        groups.value.push(currentGroup);
      }
      currentGame.value = response.data;
    }).catch(error => {
      // TODO
    }).finally(() => {
      loading.value = false;
    });
  }

  getGameToPlay();

</script>

<template>
  <Toaster />
  <div class="section-header">
    <RouterLink class="router-button" to="/games/connections">Back</RouterLink>
  </div>
  <div v-if="currentGame !== undefined">
    <h2>{{currentGame.puzzle_name}}</h2>
    <div>{{groups}}</div>
  </div>
</template>

<style scoped>

</style>
