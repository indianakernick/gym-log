import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '@/views/HomeView.vue';
import LoginView from '@/views/LoginView.vue';
import MeasurementEditView from '@/views/MeasurementEditView.vue';
import MeasurementListView from '@/views/MeasurementListView.vue';
import SignUpView from '@/views/SignUpView.vue';
import WorkoutEditView from '@/views/WorkoutEditView.vue';
import WorkoutListView from '@/views/WorkoutListView.vue';
import { authGuard } from './auth-guard';
import { dateGuard } from './date-guard';
import { uuidGuard } from './uuid-guard';

export default createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  strict: true,
  sensitive: true,
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
      beforeEnter: authGuard
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
      path: '/measurements',
      component: MeasurementListView,
      beforeEnter: authGuard
    },
    {
      path: '/measurements/:date',
      component: MeasurementEditView,
      beforeEnter: [dateGuard, authGuard],
      props: true
    },
    {
      path: '/workouts',
      component: WorkoutListView,
      beforeEnter: authGuard
    },
    {
      path: '/workouts/:id',
      component: WorkoutEditView,
      beforeEnter: [uuidGuard, authGuard],
      props: true
    }
  ]
});
