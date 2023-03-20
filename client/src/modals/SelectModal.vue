<script setup lang="ts">
import { uuid } from '@/utils/uuid';
import {
  IonButton,
  IonButtons,
  IonContent,
  IonHeader,
  IonIcon,
  IonTitle,
  IonToolbar,
  modalController
} from '@ionic/vue';
import { checkmark } from 'ionicons/icons';
import { shallowRef } from 'vue';

export type Value = string;

export interface Option {
  title: string;
  value: Value;
}

export interface OptionGroup {
  title: string;
  options: Option[];
}

defineProps<{
  title: string;
  selectTitle?: string;
  groups: OptionGroup[];
}>();

const choice = shallowRef<Value>();
const id = uuid();
</script>

<template>
  <IonHeader>
    <IonToolbar>
      <IonButtons slot="start">
        <IonButton @click="modalController.dismiss()">Cancel</IonButton>
      </IonButtons>
      <IonTitle>{{ title }}</IonTitle>
      <IonButtons slot="end">
        <IonButton
          @click="modalController.dismiss(choice)"
          :disabled="!choice"
          :strong="true"
        >{{ selectTitle ?? 'Select' }}</IonButton>
      </IonButtons>
    </IonToolbar>
  </IonHeader>

  <IonContent>
    <div class="my-3">
      <div v-for="group, g in groups" class="mt-4 first:mt-0">
        <h3
          class="mx-3 mb-1 text-sm text-neutral-500 dark:text-neutral-400"
        >{{ group.title }}</h3>
        <div class="flex flex-col">
          <div
            v-for="option, o in group.options"
            class="flex items-center mx-3 p-2 gap-2
              first:rounded-t-lg last:rounded-b-lg
              border-x border-t last:border-b border-neutral-300 dark:border-neutral-600
              bg-neutral-100 dark:bg-neutral-800"
          >
            <label :for="`${id}-${g}-${o}`" class="grow">{{ option.title }}</label>
            <div class="relative w-5 h-5">
              <input
                type="radio"
                :name="id"
                :id="`${id}-${g}-${o}`"
                @input="($event.target as HTMLInputElement).checked && (choice = option.value)"
                class="opacity-0 peer"
              />
              <IonIcon
                :icon="checkmark"
                class="absolute top-0 left-0 w-5 h-5 opacity-0 peer-checked:opacity-100
                  text-blue-500"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </IonContent>
</template>
