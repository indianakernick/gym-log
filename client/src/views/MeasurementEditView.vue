<script setup lang="ts">
import Header from '@/components/Header.vue';
import Main from '@/components/Main.vue';
import {
  MEASUREMENT_TYPES,
  type MeasurementSet,
  type MeasurementType
} from '@/model/api';
import { back } from '@/router/back';
import db from '@/services/db';
import sync from '@/services/sync';
import { displayDate, toDateString } from '@/utils/date';
import { MEASUREMENT_TYPE, MEASUREMENT_TYPE_UNIT } from '@/utils/i18n';
import { TrashIcon } from '@heroicons/vue/20/solid';
import { PlusIcon } from '@heroicons/vue/24/outline';
import { nextTick, ref, shallowRef, triggerRef, watch } from 'vue';
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

const readOnly = ref(false);
let midnightTimer: number | undefined = undefined;

watch(
  () => props.date,
  d => {
    const now = new Date();
    readOnly.value = d !== toDateString(now);
    clearTimeout(midnightTimer);

    if (!readOnly.value) {
      const midnight = new Date(now);
      midnight.setDate(midnight.getDate() + 1);
      midnight.setHours(0, 0, 0, 0);
      midnightTimer = window.setTimeout(() => {
        readOnly.value = true;
      }, +midnight - +now);
    }
  },
  { immediate: true }
);

async function save() {
  await db.stageUpdateMeasurement(measurementSet.value);
  sync.sync();
  back(router, `/measurements`);
}

function addMeasurement(type: MeasurementType) {
  measurementSet.value.measurements[type] = 0;
  triggerRef(measurementSet);
  nextTick(() => {
    inputs.get(type)?.focus();
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

const inputs = new Map<MeasurementType, HTMLInputElement>();

function setInputRef(el: HTMLInputElement | null, type: MeasurementType) {
  if (el) {
    inputs.set(type, el);
  } else {
    inputs.delete(type);
  }
}

async function deleteSet() {
  if (confirm(`Delete measurements for ${displayDate(props.date)}?`)) {
    await db.stageDeleteMeasurement(props.date);
    sync.sync();
    back(router, `/measurements`);
  }
}
</script>

<template>
  <Header
    title="Edit Measurements"
    :right-disabled="!Object.keys(measurementSet.measurements).length"
    @left="back(router, `/measurements`)"
    @right="save"
  >
    <template #left>Cancel</template>
    <template #right>Save</template>
  </Header>

  <Main>
    <div class="px-3 py-2 flex justify-between">
      <div>Capture Date</div>
      <time :d="props.date">{{ displayDate(props.date) }}</time>
    </div>

    <!-- TODO: make this grow and shrink based on its contents -->
    <textarea
      v-if="!readOnly"
      aria-label="Notes"
      placeholder="Notes"
      v-model.lazy="measurementSet.notes"
      class="mx-3 my-2 px-2 py-1 resize-none rounded-lg dark:bg-neutral-700
        dark:placeholder-neutral-400 focus:outline-none"
    ></textarea>

    <div
      v-else-if="measurementSet.notes"
      aria-label="Notes"
      class="mx-3 my-2"
    >{{ measurementSet.notes }}</div>

    <ul>
      <template v-for="ty in MEASUREMENT_TYPES">
        <li
          v-if="!readOnly || measurementSet.measurements[ty] !== undefined"
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
            v-if="!readOnly && measurementSet.measurements[ty] !== undefined"
            :id="`measurement-${ty}`"
            type="number"
            inputmode="decimal"
            min="0"
            :value="measurementSet.measurements[ty]"
            @change="setMeasurement($event, ty)"
            @focus="($event.target as HTMLInputElement | null)?.select()"
            :ref="el => setInputRef(el as any, ty)"
            class="w-16 px-2 py-1 text-right rounded-lg dark:bg-neutral-700
              dark:focus-visible:outline-blue-500"
          />

          <div
            v-else-if="readOnly"
            class="text-right"
          >{{ measurementSet.measurements[ty] }}</div>

          <button
            v-else
            :id="`measurement-${ty}`"
            @click="addMeasurement(ty)"
            class="relative w-16 py-1 rounded-lg flex justify-center
              dark:bg-neutral-800"
          >
            <!--
              border-radius doesn't apply to outlines in Safari so this was the
              next simplest thing.
            -->
            <div class="absolute inset-0 rounded-lg border
              dark:border-neutral-600"></div>
            <PlusIcon class="w-6 h-6 dark:text-neutral-300"></PlusIcon>
          </button>
        </li>
      </template>
    </ul>

    <button
      @click="deleteSet"
      class="relative py-2 rounded-lg text-red-500 font-bold mx-3 my-2 flex
        items-center justify-center gap-1 dark:bg-neutral-800"
    >
      <div class="absolute inset-0 rounded-lg border
        dark:border-neutral-600"></div>
      <TrashIcon class="w-5 h-5" />
      Delete
    </button>
  </Main>
</template>
