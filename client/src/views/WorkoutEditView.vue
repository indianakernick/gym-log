<script setup lang="ts">
import ExerciseEdit from '@/components/ExerciseEdit.vue';
import Header from '@/components/Header.vue';
import Main from '@/components/Main.vue';
import Menu from '@/components/Menu.vue';
import TextArea from '@/components/TextArea.vue';
import AdjustDatesModal from '@/modals/AdjustDatesModal.vue';
import {
  exerciseEqual,
  EXERCISE_TYPE_GROUPS,
  workoutEqual,
  type Exercise,
  type ExerciseType,
  type Workout
} from '@/model/api';
import { back } from '@/router/back';
import db from '@/services/db';
import sync from '@/services/sync';
import { displayDateTime, toDateTimeString } from '@/utils/date';
import { EXERCISE_TYPE, EXERCISE_TYPE_GROUP } from '@/utils/i18n';
import { useConfirmModal } from '@/utils/modal';
import { refresh } from '@/utils/refresh';
import { uuid } from '@/utils/uuid';
import { PlusIcon, TrashIcon } from '@heroicons/vue/20/solid';
import { ChevronLeftIcon } from '@heroicons/vue/24/outline';
import { computed, shallowRef, triggerRef } from 'vue';
import { useModal } from 'vue-final-modal';
import { useRouter } from 'vue-router';

const props = defineProps<{
  id: string;
}>();

const router = useRouter();
const adjustDatesModal = useModal({
  component: AdjustDatesModal
});
const confirmModal = useConfirmModal();

const workout = shallowRef<Workout>({
  workout_id: props.id,
  notes: '',
  start_time: null,
  finish_time: null
});
const exercises = shallowRef<Exercise[]>([]);
const deletedExercises: Exercise['workout_exercise_id'][] = [];
const editing = shallowRef(false);
const editingExercise = shallowRef<number>();

function equal(dbWorkout: Workout, dbExercises: Exercise[]): boolean {
  if (!workoutEqual(workout.value, dbWorkout)) return false;
  if (exercises.value.length !== dbExercises.length) return false;

  for (let e = 0; e < dbExercises.length; ++e) {
    if (!exerciseEqual(exercises.value[e], dbExercises[e])) return false;
  }

  return true;
}

async function load(initial: boolean) {
  const [dbWorkout, dbExercises] = await Promise.all([
    db.getWorkout(props.id),
    db.getExercisesOfWorkout(props.id)
  ]);

  if (dbWorkout) {
    if (
      !initial
      && editing.value
      && !equal(dbWorkout, dbExercises)
      && await confirmModal({
        title: 'Keep edits?',
        message: 'Changes to this workout have been pulled from another device. Do you want to keep your local edits?',
        buttons: 'keep-discard'
      })
    ) return;
    workout.value = dbWorkout;
    editing.value = false;
  } else if (!initial) {
    back(router, '/workouts');
    return;
  }

  exercises.value = dbExercises;
}

refresh(load);

async function done() {
  if (!editing.value || await confirmModal({
    title: 'Keep edits?',
    message: 'Do you want to keep the changes made to this workout?',
    buttons: 'keep-discard'
  })) {
    if (!workout.value.notes && !exercises.value.length) {
      await db.stageDeleteWorkout(props.id);
    } else {
      await db.stageUpdateWorkout(workout.value);
      await Promise.all(exercises.value.map(e => db.stageUpdateExercise(e)));
      await Promise.all(deletedExercises.map(e => db.stageDeleteExercise(e)));
    }
    sync.sync();
  }
  back(router, '/workouts');
}

function start() {
  workout.value.start_time = toDateTimeString(new Date());
  triggerRef(workout);
  saveWorkout();
}

function finish() {
  workout.value.finish_time = toDateTimeString(new Date());
  triggerRef(workout);
  saveWorkout();
}

function addExercise(event: Event) {
  const select = event.target as HTMLSelectElement | null;
  if (select?.value) {
    const type = select.value as ExerciseType;
    select.value = '';
    exercises.value.push({
      workout_exercise_id: `${workout.value.workout_id}#${uuid()}`,
      order: exercises.value.length,
      type,
      notes: '',
      sets: []
    });
    triggerRef(exercises);
  }
}

