<script setup lang="ts">
import db from '@/services/db';
import { displayDate, today } from '@/utils/date';
import { ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

let dates = ref<string[]>([]);

db.getMeasurementDates().then(d => {
  dates.value = d;
});

function add() {
  router.push(`/measurement/${today()}`);
}

</script>

<template>
  <main>
    <h1>Measurements</h1>

    <button @click="add">Add</button>

    <ol>
      <li v-for="date in dates">
        <router-link :to="`/measurement/${date}`">{{ displayDate(date) }}</router-link>
      </li>
    </ol>
  </main>
</template>
