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
    <table>
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

    <button v-if="history" @click="addRepeatingSet(repeatingSets!)">Add Set</button>
  </template>

  <template v-else-if="variableSets">
    <table>
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

    <button v-if="history" @click="addVariableSet(variableSets!)">Add Set</button>
  </template>

  <template v-else-if="fixedSets">
    <table>
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

    <button v-if="history" @click="addFixedSet(fixedSets!)">Add Set</button>
  </template>
</template>
