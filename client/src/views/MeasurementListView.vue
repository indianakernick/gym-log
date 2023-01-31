<script setup lang="ts">
import db from '@/services/db';
import { displayDate, toDateString } from '@/utils/date';
import { shallowRef } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

let dates = shallowRef<string[]>([]);

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
</script>

<template>
  <main>
    <h1>Measurements</h1>

    <button @click="addToday">Add Today</button>
    <input type="date" :max="toDateString(new Date())" @change="addPast" />

    <ol>
      <li v-for="date in dates">
        <router-link :to="`/measurements/${date}`">{{ displayDate(date) }}</router-link>
      </li>
    </ol>
  </main>
</template>
