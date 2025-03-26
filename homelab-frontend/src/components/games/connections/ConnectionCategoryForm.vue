<script setup lang="ts">
  import type { ConnectionGameRowInterface } from '@/models/games_interfaces';

  const props = defineProps<{
    gameRow: ConnectionGameRowInterface,
  }>();

</script>

<template>
  <div class="input-section">
    <div>
      <label>Category Name: </label>
      <input v-model="gameRow.category_name" />
    </div>

    <div>
      <div v-for="(clue, index) in gameRow.category_clues" :key="index">
        <label>Clue {{ index + 1 }}: </label>
        <input v-model="gameRow.category_clues[index]" />
      </div>
    </div>
  </div>
</template>

<style scoped>

</style>
