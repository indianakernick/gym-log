<script setup lang="ts">
import { MEASUREMENT_TYPES, type Measurement, type MeasurementType } from '@/model/api';
import db from '@/services/db';
import sync from '@/services/sync';
import { displayDate } from '@/utils/date';
import { uuid } from '@/utils/uuid';
import { computed, shallowRef, triggerRef } from 'vue';
import { useRouter } from 'vue-router';

// Remaining functionality:
//
//  - Create measurements in the past.
//  - Change the date of measurements.
//  - Add notes to measurements.

const props = defineProps<{
  date: string
}>();

const router = useRouter();

// Making this a shallow ref so that it can be easily written to IndexedDB.
let measurements = shallowRef<Measurement[]>([]);
let deletedMeasurements: string[] = [];

db.getMeasurementsOfDate(props.date).then(d => {
  measurements.value = d;
  sort();
});

const availableTypes = computed(() => {
  if (measurements.value.length === 0) return MEASUREMENT_TYPES;

  let available: MeasurementType[] = [];

  for (let i = 0; i < MEASUREMENT_TYPES.length; ++i) {
    if (measurements.value[i - available.length]?.type !== MEASUREMENT_TYPES[i]) {
      available.push(MEASUREMENT_TYPES[i]);
    }
  }

  return available;
});

function sort() {
  measurements.value.sort((a, b) => {
    return MEASUREMENT_TYPES.indexOf(a.type) - MEASUREMENT_TYPES.indexOf(b.type);
  });
  triggerRef(measurements);
}

async function save() {
  await Promise.all(deletedMeasurements.map(m => db.stageDeleteMeasurement(m)));
  await Promise.all(measurements.value.map(m => db.stageUpdateMeasurement(m)));
  sync.sync();
  router.back();
}

function deleteItem(index: number) {
  deletedMeasurements.push(measurements.value[index].measurement_id);
  measurements.value.splice(index, 1);
  triggerRef(measurements);
}

function addItem(event: Event) {
  const select = event.target as HTMLSelectElement | null;
  if (select?.value) {
    measurements.value.push({
      measurement_id: deletedMeasurements.pop() || uuid(),
      type: select.value as MeasurementType,
      capture_date: props.date,
      value: 0,
      notes: ''
    });
    sort();
    select.value = '';
  }
}

const DISPLAY_TYPES: { [key in MeasurementType]: string } = {
  'weight': 'Weight (kg)',
  'height': 'Height (cm)',
  'arm-right-upper': 'Right Upper Arm (cm)',
  'arm-right-lower': 'Right Lower Arm (cm)',
  'arm-left-upper': 'Left Upper Arm (cm)',
  'arm-left-lower': 'Left Lower Arm (cm)',
  'leg-right-upper': 'Right Upper Leg (cm)',
  'leg-right-lower': 'Right Lower Leg (cm)',
  'leg-left-upper': 'Left Upper Leg (cm)',
  'leg-left-lower': 'Left Lower Leg (cm)'
};

</script>

<template>
  <main>
    <h1>Edit Measurements</h1>
    <button @click="router.back()">Cancel</button>
    <button @click="save()">Save</button>

    <div>
      <time :d="props.date">{{ displayDate(props.date) }}</time>
    </div>

    <ul>
      <li v-for="(m, i) in measurements">
        <label :for="m.measurement_id">{{ DISPLAY_TYPES[m.type] }}:</label>
        <input
          type="number"
          inputmode="decimal"
          v-model.lazy="m.value"
          @focus="($event.target as HTMLInputElement | null)?.select()"
        />
        <button @click="deleteItem(i)">X</button>
      </li>
    </ul>

    <template v-if="availableTypes.length">
      <select @change="addItem">
        <option value="" disabled selected>Add a measurement</option>
        <option v-for="t of availableTypes" :value="t">{{ DISPLAY_TYPES[t] }}</option>
      </select>
    </template>
  </main>
</template>
