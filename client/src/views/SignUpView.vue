<script setup lang="ts">
import cognito from '@/services/cognito';
import { getCognitoErrorMessage } from '@/utils/error';
import { shallowRef } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

let email = '';
let password = '';
let loading = shallowRef(false);
let error = shallowRef<string>();

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

  await router.push({ path: '/confirm-signup', query: { email } });
}
</script>

<template>
  <main class="flex-center">
    <form
      @submit.prevent="signUp"
      class="grow flex flex-col form-card"
    >
      <label for="email" class="form-label">Email</label>
      <input
        type="email"
        id="email"
        v-model.lazy="email"
        :disabled="loading"
        autocomplete="email"
        required
        class="form-input"
      />

      <label for="password" class="form-label mt-4">Password</label>
      <input
        type="password"
        id="password"
        v-model.lazy="password"
        :disabled="loading"
        autocomplete="new-password"
        required
        class="form-input"
      />

      <button
        :disabled="loading"
        class="form-submit mt-5"
      >Sign Up</button>

      <p class="text-center mt-3">
        Already have an account?
        <router-link to="/login" class="link">Login</router-link>
      </p>

      <p v-if="error && !loading" class="form-error mt-3">{{ error }}</p>
    </form>
  </main>
</template>
