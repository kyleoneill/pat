<script setup lang="ts">
  import type { ListedConnectionGame } from '@/models/games_interfaces';

  import { RouterLink } from 'vue-router';

  const props = defineProps<{
    connectionGame: ListedConnectionGame
  }>()
</script>

<template>
  <div class="listed-game">
    <div>{{connectionGame.puzzle_name}}</div>
    <RouterLink class="router-button" :to="{ name: 'play-connection-game', params: { gameSlug: connectionGame.slug } }">Play Game</RouterLink>
  </div>
</template>

<style scoped>
  .listed-game {
    margin-top: 1rem;
  }
</style>
