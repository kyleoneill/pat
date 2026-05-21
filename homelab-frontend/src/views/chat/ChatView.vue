<script setup lang="ts">
  import { onMounted, type Ref } from 'vue';
  import type { ChatChannel } from '@/models/chat_interfaces';
  import { WebsocketRequestType, type WebSocketRequest } from '@/models/chat_interfaces';

  import { nextTick, ref, useTemplateRef, watch } from 'vue';
  import { globalState } from '@/stores/store';
  import { connectChat, listChatChannels, getChatChannel } from '@/api/chat_api';
  import useToasterStore from '@/stores/useToasterStore';
  import ChatMessage from '@/components/chat/ChatMessage.vue';
  import MessageInput from '@/components/chat/MessageInput.vue';
  import Timestamp from '@/components/shared/timestamp.vue';

  const DEFAULT_MESSAGES_TO_LOAD: number = 50;

  const toasterStore = useToasterStore();

  const loading = ref(false);

  const chatChannels: Ref<Array<ChatChannel>> = ref([]);
  const selectedChannel: Ref<ChatChannel | null> = ref(null);
  const selectedChannelName: Ref<string> = ref("");
  let selectedChannelUserMap: Map<String, String> = new Map();

  // let observer = null;
  // const detectTopOfScrollArea = useTemplateRef('scrollable-area-top');
  const scrollableArea = useTemplateRef('scrollable-area');

  function scrollToBottom() {
    if (scrollableArea.value) {
      // This is in a nextTick because some callers change the DOM and we need to wait for a re-render before moving the scroll position
      nextTick(() => {
        if (scrollableArea.value !== null) {
          scrollableArea.value.lastElementChild?.scrollIntoView();
        }

        // TODO: This isn't really working for a number of reasons. It fires when the area is initially loaded, and it sends multiple requests to the server (as the event fires repeatedly)
        //       causing the server to process the same request multiple times. This increases load for no reason and causes the client to get duplicate messages
        // if (detectTopOfScrollArea.value !== null) {
        //   observer = new IntersectionObserver(loadMessagesFromScroll, {
        //     threshold: 1.0,
        //   });
        //   observer.observe(detectTopOfScrollArea.value)
        // }
      })
    }
  }

  function loadMessagesFromScroll() {
    if (loading.value === false && selectedChannel.value !== null) {
      loading.value = true;
      
      let messages = globalState.chatMessages.get(selectedChannel.value._id);
      if (messages !== undefined && messages?.length > 0) {
        let most_recent_message = messages[0];
        if (most_recent_message.atomic_id > 1) {
          // TODO: This should probably be a function
          const requestChatState: WebSocketRequest = {
            "type": WebsocketRequestType.GetChatState,
            "data": {message_count: DEFAULT_MESSAGES_TO_LOAD, atomic_message_id: most_recent_message.atomic_id - 1, channel_id: selectedChannel.value._id}
          };
          globalState.websocketConnection?.send(JSON.stringify(requestChatState));
        }
      }

      // This isn't really true, but not sure how else to set it at the "right moment"?
      loading.value = false;
    }
  }

  async function selectChatChannel(channelId: String) {
    if (selectedChannel.value !== null && selectedChannel.value._id === channelId) {
      return;
    }

    // Load the selected channel so we can check its most recent message ID
    await loadChatChannel(channelId);
    // This shouldn't be null here unless the server does something unexpected
    if (selectedChannel.value == null) {
      return
    }
    let currentChannel: ChatChannel = selectedChannel.value;

    selectedChannelUserMap = new Map();
    currentChannel.subscribers.forEach(subscriber => {
      selectedChannelUserMap.set(subscriber.id, subscriber.username);
    });

    if (currentChannel.name === null) {
      selectedChannelName.value = currentChannel.slug as string;
    }
    else {
      selectedChannelName.value = currentChannel.name as string;
    }

    // If we have a websocket connection, load messages for the newly selected channel
    if (globalState.websocketConnection !== null) {
      // Verify that we don't already have the data we are about to query for
      if (globalState.chatMessages.has(currentChannel._id)) {
        let most_recent_message = globalState.chatMessages.get(currentChannel._id)?.at(-1);
        if (most_recent_message.atomic_id === currentChannel.most_recent_message_id) {
          scrollToBottom();
          return;
        }
      }

      // TODO: Should update this packet to take a -1 or some other impossible ID as the message id to just get the
      //       most recent message, in case the channels most recent id field is out of date
      const requestChatState: WebSocketRequest = {
        "type": WebsocketRequestType.GetChatState,
        "data": {message_count: DEFAULT_MESSAGES_TO_LOAD, atomic_message_id: currentChannel.most_recent_message_id, channel_id: currentChannel._id}
      };
      globalState.websocketConnection?.send(JSON.stringify(requestChatState));
      scrollToBottom();
    }
  }

  async function requestMessages() {
    if (selectedChannel.value !== null) {
      let messages = globalState.chatMessages.get(selectedChannel.value._id);
      if (messages !== undefined && messages.length > 0) {
        let most_recent_message = messages[0];
        if (most_recent_message.atomic_id > 1) {
          const requestChatState: WebSocketRequest = {
            "type": WebsocketRequestType.GetChatState,
            "data": {message_count: DEFAULT_MESSAGES_TO_LOAD, atomic_message_id: most_recent_message.atomic_id - 1, channel_id: selectedChannel.value._id}
          };
          globalState.websocketConnection?.send(JSON.stringify(requestChatState));
        }
      }
    }
  }

  async function loadChatChannel(channelId: String) {
    loading.value = true;
    try {
      let response = await getChatChannel(channelId);
      selectedChannel.value = response.data;
    }
    catch (error) {
      toasterStore.responseError({error: error});
    }
    finally {
      loading.value = false;
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
        <a @click="selectChatChannel(channel._id)">
          <span> {{ channel.name || channel.slug }}</span>
        </a>
      </div>
    </div>
    <div>
      <div class="chat-area" :key="selectedChannelName" v-if="selectedChannel !== null">
        <h2>{{ selectedChannelName }}</h2>
        <div ref="scrollable-area" class="messages-area">
          <!-- <div ref="scrollable-area-top"></div> -->
          <button :disabled="loading === true" @click="requestMessages">Load more messages</button>
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

  .chat-area button {
    margin-top: 10px;
    margin-bottom: 10px;
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
