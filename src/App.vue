<template>
  <div id="app">
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useConnectionStore, type ConnectionConfig } from './stores/connection'

const connectionStore = useConnectionStore()

onMounted(() => {
  const savedConnection = localStorage.getItem('image-connection')
  if (savedConnection) {
    try {
      const config = JSON.parse(savedConnection) as ConnectionConfig
      connectionStore.savedConfig = config
    } catch (e) {
      console.error('Failed to parse saved connection:', e)
      localStorage.removeItem('image-connection')
    }
  }
})
</script>
