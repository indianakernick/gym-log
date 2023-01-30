import { isValidDate } from '@/utils/date';
import type { RouteLocationNormalized } from 'vue-router';

export function dateGuard(to: RouteLocationNormalized): boolean {
  return !Array.isArray(to.params.date) && isValidDate(to.params.date);
}
