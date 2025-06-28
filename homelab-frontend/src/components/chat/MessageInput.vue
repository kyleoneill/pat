<script setup lang="ts">
  import type { Ref } from 'vue';
  import { ref } from 'vue';

  const MAX_ROWS_TO_RENDER = 10;

  const emit = defineEmits(['send-message']);

  const messageContents: Ref<string> = ref("");

  const inputRows: Ref<number> = ref(1);

  function keyDown(keyDownEvent: KeyboardEvent) {
    if (keyDownEvent.key === "Enter" && !keyDownEvent.shiftKey) {
      if (messageContents.value.trim().length > 0) {
        emit('send-message', messageContents.value);
        messageContents.value = "";
        resizeForNewlines();
        keyDownEvent.preventDefault();
      }
    }
  }

  function resizeForNewlines() {
    const newlines = messageContents.value.match(/\n/g)?.length || 0;
    inputRows.value = Math.min(newlines + 1, MAX_ROWS_TO_RENDER);
  }
</script>

<template>
  <textarea
    v-model="messageContents"
    v-on:keydown="keyDown"
    v-on:input="resizeForNewlines"
    :rows="inputRows"
    class="text-input"
  ></textarea>
</template>

<style scoped>
  .text-input {
    font-family:
            Inter,
            -apple-system,
            BlinkMacSystemFont,
            'Segoe UI',
            Roboto,
            Oxygen,
            Ubuntu,
            Cantarell,
            'Fira Sans',
            'Droid Sans',
            'Helvetica Neue',
            sans-serif;
    font-size: medium;
  }
</style>
