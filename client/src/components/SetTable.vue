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

function add() {
  if (repeatingSets.value) {
    addRepeatingSet(repeatingSets.value);
  } else if (variableSets.value) {
    addVariableSet(variableSets.value);
  } else if (fixedSets.value) {
    addFixedSet(fixedSets.value);
  }
}

defineExpose({
  add,
});

const repeatingSets = ref<RepeatingSet[]>();
const variableSets = ref<VariableSet[]>();
const fixedSets = ref<FixedSet[]>();

watchEffect(() => {
  repeatingSets.value = getRepeatingSets(props.exercise);
  variableSets.value = getVariableSets(props.exercise);
  fixedSets.value = getFixedSets(props.exercise);
});

function addSet<T>(sets: T[], set: T) {
  // The existence of props.history indicates whether we can add sets so it must
  // exist if we're here. But that assumption is based on the implementation of
  // the parent component...
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
  <table class="table-fixed w-full">
    <template v-if="repeatingSets">
      <thead><tr>
        <th>Reps</th>
        <th>Weight (kg)</th>
      </tr></thead>
      <tbody>
        <tr v-for="set, index in repeatingSets">
          <td>
            <slot :set="set" :index="index" :field="('repetitions' as const)"></slot>
          </td>
          <td>
            <slot :set="set" :index="index" :field="('resistance' as const)"></slot>
          </td>
        </tr>
      </tbody>
    </template>

    <template v-if="variableSets">
      <thead><tr>
        <th>Resistance</th>
        <th>Distance (km)</th>
        <th>Duration</th>
      </tr></thead>
      <tbody>
        <tr v-for="set, index in variableSets">
          <td>
            <slot :set="set" :index="index" :field="('resistance' as const)"></slot>
          </td>
          <td>
            <slot :set="set" :index="index" :field="('distance' as const)"></slot>
          </td>
          <td>
            <slot :set="set" :index="index" :field="('duration' as const)"></slot>
          </td>
        </tr>
      </tbody>
    </template>

    <template v-if="fixedSets">
      <thead><tr>
        <th>Resistance</th>
        <th>Speed (km/h)</th>
        <th>Distance (km)</th>
        <th>Duration</th>
      </tr></thead>
      <tbody>
        <tr v-for="set, index in fixedSets">
          <td>
            <slot :set="set" :index="index" :field="('resistance' as const)"></slot>
          </td>
          <td>
            <slot :set="set" :index="index" :field="('speed' as const)"></slot>
          </td>
          <td>
            <slot :set="set" :index="index" :field="('distance' as const)"></slot>
          </td>
          <td>
            <slot :set="set" :index="index" :field="('duration' as const)"></slot>
          </td>
        </tr>
      </tbody>
    </template>
  </table>
</template>

<style scoped lang="postcss">
table td, table th {
  @apply text-center pt-0 pr-1 pl-1 first:pl-0 last:pr-0;
}

table th {
  @apply pb-1 text-sm;
}

table td {
  @apply pb-2;
}

table tr:last-child td {
  @apply pb-0;
}
</style>
