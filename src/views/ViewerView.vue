<template>
  <div class="viewer-view">
    <header class="viewer-header">
      <button @click="goBack" class="back-btn">
        <ArrowLeftIcon :size="24" />
      </button>
      <h1 class="file-name">{{ fileName }}</h1>
      <div class="header-actions">
        <button v-if="hasNavigation" @click="navigatePrev" class="nav-btn" title="Previous (Left Arrow)">
          <ChevronLeftIcon :size="20" />
        </button>
        <button v-if="hasNavigation" @click="navigateNext" class="nav-btn" title="Next (Right Arrow)">
          <ChevronRightIcon :size="20" />
        </button>
        <button @click="downloadFile" class="action-btn" title="Download">
          <DownloadIcon :size="20" />
        </button>
      </div>
    </header>

    <main class="viewer-content" @wheel="handleZoom" @mousedown="startPan" @mousemove="pan" @mouseup="endPan" @mouseleave="endPan">
      <div v-if="isLoading" class="loading">
        <LoaderIcon class="spin" :size="48" />
        <p>Loading...</p>
      </div>

      <div v-else-if="error" class="error">
        <AlertCircleIcon :size="48" />
        <p>{{ error }}</p>
        <button @click="goBack" class="back-error-btn">Go Back</button>
      </div>

      <template v-else>
        <img
          v-if="isImage"
          :src="fileData"
          :alt="fileName"
          class="media-content"
          :style="imageStyle"
          @load="onImageLoad"
          @error="onImageError"
          draggable="false"
        />
        
        <video
          v-else-if="isVideo"
          :src="fileData"
          controls
          class="media-content"
          @loadeddata="isLoading = false"
          @error="onVideoError"
        />

        <div v-else class="unsupported">
          <FileIcon :size="64" />
          <p>This file type cannot be previewed</p>
          <button @click="downloadFile" class="download-btn">
            <DownloadIcon :size="16" />
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
        <span class="info-label">Type:</span>
        <span class="info-value">{{ fileInfo.mimeType || 'Unknown' }}</span>
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

    <div v-if="showZoomControls && isImage" class="zoom-controls">
      <button @click="zoomOut" class="zoom-btn">
        <ZoomOutIcon :size="20" />
      </button>
      <span class="zoom-level">{{ Math.round(zoom * 100) }}%</span>
      <button @click="zoomIn" class="zoom-btn">
        <ZoomInIcon :size="20" />
      </button>
      <button @click="resetZoom" class="zoom-btn">
        <Maximize2Icon :size="20" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useConnectionStore, type FileInfo } from '../stores/connection'
import {
  ArrowLeftIcon,
  DownloadIcon,
  LoaderIcon,
  AlertCircleIcon,
  FileIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ZoomInIcon,
  ZoomOutIcon,
  Maximize2Icon,
} from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const connectionStore = useConnectionStore()

const filePath = computed(() => route.query.file as string)
const fileType = computed(() => route.query.type as string)
const fileName = computed(() => route.query.name as string || 'Unknown')
const fileIndex = computed(() => parseInt(route.query.index as string) || 0)

const isLoading = ref(true)
const error = ref('')
const fileData = ref('')
const fileInfo = ref<FileInfo | null>(null)

const zoom = ref(1)
const panX = ref(0)
const panY = ref(0)
const isPanning = ref(false)
const panStartX = ref(0)
const panStartY = ref(0)

const isImage = computed(() => fileType.value?.startsWith('image/'))
const isVideo = computed(() => fileType.value?.startsWith('video/'))
const showZoomControls = computed(() => isImage.value && !isLoading.value && !error.value)

const hasNavigation = computed(() => {
  return connectionStore.mediaFiles.length > 1
})

const imageStyle = computed(() => ({
  transform: `scale(${zoom.value}) translate(${panX.value}px, ${panY.value}px)`,
  cursor: isPanning.value ? 'grabbing' : 'grab',
}))

function onImageLoad() {
  isLoading.value = false
}

function onImageError() {
  isLoading.value = false
  error.value = 'Failed to load image'
}

function onVideoError() {
  isLoading.value = false
  error.value = 'Failed to load video'
}

