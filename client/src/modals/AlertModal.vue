<script setup lang="ts">
import Modal from './Modal.vue';

defineProps<{
  title: string;
  message: string;
}>();

defineEmits<{
  (e: 'ok'): void;
}>();
</script>

<template>
  <Modal
    :title="title"
    :buttons="[
      {
        title: 'OK',
        handler: () => $emit('ok')
      }
    ]"
  >
    <div class="p-3">{{ message }}</div>
  </Modal>
</template>
