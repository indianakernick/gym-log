import {
  exerciseEqual,
  measurementSetEqual,
  splitWorkoutExerciseId,
  workoutEqual,
  type Exercise,
  type MeasurementSet,
  type MeasurementType,
  type UserChanges,
  type Workout
} from '@/model/api';
import {
  DELETED,
  type Deleted,
  type MergeConflict,
  type MergeConflictResolutions,
  type StagedChange
} from '@/model/db';
import {
  openDB,
  type DBSchema,
  type IDBPDatabase,
  type IDBPObjectStore,
  type IDBPTransaction,
  type StoreNames
} from 'idb';
import { AsyncInit } from '@/utils/async-init';
import { binarySearch, stringCompare } from '@/utils/binary-search';

// When the user is making changes to their data, the changes are written to a
// staging area. The staging area stores the difference between the local copy
// of remote database (the canonical database) and the local database. The user
// always sees canonical version with the the staged changes applied (unless
// they're resolving merge conflicts).
//
// Changes from the staging area will be pushed up to the remote from time to
// time. If a different client made changes to the remote database since this
// client last fetched it, then the request to push changes will be rejected and
// the client will need to fetch the remote changes to update its remote copy.
// If the request to push changes to the remote succeeds, then the change is
// removed the from the staging area and applied to the canonical version. The
// version number is also incremented so that the client knows the version of
// the remote database that is has.
//
// When a request to push changes to the remote is rejected, the client will
// fetch the new set of changes to be applied to its canonical database. At this
// point, there will be two sets of changes relative to the canonical database.
// If the sets of changes involve different objects, then they can be trivially
// merged. However, if the same object is referenced twice (perhaps modified in
// one and deleted in the other), then the user will be asked which change
// should be applied and the merging process can start again with these
// choices.

interface Schema extends DBSchema {
  auth: {
    key: 'refreshToken';
    value: string;
  },
  user: {
    key: 'version';
    value: number;
  },
  measurement: {
    key: MeasurementSet['date'];
    value: MeasurementSet;
  },
  workout: {
    key: Workout['workout_id'];
    value: Workout;
  },
  exercise: {
    key: Exercise['workout_exercise_id'];
    value: Exercise;
    indexes: {
      type: Exercise['type'];
    }
  },
  stagedMeasurement: {
    key: MeasurementSet['date'];
    value: MeasurementSet | Deleted;
  },
  stagedWorkout: {
    key: Workout['workout_id'];
    value: Workout | Deleted;
  },
  stagedExercise: {
    key: Exercise['workout_exercise_id'];
    value: Exercise | Deleted;
    indexes: {
      type: Exercise['type'];
    }
  },
}

type StagedStores = {
  measurement: 'stagedMeasurement',
  workout: 'stagedWorkout',
  exercise: 'stagedExercise',
};

