import type { Exercise, MeasurementSet, Workout } from './api';

export const DELETED = { deleted: true } as const;

export type Deleted = typeof DELETED;

type Conflict<T, V, I extends keyof V> = {
  type: T;
  id: V[I];
  /** The last known remote version. */
  original: V | Deleted;
  /** The staged local change. */
  local: V | Deleted;
  /** The new remote change. */
  remote: V | Deleted;
};

export type MergeConflict =
  | Conflict<'measurement', MeasurementSet, 'date'>
  | Conflict<'workout', Workout, 'workout_id'>
  | Conflict<'exercise', Exercise, 'workout_exercise_id'>;

export type MergeConflictResolutions = {
  [key in string]?: 'local' | 'remote';
};

export type StagedChange = {
  version: number;
} & ({
  measurementId: string;
  measurement: MeasurementSet | Deleted;
} | {
  workoutId: string;
  workout: Workout | Deleted;
} | {
  workoutId: string;
  exerciseId: string;
  exercise: Exercise | Deleted;
});
