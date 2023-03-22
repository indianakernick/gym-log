<script setup lang="ts">
import type { ChangeDesc } from '@/utils/merge';

defineProps<{
  desc: ChangeDesc;
}>();
</script>

<template>
  <div
    :class="{
      'text-green-600 dark:text-green-400': desc === 'added',
      'text-orange-600 dark:text-orange-400': desc === 'modified',
    }"
  >
    <slot></slot>
  </div>
</template>
