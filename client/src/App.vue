<script setup lang="ts">
import { ModalsContainer } from 'vue-final-modal';
import { RouterView } from 'vue-router';
import sync from './services/sync';

sync.setup();
</script>

<template>
  <ModalsContainer />
  <RouterView />
</template>
