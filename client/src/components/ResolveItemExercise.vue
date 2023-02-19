<script setup lang="ts">
import type { Exercise } from '@/model/api';
import SetEdit from './SetEdit.vue';

defineProps<{
  exercise: Exercise;
}>();
</script>

<template>
  <!-- TODO: There is a px-2 so this isn't quite right. -->
  <!-- TODO: Perhaps display some context (the workout). -->
  <SetEdit :exercise="exercise" />
</template>
