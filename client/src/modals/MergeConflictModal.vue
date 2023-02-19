<script setup lang="ts">
import type { MergeConflict, MergeConflictResolutions } from '@/model/db';
import { displayDate } from '@/utils/date';
import { ChevronDownIcon, ChevronUpIcon } from '@heroicons/vue/24/outline';
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
    background="non-interactive"
  >
    <p>
      There were conflicts when syncing changes. This can happen when a change
      is made on another device while also making changes on this device. You'll
      have to choose which changes to keep.
    </p>

    <div class="sticky top-0 flex items-center bg-neutral-800">
      <div>Resolving {{ conflictIdx + 1 }} / {{ conflicts.length }}</div>

      <button
        @click="--conflictIdx"
        :disabled="conflictIdx < 1"
        aria-label="Previous"
        class="ml-auto disabled:text-neutral-600"
      >
        <ChevronUpIcon class="w-6 h-6"></ChevronUpIcon>
      </button>

      <button
        @click="++conflictIdx"
        :disabled="conflictIdx === conflicts.length - 1"
        aria-label="Next"
        class="ml-3 disabled:text-neutral-600"
      >
        <ChevronDownIcon class="w-6 h-6"></ChevronDownIcon>
      </button>
    </div>

    <template v-for="conflict in [conflicts[conflictIdx]]">
      <template v-if="conflict.type === 'measurement'">
        <div>
          <div class="flex justify-between items-center">
            <label for="remote" class="font-bold">Remote</label>
            <input
              type="radio"
              id="remote"
              :name="conflict.id"
              @input="resolutions[conflict.id] = 'remote'"
              class="w-5 h-5"
            />
          </div>

          <div v-if="'deleted' in conflict.remote" class="">
            Deleted
          </div>

          <div v-else>
            <div class="flex justify-between">
              <div>Capture Date</div>
              <time :d="conflict.remote.date">{{ displayDate(conflict.remote.date) }}</time>
            </div>

            <div>{{ conflict.remote.notes }}</div>
          </div>
        </div>

        <div>
          <div class="flex justify-between items-center">
            <label for="local" class="font-bold">Local</label>
            <input
              type="radio"
              id="local"
              :name="conflict.id"
              @input="resolutions[conflict.id] = 'local'"
              class="w-5 h-5"
            />
          </div>

          <div v-if="'deleted' in conflict.local">
            Deleted
          </div>

          <div v-else>
            <div class="flex justify-between">
              <div>Capture Date</div>
              <time :d="conflict.local.date">{{ displayDate(conflict.local.date) }}</time>
            </div>

            <div>{{ conflict.local.notes }}</div>
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
