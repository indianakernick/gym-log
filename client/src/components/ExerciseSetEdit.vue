<script setup lang="ts">
import type { Exercise, Workout } from '@/model/api';
import { IonIcon } from '@ionic/vue';
import { addOutline } from 'ionicons/icons';
import { shallowRef } from 'vue';
import SetTable from './SetTable.vue';
import TextArea from './TextArea.vue';

defineProps<{
  exercise: Exercise;
  history?: (Exercise & { workout: Workout })[];
}>();

const emit = defineEmits<{
  (e: 'setCreated'): void;
  (e: 'setsChanged'): void;
}>();

const table = shallowRef<InstanceType<typeof SetTable>>();

function addSet() {
  if (table.value) {
    table.value.add();
    emit('setCreated');
  }
}
</script>

<template>
  <div class="mx-2 flex flex-col">
    <TextArea
      label="Notes"
      v-model="exercise.notes"
      @update:model-value="emit('setsChanged')"
      :read-only="!history"
      class="my-2 w-full"
    />

    <SetTable
      v-if="exercise.sets.length"
      ref="table"
      :exercise="exercise"
      :history="history"
      v-slot="{ set, field }"
      class="mb-2"
    >
      <input
        v-if="history"
        type="number"
        v-model="set[field]"
        @change="emit('setsChanged')"
        class="max-w-full px-2 py-1 text-center rounded-lg bg-neutral-700"
      />
      <div v-else>{{ set[field] }}</div>
    </SetTable>
  </div>

  <button
    v-if="history"
    @click="addSet"
    class="p-2 w-full font-bold text-blue-500 border-t border-neutral-600
      button-flex"
  >
    <IonIcon :icon="addOutline" class="w-5 h-5" />
    Add Set
  </button>
</template>
