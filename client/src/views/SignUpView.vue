<script setup lang="ts">
import cognito from '@/services/cognito';
import db from '@/services/db';
import { getCognitoErrorMessage } from '@/utils/error';
import { ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

let email = '';
let password = '';
let confirmationCode = '';
let confirming = ref(false);
let loading = ref(false);
let error = ref<string | undefined>(undefined);

async function signUp() {
  if (loading.value) return;
  loading.value = true;
  error.value = undefined;

  try {
    await cognito.signUp(email, password);
  } catch (e) {
    error.value = getCognitoErrorMessage(e);
    return;
  } finally {
    loading.value = false;
  }

  confirming.value = true;
}

async function confirm() {
  if (loading.value) return;
  loading.value = true;
  error.value = undefined;

  try {
    await cognito.confirmSignUp(email, confirmationCode);
  } catch (e) {
    error.value = getCognitoErrorMessage(e);
    loading.value = false;
    return;
  }

  // Should we redirect the user to the login page or just log them in?
  try {
    const result = await cognito.login(email, password);
    if (result.RefreshToken) {
      await db.setRefreshToken(result.RefreshToken);
      await router.replace('/');
    } else {
      // ?
    }
  } catch (e) {
    error.value = getCognitoErrorMessage(e);
  } finally {
    loading.value = false;
  }
}

function retry() {
  confirming.value = false;
  confirmationCode = '';
}
</script>

<template>
  <main>
    <template v-if="!confirming">
      <label for="email">Email:</label>
      <input type="email" id="email" v-model.lazy="email" :disabled="loading" />
      <br/>
      <label for="password">Password:</label>
      <input type="password" id="password" v-model.lazy="password" :disabled="loading" />
      <br/>
      <button @click="signUp" :disabled="loading">Sign Up</button>
    </template>

    <template v-else>
      <label for="code">Confirmation code:</label>
      <input type="text" id="code" v-model.lazy="confirmationCode" :disabled="loading" />
      <br/>
      <button @click="confirm" :disabled="loading">Confirm</button>
      <template v-if="error">
        <button @click="retry">Retry</button>
      </template>
    </template>

    <p v-if="error && !loading">Error: {{ error }}</p>
  </main>
</template>
