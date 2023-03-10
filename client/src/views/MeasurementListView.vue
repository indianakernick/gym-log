<script setup lang="ts">
import db from '@/services/db';
import { groupBy } from '@/utils/array';
import { displayDate, toDateString } from '@/utils/date';
import { refresh } from '@/utils/refresh';
import { itemLines } from '@/utils/style';
import {
  IonButton,
  IonButtons,
  IonContent,
  IonDatetime,
  IonHeader,
  IonIcon,
  IonItem,
  IonItemDivider,
  IonItemGroup,
  IonLabel,
  IonList,
  IonModal,
  IonPage,
  IonTitle,
  IonToolbar
} from '@ionic/vue';
import { addOutline, calendarClearOutline } from 'ionicons/icons';
import { onMounted, shallowRef } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

const years = shallowRef<string[][]>([]);
const page = shallowRef();
const presentingElement = shallowRef();
const dateModal = shallowRef();
const date = shallowRef<string>();

onMounted(() => {
  presentingElement.value = page.value.$el;
});

async function load() {
  years.value = groupBy(await db.getMeasurementDates(), date => date.substring(0, 4));
}

refresh(load);

function addToday() {
  router.push(`/measurements/${toDateString(new Date())}`);
}

function dateModalDismissed(event: CustomEvent<{ data?: string }>) {
  if (event.detail.data) {
    router.push(`/measurements/${toDateString(new Date(event.detail.data))}`);
  }
}

function openDateModal() {
  date.value = undefined;
  dateModal.value.$el.present();
}
</script>

<template>
  <IonPage ref="page">
    <IonHeader>
      <IonToolbar>
        <IonButtons slot="start">
          <IonButton @click="openDateModal">
            <IonIcon slot="icon-only" :icon="calendarClearOutline" />
          </IonButton>
        </IonButtons>
        <IonTitle>Measurements</IonTitle>
        <IonButtons slot="end">
          <IonButton @click="addToday">
            <IonIcon slot="icon-only" :icon="addOutline" />
          </IonButton>
        </IonButtons>
      </IonToolbar>
    </IonHeader>

    <IonContent :fullscreen="true">
      <IonHeader collapse="condense">
        <IonToolbar>
          <IonTitle size="large">Measurements</IonTitle>
        </IonToolbar>
      </IonHeader>

      <IonList>
        <IonItemGroup v-for="year, y in years">
          <IonItemDivider>
            <IonLabel>{{ year[0].substring(0, 4) }}</IonLabel>
          </IonItemDivider>

          <IonItem
            v-for="date, d in year"
            button
            :detail="true"
            :lines="itemLines(years, y, d)"
            @click="router.push(`/measurements/${date}`)"
          >
            <IonLabel>{{ displayDate(date) }}</IonLabel>
          </IonItem>
        </IonItemGroup>
      </IonList>

      <IonModal
        ref="dateModal"
        :presenting-element="presentingElement"
        :can-dismiss="true"
        @didDismiss="dateModalDismissed"
      >
        <IonHeader>
          <IonToolbar>
            <IonButtons slot="start">
              <IonButton @click="dateModal.$el.dismiss()">Cancel</IonButton>
            </IonButtons>
            <IonTitle>Select Date</IonTitle>
            <IonButtons slot="end">
              <IonButton :strong="true" :disabled="!date" @click="dateModal.$el.dismiss(date)">Select</IonButton>
            </IonButtons>
          </IonToolbar>
        </IonHeader>

        <IonContent class="ion-padding" :scroll-x="false" :scroll-y="false">
          <div>Select a date to record past measurements.</div>
          <IonDatetime
            :first-day-of-week="1"
            :max="new Date().toISOString()"
            v-model="date"
            presentation="date"
          />
        </IonContent>
      </IonModal>
    </IonContent>
  </IonPage>
</template>

<style scoped>
ion-item-divider {
  position: sticky;
  top: 0;
}

ion-datetime {
  border-radius: 10px;
  margin: 16px auto 0 auto;
}
</style>
