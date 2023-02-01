<script setup lang="ts">
import { MEASUREMENT_TYPES, type MeasurementSet, type MeasurementType } from '@/model/api';
import { back } from '@/router/back';
import db from '@/services/db';
import sync from '@/services/sync';
import { displayDate } from '@/utils/date';
import { MEASUREMENT_TYPE } from '@/utils/i18n';
import { computed, shallowRef, triggerRef } from 'vue';
import { useRouter } from 'vue-router';

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
  <main>
    <h1>Edit Measurements</h1>
    <button @click="back(router, `/measurements`)">Cancel</button>
    <button
      @click="save"
      :disabled="!Object.keys(measurementSet.measurements).length"
    >Save</button>

    <div>
      <time :d="props.date">{{ displayDate(props.date) }}</time>
    </div>

    <label for="notes">Notes:</label>
    <textarea id="notes" v-model.lazy="measurementSet.notes"></textarea>

    <ul>
      <template v-for="ty in MEASUREMENT_TYPES">
        <li v-if="measurementSet.measurements[ty] !== undefined">
          <label :for="`measurement-${ty}`">{{ MEASUREMENT_TYPE[ty] }}:</label>
          <input
            :id="`measurement-${ty}`"
            type="number"
            inputmode="decimal"
            v-model.lazy="measurementSet.measurements[ty]"
            @focus="($event.target as HTMLInputElement | null)?.select()"
          />
          <button @click="deleteMeasurement(ty)">X</button>
        </li>
      </template>
    </ul>

    <template v-if="availableTypes.length">
      <select @change="addMeasurement">
        <option value="" disabled selected>Add Measurement</option>
        <option v-for="t of availableTypes" :value="t">{{ MEASUREMENT_TYPE[t] }}</option>
      </select>
    </template>
  </main>
</template>
