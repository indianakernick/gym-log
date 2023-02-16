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
  <main class="flex items-center justify-center">
    <div class="grow flex flex-col max-w-md p-4 m-4 dark:bg-neutral-800
      rounded-lg border dark:border-neutral-700"
    >
      <label for="email" class="text-sm">Email</label>
      <input
        type="email"
        id="email"
        v-model.lazy="email"
        :disabled="loading"
        class="mt-1 max-w-full p-2 rounded-lg dark:bg-neutral-700
          dark:focus-visible:outline-blue-500"
      />

      <label for="password" class="text-sm mt-4">Password</label>
      <input
        type="password"
        id="password"
        v-model.lazy="password"
        :disabled="loading"
        class="mt-1 max-w-full p-2 rounded-lg dark:bg-neutral-700
          dark:focus-visible:outline-blue-500"
      />

      <button
        @click="login"
        :disabled="loading"
        class="mt-5 py-2 rounded-lg font-bold dark:bg-blue-700 border
          dark:border-blue-500"
      >Login</button>

      <p v-if="error && !loading" class="mt-2 text-center text-red-500">{{ error }}</p>

      <p class="text-center mt-2">
        Don't have an account?
        <router-link
          to="/signup"
          class="text-blue-500 hover:underline"
        >Sign up</router-link>
      </p>
    </div>
  </main>
</template>
