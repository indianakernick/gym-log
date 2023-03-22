<script setup lang="ts">
import Menu from '@/components/Menu.vue';
import TextArea from '@/components/TextArea.vue';
import {
  measurementSetEqual,
  MEASUREMENT_TYPES,
  type MeasurementSet,
  type MeasurementType
} from '@/model/api';
import { back } from '@/router/back';
import db from '@/services/db';
import sync from '@/services/sync';
import { showAlert } from '@/utils/alert';
import { displayDate, toDateString } from '@/utils/date';
import { MEASUREMENT_TYPE, MEASUREMENT_TYPE_UNIT } from '@/utils/i18n';
import { refresh } from '@/utils/refresh';
import {
  IonBackButton,
  IonButtons,
  IonContent,
  IonHeader,
  IonIcon,
  IonPage,
  IonTitle,
  IonToolbar,
  useIonRouter
} from '@ionic/vue';
import { addOutline, trashOutline } from 'ionicons/icons';
import { computed, nextTick, onUnmounted, shallowRef, triggerRef } from 'vue';
import { onBeforeRouteLeave } from 'vue-router';

const props = defineProps<{
  date: string;
}>();

const router = useIonRouter();

const measurementSet = shallowRef<MeasurementSet>({
  date: props.date,
  notes: '',
  measurements: {}
});

async function load(initial: boolean) {
  const dbMeasurements = await db.getMeasurementSet(props.date);

  if (dbMeasurements) {
    if (
      !initial
      && editing.value
      && !measurementSetEqual(measurementSet.value, dbMeasurements)
      // TODO: can/should we use the merge conflict modal here?
      // ditto for the workout page
      && await showAlert({
        title: 'Keep edits?',
        message: 'Changes to these measurements have been pulled from another device. Do you want to keep your local edits?',
        buttons: 'keep-discard'
      })
    ) return;
    measurementSet.value = dbMeasurements;
    editing.value = false;
  } else if (initial) {
    readOnly.value = false;
  } else {
    back(router, '/measurements');
  }
}

refresh(load);

// Measurements for today are editable. They become read-only at midnight.
const now = new Date();
const readOnly = shallowRef(props.date !== toDateString(now));
let midnightTimer: number | undefined = undefined;

if (!readOnly.value) {
  const midnight = new Date(now);
  midnight.setDate(midnight.getDate() + 1);
  midnight.setHours(0, 0, 0, 0);
  midnightTimer = window.setTimeout(() => {
    readOnly.value = true;
  }, +midnight - +now);
}

onUnmounted(() => {
  clearTimeout(midnightTimer);
});

const editing = shallowRef(false);

onBeforeRouteLeave(async () => {
  if (!editing.value || await showAlert({
    title: 'Keep edits?',
    message: 'Do you want to keep the changes made to these measurements?',
    buttons: 'keep-discard'
  })) {
    if (!measurementSet.value.notes && !Object.keys(measurementSet.value.measurements).length) {
      await db.stageDeleteMeasurement(measurementSet.value.date);
    } else {
      await db.stageUpdateMeasurement(measurementSet.value);
    }
    sync.sync();
  }
});

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
    if (!editing) save();
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

const options = computed(() => {
  const items: InstanceType<typeof Menu>['items'] = [];

  if (readOnly.value && !editing.value) {
    items.push({
      title: 'Edit',
      handler: () => editing.value = true
    });
  }

  items.push({
    title: 'Delete Measurements',
    theme: 'danger',
    icon: trashOutline,
    handler: deleteSet
  });

  return items;
});

async function deleteSet() {
  if (await showAlert({
    title: 'Delete measurements',
    message: `Are you sure you want to delete measurements for ${displayDate(props.date)}?`,
    buttons: 'delete-cancel'
  })) {
    await db.stageDeleteMeasurement(props.date);
    sync.sync();
    back(router, '/measurements');
  }
}

async function save() {
  await db.stageUpdateMeasurement(measurementSet.value);
  sync.sync();
}
</script>

<template>
  <IonPage>
    <IonHeader>
      <IonToolbar>
        <IonButtons slot="start">
          <IonBackButton default-href="/measurements" />
        </IonButtons>
        <IonTitle>Measurement Details</IonTitle>
        <IonButtons slot="end">
          <Menu
            title="Measurement Options"
            context="title-bar"
            :items="options"
          />
        </IonButtons>
      </IonToolbar>
    </IonHeader>

    <IonContent>
      <div class="flex flex-col gap-3 py-3">
        <div class="mx-3 flex justify-between">
          <div>Capture Date</div>
          <time :d="date">{{ displayDate(date) }}</time>
        </div>

        <TextArea
          label="Notes"
          v-model="measurementSet.notes"
          @update:modelValue="editing || save()"
          :read-only="readOnly && !editing"
          class="mx-3"
        />

        <ul class="contents">
          <template v-for="ty in MEASUREMENT_TYPES">
            <li
              v-if="!readOnly || editing || measurementSet.measurements[ty] !== undefined"
              class="mx-3 flex items-center"
            >
              <label
                :for="`measurement-${ty}`"
                class="flex-grow"
              >
                {{ MEASUREMENT_TYPE[ty] }}
                <i class="text-neutral-400">{{ MEASUREMENT_TYPE_UNIT[ty] }}</i>
              </label>

              <input
                v-if="(!readOnly || editing) && measurementSet.measurements[ty] !== undefined"
                :id="`measurement-${ty}`"
                type="number"
                inputmode="decimal"
                min="0"
                :value="measurementSet.measurements[ty]"
                @change="setMeasurement($event, ty)"
                @focus="($event.target as HTMLInputElement | null)?.select()"
                :ref="el => setInputRef(el as HTMLInputElement | null, ty)"
                class="w-16 px-2 py-1 text-right rounded-lg bg-neutral-700"
              />

              <div
                v-else-if="readOnly && !editing"
                class="text-right"
              >{{ measurementSet.measurements[ty] }}</div>

              <button
                v-else
                :id="`measurement-${ty}`"
                @click="addMeasurement(ty)"
                class="relative w-16 py-1 rounded-lg flex justify-center
                  bg-neutral-800"
              >
                <!--
                  border-radius doesn't apply to outlines in Safari so this was the
                  next simplest thing.
                -->
                <div class="absolute inset-0 rounded-lg border
                  border-neutral-600"></div>
                <IonIcon :icon="addOutline" class="w-6 h-6 text-neutral-300" />
              </button>
            </li>
          </template>
        </ul>
      </div>
    </IonContent>
  </IonPage>
</template>
