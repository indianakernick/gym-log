import type { NavigationFailure, RouteLocationRaw, Router } from 'vue-router';

export function back(
  router: Router,
  to: RouteLocationRaw,
): Promise<NavigationFailure | void | undefined> {
  // TODO: find a reliable way of checking whether router.back() will succeed
  // router.back if we can go back, otherwise router.replace
  return router.replace(to);
}
