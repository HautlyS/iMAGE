import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ConnectionConfig {
  host: string
  username: string
  pemContent: string
  port: number
}

export interface FileInfo {
  name: string
  path: string
  size: number
  isDir: boolean
  modified?: number
  mimeType?: string
  thumbnail?: string
}

export const useConnectionStore = defineStore('connection', () => {
  const isConnected = ref(false)
  const isConnecting = ref(false)
  const error = ref<string | null>(null)
  const savedConfig = ref<ConnectionConfig | null>(null)
  const currentPath = ref('/home')
  const files = ref<FileInfo[]>([])
  const isLoadingFiles = ref(false)

  const isMediaOnly = ref(false)

  const filteredFiles = computed(() => {
    if (!isMediaOnly.value) return files.value
    
    return files.value.filter(file => {
      if (file.isDir) return true
      const mime = file.mimeType || ''
      return mime.startsWith('image/') || mime === 'video'
    })
  })

  const albums = computed(() => {
    return filteredFiles.value.filter(file => file.isDir)
  })

  const mediaFiles = computed(() => {
    return filteredFiles.value.filter(file => !file.isDir)
  })

  async function connect(config: ConnectionConfig) {
    isConnecting.value = true
    error.value = null
    
    try {
      const response = await invoke<{
        success: boolean
        message: string
      }>('connect_ec2', {
        request: {
          host: config.host,
          username: config.username,
          pem_content: config.pemContent,
          port: config.port || 22,
        },
      })

      if (response.success) {
        isConnected.value = true
        savedConfig.value = config
        localStorage.setItem('ec2-connection', JSON.stringify(config))
        await loadFiles(config.username === 'root' ? '/root' : '/home/' + config.username)
        return true
      } else {
        error.value = response.message
        return false
      }
    } catch (e) {
      error.value = String(e)
      return false
    } finally {
      isConnecting.value = false
    }
  }

  async function loadFiles(path: string) {
    isLoadingFiles.value = true
    currentPath.value = path
    
    try {
      const result = await invoke<FileInfo[]>('list_files', { path })
      files.value = result
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoadingFiles.value = false
    }
  }

  async function disconnect() {
    try {
      await invoke('disconnect')
      isConnected.value = false
      files.value = []
      currentPath.value = '/home'
      localStorage.removeItem('ec2-connection')
    } catch (e) {
      console.error('Disconnect error:', e)
    }
  }

  function toggleMediaOnly() {
    isMediaOnly.value = !isMediaOnly.value
  }

  return {
    isConnected,
    isConnecting,
    error,
    savedConfig,
    currentPath,
    files,
    isLoadingFiles,
    isMediaOnly,
    filteredFiles,
    albums,
    mediaFiles,
    connect,
    loadFiles,
    disconnect,
    toggleMediaOnly,
  }
})
