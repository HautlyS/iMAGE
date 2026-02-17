<template>
  <div class="viewer-view">
    <header class="viewer-header">
      <button @click="goBack" class="back-btn">
        <ArrowLeftIcon :size="24" />
      </button>
      <h1 class="file-name">{{ fileName }}</h1>
      <button @click="downloadFile" class="action-btn">
        <DownloadIcon :size={20} />
      </button>
    </header>

    <main class="viewer-content">
      <div v-if="isLoading" class="loading">
        <LoaderIcon class="spin" :size="48" />
      </div>

      <div v-else-if="error" class="error">
        <AlertCircleIcon :size="48" />
        <p>{{ error }}</p>
      </div>

      <template v-else>
        <img
          v-if="isImage"
          :src="fileData"
          :alt="fileName"
          class="media-content"
          @load="isLoading = false"
        />
        
        <video
          v-else-if="isVideo"
          :src="fileData"
          controls
          class="media-content"
          @loadeddata="isLoading = false"
        />

        <div v-else class="unsupported">
          <FileIcon :size="64" />
          <p>This file type cannot be previewed</p>
          <button @click="downloadFile" class="download-btn">
            <DownloadIcon :size={16} />
            Download File
          </button>
        </div>
      </template>
    </main>

    <div v-if="fileInfo" class="file-info">
      <div class="info-row">
        <span class="info-label">Size:</span>
        <span class="info-value">{{ formatFileSize(fileInfo.size) }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">Modified:</span>
        <span class="info-value">{{ formatDate(fileInfo.modified) }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">Path:</span>
        <span class="info-value path">{{ fileInfo.path }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import {
  ArrowLeftIcon,
  DownloadIcon,
  LoaderIcon,
  AlertCircleIcon,
  FileIcon,
} from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()

const filePath = computed(() => route.query.file as string)
const fileType = computed(() => route.query.type as string)
const fileName = computed(() => route.query.name as string || 'Unknown')

const isLoading = ref(true)
const error = ref('')
const fileData = ref('')
const fileInfo = ref<any>(null)

const isImage = computed(() => fileType.value?.startsWith('image/'))
const isVideo = computed(() => fileType.value === 'video')

onMounted(async () => {
  if (!filePath.value) {
    error.value = 'No file specified'
    isLoading.value = false
    return
  }

  try {
    const base64Data = await invoke<string>('read_file', { path: filePath.value })
    
    if (isImage.value) {
      fileData.value = `data:${fileType.value};base64,${base64Data}`
    } else if (isVideo.value) {
      fileData.value = `data:video/mp4;base64,${base64Data}`
    }

    // Get file info from the store or reload
    fileInfo.value = {
      path: filePath.value,
      size: 0,
      modified: Date.now(),
    }
  } catch (e) {
    error.value = String(e)
  } finally {
    if (!isImage.value && !isVideo.value) {
      isLoading.value = false
    }
  }
})

function goBack() {
  router.back()
}

function downloadFile() {
  // Implementation for downloading file
  const link = document.createElement('a')
  link.href = fileData.value
  link.download = fileName.value
  link.click()
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

function formatDate(timestamp: number): string {
  if (!timestamp) return 'Unknown'
  return new Date(timestamp * 1000).toLocaleString()
}
</script>

<style scoped>
.viewer-view {
  min-height: 100vh;
  background: var(--color-bg);
  display: flex;
  flex-direction: column;
}

.viewer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-md);
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  position: sticky;
  top: 0;
  z-index: 100;
}

.back-btn,
.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  color: var(--color-text);
  cursor: pointer;
  transition: all 0.2s;
}

.back-btn:hover,
.action-btn:hover {
  background: var(--color-hover);
}

.file-name {
  flex: 1;
  font-size: 1rem;
  font-weight: 500;
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin: 0 var(--spacing-md);
}

.viewer-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-md);
  overflow: hidden;
}

.media-content {
  max-width: 100%;
  max-height: 70vh;
  object-fit: contain;
  border-radius: var(--radius-md);
}

.loading,
.error,
.unsupported {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-md);
  color: var(--color-text-secondary);
  text-align: center;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.download-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-sm);
  background: var(--color-primary);
  color: white;
  border: none;
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
  cursor: pointer;
  transition: opacity 0.2s;
}

.download-btn:hover {
  opacity: 0.9;
}

.file-info {
  background: var(--color-surface);
  border-top: 1px solid var(--color-border);
  padding: var(--spacing-md);
}

.info-row {
  display: flex;
  justify-content: space-between;
  padding: var(--spacing-xs) 0;
  border-bottom: 1px solid var(--color-border);
}

.info-row:last-child {
  border-bottom: none;
}

.info-label {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}

.info-value {
  font-size: 0.875rem;
  color: var(--color-text);
}

.info-value.path {
  max-width: 60%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: right;
}
</style>
