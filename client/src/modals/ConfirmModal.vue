<script setup lang="ts">
import { computed } from 'vue';
import Modal from './Modal.vue';

const props = defineProps<{
  title: string;
  message: string;
  buttons: 'delete-cancel' | 'keep-discard';
}>();

const emit = defineEmits<{
  (e: 'confirm', confirmed: boolean): void;
}>();

const confirmFalse = () => emit('confirm', false);
const confirmTrue = () => emit('confirm', true);

const buttons = computed<InstanceType<typeof Modal>['buttons']>(() => {
  switch (props.buttons) {
    case 'delete-cancel':
      return [
        {
          title: 'Cancel',
          handler: confirmFalse
        },
        {
          title: 'Delete',
          bold: true,
          theme: 'danger',
          handler: confirmTrue
        }
      ];
    case 'keep-discard':
      return [
        {
          title: 'Discard',
          handler: confirmFalse
        },
        {
          title: 'Keep',
          bold: true,
          handler: confirmTrue
        }
      ];
  }
});
</script>

<template>
  <Modal
    :title="title"
    :buttons="buttons"
  >{{ message }}</Modal>
</template>