const options = computed(() => {
  const items: InstanceType<typeof Menu>['items'] = [];

  if (workout.value.finish_time && !editing.value) {
    items.push({ title: 'Edit', handler: () => editing.value = true });
  }

  if (workout.value.start_time && workout.value.finish_time) {
    items.push({ title: 'Adjust Dates', handler: () => {
      adjustDatesModal.patchOptions({
        attrs: {
          start: workout.value.start_time!,
          finish: workout.value.finish_time!,
          onSave: (start, finish) => {
            workout.value.start_time = start;
            workout.value.finish_time = finish;
            triggerRef(workout);
            saveWorkout();
            adjustDatesModal.close();
          },
          onCancel: () => adjustDatesModal.close()
        }
      });
      adjustDatesModal.open();
    }});
  }

  items.push({
    title: 'Delete Workout',
    theme: 'danger',
    icon: TrashIcon,
    handler: deleteWorkout
  });

  return items;
});

async function deleteWorkout() {
  if (await confirmModal({
    title: 'Delete workout',
    message: 'Are you sure you want to delete this workout?',
    buttons: 'delete-cancel'
  })) {
    await db.stageDeleteWorkout(props.id);
    sync.sync();
    back(router, '/workouts');
  }
}

async function deleteExercise(index: number) {
  const id = exercises.value[index].workout_exercise_id;
  if (editing.value) {
    deletedExercises.push(id);
  } else {
    await db.stageDeleteExercise(id);
    sync.sync();
  }
  exercises.value.splice(index, 1);
  triggerRef(exercises);
}

async function saveWorkout() {
  await db.stageUpdateWorkout(workout.value);
  sync.sync();
}

async function saveExercise(exercise: Exercise) {
  await db.stageUpdateExercise(exercise);
  sync.sync();
}
</script>

<template>
  <Header
    title="Workout Details"
    @left="done"
  >
    <template #left>
      <ChevronLeftIcon class="w-6 h-6" />
    </template>
    <template #full-right>
      <Menu
        title="Workout Options"
        :items="options"
        theme="primary"
      ></Menu>
    </template>
  </Header>

  <Main>
    <TextArea
      label="Notes"
      v-model="workout.notes"
      @update:model-value="editing || saveWorkout()"
      :read-only="!!workout.finish_time && !editing"
      class="mx-3 my-2"
    ></TextArea>

    <div
      v-if="workout.start_time"
      class="px-3 py-2 flex justify-between"
    >
      <div>Started</div>
      <time :d="workout.start_time">{{ displayDateTime(workout.start_time) }}</time>
    </div>
    <button
      v-else
      @click="start"
      class="button-primary"
    >Start</button>

    <div
      v-if="workout.start_time && workout.finish_time"
      class="px-3 py-2 flex justify-between"
    >
      <div>Finished</div>
      <time :d="workout.finish_time">{{ displayDateTime(workout.finish_time) }}</time>
    </div>

    <ol class="flex flex-col gap-3 my-2">
      <li v-for="exercise, i in exercises">
        <ExerciseEdit
          :workout-start="workout.start_time!"
          :exercise="exercise"
          :editing-workout="!workout.finish_time || editing"
          :editing="i === (editingExercise ?? (exercises.length - 1))"
          @delete-exercise="deleteExercise(i)"
          @edit-exercise="editingExercise = i === exercises.length - 1 ? undefined : i"
          @exercise-changed="editing || saveExercise(exercise)"
        ></ExerciseEdit>
      </li>
    </ol>

    <template v-if="workout.start_time && !workout.finish_time">
      <!--
        Safari doesn't respond to text-align: center on <select> elements so we
        need to use this god awful hack. In the future this might be a regular
        button that opens a dialog so maybe it doesn't matter.
      -->
      <div
        class="relative mx-3 my-2 rounded-lg bg-neutral-800 border
          border-neutral-600"
      >
        <select
          @change="addExercise"
          class="py-2 w-full appearance-none opacity-0"
        >
          <option value="" disabled selected>Add Exercise</option>
          <optgroup v-for="group, name in EXERCISE_TYPE_GROUPS" :label="EXERCISE_TYPE_GROUP[name]">
            <option v-for="ty in group" :value="ty">{{ EXERCISE_TYPE[ty] }}</option>
          </optgroup>
        </select>

        <div
          class="absolute inset-0 pointer-events-none font-bold
            button-flex text-blue-500"
        >
          <PlusIcon class="w-5 h-5" />
          Add Exercise
        </div>
      </div>

      <button
        @click="finish"
        class="button-primary"
      >Finish</button>
    </template>
  </Main>
</template>
