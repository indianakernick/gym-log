<script setup lang="ts">
import type { Workout } from '@/model/api';
import { displayDateTime } from '@/utils/date';

defineProps<{
  workout: Workout;
}>();
</script>

<template>
  <div>
    <div aria-label="Notes" class="whitespace-pre-wrap">{{ workout.notes }}</div>

    <div
      v-if="workout.start_time"
      class="flex justify-between"
    >
      <div>Started</div>
      <time :d="workout.start_time">{{ displayDateTime(workout.start_time) }}</time>
    </div>

    <div
      v-if="workout.start_time && workout.finish_time"
      class="flex justify-between"
    >
      <div>Finished</div>
      <time :d="workout.finish_time">{{ displayDateTime(workout.finish_time) }}</time>
    </div>
  </div>
</template>
