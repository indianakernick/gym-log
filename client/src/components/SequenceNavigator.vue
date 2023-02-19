<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon } from '@heroicons/vue/24/outline';

defineProps<{
  modelValue: number;
  length: number;
}>();

defineEmits<{
  (e: 'update:modelValue', value: number): void;
}>();
</script>

<template>
  <div class="flex items-center">
    <div><slot></slot></div>

    <button
      @click="modelValue < 1 || $emit('update:modelValue', modelValue - 1)"
      :disabled="modelValue < 1"
      aria-label="Previous"
      class="ml-auto disabled:text-neutral-600"
    >
      <ChevronUpIcon class="w-6 h-6"></ChevronUpIcon>
    </button>

    <button
      @click="modelValue >= length - 1 || $emit('update:modelValue', modelValue + 1)"
      :disabled="modelValue >= length - 1"
      aria-label="Next"
      class="ml-3 disabled:text-neutral-600"
    >
      <ChevronDownIcon class="w-6 h-6"></ChevronDownIcon>
    </button>
  </div>
</template>
