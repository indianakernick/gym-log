<script setup lang="ts">
import cognito from '@/services/cognito';
import { getCognitoErrorMessage } from '@/utils/error';
import { shallowRef } from 'vue';
import { useRouter } from 'vue-router';

const props = defineProps<{
  email?: string;
}>();

let confirmationCode = '';
let loading = shallowRef(false);
let error = shallowRef<string>();

const router = useRouter();

if (!props.email) {
  // The user has manually edited the URL.
  router.replace('/signup');
}

async function confirm() {
  if (loading.value) return;
  loading.value = true;
  error.value = undefined;

  try {
    await cognito.confirmSignUp(props.email!, confirmationCode);
  } catch (e) {
    error.value = getCognitoErrorMessage(e);
    return;
  } finally {
    loading.value = false;
  }

  await router.replace('/login');
}

async function resend() {
  if (loading.value) return;
  loading.value = true;
  error.value = undefined;

  try {
    await cognito.resendConfirmSignUp(props.email!);
  } catch (e) {
    error.value = getCognitoErrorMessage(e);
    return;
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <main class="flex-center">
    <form
      @submit.prevent="confirm"
      class="grow flex flex-col form-card"
    >
      <p class="mb-4">
        A code was sent to your email. Enter it here to verify ownership of
        the email address.
      </p>

      <label for="code" class="form-label">Confirmation Code</label>
      <input
        type="text"
        id="code"
        v-model.lazy="confirmationCode"
        :disabled="loading"
        autocomplete="one-time-code"
        required
        class="form-input"
      />

      <button
        :disabled="loading"
        class="form-submit mt-5"
      >Confirm</button>

      <p class="text-center mt-3">
        Didn't receive a code?
        <button
          type="button"
          @click="resend"
          class="link"
        >Resend</button>
      </p>

      <p v-if="error && !loading" class="form-error mt-3">{{ error }}</p>
    </form>
  </main>
</template>
