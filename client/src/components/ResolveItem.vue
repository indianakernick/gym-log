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
  <div class="p-3">
    <ResolveChoose
      :id="conflict.id"
      type="remote"
      @choose="$emit('resolve', conflict.id, 'remote')"
      class="pb-2"
    />

    <i v-if="'deleted' in conflict.remote" class="text-red-500">Deleted</i>

    <slot v-else :item="conflict.remote" :other="conflict.local"></slot>
  </div>

  <div class="p-3 border-t border-neutral-600">
    <ResolveChoose
      :id="conflict.id"
      type="local"
      @choose="$emit('resolve', conflict.id, 'local')"
      class="pb-2"
    />

    <i v-if="'deleted' in conflict.local" class="text-red-500">Deleted</i>

    <slot v-else :item="conflict.local" :other="conflict.remote"></slot>
  </div>
</template>
