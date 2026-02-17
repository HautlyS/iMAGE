<template>
  <div class="media-card" :class="{ 'is-video': isVideo, 'is-image': isImage }">
    <div class="media-thumbnail">
      <template v-if="isImage">
        <div class="image-placeholder" :style="{ backgroundColor: getRandomColor() }">
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
import { computed } from 'vue'
import { ImageIcon, PlayIcon, FileIcon, VideoIcon } from 'lucide-vue-next'
import type { FileInfo } from '../stores/connection'

interface Props {
  file: FileInfo
}

const props = defineProps<Props>()

const isImage = computed(() => props.file.mimeType?.startsWith('image/'))
const isVideo = computed(() => props.file.mimeType === 'video')

const colors = [
  '#667eea', '#764ba2', '#f093fb', '#f5576c',
  '#4facfe', '#00f2fe', '#43e97b', '#38f9d7',
  '#fa709a', '#fee140', '#30cfd0', '#330867'
]

function getRandomColor() {
  const index = props.file.name.charCodeAt(0) % colors.length
  return colors[index]
}
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
