<script setup lang="ts">
import Header from '@/components/Header.vue';
import Main from '@/components/Main.vue';
import db from '@/services/db';
import { displayDate, toDateString } from '@/utils/date';
import { ChevronRightIcon } from '@heroicons/vue/20/solid';
import { CalendarIcon, PlusIcon } from '@heroicons/vue/24/solid';
import { shallowRef } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

const dates = shallowRef<string[]>([]);

db.getMeasurementDates().then(d => {
  dates.value = d;
});

function addToday() {
  router.push(`/measurements/${toDateString(new Date())}`);
}

function addPast(event: Event) {
  const date = (event.target as HTMLInputElement | null)?.valueAsDate;
  if (date) {
    // Safari doesn't support the max attribute. Also, it will emit the change
    // event with today's date when the picker is opened.
    const today = toDateString(new Date());
    const selected = toDateString(date);
    if (selected < today) {
      router.push(`/measurements/${toDateString(date)}`);
    }
  }
}
</script>

<template>
  <Header title="Measurements" @right="addToday">
    <template #left>
      <CalendarIcon class="w-6 h-6" />
      <!--
        Safari 15 doesn't support the showPicker method so we're putting the
        date picker on top of the button.
      -->
      <input
        class="absolute top-0 right-0 bottom-0 opacity-0"
        type="date"
        :max="toDateString(new Date())"
        @change="addPast"
      />
    </template>
    <template #right>
      <PlusIcon class="w-6 h-6" />
    </template>
  </Header>

  <Main>
    <ol class="my-2 mx-3">
      <li
        v-for="date in dates"
        class="border-t border-r last:border-b border-l first:rounded-t-lg
          last:rounded-b-lg dark:border-neutral-600 dark:bg-neutral-800"
      >
        <button
          @click="router.push(`/measurements/${date}`)"
          class="px-3 py-2 w-full flex justify-between items-center"
        >
          {{ displayDate(date) }}
          <ChevronRightIcon class="w-5 h-5 text-neutral-500" />
        </button>
      </li>
    </ol>
  </Main>
</template>
