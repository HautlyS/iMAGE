import { createRouter, createWebHistory } from 'vue-router'
import LoginView from '../views/LoginView.vue'
import GalleryView from '../views/GalleryView.vue'
import ViewerView from '../views/ViewerView.vue'
import { useConnectionStore } from '../stores/connection'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'login',
      component: LoginView,
      meta: { requiresConnection: false },
    },
    {
      path: '/gallery',
      name: 'gallery',
      component: GalleryView,
      meta: { requiresConnection: true },
    },
    {
      path: '/viewer',
      name: 'viewer',
      component: ViewerView,
      meta: { requiresConnection: true },
      props: (route) => ({ 
        file: route.query.file,
        type: route.query.type,
        name: route.query.name,
        index: route.query.index,
      }),
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/',
    },
  ],
})

router.beforeEach((to, _from, next) => {
  const connectionStore = useConnectionStore()
  
  if (to.meta.requiresConnection && !connectionStore.isConnected) {
    next({ name: 'login' })
  } else if (to.name === 'login' && connectionStore.isConnected) {
    next({ name: 'gallery' })
  } else {
    next()
  }
})

export default router
