<script setup lang="ts">
  import type { Ref } from 'vue';
  import type { ChatChannel } from '@/models/chat_interfaces';
  import { WebsocketRequestType, type WebSocketRequest } from '@/models/chat_interfaces';

  import { nextTick, ref, useTemplateRef, watch } from 'vue';
  import { globalState } from '@/stores/store';
  import { connectChat, listChatChannels } from '@/api/chat_api';
  import useToasterStore from '@/stores/useToasterStore';
  import ChatMessage from '@/components/chat/ChatMessage.vue';
  import MessageInput from '@/components/chat/MessageInput.vue';
  import Timestamp from '@/components/shared/timestamp.vue';

  const toasterStore = useToasterStore();

  const loading = ref(false);

  const chatChannels: Ref<Array<ChatChannel>> = ref([]);
  const selectedChannel: Ref<ChatChannel | null> = ref(null);
  const selectedChannelName: Ref<string> = ref("");
  let selectedChannelUserMap: Map<String, String> = new Map();

  const scrollableArea = useTemplateRef('scrollable-area');

  function scrollToBottom() {
    if (scrollableArea.value) {
      scrollableArea.value.scrollTop = scrollableArea.value?.scrollHeight;
    }
  }

  function selectChatChannel(channel: ChatChannel) {
    //  TODO: All of my frontend logic for loading messages and channels sucks and should be tossed and re-done in a more
    //        comprehensive and readable fashion

    if (selectedChannel.value !== null && selectedChannel.value._id === channel._id) {
      return;
    }
    selectedChannel.value = channel;
    selectedChannelUserMap = new Map();
    channel.subscribers.forEach(subscriber => {
      selectedChannelUserMap.set(subscriber.id, subscriber.username);
    });

    if (channel.name === null) {
      selectedChannelName.value = channel.slug as string;
    }
    else {
      selectedChannelName.value = channel.name as string;
    }

    // If we have a websocket connection, load messages for the newly selected channel
    if (globalState.websocketConnection !== null) {
      const channelData: ChatChannel = selectedChannel.value as ChatChannel;

      // Verify that we don't already have the data we are about to query for
      if (globalState.chatMessages.has(channelData._id)) {
        let most_recent_message = globalState.chatMessages.get(channelData._id)?.at(-1);
        if (most_recent_message.atomic_id === channelData.most_recent_message_id) {
          // TODO: This doesn't work as this is called before the area re-renders
          scrollToBottom();
          return;
        }
      }

      // TODO: Should update this packet to take a -1 or some other impossible ID as the message id to just get the
      //       most recent message, in case the channels most recent id field is out of date
      const requestChatState: WebSocketRequest = {
        "type": WebsocketRequestType.GetChatState,
        "data": {message_count: 25, atomic_message_id: channelData.most_recent_message_id, channel_id: channelData._id}
      };
      globalState.websocketConnection?.send(JSON.stringify(requestChatState));
      // TODO: This doesn't work as this is called before the area re-renders
      scrollToBottom();
    }
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
      const channelData: ChatChannel = selectedChannel.value as ChatChannel;
      const sendMessagePacket: WebSocketRequest = {
        "type": WebsocketRequestType.CreateMessage,
        "data": {channel_id: channelData._id, contents: message, reply_to: null }
      };
      globalState.websocketConnection.send(JSON.stringify(sendMessagePacket));
    } else {
      toasterStore.responseError({error: "Tried to send a message but the server connection has been closed"});
    }
  }

  // Is this the most optimal way to do this?
  // Watch global state, when it changes we assume that we have a new message
  watch(globalState, (newState, _oldState) => {
    // Wait a tick to ensure the DOM has been updated with the new message
    nextTick(() => {
      scrollToBottom();

      // If we have a new message, change the most recent message for our chat channel. Do this locally
      // rather than re-poll the server for something we can do locally
      if (selectedChannel.value !== null) {
        let most_recent_message = globalState.chatMessages.get(selectedChannel.value._id)?.at(-1);
        if (most_recent_message !== undefined) {
          selectedChannel.value.most_recent_message_id = most_recent_message.atomic_id;
        }
      }
    });
  });

  getChatChannels();
  establishWebsocketConnection();
</script>

<template>
  <div class="chat-container">
    <!-- TODO: Replace this with something that actually handles the ws connection closing -->
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
      <div class="chat-area" :key="selectedChannelName" v-if="selectedChannel !== null">
        <h2>{{ selectedChannelName }}</h2>
        <div ref="scrollable-area" class="messages-area">
          <div v-for="(message, index) in globalState.chatMessages.get(selectedChannel._id)" :class="{'sent': globalState.currentUser?.id === message.author_id, 'received': globalState.currentUser?.id !== message.author_id}" :key="index">
            <div v-if="index === 0 || (globalState.chatMessages.has(selectedChannel._id) && globalState.chatMessages.get(selectedChannel._id)[index - 1].author_id !== message.author_id)">
              <span>{{ selectedChannelUserMap.get(message.author_id) || message.author_id }}</span>
              <span class="message-datetime">
                <timestamp :timestamp="message.updated_at"/>
              </span>
            </div>
            <chat-message
              :chatMessage="message"
              :authorUsername="selectedChannelUserMap.get(message.author_id) || message.author_id"
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
    margin-right: 30px;
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

  .message-datetime {
    padding-left: 0.5vw;
  }

  .message-input {
    width: 50vw;
    display: block;
    margin-left: auto;
    margin-right: auto;
  }
</style>
