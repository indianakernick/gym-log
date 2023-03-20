<script setup lang="ts">
import type { Exercise } from '@/model/api';
import type { Deleted } from '@/model/db';
import { colorForChange } from '@/utils/merge';
import SetTable from './SetTable.vue';

const props = defineProps<{
  exercise: Exercise;
  otherExercise: Exercise | Deleted;
}>();

function composeClass(index: number, field: string): { [ key in string]: boolean } {
  if ('deleted' in props.otherExercise) return {};
  if (props.exercise.type !== props.otherExercise.type) {
    return colorForChange(props.exercise, props.otherExercise, e => e);
  }
  // We've proved that the two exercises have the same type. We have to do this
  // dodgy cast to get it to compile. I don't think it's even possible to
  // express this relationship in the type system and remove the cast. It would
  // be very complicated if it was possible! Also, it doesn't seem like set_id
  // is necessary.
  return colorForChange(
    props.exercise,
    props.otherExercise,
    e => e.sets?.[index]?.[field as 'set_id']
  );
}
</script>

<template>
  <!-- TODO: Perhaps display some context (the workout). -->

  <div>
    <div
      v-if="exercise.notes"
      aria-label="Notes"
      class="whitespace-pre-wrap mb-2"
      :class="colorForChange(exercise, otherExercise, e => e.notes)"
    >{{ exercise.notes }}</div>

    <!--
      TODO: maybe find another way of implementing colorForChange
      We have to repeat the classes here so that the Tailwind JIT can find them.
    -->
    <SetTable
      :exercise="exercise"
      v-slot="{ set, index, field }"
      :class="{
        'text-green-600 dark:text-green-400': false,
        'text-orange-600 dark:text-orange-400': false
      }"
    >
      <div :class="composeClass(index, field)">{{ set[field] }}</div>
    </SetTable>
  </div>
</template>
