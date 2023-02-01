<script setup lang="ts">
import type { Exercise, Workout } from '@/model/api';
import db from '@/services/db';
import { stringCompare } from '@/utils/binary-search';
import { displayDateTime } from '@/utils/date';
import { EXERCISE_TYPE } from '@/utils/i18n';
import { ref, shallowRef } from 'vue';
import SetEdit from './SetEdit.vue';

const props = defineProps<{
  exercise: Exercise;
  readOnly?: boolean;
}>();

let history = shallowRef<(Exercise & { workout: Workout })[]>([]);
let historyIdx = ref<number>(-1);

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
</script>

<template>
  <div>
    <strong>{{ EXERCISE_TYPE[exercise.type] }}</strong>
    <div>
      <strong>History</strong>
      <br/>
      <i v-if="historyIdx === -1">You've never done this exercise before</i>
      <template v-else>
        <button @click="--historyIdx" :disabled="historyIdx < 1">Previous</button>
        <button @click="++historyIdx" :disabled="historyIdx === history.length - 1">Next</button>
        <!--
          The only way for a historic workout to not have a start_time is if
          there was a merge.
        -->
        <time v-if="history[historyIdx].workout.start_time" :d="history[historyIdx].workout.start_time">{{
          displayDateTime(history[historyIdx].workout.start_time!)
        }}</time>
        <i v-else>Not started</i>
        <SetEdit :exercise="history[historyIdx]"></SetEdit>
        <div>{{ history[historyIdx].notes }}</div>
      </template>
    </div>

    <div>
      <strong>Current</strong>

      <SetEdit :exercise="exercise" :history="readOnly ? undefined : history"></SetEdit>

      <div v-if="readOnly">{{ exercise.notes }}</div>
      <div v-else>
        <textarea v-model.lazy="exercise.notes"></textarea>
      </div>
    </div>
  </div>
</template>
