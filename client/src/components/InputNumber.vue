<script setup lang="ts">
defineProps<{
  modelValue: number;
  readOnly?: boolean;
}>();

defineEmits<{
  (e: 'update:modelValue', v: number): void;
}>();
</script>

<template>
  <input
    v-if="!readOnly"
    type="number"
    :value="modelValue"
    @change="$emit('update:modelValue', ($event.target as HTMLInputElement).valueAsNumber)"
  />
  <div v-else>{{ modelValue }}</div>
</template>
