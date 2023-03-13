<script setup lang="ts">
import { IonApp, IonRouterOutlet } from '@ionic/vue';
import { ModalsContainer } from 'vue-final-modal';
import sync from './services/sync';

sync.setup();
</script>

<template>
  <ModalsContainer />
  <IonApp>
    <IonRouterOutlet />
  </IonApp>
</template>
