<script setup lang="ts">
import { MEASUREMENT_TYPES, type MeasurementSet } from '@/model/api';
import { displayDate } from '@/utils/date';
import { MEASUREMENT_TYPE, MEASUREMENT_TYPE_UNIT } from '@/utils/i18n';

defineProps<{
  set: MeasurementSet;
}>();
</script>

<template>
  <!--
    This is pretty similar to what's in MeasurementEditView.
    Should the two of them be merged? I don't think it will be easy for workouts
    and exercises though. Also, there are styling differences and Tailwind will
    make that tricky to deal with.
  -->

  <div>
    <div class="flex justify-between">
      <div>Capture Date</div>
      <time :d="set.date">{{ displayDate(set.date) }}</time>
    </div>

    <div aria-label="Notes" class="whitespace-pre-wrap">{{ set.notes }}</div>

    <ul>
      <template v-for="ty in MEASUREMENT_TYPES">
        <li
          v-if="set.measurements[ty] !== undefined"
          class="flex items-center"
        >
          <div class="flex-grow">
            {{ MEASUREMENT_TYPE[ty] }}
            <i class="text-neutral-400">{{ MEASUREMENT_TYPE_UNIT[ty] }}</i>
          </div>

          <div class="text-right">{{ set.measurements[ty] }}</div>
        </li>
      </template>
    </ul>
  </div>
</template>
