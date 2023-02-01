<script setup lang="ts">
import ExerciseEdit from '@/components/ExerciseEdit.vue'
import { type Workout, type Exercise, EXERCISE_TYPE_GROUPS, type ExerciseType } from '@/model/api';
import { back } from '@/router/back';
import db from '@/services/db';
import sync from '@/services/sync';
import { displayDateTime, toDateTimeString } from '@/utils/date';
import { EXERCISE_TYPE, EXERCISE_TYPE_GROUP } from '@/utils/i18n';
import { uuid } from '@/utils/uuid';
import { shallowRef, triggerRef } from 'vue';
import { useRouter } from 'vue-router';

const props = defineProps<{
  id: string;
}>();

const router = useRouter();

const workout = shallowRef<Workout>({
  workout_id: props.id,
  notes: '',
  start_time: null,
  finish_time: null
});
const exercises = shallowRef<Exercise[]>([]);
const deletedExercises: Exercise['workout_exercise_id'][] = [];

Promise.all([
  db.getWorkout(props.id),
  db.getExercisesOfWorkout(props.id)
]).then(([w, e]) => {
  if (w) workout.value = w;
  exercises.value = e;
});

async function save() {
  await db.stageUpdateWorkout(workout.value);
  await Promise.all(exercises.value.map(e => db.stageUpdateExercise(e)));
  await Promise.all(deletedExercises.map(e => db.stageDeleteExercise(e)));
  sync.sync();
  back(router, `/workouts`);
}

function start() {
  workout.value.start_time = toDateTimeString(new Date());
  triggerRef(workout);
}

function finish() {
  workout.value.finish_time = toDateTimeString(new Date());
  triggerRef(workout);
}

function addExercise(event: Event) {
  const select = event.target as HTMLSelectElement | null;
  if (select?.value) {
    const type = select.value as ExerciseType;
    select.value = '';
    exercises.value.push({
      workout_exercise_id: `${workout.value}#${uuid()}`,
      order: exercises.value.length,
      type,
      notes: '',
      sets: []
    });
    triggerRef(exercises);
  }
}
</script>

<template>
  <main>
    <h1>Edit Workout</h1>
    <button @click="back(router, `/workouts`)">Cancel</button>
    <button
      @click="save"
      :disabled="!exercises.length"
    >Save</button>

    <div>
      <label for="notes">Notes:</label>
      <textarea id="notes" v-model.lazy="workout.notes"></textarea>
    </div>

    <time v-if="workout.start_time" :d="workout.start_time">{{
      displayDateTime(workout.start_time)
    }}</time>
    <button v-else @click="start">Start</button>

    <ol>
      <li v-for="exercise in exercises">
        <ExerciseEdit :exercise="exercise"></ExerciseEdit>
      </li>
    </ol>

    <template v-if="workout.start_time && !workout.finish_time">
      <select @change="addExercise">
        <option value="" disabled selected>Add exercise</option>
        <optgroup v-for="group, name in EXERCISE_TYPE_GROUPS" :label="EXERCISE_TYPE_GROUP[name]">
          <option v-for="ty in group" :value="ty">{{ EXERCISE_TYPE[ty] }}</option>
        </optgroup>
      </select>
      <br/>
      <button @click="finish">Finish</button>
    </template>
  </main>
</template>
