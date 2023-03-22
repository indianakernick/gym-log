<script setup lang="ts">
import type { Exercise } from '@/model/api';
import type { Deleted } from '@/model/db';
import { describeChange, type ChangeDesc } from '@/utils/merge';
import ResolveColor from './ResolveColor.vue';
import SetTable from './SetTable.vue';

const props = defineProps<{
  exercise: Exercise;
  otherExercise: Exercise | Deleted;
}>();

function describeSetFieldChange(index: number, field: string): ChangeDesc {
  if ('deleted' in props.otherExercise) return 'equal';
  if (props.exercise.type !== props.otherExercise.type) {
    return describeChange(props.exercise, props.otherExercise, e => e);
  }
  // We've proved that the two exercises have the same type. We have to do this
  // dodgy cast to get it to compile. I don't think it's even possible to
  // express this relationship in the type system and remove the cast. It would
  // be very complicated if it was possible! Also, it doesn't seem like set_id
  // is necessary.
  return describeChange(
    props.exercise,
    props.otherExercise,
    e => e.sets?.[index]?.[field as 'set_id']
  );
}
</script>

<template>
  <!-- TODO: Perhaps display some context (the workout). -->

  <div>
    <ResolveColor
      v-if="exercise.notes"
      :desc="describeChange(exercise, otherExercise, e => e.notes)"
      aria-label="Notes"
      class="whitespace-pre-wrap mb-2"
    >
      {{ exercise.notes }}
    </ResolveColor>

    <SetTable
      :exercise="exercise"
      v-slot="{ set, index, field }"
    >
      <ResolveColor :desc="describeSetFieldChange(index, field)">
        {{ set[field] }}
      </ResolveColor>
    </SetTable>
  </div>
</template>
