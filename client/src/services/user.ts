import {
  splitWorkoutExerciseId,
  type Exercise,
  type MeasurementSet,
  type UserChanges,
  type Workout
} from '@/model/api';
import auth from './auth';

const BASE_URL = import.meta.env.CFN_ApiBaseUrl + '/';

export class CacheOutdatedError extends Error {}

export class BadResponseError extends Error {}

export default new class {
  private async getHeaders(json: boolean = false): Promise<HeadersInit> {
    const headers: HeadersInit = { Authorization: await auth.getAccessToken() };
    if (json) headers['Content-Type'] = 'application/json';
    return headers;
  }

  async getChanges(sinceVersion: number): Promise<UserChanges> {
    const res = await fetch(`${BASE_URL}user?since=${sinceVersion}`, {
      method: 'GET',
      headers: await this.getHeaders()
    });

    if (!res.ok) throw new BadResponseError();

    return await res.json();
  }

  async deleteMeasurement(version: number, measurementId: string): Promise<void> {
    const res = await fetch(`${BASE_URL}user/measurement/${measurementId}`, {
      method: 'DELETE',
      headers: await this.getHeaders(true),
      body: JSON.stringify({ version })
    });

    if (res.status === 409) throw new CacheOutdatedError();
    if (!res.ok) throw new BadResponseError();
  }

  async updateMeasurement(
    version: number,
    m: MeasurementSet
  ): Promise<void> {
    const res = await fetch(`${BASE_URL}user/measurement/${m.date}`, {
      method: 'PUT',
      headers: await this.getHeaders(true),
      body: JSON.stringify({
        version,
        item: {
          notes: m.notes,
          measurements: m.measurements
        }
      })
    });

    if (res.status === 409) throw new CacheOutdatedError();
    if (!res.ok) throw new BadResponseError();
  }

  async deleteWorkout(version: number, workoutId: string): Promise<void> {
    const res = await fetch(`${BASE_URL}user/workout/${workoutId}`, {
      method: 'DELETE',
      headers: await this.getHeaders(true),
      body: JSON.stringify({ version })
    });

    if (res.status === 409) throw new CacheOutdatedError();
    if (!res.ok) throw new BadResponseError();
  }

  async updateWorkout(
    version: number,
    w: Workout
  ): Promise<void> {
    const res = await fetch(`${BASE_URL}user/workout/${w.workout_id}`, {
      method: 'PUT',
      headers: await this.getHeaders(true),
      body: JSON.stringify({
        version,
        item: {
          start_time: w.start_time,
          finish_time: w.finish_time,
          notes: w.notes
        }
      })
    });

    if (res.status === 409) throw new CacheOutdatedError();
    if (!res.ok) throw new BadResponseError();
  }

  async deleteExercise(
    version: number,
    workoutExerciseId: Exercise['workout_exercise_id'],
  ): Promise<void> {
    const { workoutId, exerciseId } = splitWorkoutExerciseId(workoutExerciseId);
    const res = await fetch(`${BASE_URL}user/workout/${workoutId}/exercise/${exerciseId}`, {
      method: 'DELETE',
      headers: await this.getHeaders(true),
      body: JSON.stringify({ version })
    });

    if (res.status === 409) throw new CacheOutdatedError();
    if (!res.ok) throw new BadResponseError();
  }

  async updateExercise(
    version: number,
    exercise: Exercise
  ): Promise<void> {
    const { workoutId, exerciseId } = splitWorkoutExerciseId(exercise.workout_exercise_id);
    const res = await fetch(`${BASE_URL}user/workout/${workoutId}/exercise/${exerciseId}`, {
      method: 'PUT',
      headers: await this.getHeaders(true),
      body: JSON.stringify({
        version,
        item: {
          order: exercise.order,
          type: exercise.type,
          notes: exercise.notes,
          sets: exercise.sets
        }
      })
    });

    if (res.status === 409) throw new CacheOutdatedError();
    if (!res.ok) throw new BadResponseError();
  }

  async updateExerciseOrder(
    version: number,
    workoutId: string,
    exercises: string[]
  ): Promise<void> {
    const res = await fetch(`${BASE_URL}user/workout/${workoutId}/order`, {
      method: 'PUT',
      headers: await this.getHeaders(true),
      body: JSON.stringify({
        version,
        item: exercises
      })
    });

    if (res.status === 409) throw new CacheOutdatedError();
    if (!res.ok) throw new BadResponseError();
  }
}
