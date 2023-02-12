<script setup lang="ts">
import { getIdGenerator } from '@/utils/id';
import { EllipsisHorizontalIcon } from '@heroicons/vue/24/outline';
import { shallowRef, type FunctionalComponent } from 'vue';
import Backdrop from './Backdrop.vue';

defineProps<{
  title: string;
  items: {
    title: string;
    handler: () => void;
    theme?: 'default' | 'primary' | 'danger';
    icon?: FunctionalComponent;
  }[];
  theme?: 'default' | 'primary';
}>();

const id = getIdGenerator('menu')();
const expanded = shallowRef(false);
</script>

<template>
  <div class="relative h-6">
    <Backdrop :show="expanded" @click="expanded = false" />

    <button
      aria-haspopup="menu"
      :title="title"
      :aria-controls="id"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
      :class="{
        'dark:text-blue-500 text-blue-600': theme === 'primary'
      }"
    >
      <EllipsisHorizontalIcon class="w-6 h-6" />
    </button>

    <ul
      role="menu"
      :id="id"
      :aria-label="title"
      class="absolute top-full pt-2 right-0 z-20 transition-[opacity,transform]
        origin-top-right"
      :class="{
        'pointer-events-none opacity-0 scale-50': !expanded
      }"
    >
      <li
        v-for="item in items"
        role="presentation"
        class="border-t border-r last:border-b border-l first:rounded-t-lg
          last:rounded-b-lg dark:border-neutral-600 dark:bg-neutral-800
          shadow-lg overflow-hidden"
      >
        <!--
          An empty ontouchstart listener is required to make :active work on iOS
          Safari. This isn't necessary for the modal for some reason.
          https://stackoverflow.com/a/33681490/4093378
        -->
        <button
          role="menuitem"
          @click="expanded = false; item.handler()"
          @touchstart=""
          class="w-full p-2 whitespace-nowrap font-bold flex gap-1 items-center
            dark:active:bg-neutral-700"
          :class="{
            'dark:text-blue-500 text-blue-600': item.theme === 'primary',
            'text-red-500': item.theme === 'danger',
          }"
        >
          <component
            v-if="item.icon"
            :is="item.icon"
            class="w-5 h-5"
          ></component>
          {{ item.title }}
        </button>
      </li>
    </ul>
  </div>
</template>
