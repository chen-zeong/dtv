import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from './router';
import { useFollowStore } from './store/followStore'; 
import { useThemeStore } from './stores/theme';
import { useBilibiliStore } from './stores/bilibili';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);

const followStore = useFollowStore(); 
try {
  followStore.loadFollowedStreamers();
} catch (error) {
  console.error('[main.ts] Error initializing follow store:', error);
}


const themeStore = useThemeStore();
try {
  themeStore.initTheme(); 
} catch (error) {
  console.error('[main.ts] Error initializing theme store:', error);
}

// 初始化 B 站 w_webid，仅在软件启动时调用一次
const biliStore = useBilibiliStore();
try {
  biliStore.initWebid();
} catch (error) {
  console.error('[main.ts] Error initializing Bilibili webid:', error);
}

app.mount('#app');
