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
