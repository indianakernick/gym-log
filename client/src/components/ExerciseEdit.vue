<script setup lang="ts">
import {
  getFixedSets,
  getRepeatingSets,
  getVariableSets,
  type Exercise,
  type FixedSet,
  type RepeatingSet,
  type VariableSet
} from '@/model/api';
import db from '@/services/db';
import { EXERCISE_TYPE } from '@/utils/i18n';
import { uuid } from '@/utils/uuid';
import { ref, watchEffect } from 'vue';

const props = defineProps<{
  exercise: Exercise
}>();

let history = ref<Exercise[]>([]);
let historyIdx = ref<number>(-1);

db.getExercisesOfType(props.exercise.type).then(d => {
  // TODO: sort this
  // use the `order` field to order exercises within the same workout
  // otherwise use the `start_time` of their parent workout
  // maybe exclude any exercises within the same workout
  history.value = d;
  historyIdx.value = d.length - 1;
});

let repeatingSets = ref<RepeatingSet[]>();
let variableSets = ref<VariableSet[]>();
let fixedSets = ref<FixedSet[]>();

watchEffect(() => {
  repeatingSets.value = getRepeatingSets(props.exercise);
  variableSets.value = getVariableSets(props.exercise);
  fixedSets.value = getFixedSets(props.exercise);
});

// TODO: Use values from previous sets and previous exercises to populate new
// sets sane defaults

function addRepeatingSet(sets: RepeatingSet[]) {
  sets.push({
    set_id: uuid(),
    repetitions: 0,
    resistance: 1
  });
}

function addVariableSet(sets: VariableSet[]) {
  sets.push({
    set_id: uuid(),
    resistance: 1,
    distance: 0,
    duration: 0
  });
}

function addFixedSet(sets: FixedSet[]) {
  sets.push({
    set_id: uuid(),
    resistance: 1,
    speed: 1,
    distance: 0,
    duration: 0
  });
}
</script>

<template>
  <div>
    <strong>{{ EXERCISE_TYPE[exercise.type] }}</strong>
    <div>
      <strong>History</strong>
      <i v-if="historyIdx === -1">You've never done this exercise before</i>
      <template v-else>
        <button @click="--historyIdx" :disabled="historyIdx < 1">Previous</button>
        <button @click="++historyIdx" :disabled="historyIdx === history.length - 1">Next</button>
        <!-- Also show the start date of the workout -->
      </template>
    </div>

    <div>
      <strong>Current</strong>

      <template v-if="repeatingSets">
        <table>
          <thead><tr>
            <th>Reps</th>
            <th>Weight (kg)</th>
          </tr></thead>
          <tbody>
            <tr v-for="set in repeatingSets">
              <td><input type="number" v-model.lazy="set.repetitions"/></td>
              <td><input type="number" v-model.lazy="set.resistance"/></td>
            </tr>
          </tbody>
        </table>

        <button @click="addRepeatingSet(repeatingSets!)">Add Set</button>
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
              <td><input type="number" v-model.lazy="set.resistance"/></td>
              <td><input type="number" v-model.lazy="set.distance"/></td>
              <td><input type="number" v-model.lazy="set.duration"/></td>
            </tr>
          </tbody>
        </table>

        <button @click="addVariableSet(variableSets!)">Add Set</button>
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
              <td><input type="number" v-model.lazy="set.resistance"/></td>
              <td><input type="number" v-model.lazy="set.speed"/></td>
              <td><input type="number" v-model.lazy="set.distance"/></td>
              <td><input type="number" v-model.lazy="set.duration"/></td>
            </tr>
          </tbody>
        </table>

        <button @click="addFixedSet(fixedSets!)">Add Set</button>
      </template>

      <div>
        <textarea v-model.lazy="exercise.notes"></textarea>
      </div>
    </div>
  </div>
</template>
