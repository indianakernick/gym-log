import type { MergeConflictResolutions, StagedChange } from '@/model/db';
import db from '@/services/db';
import user, { CacheOutdatedError } from '@/services/user';
import { shallowRef, type DeepReadonly, type ShallowRef } from 'vue';
import type { Router } from 'vue-router';
import { UnauthenticatedError } from './auth';

// Do we gain anything by moving this into the service worker?

const DEBOUNCE_DURATION = 5000;
const SYNC_PERIOD = 10 * 60 * 1000;

export default new class {
  private syncing: boolean = false;
  private debounceId?: number;
  private router?: Router;
  private versionRef = shallowRef<number>();

  constructor() {
    setInterval(this.sync.bind(this), SYNC_PERIOD);
    this.sync();
    db.getCurrentVersion().then(v => this.versionRef.value = v);
  }

  setRouter(router: Router) {
    this.router = router;
  }

  get version(): DeepReadonly<ShallowRef<number | undefined>> {
    return this.versionRef;
  }

  sync() {
    clearTimeout(this.debounceId);
    this.debounceId = window.setTimeout(async () => {
      if (this.syncing) {
        this.sync();
      } else {
        this.syncing = true;
        try {
          if (!await this.doSync()) {
            await this.pullChanges(await db.getCurrentVersion());
          }
        } catch (e) {
          if (!this.handleAuthRedirect(e)) {
            console.error('Pulling changes', e);
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
            if (!this.handleAuthRedirect(e)) {
              console.error('Pulling changes', e);
            }
            return true;
          }
        }
        if (!this.handleAuthRedirect(e)) {
          console.error('Pushing changes', e);
        }
        return true;
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
        this.versionRef.value = version;
        return;
      }
    }
  }

  private handleAuthRedirect(e: unknown): boolean {
    if (e instanceof UnauthenticatedError) {
      // There's no way initializing the app is going to take up the full
      // debounce duration so the router would have been set by then.
      this.router!.push({
        path: '/login',
        query: { redirect: this.router!.currentRoute.value.path }
      });
      return true;
    }
    return false;
  }
}
