<script setup lang="ts">
import { getIdGenerator } from '@/utils/id';
import { EllipsisHorizontalIcon } from '@heroicons/vue/24/outline';
import { ref } from 'vue';
import Backdrop from './Backdrop.vue';

defineProps<{
  title: string;
  items: {
    title: string;
    handler: () => void;
    theme?: 'default' | 'primary' | 'danger';
  }[];
}>();

const id = getIdGenerator('menu')();
const expanded = ref(false);
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
      class="relative z-10"
    >
      <EllipsisHorizontalIcon class="w-6 h-6" />
    </button>

    <ul
      role="menu"
      :id="id"
      :aria-label="title"
      class="absolute top-full pt-2 right-0 z-10 transition-[opacity,transform]
        origin-top-right"
      :class="{
        'pointer-events-none opacity-0 scale-50': !expanded
      }"
    >
      <li
        v-for="item in items"
        class="border-t border-r last:border-b border-l first:rounded-t-lg
          last:rounded-b-lg dark:border-neutral-600 dark:bg-neutral-800
          shadow-lg"
      >
        <button
          role="menuitem"
          @click="expanded = false; item.handler()"
          class="p-2 whitespace-nowrap font-bold"
          :class="{
            'text-blue-500': item.theme === 'primary',
            'text-red-500': item.theme === 'danger'
          }"
        >{{ item.title }}</button>
      </li>
    </ul>
  </div>
</template>
