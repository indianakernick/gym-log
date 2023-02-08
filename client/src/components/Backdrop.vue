<script setup lang="ts">
defineProps<{
  show: boolean;
}>();
</script>

<template>
  <div
    class="fixed z-10 inset-0 bg-black transition-opacity"
    :class="{
      'opacity-0 pointer-events-none': !show,
      'opacity-50': show,
    }"
  ></div>
</template>
