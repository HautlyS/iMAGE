<template>
  <div id="app">
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useConnectionStore } from './stores/connection'

const connectionStore = useConnectionStore()

onMounted(() => {
  // Check for saved connection on startup
  const savedConnection = localStorage.getItem('ec2-connection')
  if (savedConnection) {
    try {
      const config = JSON.parse(savedConnection)
      connectionStore.savedConfig = config
    } catch (e) {
      console.error('Failed to parse saved connection:', e)
    }
  }
})
</script>
