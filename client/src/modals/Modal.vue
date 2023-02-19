<script setup lang="ts">
import { VueFinalModal } from 'vue-final-modal';

defineProps<{
  title: string;
  buttons?: {
    title: string;
    theme?: 'primary' | 'danger';
    bold?: boolean;
    disabled?: boolean;
    handler: () => void;
  }[];
  background?: InstanceType<typeof VueFinalModal>['background'];
}>();
</script>

<template>
  <VueFinalModal
    class="flex justify-center items-center"
    content-class="w-full m-6 bg-neutral-800 rounded-lg overflow-hidden border
      border-neutral-600 max-h-[calc(100%-2*theme(spacing.6))] max-w-lg flex
      flex-col"
    :overlay-transition="{
      enterActiveClass: 'transition-opacity',
      enterFromClass: 'opacity-0',
      leaveActiveClass: 'transition-opacity',
      leaveToClass: 'opacity-0'
    }"
    :content-transition="{
      enterActiveClass: 'transition-[opacity,transform]',
      enterFromClass: 'scale-50 opacity-0',
      leaveActiveClass: 'transition-[opacity,transform]',
      leaveToClass: 'scale-50 opacity-0'
    }"
    :background="background"
  >
    <h2 class="text-lg font-bold px-3 pt-2">{{ title }}</h2>

    <div class="overflow-scroll">
      <div class="p-3 flex flex-col gap-3">
        <slot></slot>
      </div>
    </div>

    <div v-if="buttons" class="grid grid-flow-col auto-cols-fr gap-[1px]
      pt-[1px] bg-neutral-600"
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
