<script setup lang="ts">
import type { Workout } from '@/model/api';
import type { Deleted } from '@/model/db';
import { displayDateTime } from '@/utils/date';
import { describeChange } from '@/utils/merge';
import ResolveColor from './ResolveColor.vue';

defineProps<{
  workout: Workout;
  otherWorkout: Workout | Deleted;
}>();
</script>

<template>
  <div>
    <ResolveColor
      :desc="describeChange(workout, otherWorkout, w => w.notes)"
      aria-label="Notes"
      class="whitespace-pre-wrap"
    >
      {{ workout.notes }}
    </ResolveColor>

    <div
      v-if="workout.start_time"
      class="flex justify-between"
    >
      <div>Started</div>
      <ResolveColor :desc="describeChange(workout, otherWorkout, w => w.start_time)">
        <time :d="workout.start_time">{{ displayDateTime(workout.start_time) }}</time>
      </ResolveColor>
    </div>

    <div
      v-if="workout.start_time && workout.finish_time"
      class="flex justify-between"
    >
      <div>Finished</div>
      <ResolveColor :desc="describeChange(workout, otherWorkout, w => w.finish_time)">
        <time :d="workout.finish_time">{{ displayDateTime(workout.finish_time) }}</time>
      </ResolveColor>
    </div>
  </div>
</template>
