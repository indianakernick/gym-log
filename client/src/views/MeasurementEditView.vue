<script setup lang="ts">
import { MEASUREMENT_TYPES, type MeasurementSet, type MeasurementType } from '@/model/api';
import { back } from '@/router/back';
import db from '@/services/db';
import sync from '@/services/sync';
import { displayDate } from '@/utils/date';
import { MEASUREMENT_TYPE, MEASUREMENT_TYPE_UNIT } from '@/utils/i18n';
import { PlusIcon } from '@heroicons/vue/24/outline';
import { nextTick, shallowRef, triggerRef } from 'vue';
import { useRouter } from 'vue-router';

// TODO: prevent users from accidentally editing historic measurements. they
// should still be able to do it if they want to, but don't make it too easy

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

async function save() {
  await db.stageUpdateMeasurement(measurementSet.value);
  sync.sync();
  back(router, `/measurements`);
}

function addMeasurement(type: MeasurementType) {
  measurementSet.value.measurements[type] = 0;
  triggerRef(measurementSet);
  nextTick(() => {
    refs.get(type)?.focus();
  });
}

function setMeasurement(event: Event, type: MeasurementType) {
  const input = event.target as HTMLInputElement | null;
  if (input) {
    if (Number.isNaN(input.valueAsNumber)) {
      delete measurementSet.value.measurements[type];
    } else {
      measurementSet.value.measurements[type] = input.valueAsNumber;
    }
    triggerRef(measurementSet);
  }
}

const refs = new Map<MeasurementType, HTMLInputElement>();

function setInputRef(el: HTMLInputElement | null, type: MeasurementType) {
  if (el) {
    refs.set(type, el);
  } else {
    refs.delete(type);
  }
}
</script>

<template>
  <header class="p-3 border-b dark:border-neutral-300 border-neutral-700 grid grid-cols-header">
    <button
      class="mr-auto dark:text-blue-500 text-blue-600"
      @click="back(router, `/measurements`)"
    >Cancel</button>
    <h1 class="font-bold text-lg text-center">Edit Measurements</h1>
    <button
      class="ml-auto font-bold dark:text-blue-500 text-blue-600"
      @click="save"
      :disabled="!Object.keys(measurementSet.measurements).length"
    >Save</button>
  </header>

  <main class="flex flex-col gap-2 py-2">
    <div class="px-3 py-2 flex justify-between">
      <div>Capture Date</div>
      <time :d="props.date">{{ displayDate(props.date) }}</time>
    </div>

    <!-- TODO: make this grow and shrink based on its contents -->
    <textarea
      aria-label="Notes"
      placeholder="Notes"
      v-model.lazy="measurementSet.notes"
      class="mx-3 p-1 resize-none rounded-lg dark:bg-neutral-700 dark:placeholder-neutral-400 focus:outline-none"
    ></textarea>

    <ul>
      <li
        v-for="ty in MEASUREMENT_TYPES"
        class="px-3 py-2 flex flex-row items-center"
      >
        <label
          :for="`measurement-${ty}`"
          class="flex-grow"
        >
          {{ MEASUREMENT_TYPE[ty] }}
          <i class="text-neutral-400">{{ MEASUREMENT_TYPE_UNIT[ty] }}</i>
        </label>

        <input
          v-if="measurementSet.measurements[ty] !== undefined"
          :id="`measurement-${ty}`"
          type="number"
          inputmode="decimal"
          min="0"
          :value="measurementSet.measurements[ty]"
          @change="setMeasurement($event, ty)"
          @focus="($event.target as HTMLInputElement | null)?.select()"
          :ref="el => setInputRef(el as any, ty)"
          class="w-16 text-right p-1 rounded-lg dark:bg-neutral-700 dark:focus-visible:outline-blue-500"
        />

        <button
          v-else
          :id="`measurement-${ty}`"
          @click="addMeasurement(ty)"
          class="w-16 py-1 rounded-lg flex justify-center relative"
        >
          <!--
            border-radius doesn't apply to outlines in Safari so this was the
            next simplest thing.
          -->
          <div class="absolute inset-0 rounded-lg border dark:border-neutral-300"></div>
          <PlusIcon class="w-6 h-6 dark:text-neutral-300"></PlusIcon>
        </button>
      </li>
    </ul>
  </main>
</template>
