import { createRouter, createWebHistory } from 'vue-router'
import LoginView from '../views/LoginView.vue'
import GalleryView from '../views/GalleryView.vue'
import AlbumView from '../views/AlbumView.vue'
import ViewerView from '../views/ViewerView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'login',
      component: LoginView,
    },
    {
      path: '/gallery',
      name: 'gallery',
      component: GalleryView,
    },
    {
      path: '/album/:path(.*)',
      name: 'album',
      component: AlbumView,
      props: true,
    },
    {
      path: '/viewer',
      name: 'viewer',
      component: ViewerView,
      props: (route) => ({ 
        file: route.query.file,
        type: route.query.type,
      }),
    },
  ],
})

export default router
