<script setup lang="ts">
import type { Exercise, Workout } from '@/model/api';
import db from '@/services/db';
import { binarySearch, stringCompare } from '@/utils/array';
import { displayDateTime } from '@/utils/date';
import { EXERCISE_TYPE } from '@/utils/i18n';
import { useConfirmModal } from '@/utils/modal';
import { TrashIcon } from '@heroicons/vue/20/solid';
import { ChevronDownIcon, ChevronUpIcon } from '@heroicons/vue/24/outline';
import { computed, shallowRef } from 'vue';
import Menu from './Menu.vue';
import SetEdit from './SetEdit.vue';

const props = defineProps<{
  workoutStart: string;
  exercise: Exercise;
  editingWorkout?: boolean;
  editing?: boolean;
}>();

const emit = defineEmits<{
  (e: 'deleteExercise'): void;
  (e: 'editExercise'): void;
}>();

const confirmModal = useConfirmModal({
  buttons: 'delete-cancel'
});

const history = shallowRef<(Exercise & { workout: Workout })[]>([]);
const historyIdx = shallowRef(-1);

db.getExercisesOfType(props.exercise.type).then(exercises => {
  // Removing exercises from the current workout.

  const workoutId = props.exercise.workout_exercise_id.substring(0, 36);
  const start = -binarySearch(exercises, 0, exercises.length, e => {
    return stringCompare(e.workout_exercise_id, workoutId);
  }) - 1;
  let end = start;

  while (
    end < exercises.length
    && exercises[end].workout_exercise_id.startsWith(workoutId)
  ) ++end;

  if (end > start) exercises.splice(start, end - start);

  db.joinWorkoutWithExercises(exercises).then(joinedExercises => {
    joinedExercises.sort((a, b) => {
      const time = stringCompare(a.workout.start_time, b.workout.start_time);
      if (time !== 0) return time;
      return a.order - b.order;
    });

    // Removing exercises whose workout start time is equal to or greater than
    // the start time of the current workout.

    let index = binarySearch(joinedExercises, 0, joinedExercises.length, e => {
      return stringCompare(e.workout.start_time, props.workoutStart);
    });

    if (index < 0) index = -index - 1;
    joinedExercises.splice(index, joinedExercises.length - index);

    history.value = joinedExercises;
    historyIdx.value = joinedExercises.length - 1;
  });
});

const setsKey = shallowRef(0);

const options = computed(() => {
  const items: InstanceType<typeof Menu>['items'] = [];

  if (!props.editing) {
    items.push({ title: 'Edit', handler: () => emit('editExercise') });
  }

  items.push({ title: 'Change Exercise type', handler: () => {} });

  // Recomputing if the sets change.
  setsKey.value;

  if (props.exercise.sets.length) {
    items.push({ title: 'Delete Last Set', theme: 'danger', icon: TrashIcon, handler: deleteLastSet });
  }

  items.push({ title: 'Delete Exercise', theme: 'danger', icon: TrashIcon, handler: deleteExercise });

  return items;
});

async function deleteExercise() {
  if (await confirmModal({
    title: 'Delete exercise',
    message: 'Are you sure you want to delete this exercise?'
  })) {
    emit('deleteExercise');
  }
}

async function deleteLastSet() {
  if (await confirmModal({
    title: 'Delete last set',
    message: 'Are you sure you want to delete the last set in this exercise?'
  })) {
    props.exercise.sets.pop();
    // Vue doesn't see the above mutation so we're manually triggering a
    // re-render. Maybe a little hacky but I don't see a simpler way.
    ++setsKey.value;
  }
}
</script>

<template>
  <div class="mx-3 rounded-lg bg-neutral-800 border border-neutral-600">
    <div class="p-2 border-b border-neutral-600 flex justify-between">
      <h2 class="font-bold">{{ EXERCISE_TYPE[exercise.type] }}</h2>
      <Menu
        v-if="editingWorkout"
        title="Exercise Options"
        :items="options"
      ></Menu>
    </div>

    <div v-if="historyIdx === -1" class="p-2 border-b border-neutral-600">
      <i>You've never done this exercise before</i>
    </div>

    <template v-else>
      <div class="p-2 border-b border-neutral-600 flex items-center">
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
          aria-label="Previous"
          class="ml-auto disabled:text-neutral-600"
        >
          <ChevronUpIcon class="w-6 h-6"></ChevronUpIcon>
        </button>
        <button
          @click="++historyIdx"
          :disabled="historyIdx === history.length - 1"
          aria-label="Next"
          class="ml-3 disabled:text-neutral-600"
        >
          <ChevronDownIcon class="w-6 h-6"></ChevronDownIcon>
        </button>
      </div>

      <div v-if="historyIdx !== -1" class="border-b border-neutral-600">
        <SetEdit :exercise="history[historyIdx]"></SetEdit>
      </div>
    </template>

    <SetEdit
      :exercise="exercise"
      :history="editingWorkout && editing ? history : undefined"
      :key="setsKey"
      @set-created="++setsKey"
    ></SetEdit>
  </div>
</template>
