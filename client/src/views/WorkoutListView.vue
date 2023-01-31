<script setup lang="ts">
import type { Workout } from '@/model/api';
import db from '@/services/db';
import sync from '@/services/sync';
import { displayDateTime } from '@/utils/date';
import { uuid } from '@/utils/uuid';
import { shallowRef, triggerRef } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

const workouts = shallowRef<Workout[]>([]);

db.getWorkouts().then(d => workouts.value = d);

function add() {
  router.push(`/workouts/${uuid()}`);
}

function deleteWorkout(index: number) {
  db.stageDeleteWorkout(workouts.value[index].workout_id).then(() => sync.sync());
  workouts.value.splice(index);
  triggerRef(workouts);
}
</script>

<template>
  <main>
    <h1>Workouts</h1>

    <button @click="add">Add</button>

    <ol>
      <li v-for="workout, i in workouts">
        <router-link :to="`/workouts/${workout.workout_id}`">
          <div>
            <time v-if="workout.start_time" :d="workout.start_time">{{
              displayDateTime(workout.start_time)
            }}</time>
            <i v-else>Not started</i>
            -
            <time v-if="workout.finish_time" :d="workout.finish_time">{{
              displayDateTime(workout.finish_time)
            }}</time>
            <i v-else>Not finished</i>
          </div>
          <!-- TODO: first line of notes truncated with ellipsis -->
          <div v-if="workout.notes">{{ workout.notes }}</div>
        </router-link>
        <button @click="deleteWorkout(i)">X</button>
      </li>
    </ol>
  </main>
</template>
