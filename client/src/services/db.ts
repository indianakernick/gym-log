import {
  openDB,
  type DBSchema,
  type IDBPDatabase,
  type IDBPTransaction,
  type StoreNames
} from 'idb';
import { AsyncInit } from '../utils/async-init';
import type { Exercise, Measurement, UserChanges, Workout } from './user';

// When the user is making changes to their data, the changes are written to a
// staging area. The staging area stores the difference between the local copy
// of remote database (the canonical database) and the local database. The user
// always sees the local version of the database (unless they're resolving merge
// conflicts) which means that they see the canonical version with the staged
// changes applied.
//
// Changes from the staging area will be uploaded from time to time. If a
// different client made changes to the remote database since this client last
// fetched it, then the request to apply changes will be rejected and the client
// will need to fetch the remote changes. If the request to push changes to the
// remote succeeds, then the change is removed the from the staging area and
// applied to the canonical version. The version number is also incremented so
// that the client knows the version of the remote database that is has.
//
// When a request to apply changes to the remote is rejected, the client will
// fetch the new set of changes to be applied to its canonical database. At this
// point, there will be two sets of changes relative to the canonical database.
// If the sets of changes involve different objects, then they can be trivially
// merged. However, if the same object is referenced twice (perhaps modified in
// one and deleted in the other), then the user will be asked which change
// should be applied and the merging process can start again with these
// choices.

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

export type StagedChange = {
  version: number;
} & ({
  measurementId: string;
  measurement: Measurement | Deleted;
} | {
  workoutId: string;
  workout: Omit<Workout, 'exercises'> | Deleted;
} | {
  workoutId: string;
  exerciseId: string;
  exercise: Exercise | Deleted;
});

type StagedStores = {
  measurement: 'stagedMeasurement',
  workout: 'stagedWorkout',
  exercise: 'stagedExercise',
};

