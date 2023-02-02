<script setup lang="ts">
import { MEASUREMENT_TYPES, type MeasurementSet, type MeasurementType } from '@/model/api';
import { back } from '@/router/back';
import db from '@/services/db';
import sync from '@/services/sync';
import { displayDate } from '@/utils/date';
import { MEASUREMENT_TYPE } from '@/utils/i18n';
import { computed, shallowRef, triggerRef } from 'vue';
import { useRouter } from 'vue-router';
import { XMarkIcon } from '@heroicons/vue/20/solid';

const props = defineProps<{
  date: string;
}>();

const router = useRouter();

const measurementSet = shallowRef<MeasurementSet>({
  date: props.date,
  notes: '',
  measurements: {}
});

db.getMeasurementSet(props.date).then(d => {
  if (d) measurementSet.value = d;
});

const availableTypes = computed(() => {
  if (!measurementSet.value) return MEASUREMENT_TYPES;

  let available: MeasurementType[] = [];

  for (const type of MEASUREMENT_TYPES) {
    if (measurementSet.value.measurements[type] === undefined) {
      available.push(type);
    }
  }

  return available;
});

async function save() {
  await db.stageUpdateMeasurement(measurementSet.value);
  sync.sync();
  back(router, `/measurements`);
}

function deleteMeasurement(type: MeasurementType) {
  delete measurementSet.value.measurements[type];
  triggerRef(measurementSet);
}

function addMeasurement(event: Event) {
  const select = event.target as HTMLSelectElement | null;
  if (select?.value) {
    measurementSet.value.measurements[select.value as MeasurementType] = 0;
    select.value = '';
    triggerRef(measurementSet);
  }
}
</script>

<template>
  <header class="p-3 border-b dark:border-neutral-300 border-neutral-700 grid grid-cols-header">
    <button
      class="px-1 mr-auto dark:text-blue-500 text-blue-600"
      @click="back(router, `/measurements`)"
    >Cancel</button>
    <h1 class="font-bold text-lg text-center">Edit Measurements</h1>
    <button
      class="px-1 ml-auto font-bold dark:text-blue-500 text-blue-600"
      @click="save"
      :disabled="!Object.keys(measurementSet.measurements).length"
    >Save</button>
  </header>

  <main class="flex flex-col gap-2 py-2">
    <div class="px-3 flex justify-between">
      <div>Capture Date</div>
      <time :d="props.date">{{ displayDate(props.date) }}</time>
    </div>

    <!-- TODO: make this grow and shrink based on its contents -->
    <textarea
      aria-label="Notes"
      placeholder="Notes"
      v-model.lazy="measurementSet.notes"
      class="p-1 resize-none rounded-none dark:bg-neutral-700 dark:placeholder-neutral-400 focus:outline-none"
    ></textarea>

    <ul>
      <template v-for="ty in MEASUREMENT_TYPES">
        <li
          v-if="measurementSet.measurements[ty] !== undefined"
          class="px-3 py-2 flex flex-row items-center gap-2"
        >
          <label
            :for="`measurement-${ty}`"
            class="flex-grow"
          >{{ MEASUREMENT_TYPE[ty] }}</label>
          <input
            :id="`measurement-${ty}`"
            type="number"
            inputmode="decimal"
            v-model.lazy="measurementSet.measurements[ty]"
            @focus="($event.target as HTMLInputElement | null)?.select()"
            class="w-16 text-right p-1 rounded-lg dark:bg-neutral-700 dark:focus-visible:outline-blue-500"
          />

          <!-- Perhaps reveal this with a swipe-left -->
          <button
            @click="deleteMeasurement(ty)"
          >
            <XMarkIcon class="w-6 h-6"></XMarkIcon>
          </button>
        </li>
      </template>
    </ul>

    <select
      v-if="availableTypes.length"
      @change="addMeasurement"
      class="h-10 rounded-none dark:bg-neutral-700 focus:outline-none"
    >
      <option value="" disabled selected>Add Measurement</option>
      <option v-for="t of availableTypes" :value="t">{{ MEASUREMENT_TYPE[t] }}</option>
    </select>
  </main>
</template>
