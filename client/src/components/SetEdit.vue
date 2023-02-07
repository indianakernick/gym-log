<script setup lang="ts">
import {
  getFixedSets,
  getRepeatingSets,
  getVariableSets,
  type Exercise,
  type FixedSet,
  type RepeatingSet,
  type VariableSet,
  type Workout
} from '@/model/api';
import { uuid } from '@/utils/uuid';
import { ref, watchEffect } from 'vue';
import TextArea from './TextArea.vue';

const props = defineProps<{
  exercise: Exercise;
  history?: (Exercise & { workout: Workout })[];
}>();

let repeatingSets = ref<RepeatingSet[]>();
let variableSets = ref<VariableSet[]>();
let fixedSets = ref<FixedSet[]>();

watchEffect(() => {
  repeatingSets.value = getRepeatingSets(props.exercise);
  variableSets.value = getVariableSets(props.exercise);
  fixedSets.value = getFixedSets(props.exercise);
});

function addSet<T>(sets: T[], set: T) {
  // The existence of props.history indicates whether we can add sets so it must
  // exist if we're here.
  if (props.history!.length) {
    const previous = props.history![props.history!.length - 1].sets as T[];
    if (sets.length < previous.length) {
      sets.push({ ...previous[sets.length] });
      return;
    }
  }
  if (sets.length > 0) {
    sets.push({ ...sets[sets.length - 1] });
    return;
  }
  sets.push(set);
}

function addRepeatingSet(sets: RepeatingSet[]) {
  addSet(sets, {
    set_id: uuid(),
    repetitions: 0,
    resistance: 1
  });
}

function addVariableSet(sets: VariableSet[]) {
  addSet(sets, {
    set_id: uuid(),
    resistance: 1,
    distance: 0,
    duration: 0
  });
}

function addFixedSet(sets: FixedSet[]) {
  addSet(sets, {
    set_id: uuid(),
    resistance: 0,
    speed: 1,
    distance: 0,
    duration: 0
  });
}
</script>

<template>
  <template v-if="repeatingSets">
    <div class="px-2 flex flex-col">
      <TextArea
        v-model="exercise.notes"
        label="Notes"
        :read-only="!history"
        class="my-2 w-full"
      ></TextArea>

      <table class="table">
        <thead><tr>
          <th>Reps</th>
          <th>Weight (kg)</th>
        </tr></thead>
        <tbody>
          <tr v-for="set in repeatingSets">
            <td><input type="number" v-model.lazy="set.repetitions" :readonly="!history" /></td>
            <td><input type="number" v-model.lazy="set.resistance" :readonly="!history" /></td>
          </tr>
        </tbody>
      </table>
    </div>

    <button v-if="history" @click="addRepeatingSet(repeatingSets!)" class="set-button">Add Set</button>
  </template>

  <template v-else-if="variableSets">
    <div class="px-2">
      <TextArea
        v-model="exercise.notes"
        label="Notes"
        :read-only="!history"
        class="my-2 w-full"
      ></TextArea>

      <table class="table">
        <thead><tr>
          <th>Resistance</th>
          <th>Distance (km)</th>
          <th>Duration</th>
        </tr></thead>
        <tbody>
          <tr v-for="set in variableSets">
            <td><input type="number" v-model.lazy="set.resistance" :readonly="!history" /></td>
            <td><input type="number" v-model.lazy="set.distance" :readonly="!history" /></td>
            <td><input type="number" v-model.lazy="set.duration" :readonly="!history" /></td>
          </tr>
        </tbody>
      </table>
    </div>

    <button v-if="history" @click="addVariableSet(variableSets!)" class="set-button">Add Set</button>
  </template>

  <template v-else-if="fixedSets">
    <div class="px-2">
      <TextArea
        v-model="exercise.notes"
        label="Notes"
        :read-only="!history"
        class="my-2 w-full"
      ></TextArea>

      <table class="table">
        <thead><tr>
          <th>Resistance</th>
          <th>Speed (km/h)</th>
          <th>Distance (km)</th>
          <th>Duration</th>
        </tr></thead>

        <tbody>
          <tr v-for="set in fixedSets">
            <td><input type="number" v-model.lazy="set.resistance" :readonly="!history" /></td>
            <td><input type="number" v-model.lazy="set.speed" :readonly="!history" /></td>
            <td><input type="number" v-model.lazy="set.distance" :readonly="!history" /></td>
            <td><input type="number" v-model.lazy="set.duration" :readonly="!history" /></td>
          </tr>
        </tbody>
      </table>
    </div>

    <button v-if="history" @click="addFixedSet(fixedSets!)" class="set-button">Add Set</button>
  </template>
</template>

<style lang="postcss">
.table {
  @apply table-fixed w-full mb-2;
}

.table:has(tbody:empty) {
  @apply hidden;
}

.table td, .table th {
  @apply pt-0 pr-1 pl-1 first:pl-0 last:pr-0;
}

.table th {
  @apply pb-1 text-center text-sm;
}

.table td {
  @apply pb-2;
}

.table tr:last-child td {
  @apply pb-0;
}

.table input {
  @apply max-w-full px-2 py-1 text-right rounded-lg dark:bg-neutral-700
    dark:focus-visible:outline-blue-500;
}

.set-button {
  @apply p-2 w-full font-bold dark:text-blue-500 border-t dark:border-neutral-600;
}
</style>
