import {
  splitWorkoutExerciseId,
  type Exercise,
  type Measurement,
  type UserChanges,
  type Workout
} from '@/model/api';
import auth from './auth';

const BASE_URL = 'https://pa36mmpygd.execute-api.ap-southeast-2.amazonaws.com/';

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

    if (!res.ok) {
      throw 'something...';
    }

    return JSON.parse(await res.json());
  }

  async deleteMeasurement(version: number, measurementId: string): Promise<void> {
    const res = await fetch(`${BASE_URL}user/measurement/${measurementId}`, {
      method: 'DELETE',
      headers: await this.getHeaders(true),
      body: JSON.stringify({ version })
    });
  }

  async updateMeasurement(
    version: number,
    m: Measurement
  ): Promise<void> {
    const res = await fetch(`${BASE_URL}user/measurement/${m.measurement_id}`, {
      method: 'PUT',
      headers: await this.getHeaders(true),
      body: JSON.stringify({
        version,
        item: {
          type: m.type,
          capture_date: m.capture_date,
          value: m.value,
          notes: m.notes
        }
      })
    });
  }

  async deleteWorkout(version: number, workoutId: string): Promise<void> {
    const res = await fetch(`${BASE_URL}user/workout/${workoutId}`, {
      method: 'DELETE',
      headers: await this.getHeaders(true),
      body: JSON.stringify({ version })
    });
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
  }
}
