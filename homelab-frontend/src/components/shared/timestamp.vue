<script setup lang="ts">
  const props = defineProps<{
    timestamp: number,
  }>();

  const date = new Date(props.timestamp * 1000);
  const formattedTime = date.getUTCMonth() + "/" + date.getUTCDate() + "/" + date.getUTCFullYear();
</script>

<template>
  <span class="timestamp">{{ date.toLocaleDateString() }} {{ date.toLocaleTimeString() }}</span>
</template>

<style>
  .timestamp {
    font-size: var(--tiny-text-size);
    color: var(--dark-color-text);
  }
</style>
