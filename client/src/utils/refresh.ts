import sync from '@/services/sync';
import { watchEffect } from 'vue';

export function refresh(load: (initial: boolean) => void | Promise<void>) {
  let previousVersion: number | undefined;

  // sync.version will initially be undefined for a brief moment until the
  // actual version is loaded from IndexedDB. Refreshing the view should not
  // happen we are simply discovering what the initial version is.

  watchEffect(() => {
    const version = sync.version.value;
    if (version !== previousVersion) {
      if (previousVersion !== undefined) {
        console.log('loaded', previousVersion, version);
        load(false);
      }
      previousVersion = version;
    }
  });

  console.log('initial load');
  load(true);
}
