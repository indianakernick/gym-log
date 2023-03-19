<script setup lang="ts">
import {
  IonIcon,
  IonLabel,
  IonPage,
  IonRouterOutlet,
  IonTabBar,
  IonTabButton,
  IonTabs
} from '@ionic/vue';
import { barbellOutline, scaleOutline } from 'ionicons/icons';

const tabs = [
  {
    prefix: '/workouts',
    label: 'Workouts',
    icon: barbellOutline
  },
  {
    prefix: '/measurements',
    label: 'Measurements',
    icon: scaleOutline
  },
] as const;
</script>

<template>
  <IonPage>
    <IonTabs>
      <IonRouterOutlet />
      <IonTabBar slot="bottom">
        <IonTabButton v-for="tab in tabs" :tab="tab.prefix" :href="tab.prefix">
          <IonIcon :icon="tab.icon" />
          <IonLabel>{{ tab.label }}</IonLabel>
        </IonTabButton>
      </IonTabBar>
    </IonTabs>
  </IonPage>
</template>

<style scoped>
/*
Tailwind's preflight stylesheet adds this:

  border-width: 0;
  border-style: solid;
  border-color: #e5e7eb;

This overrides the border on Ionic's tab bar so we're adding it back.
*/
ion-tab-bar {
  border-top: var(--border);
}
</style>
