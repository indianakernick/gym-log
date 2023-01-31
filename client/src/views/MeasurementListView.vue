<script setup lang="ts">
import db from '@/services/db';
import sync from '@/services/sync';
import { displayDate, toDateString } from '@/utils/date';
import { shallowRef, triggerRef } from 'vue';
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
    router.push(`/measurements/${toDateString(date)}`);
  }
}

function deleteSet(index: number) {
  db.stageDeleteMeasurement(dates.value[index]).then(() => sync.sync());
  dates.value.splice(index, 1);
  triggerRef(dates);
}
</script>

<template>
  <main>
    <h1>Measurements</h1>

    <button @click="addToday">Add Today</button>
    <input type="date" :max="toDateString(new Date())" @change="addPast" />

    <ol>
      <li v-for="date, i in dates">
        <router-link :to="`/measurements/${date}`">{{ displayDate(date) }}</router-link>
        <button @click="deleteSet(i)">X</button>
      </li>
    </ol>
  </main>
</template>
