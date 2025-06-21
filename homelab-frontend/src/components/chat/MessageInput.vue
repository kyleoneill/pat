<script setup lang="ts">
  import type { Ref } from 'vue';
  import { ref } from 'vue';

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
    inputRows.value = Math.min(newlines + 1, 5);
  }
</script>

<template>
  <textarea v-model="messageContents" v-on:keydown="keyDown" v-on:input="resizeForNewlines" :rows="inputRows"></textarea>
</template>

<style scoped>

</style>
