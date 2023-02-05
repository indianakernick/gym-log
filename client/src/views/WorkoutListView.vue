<script setup lang="ts">
import Header from '@/components/Header.vue';
import ListGroup from '@/components/ListGroup.vue';
import ListItem from '@/components/ListItem.vue';
import Main from '@/components/Main.vue';
import type { Workout } from '@/model/api';
import db from '@/services/db';
import { groupByFiltered } from '@/utils/binary-search';
import { displayDateTime } from '@/utils/date';
import { uuid } from '@/utils/uuid';
import { PlusIcon } from '@heroicons/vue/24/outline';
import { shallowRef } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

const workouts = shallowRef<Workout[][]>([]);
let hasIncomplete = false;

db.getWorkouts().then(d => {
  const incomplete: Workout[] = [];

  // We might get away with just looking at the first one because of the way
  // they're sorted but merges could make things weird.
  for (const workout of d) {
    if (!workout.start_time || !workout.finish_time) {
      incomplete.push(workout);
    }
  }

  hasIncomplete = !!incomplete.length;

  const groups = groupByFiltered(d, workout => {
    if (workout.start_time && workout.finish_time) {
      return workout.start_time.substring(0, 4);
    } else {
      return undefined;
    }
  });

  if (incomplete.length) {
    groups.unshift(incomplete);
  }

  workouts.value = groups;
});

function add() {
  // We might get away with just looking at the first one because of the way
  // they're sorted but merges could make things weird.
  if (hasIncomplete) {
    alert('You have incomplete workouts!');
    return;
  }
  router.push(`/workouts/${uuid()}`);
}

// Do this from the individual workout page.
/*
function deleteWorkout(index: number) {
  db.stageDeleteWorkout(workouts.value[index].workout_id).then(() => sync.sync());
  workouts.value.splice(index);
  triggerRef(workouts);
}
*/
</script>

<template>
  <Header title="Workouts" @right="add">
    <template #right>
      <PlusIcon class="w-6 h-6"></PlusIcon>
    </template>
  </Header>

  <Main>
    <ol>
      <li v-for="group in workouts">
        <ListGroup>
          <ListItem
            v-for="workout in group"
            @click="router.push(`/workouts/${workout.workout_id}`)"
          >
            <div class="min-w-0">
              <div>
                <time v-if="workout.start_time" :d="workout.start_time">{{
                  displayDateTime(workout.start_time)
                }}</time>
                <i v-else>Not started</i>
                -
                <time v-if="workout.finish_time" :d="workout.finish_time">{{
                  displayDateTime(workout.finish_time)
                }}</time>
                <i v-else>Not finished</i>
              </div>
              <div
                v-if="workout.notes"
                class="text-sm text-ellipsis overflow-hidden whitespace-nowrap
                  dark:text-neutral-400"
              >{{ workout.notes }}</div>
            </div>
          </ListItem>
        </ListGroup>
      </li>
    </ol>
  </Main>
</template>
