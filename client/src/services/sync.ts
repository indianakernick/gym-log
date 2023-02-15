import type { MergeConflictResolutions, StagedChange } from '@/model/db';
import db from '@/services/db';
import user, { CacheOutdatedError } from '@/services/user';
import auth from './auth';

// Do we gain anything by moving this into the service worker?

const DEBOUNCE_DURATION = 5000;
const SYNC_PERIOD = 10 * 60 * 1000;

export default new class {
  private syncing: boolean = false;
  private debounceId?: number;

  constructor() {
    setInterval(this.sync.bind(this), SYNC_PERIOD);
    this.sync();
  }

  sync() {
    clearTimeout(this.debounceId);
    this.debounceId = window.setTimeout(async () => {
      if (!await auth.isLoggedIn()) return;
      if (this.syncing) {
        this.sync();
      } else {
        this.syncing = true;
        try {
          if (!await this.doSync()) {
            await this.pullChanges(await db.getCurrentVersion());
          }
        } finally {
          this.syncing = false;
        }
      }
    }, DEBOUNCE_DURATION);
  }

  private async doSync(): Promise<boolean> {
    let foundStagedChanges = false;

    while (true) {
      const change = await db.getNextStagedChange();

      if (!change) return foundStagedChanges;
      foundStagedChanges = true;

      try {
        await this.pushChange(change);
      } catch (e) {
        if (e instanceof CacheOutdatedError) {
          try {
            await this.pullChanges(change.version);
          } catch (e) {
            console.error('Pulling changes', e);
            return true;
          }
        } else {
          console.error('Pushing changes', e);
          return true;
        }
      }
    }
  }

  private async pushChange(change: StagedChange): Promise<void> {
    if ('measurement' in change) {
      if ('deleted' in change.measurement) {
        await user.deleteMeasurement(change.version, change.measurementId);
        await db.applyDeleteMeasurement(change.measurementId);
      } else {
        await user.updateMeasurement(change.version, change.measurement);
        await db.applyUpdateMeasurement(change.measurement);
      }
    } else if ('workout' in change) {
      if ('deleted' in change.workout) {
        await user.deleteWorkout(change.version, change.workoutId);
        await db.applyDeleteWorkout(change.workoutId);
      } else {
        await user.updateWorkout(change.version, change.workout);
        await db.applyUpdateWorkout(change.workout);
      }
    } else {
      if ('deleted' in change.exercise) {
        await user.deleteExercise(change.version, `${change.workoutId}#${change.exerciseId}`);
        await db.applyDeleteExercise(`${change.workoutId}#${change.exerciseId}`);
      } else {
        await user.updateExercise(change.version, change.exercise);
        await db.applyUpdateExercise(change.exercise);
      }
    }
  }

  private async pullChanges(version: number): Promise<void> {
    const changes = await user.getChanges(version);
    const resolutions: MergeConflictResolutions = {};

    while (true) {
      const conflicts = await db.merge(changes, resolutions);

      if (conflicts.length) {
        // TODO: show the conflicts to the user and ask them for resolutions
        alert('Merge conflicts!!!');
      } else {
        // TODO: tell the current view to refresh itself somehow
        // the user might be looking at something that has just been deleted.
        // so we would need to make sure that we don't accidentally create it
        // again
        return;
      }
    }
  }
}
