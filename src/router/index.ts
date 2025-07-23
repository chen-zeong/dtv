import { createRouter, createWebHistory } from 'vue-router'
import DouyuHomeView from '../pages/DouyuHomeView.vue'
import DouyinHomeView from '../pages/DouyinHomeView.vue'
import DouyuPlayerView from '../pages/DouyuPlayerView.vue';
import DouyinPlayerView from '../pages/DouyinPlayerView.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'DouyuHome',
      component: DouyuHomeView
    },
    {
      path: '/douyin',
      name: 'DouyinHome',
      component: DouyinHomeView
    },
    {
      path: '/player/douyu/:roomId', 
      name: 'douyuPlayer',
      component: DouyuPlayerView,
      props: true
    },
    {
      path: '/player/douyin/:roomId',
      name: 'douyinPlayer',
      component: DouyinPlayerView,
      props: true
    }
  ]
})

export default router