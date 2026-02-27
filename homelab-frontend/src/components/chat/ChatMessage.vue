<script setup lang="ts">
  import type { ChatMessage } from '@/models/chat_interfaces';

  defineProps<{
    chatMessage: ChatMessage,
    authorUsername: String,
  }>();
</script>

<template>
  <div class="chat-message">
    <div class="chat-contents">{{ chatMessage.contents }}</div>
  </div>
</template>

<style>
  .chat-message {
    position: relative;
    max-width: 75%;
    padding: 5px 15px;
    margin-bottom: 2px;
  }

  .chat-message:hover {
    background-color: var(--vt-c-black-soft);
  }
</style>
