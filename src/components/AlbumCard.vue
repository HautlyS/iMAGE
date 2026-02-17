<template>
  <div class="album-card">
    <div class="album-thumbnail">
      <FolderIcon :size="40" />
    </div>
    <div class="album-info">
      <h3 class="album-name">{{ album.name }}</h3>
      <span class="album-count">Album</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { FolderIcon } from 'lucide-vue-next'
import type { FileInfo } from '../stores/connection'

interface Props {
  album: FileInfo
}

defineProps<Props>()
</script>

<style scoped>
.album-card {
  background: var(--color-surface);
  border-radius: var(--radius-lg);
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
  border: 1px solid var(--color-border);
}

.album-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}

.album-thumbnail {
  aspect-ratio: 1;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.album-info {
  padding: var(--spacing-sm) var(--spacing-md);
}

.album-name {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 2px;
}

.album-count {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}
</style>
