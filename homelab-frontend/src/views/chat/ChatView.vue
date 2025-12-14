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
  let selectedChannelName: String | null = null;
  let selectedChannelUserMap: Map<String, String> = new Map();

  const scrollableArea = useTemplateRef('scrollable-area');

  function selectChatChannel(channel: ChatChannel) {
    if (selectedChannel.value !== null && selectedChannel.value._id === channel._id) {
      return;
    }
    selectedChannel.value = channel;
    selectedChannelUserMap = new Map();
    channel.subscribers.forEach(subscriber => {
      selectedChannelUserMap.set(subscriber.id, subscriber.username);
    });

    // If we have a websocket connection, load messages for the newly selected channel
    if (globalState.websocketConnection !== null) {
      const channelData: ChatChannel = selectedChannel.value as ChatChannel;
      // TODO: Might want to re-pull the channel here, as the "most recent message" could be out of date (we do not know
      //       how long ago the user got this channel)
      //       Could also send updates to the client when a channel changes, in the same way that messages are sent

      // Verify that we don't already have the data we are about to query for
      if (globalState.chatMessages.has(channelData._id)) {
        for (const message of globalState.chatMessages.get(channelData._id)) {
          if (message.atomic_id === channelData.most_recent_message_id) {
            return;
          }
        }
      }

      // TODO: Should update this packet to take a -1 or some other impossible ID as the message id to just get the
      //       most recent message, in case the channels most recent id field is out of date
      const requestChatState: WebSocketRequest = {
        "type": WebsocketRequestType.GetChatState,
        "data": {message_count: 25, atomic_message_id: channelData.most_recent_message_id, channel_id: channelData._id}
      };
      globalState.websocketConnection?.send(JSON.stringify(requestChatState));
    }
    if (channel.name === null) {
      selectedChannelName = channel.slug;
    }
    else {
      selectedChannelName = channel.name as String;
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
  watch(globalState, (_newState, _oldState) => {
    // Wait a tick to ensure the DOM has been updated with the new message
    nextTick(() => {
      if (scrollableArea.value) {
        // Scroll to the bottom of the chat area, so the new message is visible
        scrollableArea.value.scrollTop = scrollableArea.value?.scrollHeight;
      }
    });
  });

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
              <span>{{ selectedChannelUserMap.get(message.author_id) || message.author_id }}</span>
              <span class="message-datetime">
                <timestamp :timestamp="message.updated_at"/>
              </span>
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
