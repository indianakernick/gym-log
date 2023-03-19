<script setup lang="ts">
import { IonIcon } from '@ionic/vue';
import { chevronDown, chevronUp } from 'ionicons/icons';

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
      <IonIcon :icon="chevronUp" class="block w-6 h-6" />
    </button>

    <button
      @click="modelValue >= length - 1 || $emit('update:modelValue', modelValue + 1)"
      :disabled="modelValue >= length - 1"
      aria-label="Next"
      class="ml-3 disabled:text-neutral-600"
    >
      <IonIcon :icon="chevronDown" class="block w-6 h-6" />
    </button>
  </div>
</template>
