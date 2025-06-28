<script setup lang="ts">
  import type { Ref } from 'vue';
  import type { ChatChannel } from '@/models/chat_interfaces';

  import { nextTick, ref, useTemplateRef, watch } from 'vue';
  import { globalState } from '@/stores/store';
  import { connectChat, listChatChannels } from '@/api/chat_api';
  import useToasterStore from '@/stores/useToasterStore';
  import ChatMessage from '@/components/chat/ChatMessage.vue';
  import MessageInput from '@/components/chat/MessageInput.vue';

  const toasterStore = useToasterStore();

  const loading = ref(false);

  const chatChannels: Ref<Array<ChatChannel>> = ref([]);
  const selectedChannel: Ref<ChatChannel | null> = ref(null);
  let selectedChannelName: String | null = null;
  let selectedChannelUserMap: Map<String, String> = new Map();

  const scrollableArea = useTemplateRef('scrollable-area');

  function selectChatChannel(channel: ChatChannel) {
    selectedChannel.value = channel;
    selectedChannelUserMap = new Map();
    channel.subscribers.forEach(subscriber => {
      selectedChannelUserMap.set(subscriber.id, subscriber.username);
    });
    if (channel.name === null) {
      selectedChannelName = channel.slug;
    }
    else {
      selectedChannelName = channel.name as String;
    }
    // TODO: LOAD THE CHAT - SEND WEBSOCKET ReceiveChatUpdateRequest PACKET
  }

  function getChatChannels() {
    loading.value = true;
    listChatChannels({subscribed: true}).then(response => {
      chatChannels.value = response.data;
    }).catch(error => {
      toasterStore.responseError({error: error});
    }).finally(() => {
      loading.value = false;
    })
  }

  function establishWebsocketConnection() {
    loading.value = true;
    connectChat(globalState.token).then(() => {
      // This no longer returns anything?
    }).catch(_error => {
      toasterStore.responseError({error: "Failed to establish WebSocket connection to server"});
    }).finally(() => {
      loading.value = false;
    })
  }

  function sendMessage(message: String) {
    if (globalState.websocketConnection !== null && globalState.websocketConnection.readyState === WebSocket.OPEN) {
      const sendMessagePacket = {
        "type": "CreateMessage",
        "data": {channel_id: selectedChannel.value?._id, contents: message, reply_to: null }
      };
      globalState.websocketConnection.send(JSON.stringify(sendMessagePacket));
    } else {
      toasterStore.responseError({error: "Tried to send a message but the server connection has been closed"});
    }
  }

  // Is this the most optimal way to do this?
  // Watch global state, when it changes we assume that we have a new message
  watch(globalState, (_newState, _oldState) => {
    // Wait a tick to ensure the DOM has been updated with the new message
    nextTick(() => {
      if (scrollableArea.value) {
        // Scroll to the bottom of the chat area, so the new message is visible
        scrollableArea.value.scrollTop = scrollableArea.value?.scrollHeight;
      }
    });
  });

  // TODO:
  //  - load messages for a chat on chat select
  //  - send/receive messages in the chat section
  // HANDLE STORING USER ID AND THEN DISCRIMINATING MESSAGES ON IF THEY ARE AUTHORED BY CURRENT USER

  getChatChannels();
  establishWebsocketConnection();
</script>

<template>
  <div class="chat-container">
    <div v-if="globalState.websocketConnection === null">DEBUG: WS CONNECTION IS CLOSED</div>
    <div class="channel-list">
      <div>
        <a>Create Channel</a>
      </div>
      <hr />
      <div v-for="(channel, index) in chatChannels" :key="index" class="channel-listing">
        <a @click="selectChatChannel(channel)">
          <span> {{ channel.name || channel.slug }}</span>
        </a>
      </div>
    </div>
    <div>
      <div class="chat-area" v-if="selectedChannel !== null">
        <h2>{{ selectedChannelName }}</h2>
        <div ref="scrollable-area" class="messages-area">
          <div v-for="(message, index) in globalState.chatMessages.get(selectedChannel._id)" :key="index">
            <div v-if="index === 0 || (globalState.chatMessages.has(selectedChannel._id) && globalState.chatMessages.get(selectedChannel._id)[index - 1].author_id !== message.author_id)">
              {{ selectedChannelUserMap.get(message.author_id) || message.author_id }}
            </div>
            <chat-message
              :chatMessage="message"
              :authorUsername="selectedChannelUserMap.get(message.author_id) || message.author_id"
              :sent="globalState.currentUser?.id === message.author_id"
            />
          </div>
        </div>
        <MessageInput class="message-input" @send-message="sendMessage"/>
      </div>
    </div>
  </div>
</template>

<style>
  .chat-container {
    display: flex;
  }

  .channel-list {
    max-width: 15vw;
    margin-right: 25px;
  }

  .channel-list > hr {
    margin-top: 8px;
    margin-bottom: 8px;
  }

  .channel-listing {
    margin-bottom: 5px;
  }

  textarea {
    resize: none;
  }

  .chat-area {
    height: 94vh;
    min-width: 60vw;
    display: flex;
    flex-direction: column;
  }

  .messages-area {
    flex-grow: 1;
    overflow: auto;
    flex-direction: column-reverse;
  }

  .message-input {
    width: 50vw;
    display: block;
    margin-left: auto;
    margin-right: auto;
  }
</style>