function getVideoMimeType(): string {
  const ext = fileName.value.split('.').pop()?.toLowerCase() || ''
  const mimeTypes: Record<string, string> = {
    mp4: 'video/mp4',
    mov: 'video/quicktime',
    avi: 'video/x-msvideo',
    mkv: 'video/x-matroska',
    webm: 'video/webm',
    m4v: 'video/x-m4v',
  }
  return mimeTypes[ext] || 'video/mp4'
}

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
      fileData.value = `data:${getVideoMimeType()};base64,${base64Data}`
    }

    const foundFile = connectionStore.mediaFiles.find(f => f.path === filePath.value)
    if (foundFile) {
      fileInfo.value = foundFile
    } else {
      fileInfo.value = {
        name: fileName.value,
        path: filePath.value,
        size: 0,
        isDir: false,
        modified: undefined,
        mimeType: fileType.value,
      }
    }
  } catch (e) {
    error.value = String(e)
    isLoading.value = false
  }

  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowLeft') {
    navigatePrev()
  } else if (e.key === 'ArrowRight') {
    navigateNext()
  } else if (e.key === 'Escape') {
    goBack()
  } else if (e.key === '+' || e.key === '=') {
    zoomIn()
  } else if (e.key === '-') {
    zoomOut()
  } else if (e.key === '0') {
    resetZoom()
  }
}

function navigatePrev() {
  if (!hasNavigation.value) return
  const files = connectionStore.mediaFiles
  const currentIndex = fileIndex.value
  const prevIndex = currentIndex > 0 ? currentIndex - 1 : files.length - 1
  const prevFile = files[prevIndex]
  
  router.replace({
    name: 'viewer',
    query: {
      file: prevFile.path,
      type: prevFile.mimeType,
      name: prevFile.name,
      index: prevIndex.toString(),
    },
  })
}

function navigateNext() {
  if (!hasNavigation.value) return
  const files = connectionStore.mediaFiles
  const currentIndex = fileIndex.value
  const nextIndex = currentIndex < files.length - 1 ? currentIndex + 1 : 0
  const nextFile = files[nextIndex]
  
  router.replace({
    name: 'viewer',
    query: {
      file: nextFile.path,
      type: nextFile.mimeType,
      name: nextFile.name,
      index: nextIndex.toString(),
    },
  })
}

function handleZoom(e: WheelEvent) {
  if (!isImage.value || isLoading.value) return
  if (e.deltaY < 0) {
    zoom.value = Math.min(zoom.value * 1.1, 5)
  } else {
    zoom.value = Math.max(zoom.value / 1.1, 0.1)
  }
}

function startPan(e: MouseEvent) {
  if (!isImage.value || zoom.value <= 1) return
  isPanning.value = true
  panStartX.value = e.clientX - panX.value
  panStartY.value = e.clientY - panY.value
}

function pan(e: MouseEvent) {
  if (!isPanning.value) return
  panX.value = e.clientX - panStartX.value
  panY.value = e.clientY - panStartY.value
}

function endPan() {
  isPanning.value = false
}

function zoomIn() {
  zoom.value = Math.min(zoom.value * 1.2, 5)
}

function zoomOut() {
  zoom.value = Math.max(zoom.value / 1.2, 0.1)
}

function resetZoom() {
  zoom.value = 1
  panX.value = 0
  panY.value = 0
}

function goBack() {
  router.back()
}

function downloadFile() {
  if (!fileData.value) return
  const link = document.createElement('a')
  link.href = fileData.value
  link.download = fileName.value
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

function formatDate(timestamp: number | undefined): string {
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

.header-actions {
  display: flex;
  gap: var(--spacing-xs);
}

.back-btn,
.action-btn,
.nav-btn {
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
.action-btn:hover,
.nav-btn:hover {
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
  position: relative;
}

.media-content {
  max-width: 100%;
  max-height: 70vh;
  object-fit: contain;
  border-radius: var(--radius-md);
  transition: transform 0.1s ease-out;
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

.back-error-btn,
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

.back-error-btn:hover,
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

.zoom-controls {
  position: fixed;
  bottom: 100px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  background: var(--color-surface);
  padding: var(--spacing-sm);
  border-radius: var(--radius-lg);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.zoom-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  color: var(--color-text);
  cursor: pointer;
  transition: background 0.2s;
}

.zoom-btn:hover {
  background: var(--color-hover);
}

.zoom-level {
  font-size: 0.875rem;
  color: var(--color-text);
  min-width: 50px;
  text-align: center;
}
</style>
