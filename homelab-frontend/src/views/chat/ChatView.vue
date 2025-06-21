<script setup lang="ts">
  import type { Ref } from 'vue';

  import type { ChatChannel, WebSocketSendPacket } from '@/models/chat_interfaces';

  import { RouterLink } from 'vue-router';
  import { ref } from 'vue';

  import { globalState } from '@/stores/store';

  import { connectChat, listChatChannels } from '@/api/chat_api';

  import useToasterStore from '@/stores/useToasterStore';

  import MessageInput from '@/components/chat/MessageInput.vue';

  const toasterStore = useToasterStore();

  const loading = ref(false);

  const chatChannels: Ref<Array<ChatChannel>> = ref([]);
  const selectedChannel: Ref<ChatChannel | null> = ref(null);
  const selectedChannelName: Ref<String | null> = ref(null);

  function selectChatChannel(channel: ChatChannel) {
    selectedChannel.value = channel;
    if (channel.name === null) {
      selectedChannelName.value = channel.slug;
    }
    else {
      selectedChannelName.value = channel.name as String;
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
    connectChat(globalState.token).then(socket => {
      globalState.setWebsocketConnection(socket);
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

  // TODO:
  //  - establish a ws connection on page load
  //  - load messages for a chat on chat select
  //  - send/receive messages in the chat section

  getChatChannels();
  establishWebsocketConnection();
</script>

<template>
  <div class="chat-container">
    <div class="channel-list">
      <div>
        <a>Create Channel</a>
      </div>
      <hr />
      <div v-for="(channel, index) in chatChannels" :key="index" class="channel-listing">
        <a @click="selectChatChannel(channel)">
          <span v-if="channel.name !== null">{{ channel.name }}</span>
          <span v-else>{{ channel.slug }}</span>
        </a>
      </div>
    </div>
    <div>
      <div class="chat-area" v-if="selectedChannel !== null">
        <h2>{{ selectedChannelName }}</h2>
        <div class="messages-area">
          <div>foo</div>
          <div>bar</div>
        </div>
        <MessageInput class="message-input" @send-message="sendMessage"/>
      </div>
    </div>
  </div>
</template>

<style scoped>
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
  }

  .message-input {
    width: 50vw;
    display: block;
    margin-left: auto;
    margin-right: auto;
  }
</style>
