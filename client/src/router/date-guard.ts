import { toDbDate } from "@/utils/date";
import type { RouteLocationNormalized } from "vue-router";

export function dateGuard(to: RouteLocationNormalized): boolean {
  if (Array.isArray(to.params.date)) return false;
  const date = new Date(to.params.date);
  if (Number.isNaN(date.valueOf())) return false;
  return toDbDate(date) === to.params.date;
}
