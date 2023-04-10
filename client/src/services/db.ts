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
import { binarySearch, stringCompare } from '@/utils/array';

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
    };
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
    };
  },
}

type StagedStores = {
  measurement: 'stagedMeasurement',
  workout: 'stagedWorkout',
  exercise: 'stagedExercise',
};

// start_time must be present if the workout has exercises.
type ExerciseWorkoutJoin = (Exercise & { workout: Workout & { start_time: string } });

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
    return this.stageDelete('stagedMeasurement', date);
  }

  /**
   * Stage an update-measurement request.
   */
  stageUpdateMeasurement(measurement: MeasurementSet): Promise<void> {
    return this.stageUpdate('stagedMeasurement', measurement, measurement.date);
  }

  /**
   * Stage a delete-workout request.
   */
  stageDeleteWorkout(workoutId: Workout['workout_id']): Promise<void> {
    return this.stageDelete('stagedWorkout', workoutId);
  }

  /**
   * Stage an update-workout request.
   */
  stageUpdateWorkout(workout: Workout): Promise<void> {
    return this.stageUpdate('stagedWorkout', workout, workout.workout_id);
  }

  /**
   * Stage a delete-exercise request.
   */
  stageDeleteExercise(workoutExerciseId: Exercise['workout_exercise_id']): Promise<void> {
    return this.stageDelete('stagedExercise', workoutExerciseId);
  }

  /**
   * Stage an update-exercise request.
   */
  stageUpdateExercise(exercise: Exercise): Promise<void> {
    return this.stageUpdate('stagedExercise', exercise, exercise.workout_exercise_id);
  }

  private async stageDelete<S extends keyof StagedStores>(
    staged: StagedStores[S],
    id: Schema[S]['key']
  ): Promise<void> {
    await (await this.db.get()).put(staged, DELETED, id);
  }

  private async stageUpdate<S extends keyof StagedStores>(
    staged: StagedStores[S],
    stagedItem: Schema[S]['value'],
    id: Schema[S]['key']
  ): Promise<void> {
    await (await this.db.get()).put(staged, stagedItem, id);
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
        (id, original, remote, local) =>
          ({ type: 'measurement', id, original, remote, local })
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
        (id, original, remote, local) =>
          ({ type: 'workout', id, original, remote, local })
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
        (id, original, remote, local) =>
          ({ type: 'exercise', id, original, remote, local })
      )
    ]);

    if (conflicts.length) {
      tx.abort();
      // IDB will throw a DOMException('AbortError', 'AbortError').
      try { await tx.done } catch {}
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
    makeConflict: (
      id: Schema[S]['key'],
      original: Schema[S]['value'] | Deleted,
      remote: Schema[S]['value'] | Deleted,
      local: Schema[S]['value'] | Deleted
    ) => MergeConflict
  ): Promise<void> {
    for (const r of remoteUpdates) {
      const id = getId(r);
      const original = (await canonStore.get(id)) ?? DELETED;

      await canonStore.put(r);

      await this.mergeUpdate(
        resolutions[id],
        await stagedStore.get(id),
        r,
        equal,
        () => stagedStore.delete(id),
        local => conflicts.push(makeConflict(id, original, r, local))
      );
    }

    for (const r of remoteDeletes) {
      const original = (await canonStore.get(r)) ?? DELETED;

      await canonStore.delete(r);

      await this.mergeDelete(
        resolutions[r],
        await stagedStore.get(r),
        () => stagedStore.delete(r),
        local => conflicts.push(makeConflict(r, original, DELETED, local))
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
   *
   * In the process of finding a staged change, this will remove unnecessary
   * staged changes to avoid pushing things unnecessarily.
   */
  async getNextStagedChange(): Promise<StagedChange | undefined> {
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

    const version = await this.getVersion(tx);

    const measurement = await this.getStaged(
      tx.objectStore('measurement'),
      tx.objectStore('stagedMeasurement'),
      measurementSetEqual,
      (id, value) => ({
        version,
        measurementId: id,
        measurement: value
      })
    );
    if (measurement) return measurement;

    const workout = await this.getStaged(
      tx.objectStore('workout'),
      tx.objectStore('stagedWorkout'),
      workoutEqual,
      (id, value) => ({
        version,
        workoutId: id,
        workout: value
      })
    );
    if (workout) return workout;

    const exercise = await this.getStaged(
      tx.objectStore('exercise'),
      tx.objectStore('stagedExercise'),
      exerciseEqual,
      (id, value) => ({
        version,
        ...splitWorkoutExerciseId(id),
        exercise: value
      })
    );
    if (exercise) return exercise;

    return undefined;
  }

  private async getStaged<S extends keyof StagedStores, T extends StoreNames<Schema>>(
    canonStore: IDBPObjectStore<Schema, T[], S, 'readwrite'>,
    stagedStore: IDBPObjectStore<Schema, T[], StagedStores[S], 'readwrite'>,
    equal: (a: Schema[S]['value'], b: Schema[S]['value']) => boolean,
    makeChange: (id: Schema[StagedStores[S]]['key'], value: Schema[StagedStores[S]]['value']) => StagedChange
  ): Promise<StagedChange | undefined> {
    let cursor = await stagedStore.openCursor();

    while (cursor) {
      if ('deleted' in cursor.value) {
        if (await canonStore.count(cursor.primaryKey) > 0) {
          return makeChange(cursor.primaryKey, cursor.value);
        } else {
          await cursor.delete();
        }
      } else {
        const canonItem = await canonStore.get(cursor.primaryKey);
        if (!canonItem || !equal(canonItem, cursor.value as Schema[S]['value'])) {
          return makeChange(cursor.primaryKey, cursor.value);
        } else {
          await cursor.delete();
        }
      }

      cursor = await cursor.continue();
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

    const stagedItem = await stagedStore.get(id);
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
   * Get all measurement dates, ordered by descending `date`.
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

      if (canonIdx < 0) {
        if (!('deleted' in stagedItem)) {
          newDates.push(id);
        }
      } else {
        if ('deleted' in stagedItem) {
          canonKeys.splice(canonIdx, 1);
          start = canonIdx;
        } else {
          start = canonIdx + 1;
        }
      }
    }

    canonKeys.push(...newDates);

    if (newDates.length) canonKeys.sort();

    return canonKeys.reverse();
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
    // time from most recent to least recent. Note that `sort` is stable so if
    // two workouts somehow have the same start time, they will be ordered by
    // the ID.
    canon.sort((a, b) => {
      return stringCompare(b.start_time || 'Z', a.start_time || 'Z');
    });

    return canon;
  }

  /**
   * Get a workout with particular ID.
   */
  async getWorkout(workoutId: string): Promise<Workout | undefined> {
    const db = await this.db.get();
    const tx = db.transaction(['workout', 'stagedWorkout']);
    const stagedStore = tx.objectStore('stagedWorkout');

    const [canon, staged, stagedKeys] = await Promise.all([
      tx.objectStore('workout').getAll(workoutId),
      stagedStore.getAll(workoutId),
      stagedStore.getAllKeys(workoutId)
    ]);

    this.applyStaged(canon, staged, stagedKeys, (workout, id) => {
      return stringCompare(workout.workout_id, id);
    });

    return canon[0];
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

  // TODO: is there a situation where we can use getExercisesOfType on its own
  // without joinWorkoutWithExercises? If not, it might make sense to merge the
  // two functions together.

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
   * Add a `workout` field containing the workout associated with each exercise.
   * Assumes that the exercises are ordered by ascending `workout_exercise_id`.
   * The resulting array is in the same order but exercises whose workout is not
   * found (perhaps deleted) will be removed.
   */
  async joinWorkoutWithExercises(
    exercises: Exercise[]
  ): Promise<ExerciseWorkoutJoin[]> {
    const db = await this.db.get();
    const tx = db.transaction(['workout', 'stagedWorkout']);
    const canonStore = tx.objectStore('workout');
    const stagedStore = tx.objectStore('stagedWorkout');
    const result: (Exercise & { workout: Workout })[] = [];

    for (const [i, exercise] of exercises.entries()) {
      const workoutId = exercise.workout_exercise_id.substring(0, 36);

      if (exercises[i - 1]?.workout_exercise_id.substring(0, 36) === workoutId) {
        const workout = result[result.length - 1]?.workout;
        if (workout?.workout_id === workoutId) {
          result.push({ ...exercise, workout });
        }
        continue;
      }

      const [canon, staged] = await Promise.all([
        canonStore.get(workoutId),
        stagedStore.get(workoutId)
      ]);

      const workout = this.applyStagedOne(canon, staged);

      if (workout) result.push({ ...exercise, workout });
    }

    return result as ExerciseWorkoutJoin[];
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

      if (canonIdx < 0) {
        if (!('deleted' in stagedItem)) {
          newItems.push(stagedItem);
        }
      } else {
        if ('deleted' in stagedItem) {
          canon.splice(canonIdx, 1);
          start = canonIdx;
        } else {
          canon[canonIdx] = stagedItem;
          start = canonIdx + 1;
        }
      }
    }

    canon.push(...newItems);

    return newItems.length;
  }

  private applyStagedOne<T extends object>(
    canon: T | undefined,
    staged: T | Deleted | undefined
  ): T | undefined {
    if (staged) {
      return 'deleted' in staged ? undefined : staged;
    } else {
      return canon;
    }
  }
}
