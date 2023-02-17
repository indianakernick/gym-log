import ConfirmSignUpView from '@/views/ConfirmSignUpView.vue';
import LoginView from '@/views/LoginView.vue';
import MeasurementEditView from '@/views/MeasurementEditView.vue';
import MeasurementListView from '@/views/MeasurementListView.vue';
import SignUpView from '@/views/SignUpView.vue';
import TabsView from '@/views/TabsView.vue';
import WorkoutEditView from '@/views/WorkoutEditView.vue';
import WorkoutListView from '@/views/WorkoutListView.vue';
import { nextTick } from 'vue';
import { createRouter, createWebHistory } from 'vue-router';
import { authGuard } from './auth-guard';
import { dateGuard } from './date-guard';
import { uuidGuard } from './uuid-guard';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  strict: true,
  sensitive: true,
  routes: [
    {
      name: 'confirm-signup',
      path: '/confirm-signup',
      component: ConfirmSignUpView,
      props: route => ({ email: route.query.email }),
      meta: { title: 'Confirm Sign-up' },
    },
    {
      path: '/login',
      component: LoginView,
      props: route => ({ redirect: route.query.redirect }),
      meta: { title: 'Login' },
    },
    {
      path: '/signup',
      component: SignUpView,
      meta: { title: 'Sign-up' },
    },
    {
      path: '/',
      component: TabsView,
      beforeEnter: authGuard,
      children: [
        {
          path: '',
          redirect: 'workouts'
        },
        {
          path: 'measurements',
          component: MeasurementListView,
          meta: { title: 'Measurement List' },
        },
        {
          path: 'measurements/:date',
          component: MeasurementEditView,
          beforeEnter: dateGuard,
          props: true,
          meta: { title: 'Measurement Details' },
        },
        {
          path: 'workouts',
          component: WorkoutListView,
          meta: { title: 'Workout List' },
        },
        {
          path: 'workouts/:id',
          component: WorkoutEditView,
          beforeEnter: uuidGuard,
          props: true,
          meta: { title: 'Workout Details' },
        },
      ]
    },
  ]
});

router.afterEach(to => {
  nextTick(() => {
    const app = 'Gym Log';
    document.title = to.meta.title ? `${to.meta.title} - ${app}` : app;
  });
});

export default router;
