<template>
  <div class="login-view">
    <div class="login-container">
      <div class="logo">
        <div class="logo-icon">
          <ImageIcon :size="48" />
        </div>
        <h1>iMAGE</h1>
        <p class="subtitle">Connect to your EC2 instance</p>
      </div>

      <form @submit.prevent="handleConnect" class="login-form">
        <div class="form-group">
          <label for="host">IP Address / Host</label>
          <input
            id="host"
            v-model="form.host"
            type="text"
            placeholder="192.168.1.1 or ec2-xxx.compute.amazonaws.com"
            required
          />
        </div>

        <div class="form-group">
          <label for="username">Username</label>
          <input
            id="username"
            v-model="form.username"
            type="text"
            placeholder="ec2-user, ubuntu, root..."
            required
          />
        </div>

        <div class="form-group">
          <label for="port">Port</label>
          <input
            id="port"
            v-model="form.port"
            type="number"
            placeholder="22"
          />
        </div>

        <div class="form-group">
          <label for="pem">SSH Key (.pem file)</label>
          <div class="file-input-wrapper">
            <input
              type="file"
              id="pem"
              accept=".pem,.key,.txt"
              @change="handleFileChange"
              class="file-input"
            />
            <label for="pem" class="file-label">
              <UploadIcon :size="20" />
              <span>{{ pemFileName || 'Choose PEM file' }}</span>
            </label>
          </div>
        </div>

        <div v-if="connectionStore.error" class="error-message">
          <AlertCircleIcon :size="16" />
          <span>{{ connectionStore.error }}</span>
        </div>

        <button
          type="submit"
          class="connect-btn"
          :disabled="connectionStore.isConnecting || !isFormValid"
        >
          <LoaderIcon v-if="connectionStore.isConnecting" class="spin" :size="20" />
          <LogInIcon v-else :size="20" />
          <span>{{ connectionStore.isConnecting ? 'Connecting...' : 'Connect' }}</span>
        </button>
      </form>

      <div v-if="connectionStore.savedConfig" class="saved-connection">
        <p>Previously connected to:</p>
        <button @click="useSavedConnection" class="saved-btn">
          <ServerIcon :size="16" />
          <span>{{ connectionStore.savedConfig.username }}@{{ connectionStore.savedConfig.host }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useConnectionStore, type ConnectionConfig } from '../stores/connection'
import {
  ImageIcon,
  UploadIcon,
  LogInIcon,
  LoaderIcon,
  AlertCircleIcon,
  ServerIcon,
} from 'lucide-vue-next'

const router = useRouter()
const connectionStore = useConnectionStore()

const form = ref({
  host: '',
  username: '',
  port: 22,
  pemContent: '',
})

const pemFileName = ref('')

const isFormValid = computed(() => {
  return form.value.host && form.value.username && form.value.pemContent
})

async function handleFileChange(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  
  if (file) {
    pemFileName.value = file.name
    const reader = new FileReader()
    reader.onload = (e) => {
      const content = e.target?.result as string
      // Remove header/footer and whitespace for base64 encoding
      form.value.pemContent = btoa(content)
    }
    reader.readAsText(file)
  }
}

async function handleConnect() {
  const config: ConnectionConfig = {
    host: form.value.host,
    username: form.value.username,
    pemContent: form.value.pemContent,
    port: form.value.port || 22,
  }

  const success = await connectionStore.connect(config)
  if (success) {
    router.push('/gallery')
  }
}

async function useSavedConnection() {
  if (connectionStore.savedConfig) {
    const success = await connectionStore.connect(connectionStore.savedConfig)
    if (success) {
      router.push('/gallery')
    }
  }
}
</script>

<style scoped>
.login-view {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-md);
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
}

.login-container {
  width: 100%;
  max-width: 400px;
  background: var(--color-surface);
  border-radius: var(--radius-xl);
  padding: var(--spacing-xl);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
}

.logo {
  text-align: center;
  margin-bottom: var(--spacing-xl);
}

.logo-icon {
  width: 80px;
  height: 80px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: var(--radius-lg);
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto var(--spacing-md);
}

.logo h1 {
  font-size: 1.75rem;
  font-weight: 700;
  margin-bottom: var(--spacing-xs);
}

.subtitle {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.form-group label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.form-group input {
  background: var(--color-surface-light);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-sm) var(--spacing-md);
  color: var(--color-text);
  font-size: 1rem;
  transition: border-color 0.2s;
}

.form-group input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.file-input-wrapper {
  position: relative;
}

.file-input {
  position: absolute;
  opacity: 0;
  width: 100%;
  height: 100%;
  cursor: pointer;
}

.file-label {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  background: var(--color-surface-light);
  border: 2px dashed var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  cursor: pointer;
  transition: all 0.2s;
}

.file-label:hover {
  border-color: var(--color-primary);
  background: var(--color-hover);
}

.file-label span {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}

.error-message {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
}

.connect-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-sm);
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.2s, transform 0.2s;
}

.connect-btn:hover:not(:disabled) {
  opacity: 0.9;
  transform: translateY(-1px);
}

.connect-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.saved-connection {
  margin-top: var(--spacing-lg);
  padding-top: var(--spacing-lg);
  border-top: 1px solid var(--color-border);
  text-align: center;
}

.saved-connection p {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  margin-bottom: var(--spacing-sm);
}

.saved-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-sm);
  background: var(--color-surface-light);
  border: 1px solid var(--color-border);
  color: var(--color-text);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: 0.875rem;
  transition: all 0.2s;
}

.saved-btn:hover {
  background: var(--color-hover);
  border-color: var(--color-primary);
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
