<script setup lang="ts">
import ResolveItem from '@/components/ResolveItem.vue';
import ResolveItemExercise from '@/components/ResolveItemExercise.vue';
import ResolveItemMeasurement from '@/components/ResolveItemMeasurement.vue';
import ResolveItemWorkout from '@/components/ResolveItemWorkout.vue';
import SequenceNavigator from '@/components/SequenceNavigator.vue';
import type { Exercise, MeasurementSet, Workout } from '@/model/api';
import type { Deleted, MergeConflict, MergeConflictResolutions } from '@/model/db';
import {
  IonButton,
  IonButtons,
  IonContent,
  IonHeader,
  IonTitle,
  IonToolbar,
  modalController
} from '@ionic/vue';
import { ref, shallowRef } from 'vue';

defineProps<{
  conflicts: MergeConflict[];
}>();

defineEmits<{
  (e: 'resolved', resolutions: MergeConflictResolutions): void;
}>();

const conflictIdx = shallowRef(0);
const resolutions = ref<MergeConflictResolutions>({});

function resolve(id: string, which: 'local' | 'remote') {
  resolutions.value[id] = which;
}
</script>

<template>
  <IonHeader>
    <IonToolbar>
      <IonTitle>Sync Conflicts</IonTitle>
      <IonButtons slot="end">
        <IonButton
          @click="modalController.dismiss(resolutions)"
          :disabled="Object.keys(resolutions).length < conflicts.length"
          :strong="true"
        >Resolve</IonButton>
      </IonButtons>
    </IonToolbar>
  </IonHeader>

  <IonContent>
    <p
      class="p-3"
      :class="{
        'border-b border-neutral-300 dark:border-neutral-600': conflicts.length === 1
      }"
    >
      There were conflicts when syncing changes. This can happen when a change
      is made on another device while also making changes on this device. You'll
      have to choose which changes to keep.
    </p>

    <SequenceNavigator
      v-if="conflicts.length > 1"
      v-model="conflictIdx"
      :length="conflicts.length"
      class="sticky -top-px px-3 py-2 border-t border-b
        border-neutral-300 dark:border-neutral-600
        bg-neutral-100 dark:bg-neutral-800"
    >
      Resolving {{ conflictIdx + 1 }} / {{ conflicts.length }}
    </SequenceNavigator>

    <template v-for="conflict in [conflicts[conflictIdx]]">
      <template v-if="conflict.type === 'measurement'">
        <ResolveItem v-slot="{ item, other }" :conflict="conflict" @resolve="resolve">
          <!--
            We'd need generically typed components to remove the cast.
            https://github.com/vuejs/rfcs/discussions/436
            That would be pretty cool!
          -->
          <ResolveItemMeasurement
            :set="(item as MeasurementSet)"
            :other-set="(other as MeasurementSet | Deleted)"
          />
        </ResolveItem>
      </template>

      <template v-else-if="conflict.type === 'workout'">
        <ResolveItem v-slot="{ item, other }" :conflict="conflict" @resolve="resolve">
          <ResolveItemWorkout
            :workout="(item as Workout)"
            :other-workout="(other as Workout | Deleted)"
          />
        </ResolveItem>
      </template>

      <template v-else-if="conflict.type === 'exercise'">
        <ResolveItem v-slot="{ item, other }" :conflict="conflict" @resolve="resolve">
          <ResolveItemExercise
            :exercise="(item as Exercise)"
            :other-exercise="(other as Exercise | Deleted)"
          />
        </ResolveItem>
      </template>
    </template>
  </IonContent>
</template>
