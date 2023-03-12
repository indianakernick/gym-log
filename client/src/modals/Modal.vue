<script setup lang="ts">
import { computed } from 'vue';
import { VueFinalModal } from 'vue-final-modal';

const props = defineProps<{
  title: string;
  buttons?: {
    title: string;
    theme?: 'primary' | 'danger';
    bold?: boolean;
    disabled?: boolean;
    handler: () => void;
  }[];
  trap?: boolean;
  fullscreen?: boolean;
}>();

const contentClass = computed(() => {
  const common = 'w-full overflow-hidden flex flex-col ';
  if (props.fullscreen) {
    return common + 'h-full bg-neutral-900';
  } else {
    return common + 'bg-neutral-800 rounded-lg border border-neutral-600 max-h-[calc(100%-2*theme(spacing.6))] max-w-lg m-6';
  }
});

const contentTransition = computed(() => {
  if (props.fullscreen) {
    return {
      enterActiveClass: 'transition-transform',
      enterFromClass: 'translate-y-full',
      leaveActiveClass: 'transition-transform',
      leaveToClass: 'translate-y-full'
    };
  } else {
    return {
      enterActiveClass: 'transition-[opacity,transform]',
      enterFromClass: 'scale-50 opacity-0',
      leaveActiveClass: 'transition-[opacity,transform]',
      leaveToClass: 'scale-50 opacity-0'
    };
  }
});
</script>

<template>
  <VueFinalModal
    class="flex justify-center items-center"
    :content-class="contentClass"
    :overlay-transition="{
      enterActiveClass: 'transition-opacity',
      enterFromClass: 'opacity-0',
      leaveActiveClass: 'transition-opacity',
      leaveToClass: 'opacity-0'
    }"
    :content-transition="contentTransition"
    :click-to-close="!trap"
    :esc-to-close="!trap"
  >
    <h2 class="text-lg font-bold px-3 py-2 border-b border-neutral-600">{{ title }}</h2>

    <div class="overflow-auto" :aria-label="title">
      <slot></slot>
    </div>

    <div v-if="buttons" class="grid grid-flow-col auto-cols-fr gap-px
      pt-px bg-neutral-600"
    >
      <button
        v-for="button in buttons"
        @click="button.handler"
        :disabled="button.disabled"
        :class="{
          'font-bold': button.bold,
          'text-blue-500': !button.theme || button.theme === 'primary',
          'text-red-500': button.theme === 'danger',
        }"
        class="p-2 bg-neutral-800 disabled:text-neutral-500
          active:bg-neutral-700"
      >{{ button.title }}</button>
    </div>
  </VueFinalModal>
</template>
