<script setup lang="ts">
defineProps<{
  leftText?: string;
  rightText?: string;
  leftDisabled?: boolean;
  rightDisabled?: boolean;
  title: string;
}>();

defineEmits<{
  (e: 'left'): void;
  (e: 'right'): void;
}>();
</script>

<template>
  <header class="p-3 border-b dark:border-neutral-300 border-neutral-700 grid">
    <button
      v-if="leftText"
      class="mr-auto dark:text-blue-500 text-blue-600"
      @click="$emit('left')"
      :disabled="leftDisabled"
    >{{ leftText }}</button>
    <div v-else></div>

    <h1 class="font-bold text-lg text-center">{{ title }}</h1>

    <button
      v-if="rightText"
      class="ml-auto font-bold dark:text-blue-500 text-blue-600"
      @click="$emit('right')"
      :disabled="rightDisabled"
    >{{ rightText }}</button>
    <div v-else></div>
  </header>
</template>

<style scoped>
header {
  grid-template-columns: 1fr auto 1fr;
}
</style>
