<script setup lang="ts">
import { VueFinalModal } from 'vue-final-modal';

defineProps<{
  title: string;
}>();

defineEmits<{
  (e: 'done'): void;
}>()
</script>

<template>
  <VueFinalModal
    class="flex justify-center items-center"
    content-class="w-full m-6 p-3 bg-neutral-800 rounded-lg border border-neutral-600"
    :overlay-transition="{
      enterActiveClass: 'transition-opacity',
      enterFromClass: 'opacity-0',
      leaveActiveClass: 'transition-opacity',
      leaveToClass: 'opacity-0'
    }"
    :content-transition="{
      enterActiveClass: 'transition-[opacity,transform]',
      enterFromClass: 'scale-50 opacity-0',
      leaveActiveClass: 'transition-[opacity,transform]',
      leaveToClass: 'scale-50 opacity-0'
    }"
  >
    <h2>{{ title }}</h2>
    <button @click="$emit('done')">Done</button>
  </VueFinalModal>
</template>
