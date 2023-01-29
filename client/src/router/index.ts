import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '@/views/HomeView.vue';
import LoginView from '@/views/LoginView.vue';
import SignUpView from '@/views/SignUpView.vue';
import { authGuard } from './auth-guard';

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
    }
  ]
});