export default new class {
  // It would probably be more efficient to use an async factory function
  // instead of checking and waiting for the initialisation of this object every
  // time it's accessed. Although it's unclear how to achieve that. To avoid
  // having to check, the whole app has to wait for the database connection to
  // open.
  private db = new AsyncInit<IDBPDatabase<Schema>>();

  constructor() {
    openDB<Schema>(
      'main',
      1,
      {
        upgrade(db) {
          db.createObjectStore('auth');
          db.createObjectStore('user');
          db.createObjectStore('measurement', { keyPath: 'date' });
          db.createObjectStore('workout', { keyPath: 'workout_id' });
          const e = db.createObjectStore('exercise', { keyPath: 'workout_exercise_id' });
          e.createIndex('type', 'type', { unique: false });
          db.createObjectStore('stagedMeasurement');
          db.createObjectStore('stagedWorkout');
          const se = db.createObjectStore('stagedExercise');
          se.createIndex('type', 'type', { unique: false });
        }
      }
    ).then(db => this.db.set(db));

    // Should call this at some point.
    // navigator.storage.persist()
  }

  // ---------------------------- Authentication ---------------------------- //

  async getRefreshToken(): Promise<string | undefined> {
    const db = await this.db.get();
    return db.get('auth', 'refreshToken');
  }

  async setRefreshToken(refreshToken: string): Promise<void> {
    const db = await this.db.get();
    await db.put('auth', refreshToken, 'refreshToken');
  }

  async clearRefreshToken(): Promise<void> {
    const db = await this.db.get();
    await db.delete('auth', 'refreshToken');
  }

  // ----------------------- Stage changes for upload ----------------------- //
  //
  // Stage changes to the local database and prepare them to be pushed to the
  // remote database.

  /**
   * Stage a delete-measurement request.
   */
  stageDeleteMeasurement(date: MeasurementSet['date']): Promise<void> {
    return this.stageDelete('measurement', 'stagedMeasurement', date);
  }

  /**
   * Stage an update-measurement request.
   */
  async stageUpdateMeasurement(measurement: MeasurementSet): Promise<void> {
    const db = await this.db.get();
    await db.put('stagedMeasurement', measurement, measurement.date);
  }

  /**
   * Stage a delete-workout request.
   */
  stageDeleteWorkout(workoutId: Workout['workout_id']): Promise<void> {
    return this.stageDelete('workout', 'stagedWorkout', workoutId);
  }

  /**
   * Stage an update-workout request.
   */
  async stageUpdateWorkout(workout: Workout): Promise<void> {
    const db = await this.db.get();
    await db.put('stagedWorkout', workout, workout.workout_id);
  }

  /**
   * Stage a delete-exercise request.
   */
  stageDeleteExercise(workoutExerciseId: Exercise['workout_exercise_id']): Promise<void> {
    return this.stageDelete('exercise', 'stagedExercise', workoutExerciseId);
  }

  /**
   * Stage an update-exercise request.
   */
  async stageUpdateExercise(exercise: Exercise): Promise<void> {
    const db = await this.db.get();
    await db.put('stagedExercise', exercise, exercise.workout_exercise_id);
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

  /**
   * Apply a set of changes to the canonical version and handle conflicts with
   * the staged changes.
   *
   * If there is a remote change and a staged change being applied to the same
   * object and those two changes are different, then there is a merge conflict.
   * When this is detected, the user will be shown the two changes and asked to
   * choose one of them to keep. The decisions that the user makes can be used
   * to restart the process and either keep or delete the conflicting staged
   * changes.
   *
   * @param remote changes from the remote relative to the canonical version.
   * @param resolutions decisions made by the user to resolve merge conflicts.
   * @returns a list of merge conflicts for the user to resolve.
   */
  async merge(
    remote: UserChanges,
    resolutions: MergeConflictResolutions = {}
  ): Promise<MergeConflict[]> {
    const conflicts: MergeConflict[] = [];
    const db = await this.db.get();
    const tx = db.transaction([
      'user',
      'measurement',
      'workout',
      'exercise',
      'stagedMeasurement',
      'stagedWorkout',
      'stagedExercise'
    ], 'readwrite');

    await tx.objectStore('user').put(remote.version, 'version');

    await Promise.all([
      this.mergeEntity(
        resolutions,
        conflicts,
        tx.objectStore('measurement'),
        tx.objectStore('stagedMeasurement'),
        remote.measurement_sets,
        remote.deleted_measurement_sets,
        measurementSetEqual,
        m => m.date,
        (id, remote, local) => ({ type: 'measurement', id, remote, local })
      ),
      this.mergeEntity(
        resolutions,
        conflicts,
        tx.objectStore('workout'),
        tx.objectStore('stagedWorkout'),
        remote.workouts,
        remote.deleted_workouts,
        workoutEqual,
        m => m.workout_id,
        (id, remote, local) => ({ type: 'workout', id, remote, local })
      ),
      this.mergeEntity(
        resolutions,
        conflicts,
        tx.objectStore('exercise'),
        tx.objectStore('stagedExercise'),
        remote.exercises,
        remote.deleted_exercises,
        exerciseEqual,
        m => m.workout_exercise_id,
        (id, remote, local) => ({ type: 'exercise', id, remote, local })
      )
    ]);

    if (conflicts.length) {
      tx.abort();
    } else {
      tx.commit();
    }

    return conflicts;
  }

  private async mergeEntity<
    S extends keyof StagedStores,
    T extends StoreNames<Schema>
  >(
    resolutions: MergeConflictResolutions,
    conflicts: MergeConflict[],
    canonStore: IDBPObjectStore<Schema, T[], S, 'readwrite'>,
    stagedStore: IDBPObjectStore<Schema, T[], StagedStores[S], 'readwrite'>,
    remoteUpdates: Schema[S]['value'][],
    remoteDeletes: Schema[S]['key'][],
    equal: (a: Schema[S]['value'], b: Schema[S]['value']) => boolean,
    getId: (item: Schema[S]['value']) => Schema[S]['key'],
    makeConflict: (id: Schema[S]['key'], remote: Schema[S]['value'] | Deleted, local: Schema[S]['value'] | Deleted) => MergeConflict
  ): Promise<void> {
    for (const r of remoteUpdates) {
      const id = getId(r);

      await canonStore.put(r);

      await this.mergeUpdate(
        resolutions[id],
        await stagedStore.get(id),
        r,
        equal,
        () => stagedStore.delete(id),
        local => conflicts.push(makeConflict(id, r, local))
      );
    }

    for (const r of remoteDeletes) {
      await canonStore.delete(r);

      await this.mergeDelete(
        resolutions[r],
        await stagedStore.get(r),
        () => stagedStore.delete(r),
        local => conflicts.push(makeConflict(r, DELETED, local))
      );
    }
  }

  private async mergeUpdate<T extends object>(
    res: MergeConflictResolutions[string],
    staged: T | Deleted | undefined,
    remote: T,
    equal: (a: T, b: T) => boolean,
    revert: () => Promise<void>,
    conflict: (local: T | Deleted) => void
  ) {
    if (!staged) return;

    if ('deleted' in staged) {
      if (res === 'remote') {
        return revert();
      } else if (res === undefined) {
        conflict(staged);
      }
    } else {
      if (res === 'remote' || equal(staged, remote)) {
        return revert();
      } else if (res === undefined) {
        conflict(staged);
      }
    }
  }

  private async mergeDelete<T extends object>(
    res: MergeConflictResolutions[string],
    staged: T | Deleted | undefined,
    revert: () => Promise<void>,
    conflict: (local: T) => void
  ): Promise<void> {
    if (!staged) return;

    if ('deleted' in staged || res === 'remote') {
      return revert();
    } else if (res === undefined) {
      conflict(staged);
    }
  }

  // --------------------- Get a staged change to upload -------------------- //
  //
  // Individual changes are uploaded one at a time. After uploading, the changes
  // are applied. When preparing to upload the next change, the staged changes
  // could have been changed by a user action so it needs to be read again.

  /**
   * Find one staged change that's ready to push.
   */
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
        ...splitWorkoutExerciseId(exercise.primaryKey),
        exercise: exercise.value
      };
    }

    return undefined;
  }

  /**
   * Get the current version of the database to use when pull changes.
   */
  async getCurrentVersion(): Promise<number> {
    return this.getVersion((await this.db.get()).transaction(['user']));
  }

  // ------------- Apply remote changes after successful upload ------------- //
  //
  // When uploading a change succeeds, the version number is incremented and the
  // change is applied to the canonical version. If the change that was uploaded
  // is still staged and hasn't changed, then the change is removed from the
  // staging area.

  /**
   * Apply a delete-measurement change to the canonical version after
   * successfully pushing that change. Remove it from the staging area if it
   * exists.
   */
  applyDeleteMeasurement(measurementId: string): Promise<void> {
    return this.applyDelete('measurement', 'stagedMeasurement', measurementId);
  }

  /**
   * Apply an update-measurement change to the canonical version after
   * successfully pushing that change. Remove it from the staging area if it
   * exists.
   */
  applyUpdateMeasurement(measurement: MeasurementSet): Promise<void> {
    return this.applyUpdate(
      'measurement',
      'stagedMeasurement',
      measurement,
      measurement.date,
      measurementSetEqual
    )
  }

  /**
   * Apply a delete-workout change to the canonical version after successfully
   * pushing that change. Remove it from the staging area if it exists.
   */
  applyDeleteWorkout(workoutId: string): Promise<void> {
    return this.applyDelete('workout', 'stagedWorkout', workoutId);
  }

  /**
   * Apply an update-workout change to the canonical version after successfully
   * pushing that change. Remove it from the staging area if it exists.
   */
  applyUpdateWorkout(workout: Workout): Promise<void> {
    return this.applyUpdate(
      'workout',
      'stagedWorkout',
      workout,
      workout.workout_id,
      workoutEqual
    );
  }

  /**
   * Apply a delete-exercise change to the canonical version after successfully
   * pushing that change. Remove it from the staging area if it exists.
   */
  applyDeleteExercise(workoutExerciseId: Exercise['workout_exercise_id']): Promise<void> {
    return this.applyDelete('exercise', 'stagedExercise', workoutExerciseId);
  }

  /**
   * Apply an update-exercise change to the canonical version after successfully
   * pushing that change. Remove it from the staging area if it exists.
   */
  applyUpdateExercise(exercise: Exercise): Promise<void> {
    return this.applyUpdate(
      'exercise',
      'stagedExercise',
      exercise,
      exercise.workout_exercise_id,
      exerciseEqual
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
      // that the resultant type has the `Deleted` union member excluded. Or
      // something like that...
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

  // ------------------------- Inspect the database ------------------------- //
  //
  // The user sees the canonical version with the staged changes applied.

  /**
   * Get all measurements of a particular type, ordered by ascending `date`.
   */
  async getMeasurementsOfType(
    type: MeasurementType
  ): Promise<{ date: MeasurementSet['date'], value: number }[]> {
    const db = await this.db.get();
    const tx = db.transaction(['measurement', 'stagedMeasurement']);
    const stagedStore = tx.objectStore('stagedMeasurement');

    const [canon, staged, stagedKeys] = await Promise.all([
      tx.objectStore('measurement').getAll(),
      stagedStore.getAll(),
      stagedStore.getAllKeys()
    ]);

    this.applyStaged(canon, staged, stagedKeys, (measurement, id) => {
      return stringCompare(measurement.date, id);
    });

    return canon
      .map(set => ({ date: set.date, value: set.measurements[type] }))
      .filter((set): set is { date: MeasurementSet['date'], value: number } => set.value !== undefined);
  }

  /**
   * Get the set of measurements for a date.
   */
  async getMeasurementSet(date: MeasurementSet['date']): Promise<MeasurementSet | undefined> {
    const db = await this.db.get();
    const tx = db.transaction(['measurement', 'stagedMeasurement']);
    const stagedStore = tx.objectStore('stagedMeasurement');

    const [canon, staged, stagedKeys] = await Promise.all([
      tx.objectStore('measurement').getAll(date),
      stagedStore.getAll(date),
      stagedStore.getAllKeys(date)
    ]);

    this.applyStaged(canon, staged, stagedKeys, (measurement, id) => {
      return stringCompare(measurement.date, id);
    });

    return canon[0];
  }

  /**
   * Get all unique values of `capture_date` on measurements, ordered by
   * ascending `capture_date`.
   */
  async getMeasurementDates(): Promise<MeasurementSet['date'][]> {
    const db = await this.db.get();
    const tx = db.transaction(['measurement', 'stagedMeasurement']);
    const stagedStore = tx.objectStore('stagedMeasurement');

    const [canonKeys, staged, stagedKeys] = await Promise.all([
      tx.objectStore('measurement').getAllKeys(),
      stagedStore.getAll(),
      stagedStore.getAllKeys()
    ]);

    let start = 0;
    let newDates: string[] = [];

    for (let stagedIdx = 0; stagedIdx < staged.length; ++stagedIdx) {
      const id = stagedKeys[stagedIdx];
      const canonIdx = binarySearch(
        canonKeys,
        start,
        canonKeys.length,
        item => stringCompare(item, id)
      );
      const stagedItem = staged[stagedIdx];

      if ('deleted' in stagedItem) {
        canonKeys.splice(canonIdx, 1);
        start = canonIdx;
      } else if (canonIdx !== -1) {
        start = canonIdx + 1;
      } else {
        newDates.push(id);
      }
    }

    canonKeys.push(...newDates);

    return canonKeys;
  }

  /**
   * Get all workouts, ordered by descending `start_time`.
   */
  async getWorkouts(): Promise<Workout[]> {
    const db = await this.db.get();
    const tx = db.transaction(['workout', 'stagedWorkout']);
    const stagedStore = tx.objectStore('stagedWorkout');

    const [canon, staged, stagedKeys] = await Promise.all([
      tx.objectStore('workout').getAll(),
      stagedStore.getAll(),
      stagedStore.getAllKeys()
    ]);

    this.applyStaged(canon, staged, stagedKeys, (workout, id) => {
      return stringCompare(workout.workout_id, id);
    });

    // Workouts without a start time first, followed by workouts with a start
    // time from most recent to least recent.
    canon.sort((a, b) => {
      return stringCompare(b.start_time || 'Z', a.start_time || 'Z');
    });

    return canon;
  }

  /**
   * Get all exercises within a workout, ordered by ascending `order`.
   */
  async getExercisesOfWorkout(workoutId: string): Promise<Exercise[]> {
    const db = await this.db.get();
    const tx = db.transaction(['exercise', 'stagedExercise']);
    const stagedStore = tx.objectStore('stagedExercise');
    const query = IDBKeyRange.bound(workoutId + '#', workoutId + '$', false, true);

    const [canon, staged, stagedKeys] = await Promise.all([
      tx.objectStore('exercise').getAll(query),
      stagedStore.getAll(query),
      stagedStore.getAllKeys(query)
    ]);

    this.applyStaged(canon, staged, stagedKeys, (exercise, id) => {
      return stringCompare(exercise.workout_exercise_id, id);
    });

    canon.sort((a, b) => a.order - b.order);

    return canon;
  }

  /**
   * Get all exercises of a particular type, ordered by ascending
   * `workout_exercise_id`.
   */
  async getExercisesOfType<T extends Exercise['type']>(
    type: T
  ): Promise<(Exercise & { type: T })[]> {
    const db = await this.db.get();
    const tx = db.transaction(['exercise', 'stagedExercise']);
    const stagedIndex = tx.objectStore('stagedExercise').index('type');

    const [canon, staged, stagedKeys] = await Promise.all([
      tx.objectStore('exercise').index('type').getAll(type),
      stagedIndex.getAll(type),
      stagedIndex.getAllKeys(type)
    ]);

    const added = this.applyStaged(canon, staged, stagedKeys, (exercise, id) => {
      return stringCompare(exercise.workout_exercise_id, id);
    });

    if (added > 0) {
      // We have two sorted sub-arrays. We could merge them instead of sorting.
      // Although it's unclear whether a JavaScript merge function will beat a
      // highly optimized sorting function even if it has better time
      // complexity.
      canon.sort((a, b) => {
        return stringCompare(a.workout_exercise_id, b.workout_exercise_id);
      });
    }

    return canon as (Exercise & { type: T })[];
  }

  /**
   * Apply the staged changes to the canonical version to materialize the local
   * version of the database.
   */
  private applyStaged<T extends object>(
    canon: T[],
    staged: (T | Deleted)[],
    stagedKeys: string[],
    compare: (item: T, id: string) => number
  ): number {
    let start = 0;
    let newItems: T[] = [];

    for (let stagedIdx = 0; stagedIdx < staged.length; ++stagedIdx) {
      const id = stagedKeys[stagedIdx];
      const canonIdx = binarySearch(canon, start, canon.length, item => compare(item, id));
      const stagedItem = staged[stagedIdx];

      if ('deleted' in stagedItem) {
        canon.splice(canonIdx, 1);
        start = canonIdx;
      } else if (canonIdx !== -1) {
        canon[canonIdx] = stagedItem;
        start = canonIdx + 1;
      } else {
        newItems.push(stagedItem);
      }
    }

    canon.push(...newItems);

    return newItems.length;
  }
}
