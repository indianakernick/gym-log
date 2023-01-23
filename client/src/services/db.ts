import {
  openDB,
  type DBSchema,
  type IDBPDatabase,
  type IDBPTransaction,
  type StoreNames
} from 'idb';
import { AsyncInit } from '../utils/async-init';
import type { Exercise, Measurement, UserChanges, Workout } from './user';

interface Schema extends DBSchema {
  auth: {
    key: 'refresh_token';
    value: string;
  },
  user: {
    key: 'version';
    value: number;
  },
  measurement: {
    key: Measurement['measurement_id'];
    value: Measurement;
    indexes: {
      // Interpolate and extrapolate multiple measurements of the same type.
      type: Measurement['type'];
      // Display measurements by date.
      date: Measurement['capture_date'];
    }
  },
  workout: {
    key: Workout['workout_id'];
    value: Omit<Workout, 'exercises'>;
  },
  exercise: {
    key: `${Workout['workout_id']}#${Exercise['exercise_id']}`;
    value: Exercise;
  },
  stagedMeasurement: {
    key: Measurement['measurement_id'];
    value: Measurement | Deleted;
  },
  stagedWorkout: {
    key: Workout['workout_id'];
    value: Omit<Workout, 'exercises'> | Deleted;
  },
  stagedExercise: {
    key: `${Workout['workout_id']}#${Exercise['exercise_id']}`;
    value: Exercise | Deleted;
  },
}

export const DELETED = { deleted: true } as const;

export type Deleted = typeof DELETED;

export type MergeConflict = {
  type: 'measurement';
  local: Measurement | Deleted;
  remote: Measurement | Deleted;
} | {
  type: 'workout';
  local: Omit<Workout, 'exercises'> | Deleted;
  remote: Omit<Workout, 'exercises'> | Deleted;
} | {
  type: 'exercise';
  local: Exercise | Deleted;
  remote: Exercise | Deleted;
};

export type MergeConflictResolutions = {
  [key: string]: 'local' | 'remote';
};

export interface StagedChanges {
  version: number;
  measurements: (Measurement | Deleted)[];
  workouts: (Omit<Workout, 'exercises'> | Deleted)[];
  exercises: (Exercise | Deleted)[];
}

export default new class {
  private db = new AsyncInit<IDBPDatabase<Schema>>();

  constructor() {
    openDB<Schema>(
      'main',
      0,
      {
        upgrade(db) {
          db.createObjectStore('auth');
          db.createObjectStore('user');
          db.createObjectStore('measurement', { keyPath: 'measurement_id' });
          db.createObjectStore('workout', { keyPath: 'workout_id' });
          db.createObjectStore('exercise');
          db.createObjectStore('stagedMeasurement');
          db.createObjectStore('stagedWorkout');
          db.createObjectStore('stagedExercise');
        }
      }
    ).then(db => this.db.set(db));

    // Should call this at some point.
    // navigator.storage.persist()
  }

  async getRefreshToken(): Promise<string | undefined> {
    const db = await this.db.get();
    return db.get('auth', 'refresh_token');
  }

  async setRefreshToken(refreshToken: string): Promise<void> {
    const db = await this.db.get();
    await db.put('auth', refreshToken, 'refresh_token');
  }

  // The next six methods are used when the user is making changes to their data

  stageDeleteMeasurement(measurementId: string): Promise<void> {
    return this.stageDeletion('measurement', 'stagedMeasurement', measurementId);
  }

  async stageUpdateMeasurement(measurement: Measurement): Promise<void> {
    const db = await this.db.get();
    await db.put('stagedMeasurement', measurement, measurement.measurement_id);
  }

  stageDeleteWorkout(workoutId: string): Promise<void> {
    return this.stageDeletion('workout', 'stagedWorkout', workoutId);
  }

  async stageUpdateWorkout(workout: Omit<Workout, 'exercises'>): Promise<void> {
    const db = await this.db.get();
    await db.put('stagedWorkout', workout, workout.workout_id);
  }

  stageDeleteExercise(workoutId: string, exerciseId: string): Promise<void> {
    return this.stageDeletion('exercise', 'stagedExercise', `${workoutId}#${exerciseId}`);
  }

  async stageUpdateExercise(workoutId: string, exercise: Exercise): Promise<void> {
    const db = await this.db.get();
    await db.put('stagedExercise', exercise, `${workoutId}#${exercise.exercise_id}`);
  }

  private async stageDeletion<
    Canon extends StoreNames<Schema>,
    Staged extends StoreNames<Schema>
  >(
    canon: Canon,
    staged: Staged,
    id: Schema[Canon]['key'] & Schema[Staged]['key']
  ): Promise<void> {
    const db = await this.db.get();
    const tx = db.transaction([canon, staged], 'readwrite');

    if (await tx.objectStore(canon).count(id) > 0) {
      await tx.objectStore(staged).put(DELETED, id);
    } else {
      await tx.objectStore(staged).delete(id);
    }

    tx.commit();
  }

  // Called after receiving a response from GET /user
  // Could happen after periodically polling or after a modification request was
  // rejected
  async merge(
    remote: UserChanges,
    resolutions: MergeConflictResolutions = {}
  ): Promise<MergeConflict[]> {
    // merge changes from the remote with the changes in the staged stores.
    // if any merge conflicts are encountered, return the full list to the user
    // for them to resolve and try again later.
    // if this is successful, then all of the staged object stores will be empty
    return [];
  }

  // get the staged changes to prepare for upload
  async getStagedChanges(): Promise<StagedChanges> {
    throw 'todo';
  }

  // uploading a staged change has succeeded so move the data out of the staged
  // object stores. the staged entities might have changed while they were being
  // uploaded. in that case, they should remain in the staging area.
  // uploads will happen from one place (service worker) but the staging area
  // could change from multiple tabs.

  // could put a modification time or a UUID on staged items whenever they change
  // when they finish uploading, the tag can be compared and only if the tags differ
  // will the item be removed from the staging area. it's possible that the item
  // could be completely removed from the staging area, in that case, a DELETED
  // object will need to be put in its place

  async applyDeleteMeasurement(measurementId: string): Promise<void> {
    return this.applyDeletion('measurement', 'stagedMeasurement', measurementId);
  }

  async applyDeleteWorkout(workoutId: string): Promise<void> {
    return this.applyDeletion('workout', 'stagedWorkout', workoutId);
  }

  async applyDeleteExercise(workoutId: string, exerciseId: string): Promise<void> {
    return this.applyDeletion('exercise', 'stagedExercise', `${workoutId}#${exerciseId}`);
  }

  private async incrementVersion<
    Stores extends StoreNames<Schema>
  >(tx: IDBPTransaction<Schema, (Stores | 'user')[], 'readwrite'>) {
    const store = tx.objectStore('user');
    const version = await store.get('version') || 0;
    await store.put(version + 1, 'version');
  }

  private async applyDeletion<
    Canon extends StoreNames<Schema>,
    Staged extends StoreNames<Schema>
  >(
    canon: Canon,
    staged: Staged,
    id: Schema[Canon]['key'] & Schema[Staged]['key']
  ): Promise<void> {
    const db = await this.db.get();
    const tx = db.transaction(['user', canon, staged], 'readwrite');
    const stagedStore = tx.objectStore(staged);

    this.incrementVersion(tx);

    const stagedItem = stagedStore.get(id);
    if (stagedItem && 'deleted' in stagedItem) {
      stagedStore.delete(id);
    }

    tx.objectStore(canon).delete(id);
    tx.commit();
  }
}
