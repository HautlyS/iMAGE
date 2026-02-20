import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export type StorageType = 'ec2' | 'github'

export interface Ec2ConnectionConfig {
  type: 'ec2'
  host: string
  username: string
  pemContent: string
  port: number
}

export interface GitHubConnectionConfig {
  type: 'github'
  repoUrl: string
  username: string
  sshKeyContent: string
  branch: string
  localPath: string
}

export type ConnectionConfig = Ec2ConnectionConfig | GitHubConnectionConfig

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
  const currentPath = ref('/')
  const files = ref<FileInfo[]>([])
  const isLoadingFiles = ref(false)
  const storageType = ref<StorageType | null>(null)
  const rootPath = ref('/')

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

  async function connectEc2(config: Ec2ConnectionConfig) {
    isConnecting.value = true
    error.value = null
    
    try {
      const response = await invoke<{
        success: boolean
        message: string
        storage_type?: string
        root_path?: string
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
        storageType.value = 'ec2'
        rootPath.value = response.root_path || '/'
        localStorage.setItem('image-connection', JSON.stringify(config))
        await loadFiles(rootPath.value)
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

  async function connectGithub(config: GitHubConnectionConfig) {
    isConnecting.value = true
    error.value = null
    
    try {
      const response = await invoke<{
        success: boolean
        message: string
        storage_type?: string
        root_path?: string
      }>('connect_github', {
        request: {
          repo_url: config.repoUrl,
          username: config.username,
          ssh_key_content: config.sshKeyContent,
          branch: config.branch || 'main',
          local_path: config.localPath || '/tmp/image-repo',
        },
      })

      if (response.success) {
        isConnected.value = true
        savedConfig.value = config
        storageType.value = 'github'
        rootPath.value = response.root_path || '/'
        localStorage.setItem('image-connection', JSON.stringify(config))
        await loadFiles(rootPath.value)
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

  async function connect(config: ConnectionConfig) {
    if (config.type === 'ec2') {
      return connectEc2(config)
    } else if (config.type === 'github') {
      return connectGithub(config)
    }
    error.value = 'Unknown storage type'
    return false
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
      currentPath.value = '/'
      storageType.value = null
      localStorage.removeItem('image-connection')
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
    storageType,
    rootPath,
    connect,
    connectEc2,
    connectGithub,
    loadFiles,
    disconnect,
    toggleMediaOnly,
  }
})
