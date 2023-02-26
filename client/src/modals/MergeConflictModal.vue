<script setup lang="ts">
import ResolveItem from '@/components/ResolveItem.vue';
import ResolveItemExercise from '@/components/ResolveItemExercise.vue';
import ResolveItemMeasurement from '@/components/ResolveItemMeasurement.vue';
import ResolveItemWorkout from '@/components/ResolveItemWorkout.vue';
import SequenceNavigator from '@/components/SequenceNavigator.vue';
import type { Exercise, MeasurementSet, Workout } from '@/model/api';
import type { Deleted, MergeConflict, MergeConflictResolutions } from '@/model/db';
import { ref, shallowRef } from 'vue';
import Modal from './Modal.vue';

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
  <Modal
    title="Sync Conflicts"
    :buttons="[{
      title: 'Resolve',
      theme: 'primary',
      bold: true,
      disabled: Object.keys(resolutions).length < conflicts.length,
      handler: () => $emit('resolved', resolutions)
    }]"
    :trap="true"
  >
    <p
      class="p-3"
      :class="{
        'border-b border-neutral-600': conflicts.length === 1
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
      class="sticky -top-[1px] bg-neutral-800 px-3 py-2 border-t border-b
        border-neutral-600"
    >
      Resolving {{ conflictIdx + 1 }} / {{ conflicts.length }}
    </SequenceNavigator>

    <!--
      TODO: maybe do some color coding to highlight the individual pieces of
      data that differ between the remote and local versions.
    -->

    <template v-for="conflict in [conflicts[conflictIdx]]">
      <template v-if="conflict.type === 'measurement'">
        <ResolveItem v-slot="slotProps" :conflict="conflict" @resolve="resolve">
          <!--
            We'd need generically typed components to remove the cast.
            https://github.com/vuejs/rfcs/discussions/436
            That would be pretty cool!
          -->
          <ResolveItemMeasurement
            :set="(slotProps.item as MeasurementSet)"
            :other-set="(slotProps.other as MeasurementSet | Deleted)"
          />
        </ResolveItem>
      </template>

      <template v-else-if="conflict.type === 'workout'">
        <ResolveItem v-slot="slotProps" :conflict="conflict" @resolve="resolve">
          <ResolveItemWorkout :workout="(slotProps.item as Workout)"/>
        </ResolveItem>
      </template>

      <template v-else-if="conflict.type === 'exercise'">
        <ResolveItem v-slot="slotProps" :conflict="conflict" @resolve="resolve">
          <ResolveItemExercise :exercise="(slotProps.item as Exercise)" />
        </ResolveItem>
      </template>
    </template>
  </Modal>
</template>
