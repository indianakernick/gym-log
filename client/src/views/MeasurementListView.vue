<script setup lang="ts">
import Header from '@/components/Header.vue';
import ListGroup from '@/components/ListGroup.vue';
import ListItem from '@/components/ListItem.vue';
import Main from '@/components/Main.vue';
import db from '@/services/db';
import { groupBy } from '@/utils/array';
import { displayDate, toDateString } from '@/utils/date';
import { CalendarIcon, PlusIcon } from '@heroicons/vue/24/solid';
import { shallowRef } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

const years = shallowRef<string[][]>([]);

db.getMeasurementDates().then(d => {
  years.value = groupBy(d, date => date.substring(0, 4));
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
    <ol>
      <li
        v-for="year in years"
        :aria-label="year[0].substring(0, 4)"
      >
        <ListGroup>
          <ListItem
            v-for="date in year"
            @click="router.push(`/measurements/${date}`)"
          >{{ displayDate(date) }}</ListItem>
        </ListGroup>
      </li>
    </ol>
  </Main>
</template>
