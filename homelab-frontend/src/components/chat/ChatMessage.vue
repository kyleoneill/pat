<script setup lang="ts">
  import type { ChatMessage } from '@/models/chat_interfaces';

  defineProps<{
    chatMessage: ChatMessage,
    authorUsername: String,
    sent: boolean,
  }>();
</script>

<template>
  <div class="chat-message" :class="{'sent': sent, 'received': !sent}">
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
</style>
