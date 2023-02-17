<script setup lang="ts">
import { ChevronRightIcon } from '@heroicons/vue/20/solid';

defineEmits<{
  (e: 'click'): void;
}>();
</script>

<template>
  <li
    class="border-t border-r last:border-b border-l first:rounded-t-lg
      last:rounded-b-lg border-neutral-600 bg-neutral-800"
  >
    <button
      @click="$emit('click')"
      class="px-3 py-2 w-full flex justify-between items-center text-left"
    >
      <slot></slot>
      <ChevronRightIcon class="w-5 h-5 shrink-0 text-neutral-500" />
    </button>
  </li>
</template>
