import type { RouteLocationNormalized, RouteLocationRaw } from 'vue-router';
import auth from '@/services/auth';

export async function authGuard(to: RouteLocationNormalized): Promise<true | RouteLocationRaw> {
  if (await auth.isLoggedIn()) {
    return true;
  } else {
    return { path: '/login', query: { redirect: to.path } };
  }
}
