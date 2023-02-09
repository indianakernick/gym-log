import { createApp } from 'vue';
import { createVfm } from 'vue-final-modal';
import App from './App.vue';
import router from './router';

import './assets/main.css';
import 'vue-final-modal/style.css'

const app = createApp(App);

app.use(router);
app.use(createVfm());

app.mount('#app');
