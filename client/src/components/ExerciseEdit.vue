<script setup lang="ts">
import type { Exercise, Workout } from '@/model/api';
import db from '@/services/db';
import { stringCompare } from '@/utils/array';
import { displayDateTime } from '@/utils/date';
import { EXERCISE_TYPE } from '@/utils/i18n';
import { TrashIcon } from '@heroicons/vue/20/solid';
import { ChevronDownIcon, ChevronUpIcon } from '@heroicons/vue/24/outline';
import { computed, ref, shallowRef } from 'vue';
import Menu from './Menu.vue';
import SetEdit from './SetEdit.vue';

const props = defineProps<{
  exercise: Exercise;
  readOnly?: boolean;
}>();

const history = shallowRef<(Exercise & { workout: Workout })[]>([]);
const historyIdx = ref<number>(-1);

db.getExercisesOfType(props.exercise.type).then(d => {
  // TODO: this can be done in O(log n) because it's ordered by ID
  // also consider whether it makes sense to do this.
  // should we remove this exercise or all exercises in the current workout?
  const workoutId = props.exercise.workout_exercise_id.substring(0, 36);
  for (let i = 0; i < d.length; ++i) {
    if (d[i].workout_exercise_id.startsWith(workoutId)) {
      d.splice(i, 1);
      --i;
    }
  }

  // TODO: remove workouts later than the current workout from the history.
  db.joinWorkoutWithExercises(d).then(d => {
    d.sort((a, b) => {
      const time = stringCompare(a.workout.start_time || '', b.workout.start_time || '');
      if (time !== 0) return time;
      return a.order - b.order;
    });
    history.value = d;
    historyIdx.value = d.length - 1;
  });
});

const setsKey = ref<number>(0);

const options = computed(() => {
  const items: InstanceType<typeof Menu>['items'] = [
    { title: 'Change Exercise type', handler: () => {} },
    { title: 'Delete Exercise', theme: 'danger', icon: TrashIcon, handler: () => {} },
    { title: 'Delete Last Set', theme: 'danger', icon: TrashIcon, handler: deleteLastSet },
  ];
  // Recomputing if the sets change.
  setsKey.value;
  if (!props.exercise.sets.length) items.pop();
  return items;
});

function deleteLastSet() {
  props.exercise.sets.pop();
  // Vue doesn't see the above mutation so we're manually triggering a
  // re-render. Maybe a little hacky but I don't see a simpler way.
  ++setsKey.value;
}
</script>

<template>
  <div class="mx-3 rounded-lg dark:bg-neutral-800 border dark:border-neutral-600">
    <div class="p-2 border-b dark:border-neutral-600 flex justify-between">
      <h2 class="font-bold">{{ EXERCISE_TYPE[exercise.type] }}</h2>
      <Menu title="Exercise Options" :items="options"></Menu>
    </div>

    <div v-if="historyIdx === -1" class="p-2 border-b dark:border-neutral-600">
      <i>You've never done this exercise before</i>
    </div>

    <template v-else>
      <div class="p-2 border-b dark:border-neutral-600 flex items-center">
        <!--
          The only way for a historic workout to not have a start_time is if
          there was a merge.
        -->
        <time v-if="history[historyIdx].workout.start_time" :d="history[historyIdx].workout.start_time">{{
          displayDateTime(history[historyIdx].workout.start_time!)
        }}</time>
        <i v-else>Not started</i>

        <button
          @click="--historyIdx"
          :disabled="historyIdx < 1"
          class="ml-auto disabled:text-neutral-600"
        >
          <ChevronUpIcon class="w-6 h-6"></ChevronUpIcon>
        </button>
        <button
          @click="++historyIdx"
          :disabled="historyIdx === history.length - 1"
          class="ml-3 disabled:text-neutral-600"
        >
          <ChevronDownIcon class="w-6 h-6"></ChevronDownIcon>
        </button>
      </div>

      <div v-if="historyIdx !== -1" class="border-b dark:border-neutral-600">
        <SetEdit :exercise="history[historyIdx]"></SetEdit>
      </div>
    </template>

    <div>
      <SetEdit
        :exercise="exercise"
        :history="readOnly ? undefined : history"
        :key="setsKey"
        @set-created="++setsKey"
      ></SetEdit>
    </div>
  </div>
</template>
