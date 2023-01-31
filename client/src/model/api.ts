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
  type: RepeatingExerciseType;
  sets: RepeatingSet[];
} | {
  type: VariableExerciseType;
  sets: VariableSet[];
} | {
  type: FixedExerciseType;
  sets: FixedSet[];
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

// exercises that involve repeating an action multiple times
export const REPEATING_EXERCISE_TYPES = [
  'biceps-curl',
  'chest-press',
  'dumbbell-wrist-curl',
  'fixed-pulldown',
  'leg-extension',
  'pectoral-fly',
  'seated-row',
  'shoulder-press',
  'triceps-extension',
] as const;

export type RepeatingExerciseType = typeof REPEATING_EXERCISE_TYPES[number];

// exercises whose speed is variable and depends on how the user performs
export const VARIABLE_EXERCISE_TYPES = [
  'elliptical',
  'recumbent-bike',
  'upright-bike',
] as const;

export type VariableExerciseType = typeof VARIABLE_EXERCISE_TYPES[number];

// exercises whose speed is fixed and depends on the machine
export const FIXED_EXERCISE_TYPES = [
  'treadmill',
] as const;

export type FixedExerciseType = typeof FIXED_EXERCISE_TYPES[number];

export type ExerciseType =
  | RepeatingExerciseType
  | VariableExerciseType
  | FixedExerciseType;

// TODO: is there a way to use the type system to ensure that each exercise type
// is mentioned exactly once?
export const EXERCISE_TYPE_GROUPS = {
  arms: [
    'biceps-curl',
    'chest-press',
    'dumbbell-wrist-curl',
    'fixed-pulldown',
    'pectoral-fly',
    'seated-row',
    'shoulder-press',
    'triceps-extension',
  ],
  legs: [
    'leg-extension',
  ],
  cardio: [
    'elliptical',
    'recumbent-bike',
    'treadmill',
    'upright-bike',
  ],
} as const satisfies { [key: string]: readonly ExerciseType[] };

export type ExerciseTypeGroup = keyof typeof EXERCISE_TYPE_GROUPS;

interface Set {
  set_id: string;
}

export interface RepeatingSet extends Set {
  repetitions: number;
  resistance: number;
}

export interface VariableSet extends Set {
  resistance: number;
  distance: number;
  duration: number;
}

export interface FixedSet extends Set {
  resistance: number;
  speed: number;
  // distance is calculated from speed and duration but user can override based
  // on machine display
  distance: number;
  duration: number;
}
