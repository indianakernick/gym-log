<script setup lang="ts">
import { shallowRef } from 'vue';
import { VueFinalModal } from 'vue-final-modal';

// TODO: validate that start is before finish and they're both in the past.

const props = defineProps<{
  start: string;
  finish: string;
}>();

const emit = defineEmits<{
  (e: 'cancel'): void;
  (e: 'save', start: string, finish: string): void;
}>();

const tz = new Date().getTimezoneOffset() * 60000;

function toInputDateTime(dateTimeStr: string) {
  return (new Date(Date.parse(dateTimeStr) - tz)).toISOString().substring(0, 19);
}

function fromInputDateTime(iso: string): string {
  // The date string does not have a Z suffix so the Date constructor is
  // interpreting it as a local date. No need to add the time zone offset.
  return new Date(iso).toISOString().substring(0, 19) + 'Z';
}

const start = shallowRef(toInputDateTime(props.start));
const finish = shallowRef(toInputDateTime(props.finish));

function save() {
  emit('save', fromInputDateTime(start.value), fromInputDateTime(finish.value));
}
</script>

<template>
  <VueFinalModal
    class="flex justify-center items-center"
    content-class="w-full m-6 bg-neutral-800 rounded-lg border border-neutral-600"
    :overlay-transition="{
      enterActiveClass: 'transition-opacity',
      enterFromClass: 'opacity-0',
      leaveActiveClass: 'transition-opacity',
      leaveToClass: 'opacity-0'
    }"
    :content-transition="{
      enterActiveClass: 'transition-[opacity,transform]',
      enterFromClass: 'scale-50 opacity-0',
      leaveActiveClass: 'transition-[opacity,transform]',
      leaveToClass: 'scale-50 opacity-0'
    }"
  >
    <h2 class="text-lg font-bold mb-3 px-3 pt-2">Adjust Dates</h2>

    <div class="mb-3 px-3 flex items-center justify-between">
      <label for="start">Started</label>
      <input
        id="started"
        type="datetime-local"
        v-model="start"
        step="1"
        class="appearance-none px-2 py-1 rounded-lg dark:bg-neutral-700"
      />
    </div>

    <div class="px-3 flex items-center justify-between">
      <label for="finished">Finished</label>
      <input
        id="finished"
        type="datetime-local"
        v-model="finish"
        step="1"
        class="appearance-none px-2 py-1 rounded-lg dark:bg-neutral-700"
      />
    </div>

    <div class="mt-3 grid grid-cols-2 border-t border-neutral-600 text-blue-500">
      <button @click="$emit('cancel')" class="p-2 border-r border-neutral-600">Cancel</button>
      <button @click="save" class="p-2 font-bold">Save</button>
    </div>
  </VueFinalModal>
</template>
