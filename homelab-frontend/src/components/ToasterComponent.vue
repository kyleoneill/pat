<script setup lang="ts">
import type { TToastStatus } from "@/stores/useToasterStore";

import useToasterStore from "@/stores/useToasterStore";

const toastClassMap: Record<TToastStatus, string> = {
  warning: "warning",
  error: "error",
  success: "success",
};

const toastStore = useToasterStore();
</script>

<template>
  <Teleport to="body">
    <ul v-if="toastStore.toasts.length" class="toaster-wrapper">
      <li v-for="toast in toastStore.toasts" :class="['toaster-inner', toastClassMap[toast.status]]" :key="toast.id">
        <img class="toaster-inner-icon" v-if="toast.status == 'success'" src="../assets/icons/success.svg" alt="check-icon" />
        <img class="toaster-inner-icon" v-if="toast.status == 'error'" src="../assets/icons/error.svg" alt="error-icon" />
        <img class="toaster-inner-icon" v-if="toast.status == 'warning'" src="../assets/icons/warning.svg" alt="warning-icon" />
        <span class="toaster-inner-text">{{ toast.text }}</span>
        <img class="toaster-inner-icon" @click="toastStore.deleteToast(toast.id)" src="../assets/icons/close.svg" alt="close-icon" />
      </li>
    </ul>
  </Teleport>
</template>

<style scoped>
.toaster-wrapper {
  position: fixed;
  top: 3%;
  right: 5%;

  z-index: 100;

  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.toaster-inner {
  --color: black;
  display: flex;
  align-items: center;
  gap: 1rem;

  border-radius: 0.5rem;

  border: 1px solid transparent;

  background-color: white;

  padding: 1rem 1.2rem;

  border-color: var(--color);
  color: black;
}

.toaster-inner img {
  fill: var(--color);
  stroke: var(--color);
}

.success {
  --color: green;
  background-color: #EFFFED;
}

.warning {
  --color: orange;
  background-color: #FFF4DE;
}

.error {
  --color: red;
  background-color: #FAE8E3;
}

.toaster-inner-icon {
  width: 1.8rem;
  aspect-ratio: 1/1;
}

.toaster-inner-text {
  font-size: 1.5rem;
  font-weight: 450;
}
</style>
