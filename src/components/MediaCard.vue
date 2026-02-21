<template>
  <div ref="cardRef" class="media-card" :class="{ 'is-video': isVideo, 'is-image': isImage }">
    <div class="media-thumbnail">
      <template v-if="isImage">
        <img
          v-if="thumbnailLoaded"
          :src="thumbnailUrl"
          :alt="file.name"
          class="thumbnail-image"
          @error="onThumbnailError"
        />
        <div v-else class="image-placeholder" :style="{ backgroundColor: getRandomColor() }">
          <ImageIcon :size="24" />
        </div>
      </template>
      <template v-else-if="isVideo">
        <div class="video-placeholder">
          <PlayIcon :size="24" />
        </div>
      </template>
      <template v-else>
        <div class="file-placeholder">
          <FileIcon :size="24" />
        </div>
      </template>
      
      <div v-if="isVideo" class="video-indicator">
        <VideoIcon :size="12" />
      </div>
    </div>
    <div class="media-info">
      <span class="media-name">{{ file.name }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ImageIcon, PlayIcon, FileIcon, VideoIcon } from 'lucide-vue-next'
import type { FileInfo } from '../stores/connection'

interface Props {
  file: FileInfo
  index?: number
}

const props = defineProps<Props>()

const isImage = computed(() => props.file.mimeType?.startsWith('image/'))
const isVideo = computed(() => props.file.mimeType?.startsWith('video/'))

const thumbnailLoaded = ref(false)
const thumbnailUrl = ref('')
const cardRef = ref<HTMLElement | null>(null)
const hasBeenVisible = ref(false)

const colors = [
  '#667eea', '#764ba2', '#f093fb', '#f5576c',
  '#4facfe', '#00f2fe', '#43e97b', '#38f9d7',
  '#fa709a', '#fee140', '#30cfd0', '#330867'
]

function getRandomColor() {
  const index = props.file.name.charCodeAt(0) % colors.length
  return colors[index]
}

function onThumbnailError() {
  thumbnailLoaded.value = false
}

async function loadThumbnail() {
  if (!isImage.value || thumbnailLoaded.value) return
  
  try {
    const dataUrl = await invoke<string>('get_file_thumbnail', { 
      path: props.file.path, 
      maxSize: 200 
    })
    thumbnailUrl.value = dataUrl
    thumbnailLoaded.value = true
  } catch {
    thumbnailLoaded.value = false
  }
}

onMounted(() => {
  if (!isImage.value) return
  
  // Use Intersection Observer for lazy loading
  if (cardRef.value) {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting && !hasBeenVisible.value) {
            hasBeenVisible.value = true
            loadThumbnail()
            observer.disconnect()
          }
        })
      },
      { rootMargin: '100px' } // Start loading 100px before visible
    )
    observer.observe(cardRef.value)
    
    // Store observer for cleanup
    onUnmounted(() => observer.disconnect())
  }
})
</script>

<style scoped>
.media-card {
  position: relative;
  aspect-ratio: 1;
  background: var(--color-surface);
  border-radius: var(--radius-sm);
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.2s;
}

.media-card:hover {
  transform: scale(1.02);
}

.media-thumbnail {
  width: 100%;
  height: 100%;
  position: relative;
}

.thumbnail-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.image-placeholder,
.video-placeholder,
.file-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.image-placeholder {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.video-placeholder {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.file-placeholder {
  background: var(--color-surface-light);
  color: var(--color-text-secondary);
}

.video-indicator {
  position: absolute;
  top: var(--spacing-xs);
  right: var(--spacing-xs);
  background: rgba(0, 0, 0, 0.6);
  border-radius: var(--radius-sm);
  padding: 2px 4px;
  color: white;
}

.media-info {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.8), transparent);
  padding: var(--spacing-sm);
  padding-top: var(--spacing-lg);
}

.media-name {
  font-size: 0.75rem;
  color: white;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: block;
}
</style>