export default new class {
  // It would probably make more sense to have an async factory function instead
  // of checking and waiting for the initialisation of this object every time
  // it's accessed.
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

  // ---------------------------- Authentication ---------------------------- //

  async getRefreshToken(): Promise<string | undefined> {
    const db = await this.db.get();
    return db.get('auth', 'refresh_token');
  }

  async setRefreshToken(refreshToken: string): Promise<void> {
    const db = await this.db.get();
    await db.put('auth', refreshToken, 'refresh_token');
  }

  // ----------------------- Stage changes for upload ----------------------- //
  //
  // Stage changes to the local database and prepare them to be uploaded to the
  // remote database.

  stageDeleteMeasurement(measurementId: string): Promise<void> {
    return this.stageDelete('measurement', 'stagedMeasurement', measurementId);
  }

  async stageUpdateMeasurement(measurement: Measurement): Promise<void> {
    const db = await this.db.get();
    await db.put('stagedMeasurement', measurement, measurement.measurement_id);
  }

  stageDeleteWorkout(workoutId: string): Promise<void> {
    return this.stageDelete('workout', 'stagedWorkout', workoutId);
  }

  async stageUpdateWorkout(workout: Omit<Workout, 'exercises'>): Promise<void> {
    const db = await this.db.get();
    await db.put('stagedWorkout', workout, workout.workout_id);
  }

  stageDeleteExercise(workoutId: string, exerciseId: string): Promise<void> {
    return this.stageDelete('exercise', 'stagedExercise', `${workoutId}#${exerciseId}`);
  }

  async stageUpdateExercise(workoutId: string, exercise: Exercise): Promise<void> {
    const db = await this.db.get();
    await db.put('stagedExercise', exercise, `${workoutId}#${exercise.exercise_id}`);
  }

  private async stageDelete<S extends keyof StagedStores>(
    canon: S,
    staged: StagedStores[S],
    id: Schema[S]['key']
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

  // --------------- Merge remote changes with staged changes --------------- //
  //
  // Merge a set of remote changes into the staged changes. This could happen
  // after periodically polling or after a modification request was rejected.
  // The user will be asked to resolve merge conflicts if they arise.

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

  // --------------------- Get a staged change to upload -------------------- //
  //
  // Individual changes are uploaded one at a time. After uploading, the changes
  // are applied. When preparing to upload the next change, the staged changes
  // could have been changed by a user action so it needs to be read again.

  async getNextStagedChange(): Promise<StagedChange | undefined> {
    const db = await this.db.get();
    const tx = db.transaction(['user', 'stagedMeasurement', 'stagedWorkout', 'stagedExercise']);

    const measurement = await tx.objectStore('stagedMeasurement').openCursor();
    if (measurement) {
      return {
        version: await this.getVersion(tx),
        measurementId: measurement.primaryKey,
        measurement: measurement.value
      };
    }

    const workout = await tx.objectStore('stagedWorkout').openCursor();
    if (workout) {
      return {
        version: await this.getVersion(tx),
        workoutId: workout.primaryKey,
        workout: workout.value
      };
    }

    const exercise = await tx.objectStore('stagedExercise').openCursor();
    if (exercise) {
      return {
        version: await this.getVersion(tx),
        workoutId: exercise.primaryKey.substring(0, 36),
        exerciseId: exercise.primaryKey.substring(37),
        exercise: exercise.value
      };
    }

    return undefined;
  }

  // ------------- Apply remote changes after successful upload ------------- //
  //
  // When uploading a change succeeds, the version number is incremented and the
  // change is applied to the canonical version. If the change that was uploaded
  // is still staged and hasn't changed, then the change is removed from the
  // staging area.

  applyDeleteMeasurement(measurementId: string): Promise<void> {
    return this.applyDelete('measurement', 'stagedMeasurement', measurementId);
  }

  applyUpdateMeasurement(measurement: Measurement): Promise<void> {
    return this.applyUpdate(
      'measurement',
      'stagedMeasurement',
      measurement,
      measurement.measurement_id,
      (a, b) => {
        return a.type === b.type
          && a.capture_date === b.capture_date
          && a.value === b.value
          && a.notes === b.notes;
      }
    )
  }

  applyDeleteWorkout(workoutId: string): Promise<void> {
    return this.applyDelete('workout', 'stagedWorkout', workoutId);
  }

  applyUpdateWorkout(workout: Omit<Workout, 'exercises'>): Promise<void> {
    return this.applyUpdate(
      'workout',
      'stagedWorkout',
      workout,
      workout.workout_id,
      (a, b) => {
        return a.start_time === b.start_time
          && a.finish_time === b.finish_time
          && a.notes === b.notes;
      }
    );
  }

  applyDeleteExercise(workoutId: string, exerciseId: string): Promise<void> {
    return this.applyDelete('exercise', 'stagedExercise', `${workoutId}#${exerciseId}`);
  }

  applyUpdateExercise(workoutId: string, exercise: Exercise): Promise<void> {
    return this.applyUpdate(
      'exercise',
      'stagedExercise',
      exercise,
      `${workoutId}#${exercise.exercise_id}`,
      (a, b) => {
        return a.order === b.order
          && a.type === b.type
          && a.notes === b.notes
          && JSON.stringify(a.sets) === JSON.stringify(b.sets);
      }
    );
  }

  private async applyDelete<S extends keyof StagedStores>(
    canon: S,
    staged: StagedStores[S],
    id: Schema[S]['key']
  ): Promise<void> {
    const db = await this.db.get();
    const tx = db.transaction(['user', canon, staged], 'readwrite');
    const stagedStore = tx.objectStore(staged);

    await this.incrementVersion(tx);

    const stagedItem = stagedStore.get(id);
    if (stagedItem && 'deleted' in stagedItem) {
      await stagedStore.delete(id);
    }

    await tx.objectStore(canon).delete(id);
    tx.commit();
  }

  private async applyUpdate<S extends keyof StagedStores>(
    canon: S,
    staged: StagedStores[S],
    item: Schema[S]['value'],
    id: Schema[S]['key'],
    equal: (a: Exclude<Schema[StagedStores[S]]['value'], Deleted>, b: Schema[S]['value']) => boolean
  ): Promise<void> {
    const db = await this.db.get();
    const tx = db.transaction(['user', canon, staged], 'readwrite');
    const stagedStore = tx.objectStore(staged);

    await this.incrementVersion(tx);

    const stagedItem = await stagedStore.get(id);
    if (!stagedItem) {
      // The item was removed from the staging area entirely because it wasn't
      // in the canonical version. So we need to add a tombstone since it now
      // exists but needs to be removed.
      await stagedStore.put(DELETED, id);
    } else if (
      !('deleted' in stagedItem)
      // Despite narrowing away the `deleted` property, the compiler doesn't see
      // that the resultant type has the `Deleted` union member excluded.
      && equal(stagedItem as Exclude<Schema[StagedStores[S]]['value'], Deleted>, item)
    ) {
      await stagedStore.delete(id);
    }

    await tx.objectStore(canon).put(item);
  }

  private async incrementVersion<
    Stores extends StoreNames<Schema>
  >(tx: IDBPTransaction<Schema, (Stores | 'user')[], 'readwrite'>): Promise<void> {
    const store = tx.objectStore('user');
    const version = await store.get('version') || 0;
    await store.put(version + 1, 'version');
  }

  private async getVersion<
    Stores extends StoreNames<Schema>,
    Mode extends IDBTransactionMode
  >(tx: IDBPTransaction<Schema, (Stores | 'user')[], Mode>): Promise<number> {
    return await tx.objectStore('user').get('version') || 0;
  }
}
