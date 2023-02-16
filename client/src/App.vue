<script setup lang="ts">
import { ModalsContainer } from 'vue-final-modal';
import { RouterView, useRouter } from 'vue-router';
import sync from './services/sync';

sync.setRouter(useRouter());
</script>

<template>
  <ModalsContainer />
  <RouterView />
</template>
