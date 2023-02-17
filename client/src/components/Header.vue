<script setup lang="ts">
defineProps<{
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
  <header class="p-3 border-b border-neutral-500 grid items-center">
    <slot name="full-left">
      <button
        class="text-blue-500 relative"
        @click="$emit('left')"
        :disabled="leftDisabled"
      >
        <slot name="left"></slot>
      </button>
    </slot>

    <h1 class="font-bold text-lg text-center">{{ title }}</h1>

    <slot name="full-right">
      <button
        class="font-bold text-blue-500 relative"
        @click="$emit('right')"
        :disabled="rightDisabled"
      >
        <slot name="right"></slot>
      </button>
    </slot>
  </header>
</template>

<style scoped>
header {
  grid-template-columns: 1fr auto 1fr;
}

header > :first-child {
  margin-right: auto;
}

header > :last-child {
  margin-left: auto;
}
</style>
