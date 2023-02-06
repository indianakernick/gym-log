<script setup lang="ts">
import ExerciseEdit from '@/components/ExerciseEdit.vue';
import Header from '@/components/Header.vue';
import Main from '@/components/Main.vue';
import TextArea from '@/components/TextArea.vue';
import { EXERCISE_TYPE_GROUPS, type Exercise, type ExerciseType, type Workout } from '@/model/api';
import { back } from '@/router/back';
import db from '@/services/db';
import sync from '@/services/sync';
import { displayDateTime, toDateTimeString } from '@/utils/date';
import { EXERCISE_TYPE, EXERCISE_TYPE_GROUP } from '@/utils/i18n';
import { uuid } from '@/utils/uuid';
import { shallowRef, triggerRef } from 'vue';
import { useRouter } from 'vue-router';

// TODO: support creating workouts in the past.
// TODO: add an escape hatch to edit exercises after the workout has finished.

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
      workout_exercise_id: `${workout.value.workout_id}#${uuid()}`,
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
  <Header
    title="Edit Workout"
    @right="back(router, '/workouts')"
  >
    <template #right>Done</template>
  </Header>

  <Main>
    <TextArea
      v-model="workout.notes"
      label="Notes"
      :read-only="!!workout.finish_time"
      class="mx-3 my-2"
    ></TextArea>

    <div
      v-if="workout.start_time"
      class="px-3 py-2 flex justify-between"
    >
      <div>Started</div>
      <time :d="workout.start_time">{{ displayDateTime(workout.start_time) }}</time>
    </div>
    <button
      v-else
      @click="start"
      class="button-primary"
    >Start</button>

    <div v-if="workout.start_time && workout.finish_time" class="px-3 py-2 flex justify-between">
      <div>Finished</div>
      <time :d="workout.finish_time">{{ displayDateTime(workout.finish_time) }}</time>
    </div>

    <ol class="flex flex-col gap-3 my-2">
      <li v-for="exercise, i in exercises">
        <ExerciseEdit
          :exercise="exercise"
          :read-only="!!workout.finish_time || i < exercises.length - 1"
        ></ExerciseEdit>
      </li>
    </ol>

    <template v-if="workout.start_time && !workout.finish_time">
      <!--
        Safari doesn't respond to text-align: center on <select> elements so we
        need to use this god awful hack. In the future this might be a regular
        button that opens a dialog so maybe it doesn't matter.
      -->
      <div
        class="relative mx-3 my-2 rounded-lg dark:bg-neutral-800 border
        dark:border-neutral-600"
      >
        <select
          @change="addExercise"
          class="py-2 w-full appearance-none opacity-0"
        >
          <option value="" disabled selected>Add Exercise</option>
          <optgroup v-for="group, name in EXERCISE_TYPE_GROUPS" :label="EXERCISE_TYPE_GROUP[name]">
            <option v-for="ty in group" :value="ty">{{ EXERCISE_TYPE[ty] }}</option>
          </optgroup>
        </select>

        <div
          class="absolute inset-0 pointer-events-none font-bold text-center
            flex items-center justify-center dark:text-blue-500"
        >Add Exercise</div>
      </div>

      <button
        @click="finish"
        class="button-primary"
      >Finish</button>
    </template>
  </Main>
</template>
