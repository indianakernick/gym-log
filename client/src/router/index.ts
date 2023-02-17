import ConfirmSignUpView from '@/views/ConfirmSignUpView.vue';
import LoginView from '@/views/LoginView.vue';
import MeasurementEditView from '@/views/MeasurementEditView.vue';
import MeasurementListView from '@/views/MeasurementListView.vue';
import SignUpView from '@/views/SignUpView.vue';
import TabsView from '@/views/TabsView.vue';
import WorkoutEditView from '@/views/WorkoutEditView.vue';
import WorkoutListView from '@/views/WorkoutListView.vue';
import { createRouter, createWebHistory } from 'vue-router';
import { authGuard } from './auth-guard';
import { dateGuard } from './date-guard';
import { uuidGuard } from './uuid-guard';

export default createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  strict: true,
  sensitive: true,
  routes: [
    {
      name: 'confirm-signup',
      path: '/confirm-signup',
      component: ConfirmSignUpView,
      props: route => ({ email: route.query.email })
    },
    {
      path: '/login',
      component: LoginView,
      props: route => ({ redirect: route.query.redirect })
    },
    {
      path: '/signup',
      component: SignUpView
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
          component: MeasurementListView
        },
        {
          path: 'measurements/:date',
          component: MeasurementEditView,
          beforeEnter: dateGuard,
          props: true
        },
        {
          path: 'workouts',
          component: WorkoutListView
        },
        {
          path: 'workouts/:id',
          component: WorkoutEditView,
          beforeEnter: uuidGuard,
          props: true
        },
      ]
    },
  ]
});
