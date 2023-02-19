<script setup lang="ts">
import type { MergeConflict } from '@/model/db';
import ResolveChoose from './ResolveChoose.vue';

defineProps<{
  conflict: MergeConflict;
}>();

defineEmits<{
  (e: 'resolve', id: string, which: 'local' | 'remote'): void;
}>();
</script>

<template>
  <div>
    <ResolveChoose
      :id="conflict.id"
      type="remote"
      @choose="$emit('resolve', conflict.id, 'remote')"
    />

    <i v-if="'deleted' in conflict.remote" class="text-red-500">Deleted</i>

    <slot v-else :item="conflict.remote"></slot>
  </div>

  <div>
    <ResolveChoose
      :id="conflict.id"
      type="local"
      @choose="$emit('resolve', conflict.id, 'local')"
    />

    <i v-if="'deleted' in conflict.local" class="text-red-500">Deleted</i>

    <slot v-else :item="conflict.local"></slot>
  </div>
</template>
