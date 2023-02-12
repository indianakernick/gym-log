<script setup lang="ts">
import { DocumentTextIcon, UserIcon } from '@heroicons/vue/24/outline';
import { shallowRef, watchEffect } from 'vue';
import { RouterView, useRouter } from 'vue-router';

// Really struggling to find appropriate icons. Might need to choose a different
// icon library.

// We're navigating to the root of each tab rather than maintaining state for
// each tab. If you go to an individual workout, then go to the measurements
// tab, navigating to the workouts tab should take you back to that same
// individual workout. Making that work while using the browser history API is
// kind of complicated. Making it truly behave like a native mobile app by
// preserving scroll position and other state might require bypassing the Vue
// router entirely.

const router = useRouter();
const current = shallowRef<typeof tabs[number]['prefix']>();

const tabs = [
  {
    prefix: '/workouts',
    label: 'Workouts',
    icon: UserIcon
  },
  {
    prefix: '/measurements',
    label: 'Measurements',
    icon: DocumentTextIcon
  },
] as const;

watchEffect(() => {
  const route = router.currentRoute.value;

  for (const tab of tabs) {
    if (route.path.startsWith(tab.prefix)) {
      current.value = tab.prefix;
      return;
    }
  }

  current.value = undefined;
});
</script>

<template>
  <RouterView />
  <footer>
    <div
      role="tablist"
      class="p-1 border-t dark:border-neutral-500 grid grid-flow-col auto-cols-fr items-center"
    >
      <button
        v-for="tab in tabs"
        role="tab"
        :aria-selected="current === tab.prefix"
        @click="router.push(tab.prefix)"
        class="text-sm flex flex-col items-center"
        :class="{
          'text-blue-500': current === tab.prefix
        }"
      >
        <component :is="tab.icon" class="w-6 h-6"></component>
        {{ tab.label }}
      </button>
    </div>
  </footer>
</template>
