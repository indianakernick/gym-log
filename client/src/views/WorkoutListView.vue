<script setup lang="ts">
import AlertModal from '@/modals/AlertModal.vue';
import type { Workout } from '@/model/api';
import db from '@/services/db';
import { groupByFiltered } from '@/utils/array';
import { displayDateTime, displayTime } from '@/utils/date';
import { refresh } from '@/utils/refresh';
import { uuid } from '@/utils/uuid';
import {
  IonButton,
  IonButtons,
  IonContent,
  IonHeader,
  IonIcon,
  IonItem,
  IonItemDivider,
  IonItemGroup,
  IonLabel,
  IonList,
  IonPage,
  IonTitle,
  IonToolbar
} from '@ionic/vue';
import { shallowRef } from 'vue';
import { useModal } from 'vue-final-modal';
import { useRouter } from 'vue-router';
import { addOutline } from 'ionicons/icons';
import { itemLines } from '@/utils/style';

const router = useRouter();

const alertModal = useModal({
  component: AlertModal,
  attrs: {
    title: 'You have incomplete workouts',
    message: 'Finish your current workout before starting a new one.',
    onOk: () => alertModal.close()
  }
});

const workouts = shallowRef<Workout[][]>([]);
let hasIncomplete = false;

async function load() {
  // For extracting out the incomplete workouts, we might get away with only
  // looking at the first element because of the way they're sorted but merges
  // could make things weird so we'll check the whole array.

  const { groups, filtered } = groupByFiltered(await db.getWorkouts(), workout => {
    if (workout.start_time && workout.finish_time) {
      return workout.start_time.substring(0, 4);
    } else {
      return undefined;
    }
  });

  hasIncomplete = !!filtered.length;
  if (hasIncomplete) groups.unshift(filtered);
  workouts.value = groups;
}

refresh(load);

function add() {
  if (hasIncomplete) {
    alertModal.open();
    return;
  }
  router.push(`/workouts/${uuid()}`);
}

function groupLabel(group: Workout[]) {
  return group[0].start_time && group[0].finish_time
    ? group[0].start_time.substring(0, 4)
    : 'In-progress';
}
</script>

<template>
  <IonPage>
    <IonHeader>
      <IonToolbar>
        <IonTitle>Workouts</IonTitle>
        <IonButtons slot="end">
          <IonButton @click="add">
            <IonIcon slot="icon-only" :icon="addOutline"></IonIcon>
          </IonButton>
        </IonButtons>
      </IonToolbar>
    </IonHeader>

    <IonContent :fullscreen="true">
      <IonHeader collapse="condense">
        <IonToolbar>
          <IonTitle size="large">Workouts</IonTitle>
        </IonToolbar>
      </IonHeader>

      <IonList class="mt-3">
        <IonItemGroup v-for="group, g in workouts">
          <IonItemDivider class="sticky top-0">
            <IonLabel>{{ groupLabel(group) }}</IonLabel>
          </IonItemDivider>

          <IonItem
            v-for="workout, w in group"
            button
            :detail="true"
            :lines="itemLines(workouts, g, w)"
            @click="router.push(`/workouts/${workout.workout_id}`)"
          >
            <IonLabel>
              <h3>
                <template v-if="workout.start_time">
                  <time :d="workout.start_time">{{
                    displayDateTime(workout.start_time)
                  }}</time>
                  -
                  <time v-if="workout.finish_time" :d="workout.finish_time">{{
                    // If the workout started and finished on the same day,
                    // don't show the date twice.
                    workout.start_time.substring(0, 10) === workout.finish_time.substring(0, 10)
                      ? displayTime(workout.finish_time)
                      : displayDateTime(workout.finish_time)
                  }}</time>
                  <i v-else>Not finished</i>
                </template>
                <i v-else>Not started</i>
              </h3>
              <p v-if="workout.notes">{{ workout.notes }}</p>
            </IonLabel>
          </IonItem>
        </IonItemGroup>
      </IonList>
    </IonContent>
  </IonPage>
</template>
