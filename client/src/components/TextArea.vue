<script setup lang="ts">
import { nextTick, shallowRef, watch } from 'vue';

const props = defineProps<{
  modelValue: string;
  label: string;
  readOnly?: boolean;
}>();

defineEmits<{
  (e: 'update:modelValue', v: string): void;
}>();

const area = shallowRef<HTMLTextAreaElement>();

function setHeight(area: HTMLTextAreaElement) {
  area.style.height = '';
  area.style.height = area.scrollHeight + 'px';
}

watch(
  () => [props.modelValue, props.readOnly],
  () => {
    nextTick(() => {
      if (area.value) setHeight(area.value);
    });
  }
);
</script>

<template>
  <textarea
    v-if="!readOnly"
    :aria-label="label"
    :placeholder="label"
    ref="area"
    :value="modelValue"
    @input="setHeight($event.target as HTMLTextAreaElement)"
    @change="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    class="px-2 py-1 resize-none rounded-lg bg-neutral-200 dark:bg-neutral-700
      placeholder-neutral-400 focus:outline-none"
  ></textarea>

  <div
    v-else-if="modelValue"
    :aria-label="label"
    class="whitespace-pre-wrap"
  >{{ modelValue }}</div>
</template>
