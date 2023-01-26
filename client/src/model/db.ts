import type { Exercise, Measurement, Workout } from './api';

export const DELETED = { deleted: true } as const;

export type Deleted = typeof DELETED;

export type MergeConflict = {
  type: 'measurement';
  id: Measurement['measurement_id'];
  local: Measurement | Deleted;
  remote: Measurement | Deleted;
} | {
  type: 'workout';
  id: Workout['workout_id'];
  local: Workout | Deleted;
  remote: Workout | Deleted;
} | {
  type: 'exercise';
  id: Exercise['workout_exercise_id'];
  local: Exercise | Deleted;
  remote: Exercise | Deleted;
};

export type MergeConflictResolutions = {
  [key in string]?: 'local' | 'remote';
};

export type StagedChange = {
  version: number;
} & ({
  measurementId: string;
  measurement: Measurement | Deleted;
} | {
  workoutId: string;
  workout: Workout | Deleted;
} | {
  workoutId: string;
  exerciseId: string;
  exercise: Exercise | Deleted;
});
