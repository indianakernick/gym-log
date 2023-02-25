import MergeConflictModal from '@/modals/MergeConflictModal.vue';
import type { MergeConflictResolutions, StagedChange } from '@/model/db';
import db from '@/services/db';
import user, { CacheOutdatedError } from '@/services/user';
import { shallowRef, type DeepReadonly, type ShallowRef } from 'vue';
import { useModal, type UseModalReturnType } from 'vue-final-modal';
import { useRouter, type Router } from 'vue-router';
import { UnauthenticatedError } from './auth';

// Do we gain anything by moving this into the service worker?

// TODO: let the user see the sync status. perhaps an icon in the header?
// TODO: let the user manually trigger a sync

const THROTTLE_DURATION = 10 * 1000;
const SYNC_PERIOD = 10 * 60 * 1000;

export default new class {
  private syncing: boolean = false;
  private syncAfterThrottle: boolean = false;
  private throttleId?: number;
  private router?: Router;
  private modal?: UseModalReturnType<InstanceType<typeof MergeConflictModal>['$props']>;
  private versionRef = shallowRef<number>();

  constructor() {
    setInterval(this.sync.bind(this), SYNC_PERIOD);
    db.getCurrentVersion().then(v => this.versionRef.value = v);
  }

  setup() {
    this.router = useRouter();
    this.modal = useModal({
      component: MergeConflictModal
    });
    this.sync();
  }

  get version(): DeepReadonly<ShallowRef<number | undefined>> {
    return this.versionRef;
  }

  sync() {
    if (this.throttleId !== undefined) {
      this.syncAfterThrottle = true;
      return;
    }

    if (this.syncing) {
      this.syncAfterThrottle = true;
    } else {
      this.syncAfterThrottle = false;
      this.syncPushOrPull();
    }

    this.throttleId = window.setTimeout(() => {
      this.throttleId = undefined;
      if (this.syncAfterThrottle) this.sync();
    }, THROTTLE_DURATION);
  }

  private async syncPushOrPull(): Promise<void> {
    this.syncing = true;

    try {
      if (!await this.syncPush()) {
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

  private async syncPush(): Promise<boolean> {
    let didPush = false;

    while (true) {
      const change = await db.getNextStagedChange();

      if (!change) return didPush;
      didPush = true;

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
          }
        } else if (!this.handleAuthRedirect(e)) {
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
        await new Promise<void>(accept => {
          this.modal!.patchOptions({
            attrs: {
              conflicts,
              onResolved: res => {
                Object.assign(resolutions, res);
                this.modal!.close();
                accept();
              }
            }
          });
          this.modal!.open();
        });
      } else {
        this.versionRef.value = changes.version;
        return;
      }
    }
  }

  private handleAuthRedirect(e: unknown): boolean {
    if (e instanceof UnauthenticatedError) {
      this.router!.push({
        path: '/login',
        query: { redirect: this.router!.currentRoute.value.path }
      });
      return true;
    }
    return false;
  }
}
