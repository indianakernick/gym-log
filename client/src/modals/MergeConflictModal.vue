<script setup lang="ts">
import ResolveChoose from '@/components/ResolveChoose.vue';
import ResolveMeasurement from '@/components/ResolveMeasurement.vue';
import SequenceNavigator from '@/components/SequenceNavigator.vue';
import type { MergeConflict, MergeConflictResolutions } from '@/model/db';
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
    <p>
      There were conflicts when syncing changes. This can happen when a change
      is made on another device while also making changes on this device. You'll
      have to choose which changes to keep.
    </p>

    <SequenceNavigator
      v-model="conflictIdx"
      :length="conflicts.length"
      class="sticky top-0 bg-neutral-800"
    >
      Resolving {{ conflictIdx + 1 }} / {{ conflicts.length }}
    </SequenceNavigator>

    <template v-for="conflict in [conflicts[conflictIdx]]">
      <template v-if="conflict.type === 'measurement'">
        <div>
          <ResolveChoose :id="conflict.id" type="remote" @choose="resolve" />

          <i v-if="'deleted' in conflict.remote" class="text-red-500">
            Deleted
          </i>

          <div v-else>
            <ResolveMeasurement :set="conflict.remote"></ResolveMeasurement>
          </div>
        </div>

        <div>
          <ResolveChoose :id="conflict.id" type="local" @choose="resolve" />

          <i v-if="'deleted' in conflict.local" class="text-red-500">
            Deleted
          </i>

          <div v-else>
            <ResolveMeasurement :set="conflict.local"></ResolveMeasurement>
          </div>
        </div>
      </template>

      <template v-else-if="conflict.type === 'workout'">

      </template>

      <template v-else-if="conflict.type === 'exercise'">

      </template>
    </template>
  </Modal>
</template>
