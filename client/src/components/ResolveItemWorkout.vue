<script setup lang="ts">
import type { Workout } from '@/model/api';
import type { Deleted } from '@/model/db';
import { displayDateTime } from '@/utils/date';
import { colorForChange } from '@/utils/merge';

defineProps<{
  workout: Workout;
  otherWorkout: Workout | Deleted;
}>();
</script>

<template>
  <div>
    <div
      aria-label="Notes"
      class="whitespace-pre-wrap"
      :class="colorForChange(workout, otherWorkout, w => w.notes)"
    >{{ workout.notes }}</div>

    <div
      v-if="workout.start_time"
      class="flex justify-between"
    >
      <div>Started</div>
      <time
        :d="workout.start_time"
        :class="colorForChange(workout, otherWorkout, w => w.start_time)"
      >{{ displayDateTime(workout.start_time) }}</time>
    </div>

    <div
      v-if="workout.start_time && workout.finish_time"
      class="flex justify-between"
    >
      <div>Finished</div>
      <time
        :d="workout.finish_time"
        :class="colorForChange(workout, otherWorkout, w => w.finish_time)"
      >{{ displayDateTime(workout.finish_time) }}</time>
    </div>
  </div>
</template>
