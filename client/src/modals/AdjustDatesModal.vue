<script setup lang="ts">
import {
  IonButton,
  IonButtons,
  IonContent,
  IonHeader,
  IonTitle,
  IonToolbar,
  modalController
} from '@ionic/vue';
import { shallowRef, watchEffect } from 'vue';

const props = defineProps<{
  start: string;
  finish: string;
}>();

const tz = new Date().getTimezoneOffset() * 60000;

function toInputDateTime(dateTimeStr: string) {
  return (new Date(Date.parse(dateTimeStr) - tz)).toISOString().substring(0, 19);
}

function fromInputDateTime(iso: string): string {
  // The date string does not have a Z suffix so the Date constructor is
  // interpreting it as a local date. No need to add the time zone offset.
  return new Date(iso).toISOString().substring(0, 19) + 'Z';
}

const start = shallowRef(toInputDateTime(props.start));
const finish = shallowRef(toInputDateTime(props.finish));
const error = shallowRef<'start-before-finish' | 'past'>();

// Safari doesn't support min, max or step attributes.

watchEffect(() => {
  const startTime = new Date(start.value);
  const finishTime = new Date(finish.value);
  const now = Date.now();

  if (startTime >= finishTime) {
    error.value = 'start-before-finish';
  } else if (+startTime >= now || +finishTime > now) { // finish now is fine
    error.value = 'past';
  } else {
    error.value = undefined;
  }
});

function save() {
  if (!error.value) {
    modalController.dismiss({
      start: fromInputDateTime(start.value),
      finsih: fromInputDateTime(finish.value),
    });
  }
}
</script>

<template>
  <IonHeader>
    <IonToolbar>
      <IonButtons slot="start">
        <IonButton @click="modalController.dismiss()">Cancel</IonButton>
      </IonButtons>
      <IonTitle>Adjust Dates</IonTitle>
      <IonButtons slot="end">
        <IonButton
          @click="save"
          :disabled="!!error"
          :strong="true"
        >Save</IonButton>
      </IonButtons>
    </IonToolbar>
  </IonHeader>

  <IonContent>
    <div class="p-3 flex flex-col gap-3">
      <div
        v-if="error"
        class="p-2 rounded-lg border border-red-500 border-opacity-50
          text-red-500 bg-red-500 bg-opacity-10"
      >
        <template v-if="error === 'start-before-finish'">
          Start time must be before finish time.
        </template>
        <template v-else-if="error === 'past'">
          Start and finish times must be in the past.
        </template>
      </div>

      <div class="flex items-center justify-between">
        <label for="start">Started</label>
        <input
          id="started"
          type="datetime-local"
          v-model="start"
          step="1"
          class="appearance-none px-2 py-1 rounded-lg
            bg-neutral-300 dark:bg-neutral-700"
        />
      </div>

      <div class="flex items-center justify-between">
        <label for="finished">Finished</label>
        <input
          id="finished"
          type="datetime-local"
          v-model="finish"
          step="1"
          class="appearance-none px-2 py-1 rounded-lg
            bg-neutral-300 dark:bg-neutral-700"
        />
      </div>
    </div>
  </IonContent>
</template>
