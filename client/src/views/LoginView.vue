<script setup lang="ts">
import cognito from '@/services/cognito';
import db from '@/services/db';
import { getCognitoErrorMessage } from '@/utils/error';
import { shallowRef } from 'vue';
import { useRouter, type RouteLocationRaw } from 'vue-router';

const props = defineProps<{
  redirect?: RouteLocationRaw;
}>();

const router = useRouter();

let email = '';
let password = '';
let loading = shallowRef(false);
let error = shallowRef<string>();

async function login() {
  if (loading.value) return;
  loading.value = true;
  error.value = undefined;

  try {
    const result = await cognito.login(email, password);
    if (result.RefreshToken) {
      await db.setRefreshToken(result.RefreshToken);
      await router.replace(props.redirect || '/');
    } else {
      // ?
    }
  } catch (e) {
    error.value = getCognitoErrorMessage(e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <main class="flex-center">
    <form
      @submit.prevent="login"
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
        autocomplete="current-password"
        required
        class="form-input"
      />

      <button
        :disabled="loading"
        class="form-submit mt-5"
      >Login</button>

      <p class="text-center mt-3">
        Don't have an account?
        <router-link to="/signup" class="link">Sign up</router-link>
      </p>

      <p v-if="error && !loading" class="form-error mt-3">{{ error }}</p>
    </form>
  </main>
</template>
