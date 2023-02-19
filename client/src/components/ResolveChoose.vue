<script setup lang="ts">
defineProps<{
  id: string;
  type: 'local' | 'remote';
}>();

defineEmits<{
  (e: 'choose'): void;
}>();

const LABEL = {
  local: 'Local',
  remote: 'Remote',
};
</script>

<template>
  <div class="flex justify-between items-center">
    <label :for="type" class="font-bold">{{ LABEL[type] }}</label>
    <input
      type="radio"
      id="remote"
      :name="id"
      @input="($event.target as HTMLInputElement).checked && $emit('choose')"
      class="w-5 h-5 accent-blue-500"
    />
  </div>
</template>
