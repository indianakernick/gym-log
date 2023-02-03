<script setup lang="ts">
import Header from '@/components/Header.vue';
import Main from '@/components/Main.vue';
import db from '@/services/db';
import sync from '@/services/sync';
import { displayDate, toDateString } from '@/utils/date';
import { ChevronRightIcon } from '@heroicons/vue/20/solid';
import { CalendarIcon, PlusIcon, TrashIcon } from '@heroicons/vue/24/solid';
import { ref, shallowRef, triggerRef } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

const dates = shallowRef<string[]>([]);

db.getMeasurementDates().then(d => {
  dates.value = d;
});

const items = ref<HTMLElement[]>([]);

function addToday() {
  router.push(`/measurements/${toDateString(new Date())}`);
}

function addPast(event: Event) {
  const date = (event.target as HTMLInputElement | null)?.valueAsDate;
  if (date) {
    // Safari doesn't support the max attribute. Also, it will emit the change
    // event with today's date when the picker is opened.
    const today = toDateString(new Date());
    const selected = toDateString(date);
    if (selected < today) {
      router.push(`/measurements/${toDateString(date)}`);
    }
  }
}

function deleteSet(index: number) {
  db.stageDeleteMeasurement(dates.value[index]).then(() => sync.sync());
  dates.value.splice(index, 1);
  triggerRef(dates);
}

// TODO: various improvements could be made to this:
//  - 80px comes from tailwind
//  - extract this out into a component
//  - use transitions to make it feel nicer
//  - perhaps have the delete button grow (with the icon always in the center)
//  - multiple fingers

// Or maybe this is to complicated and we should just put a button on the
// measurement editing page?

let dragging: number | undefined = undefined;
let open: number | undefined = undefined;
let dragStartX = 0;
let dragOffsetX = 0;

function start(index: number, event: TouchEvent) {
  if (open !== undefined) {
    items.value[open].style.transform = '';
  }
  dragging = index;
  dragStartX = event.touches[0].clientX;
}

function move(index: number, event: TouchEvent) {
  if (index === dragging) {
    dragOffsetX = Math.max(-80, Math.min(0, event.touches[0].clientX - dragStartX));
    items.value[dragging].style.transform = `translateX(${dragOffsetX}px)`;
  }
}

function end(index: number, event: TouchEvent) {
  if (index === dragging) {
    if (dragOffsetX <= -80 * 0.8) {
      items.value[dragging].style.transform = `translateX(-80px)`;
      open = dragging;
    } else {
      items.value[dragging].style.transform = '';
      open = undefined;
    }
    dragging = undefined;
  }
}

function cancel(index: number, event: TouchEvent) {
  if (index === dragging) {
    items.value[dragging].style.transform = '';
    open = undefined;
  }
}
</script>

<template>
  <Header title="Measurements" @right="addToday">
    <template #left>
      <CalendarIcon class="w-6 h-6" />
      <!--
        Safari 15 doesn't support the showPicker method so we're putting the
        date picker on top of the button.
      -->
      <input
        class="opacity-0 absolute top-0 right-0"
        type="date"
        :max="toDateString(new Date())"
        @change="addPast"
      />
    </template>
    <template #right>
      <PlusIcon class="w-6 h-6" />
    </template>
  </Header>

  <Main>
    <ol class="my-2">
      <li
        v-for="date, i in dates"
        class="border-t last:border-b border-neutral-500"
      >
        <div
          class="relative"
          ref="items"
          @touchstart="start(i, $event)"
          @touchmove="move(i, $event)"
          @touchend="end(i, $event)"
          @touchcancel="cancel(i, $event)"
        >
          <button
            class="px-3 py-2 w-full flex justify-between items-center"
            @click="router.push(`/measurements/${date}`)"
          >
            {{ displayDate(date) }}
            <ChevronRightIcon class="w-5 h-5 text-neutral-500" />
          </button>
          <button
            @click="deleteSet(i)"
            class="bg-red-500 absolute -right-20 top-0 h-full w-20 flex justify-center items-center"
          >
            <TrashIcon class="w-6 h-6 text-white"/>
          </button>
        </div>
      </li>
    </ol>
  </Main>
</template>
