<template>
  <div class="gallery-view">
    <header class="gallery-header">
      <div class="header-left">
        <h1>iMAGE</h1>
        <span class="connection-status" :class="{ connected: connectionStore.isConnected }">
          <WifiIcon :size="14" />
          <template v-if="connectionStore.isConnected">
            {{ connectionStore.storageType === 'github' ? 'GitHub' : 'EC2' }} Connected
          </template>
          <template v-else>Disconnected</template>
        </span>
      </div>
      <div class="header-actions">
        <button
          class="icon-btn"
          :class="{ active: connectionStore.isMediaOnly }"
          @click="connectionStore.toggleMediaOnly"
          title="Show only photos/videos"
        >
          <ImageIcon :size="20" />
        </button>
        <button class="icon-btn" @click="handleLogout" title="Disconnect">
          <LogOutIcon :size="20" />
        </button>
      </div>
    </header>

    <div class="breadcrumb">
      <button @click="goToRoot" class="breadcrumb-item root">
        <HomeIcon :size="16" />
        <span>Home</span>
      </button>
      <template v-if="currentPathParts.length > 0">
        <ChevronRightIcon :size="14" class="breadcrumb-separator" />
        <template v-for="(part, index) in currentPathParts" :key="index">
          <button
            class="breadcrumb-item"
            @click="navigateToPath(index)"
            :class="{ active: index === currentPathParts.length - 1 }"
          >
            {{ part }}
          </button>
          <ChevronRightIcon
            v-if="index < currentPathParts.length - 1"
            :size="14"
            class="breadcrumb-separator"
          />
        </template>
      </template>
    </div>

    <main class="gallery-content">
      <div v-if="connectionStore.isLoadingFiles" class="loading">
        <LoaderIcon class="spin" :size="32" />
        <p>Loading files...</p>
      </div>

      <div v-else-if="connectionStore.albums.length === 0 && connectionStore.mediaFiles.length === 0" class="empty">
        <FolderOpenIcon :size="48" />
        <p>This folder is empty</p>
      </div>

      <template v-else>
        <!-- Albums Section -->
        <section v-if="connectionStore.albums.length > 0" class="albums-section">
          <h2 class="section-title">Albums</h2>
          <div class="albums-grid">
            <AlbumCard
              v-for="album in connectionStore.albums"
              :key="album.path"
              :album="album"
              @click="openAlbum(album)"
            />
          </div>
        </section>

        <!-- Media Section -->
        <section v-if="connectionStore.mediaFiles.length > 0" class="media-section">
          <h2 class="section-title">
            {{ connectionStore.isMediaOnly ? 'Photos & Videos' : 'All Files' }}
          </h2>
          <div class="media-grid">
            <MediaCard
              v-for="file in connectionStore.mediaFiles"
              :key="file.path"
              :file="file"
              @click="openFile(file)"
            />
          </div>
        </section>
      </template>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useConnectionStore, type FileInfo } from '../stores/connection'
import AlbumCard from '../components/AlbumCard.vue'
import MediaCard from '../components/MediaCard.vue'
import {
  WifiIcon,
  LogOutIcon,
  HomeIcon,
  ChevronRightIcon,
  LoaderIcon,
  FolderOpenIcon,
  ImageIcon,
} from 'lucide-vue-next'

const router = useRouter()
const connectionStore = useConnectionStore()

const currentPathParts = computed(() => {
  const parts = connectionStore.currentPath.split('/').filter(Boolean)
  return parts
})

function goToRoot() {
  connectionStore.loadFiles(connectionStore.rootPath)
}

function navigateToPath(index: number) {
  const parts = currentPathParts.value.slice(0, index + 1)
  const path = '/' + parts.join('/')
  connectionStore.loadFiles(path)
}

function openAlbum(album: FileInfo) {
  connectionStore.loadFiles(album.path)
}

function openFile(file: FileInfo) {
  const isImage = file.mimeType?.startsWith('image/')
  const isVideo = file.mimeType === 'video'
  
  if (isImage || isVideo) {
    router.push({
      name: 'viewer',
      query: {
        file: file.path,
        type: file.mimeType,
        name: file.name,
      },
    })
  }
}

async function handleLogout() {
  await connectionStore.disconnect()
  router.push('/')
}
</script>

<style scoped>
.gallery-view {
  min-height: 100vh;
  background: var(--color-bg);
  display: flex;
  flex-direction: column;
}

.gallery-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-md) var(--spacing-md);
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  position: sticky;
  top: 0;
  z-index: 100;
}

.header-left {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.header-left h1 {
  font-size: 1.25rem;
  font-weight: 600;
}

.connection-status {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.connection-status.connected {
  color: #22c55e;
}

.header-actions {
  display: flex;
  gap: var(--spacing-sm);
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  color: var(--color-text);
  cursor: pointer;
  transition: all 0.2s;
}

.icon-btn:hover {
  background: var(--color-hover);
}

.icon-btn.active {
  background: var(--color-primary);
  color: white;
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  overflow-x: auto;
  white-space: nowrap;
  -webkit-overflow-scrolling: touch;
}

.breadcrumb-item {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-xs);
  background: none;
  border: none;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  cursor: pointer;
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  transition: all 0.2s;
}

.breadcrumb-item:hover {
  background: var(--color-hover);
  color: var(--color-text);
}

.breadcrumb-item.active {
  color: var(--color-text);
  font-weight: 500;
}

.breadcrumb-separator {
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.gallery-content {
  flex: 1;
  padding: var(--spacing-md);
  overflow-y: auto;
}

.loading,
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-md);
  min-height: 300px;
  color: var(--color-text-secondary);
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.section-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-md);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.albums-section {
  margin-bottom: var(--spacing-xl);
}

.albums-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: var(--spacing-md);
}

.media-section {
  margin-bottom: var(--spacing-xl);
}

.media-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: var(--spacing-xs);
}

@media (min-width: 768px) {
  .albums-grid {
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  }
  
  .media-grid {
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  }
}
</style>
