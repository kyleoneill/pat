<script setup lang="ts">
  import { ref } from 'vue';

  import { globalState } from '@/stores/store';

  import { updateUser } from '@/api/user_api';

  import useToasterStore from '@/stores/useToasterStore';
  const toasterStore = useToasterStore();

  const loading = ref(false);
  const newUsername = ref('');
  const newPassword = ref('');
  const confirmPassword = ref('');

  function updateAccount() {
    if (newPassword.value !== confirmPassword.value) {
      toasterStore.error({text: "Entered passwords do not match"});
      return
    }

    loading.value = true;
    updateUser(newUsername.value, newPassword.value)
      .then(updateUserResponse => {
        globalState.setCurrentUser(updateUserResponse.data);
        toasterStore.success({text: "Successfully updated account"})
      }).catch(error => {
        if (error.status === 400) {
          toasterStore.error({text: error.response.data.msg});
        }
        else {
          toasterStore.error({text: "Failed to update account"});
        }
      }).finally(() => {
        loading.value = false;
        newUsername.value = '';
        newPassword.value = '';
        confirmPassword.value = '';
      });
  }
</script>

<template>
  <main>
    <h1>Account Settings</h1>
    <h3>Username: {{ globalState.currentUser?.username }}</h3>
    <div class="account-settings">
      <div>
        <p>New Username:</p>
        <input v-model="newUsername"></input>
      </div>
      <div>
        <p>New password:</p>
        <input v-model="newPassword" type="password"></input>
        <p>Confirm password:</p>
        <input v-model="confirmPassword" type="password"></input>
      </div>
      <button :disabled="loading === true" @click="updateAccount">Update Account</button>
    </div>
  </main>
</template>

<style scoped>
  .account-settings {
    margin-bottom: 1rem;
  }
  .account-settings > button {
    margin-top: 1.5rem;
  }
  .account-settings > div {
    margin-top: 1.5rem;
  }
</style>
