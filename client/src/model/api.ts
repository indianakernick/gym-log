export interface UserChanges {
  version: number;
  measurement_sets: MeasurementSet[];
  workouts: Workout[];
  exercises: Exercise[];
  deleted_measurement_sets: MeasurementSet['date'][];
  deleted_workouts: Workout['workout_id'][];
  deleted_exercises: Exercise['workout_exercise_id'][];
}

export interface MeasurementSet {
  date: string;
  notes: string;
  measurements: { [key in MeasurementType]?: number };
}

export function measurementSetEqual(a: MeasurementSet, b: MeasurementSet): boolean {
  if (a.notes !== b.notes) return false;

  const aKeys = Object.keys(a.measurements) as MeasurementType[];
  const bKeys = Object.keys(b.measurements) as MeasurementType[];

  if (aKeys.length !== bKeys.length) return false;

  aKeys.sort();
  bKeys.sort();

  for (let k = 0; k < aKeys.length; ++k) {
    if (aKeys[k] !== bKeys[k]) return false;
    if (a.measurements[aKeys[k]] !== b.measurements[aKeys[k]]) return false;
  }

  return true;
}

export const MEASUREMENT_TYPES = [
  'weight',
  'height',
  'arm-right-upper',
  'arm-right-lower',
  'arm-left-upper',
  'arm-left-lower',
  'leg-right-upper',
  'leg-right-lower',
  'leg-left-upper',
  'leg-left-lower'
] as const;

export type MeasurementType = typeof MEASUREMENT_TYPES[number];

export interface Workout {
  workout_id: string;
  start_time: string | null;
  finish_time: string | null;
  notes: string;
}

export function workoutEqual(a: Workout, b: Workout): boolean {
  return a.start_time === b.start_time
    && a.finish_time === b.finish_time
    && a.notes === b.notes;
}

export type Exercise = {
  workout_exercise_id: `${string}#${string}`;
  order: number;
  notes: string;
} & ({
  type: LiftingExerciseType;
  sets: LiftingSet[];
} | {
  type: BikeExerciseType;
  sets: BikeSet[];
} | {
  type: TreadmillExerciseType;
  sets: TreadmillSet[];
});

export function exerciseEqual(a: Exercise, b: Exercise): boolean {
  return a.order === b.order
    && a.type === b.type
    && a.notes === b.notes
    && JSON.stringify(a.sets) === JSON.stringify(b.sets);
}

export function splitWorkoutExerciseId(workoutExerciseId: Exercise['workout_exercise_id']) {
  return {
    workoutId: workoutExerciseId.substring(0, 36),
    exerciseId: workoutExerciseId.substring(37)
  };
}

export type LiftingExerciseType =
  | 'list'
  | 'of'
  | 'lifting'
  | 'exercises';

export type BikeExerciseType =
  | 'elliptical'
  | 'recumbent_bike'
  | 'upright_bike';

export type TreadmillExerciseType = 'treadmill';

interface Set {
  set_id: string;
}

export interface LiftingSet extends Set {
  repetitions: number;
  resistance: number;
}

export interface BikeSet extends Set {
  resistance: number;
  distance: number;
  duration: number;
}

export interface TreadmillSet extends Set {
  resistance: number;
  speed: number;
  // distance is calculated from speed and duration but user can override based
  // on machine display
  distance: number;
  duration: number;
}
