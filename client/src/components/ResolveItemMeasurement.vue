<script setup lang="ts">
import { MEASUREMENT_TYPES, type MeasurementSet } from '@/model/api';
import type { Deleted } from '@/model/db';
import { displayDate } from '@/utils/date';
import { MEASUREMENT_TYPE, MEASUREMENT_TYPE_UNIT } from '@/utils/i18n';
import { describeChange } from '@/utils/merge';
import ResolveColor from './ResolveColor.vue';

defineProps<{
  set: MeasurementSet;
  otherSet: MeasurementSet | Deleted;
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
      <ResolveColor :desc="describeChange(set, otherSet, s => s.date)">
        <time :d="set.date">{{ displayDate(set.date) }}</time>
      </ResolveColor>
    </div>

    <ResolveColor
      :desc="describeChange(set, otherSet, s => s.notes)"
      aria-label="Notes"
      class="whitespace-pre-wrap"
    >
      {{ set.notes }}
    </ResolveColor>

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

          <ResolveColor :desc="describeChange(set, otherSet, s => s.measurements[ty])">
            {{ set.measurements[ty] }}
          </ResolveColor>
        </li>
      </template>
    </ul>
  </div>
</template>
