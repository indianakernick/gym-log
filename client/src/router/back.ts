import type { UseIonRouterResult } from '@ionic/vue';
import type { RouteLocationRaw } from 'vue-router';

export function back(router: UseIonRouterResult, fallback: RouteLocationRaw) {
  if (router.canGoBack()) {
    router.back();
  } else {
    router.navigate(fallback, 'back', 'replace');
  }
}
