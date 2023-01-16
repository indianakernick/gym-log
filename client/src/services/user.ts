import auth from './auth';

const BASE_URL = 'https://pa36mmpygd.execute-api.ap-southeast-2.amazonaws.com/';

export default new class {
  async getUser(): Promise<User> {
    const res = await fetch(`${BASE_URL}user`, {
      headers: {
        Authorization: await auth.getAccessToken(),
      }
    });

    if (!res.ok) {
      throw 'something...';
    }

    return JSON.parse(await res.json())
  }
};

export interface User {
  measurements: Measurement[];
  workouts: Workout[];
}

export interface Measurement {
  measurement_id: string;
  type: MeasurementType;
  capture_date: string;
  value: number;
  notes: string;
}

export type MeasurementType =
  | 'weight'
  | 'list'
  | 'of'
  | 'body'
  | 'parts';

export interface Workout {
  workout_id: string;
  start_time: string | null;
  finish_time: string | null;
  notes: string;
  exercises: Exercise[];
}

export type Exercise = {
  exercise_id: string;
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
  order: number;
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
