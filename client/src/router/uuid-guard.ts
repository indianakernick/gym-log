import type { RouteLocationNormalized } from 'vue-router';

export function uuidGuard(to: RouteLocationNormalized): boolean {
  if (Array.isArray(to.params.id)) return false;
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[4][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(to.params.id);
}
