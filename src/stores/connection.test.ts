import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useConnectionStore, type Ec2ConnectionConfig, type GitHubConnectionConfig } from '../stores/connection'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'

describe('connection store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    localStorage.clear()
  })

  describe('initial state', () => {
    it('should start disconnected', () => {
      const store = useConnectionStore()
      expect(store.isConnected).toBe(false)
      expect(store.isConnecting).toBe(false)
      expect(store.error).toBeNull()
      expect(store.savedConfig).toBeNull()
      expect(store.storageType).toBeNull()
    })
  })

  describe('filteredFiles', () => {
    it('should return all files when isMediaOnly is false', () => {
      const store = useConnectionStore()
      store.files = [
        { name: 'image.jpg', path: '/image.jpg', size: 100, isDir: false, mimeType: 'image/jpeg' },
        { name: 'doc.pdf', path: '/doc.pdf', size: 200, isDir: false, mimeType: 'application/pdf' },
        { name: 'folder', path: '/folder', size: 0, isDir: true }
      ]
      
      expect(store.filteredFiles).toHaveLength(3)
    })

    it('should filter to only media files when isMediaOnly is true', () => {
      const store = useConnectionStore()
      store.files = [
        { name: 'image.jpg', path: '/image.jpg', size: 100, isDir: false, mimeType: 'image/jpeg' },
        { name: 'video.mp4', path: '/video.mp4', size: 100, isDir: false, mimeType: 'video' },
        { name: 'doc.pdf', path: '/doc.pdf', size: 200, isDir: false, mimeType: 'application/pdf' },
        { name: 'folder', path: '/folder', size: 0, isDir: true }
      ]
      store.isMediaOnly = true
      
      expect(store.filteredFiles).toHaveLength(3)
      expect(store.filteredFiles.find(f => f.name === 'doc.pdf')).toBeUndefined()
      expect(store.filteredFiles.find(f => f.name === 'folder')).toBeDefined()
    })
  })

  describe('albums and mediaFiles', () => {
    it('should separate albums and media files', () => {
      const store = useConnectionStore()
      store.files = [
        { name: 'image.jpg', path: '/image.jpg', size: 100, isDir: false, mimeType: 'image/jpeg' },
        { name: 'folder1', path: '/folder1', size: 0, isDir: true },
        { name: 'folder2', path: '/folder2', size: 0, isDir: true }
      ]
      
      expect(store.albums).toHaveLength(2)
      expect(store.mediaFiles).toHaveLength(1)
    })
  })

  describe('toggleMediaOnly', () => {
    it('should toggle isMediaOnly', () => {
      const store = useConnectionStore()
      expect(store.isMediaOnly).toBe(false)
      
      store.toggleMediaOnly()
      expect(store.isMediaOnly).toBe(true)
      
      store.toggleMediaOnly()
      expect(store.isMediaOnly).toBe(false)
    })
  })

  describe('connectEc2', () => {
    it('should call connect_ec2 with correct parameters', async () => {
      const store = useConnectionStore()
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockResolvedValueOnce({
        success: true,
        message: 'Connected',
        storage_type: 'ec2',
        root_path: '/home/ubuntu'
      })

      const config: Ec2ConnectionConfig = {
        type: 'ec2',
        host: '192.168.1.1',
        username: 'ubuntu',
        pemContent: 'dGVzdCBrZXk=',
        port: 22
      }

      const result = await store.connectEc2(config)
      
      expect(result).toBe(true)
      expect(mockInvoke).toHaveBeenCalledWith('connect_ec2', {
        request: {
          host: '192.168.1.1',
          username: 'ubuntu',
          pem_content: 'dGVzdCBrZXk=',
          port: 22
        }
      })
      expect(store.isConnected).toBe(true)
      expect(store.storageType).toBe('ec2')
    })

    it('should handle connection failure', async () => {
      const store = useConnectionStore()
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockResolvedValueOnce({
        success: false,
        message: 'Connection failed',
        storage_type: null,
        root_path: null
      })

      const config: Ec2ConnectionConfig = {
        type: 'ec2',
        host: 'invalid-host',
        username: 'ubuntu',
        pemContent: 'dGVzdA==',
        port: 22
      }

      const result = await store.connectEc2(config)
      
      expect(result).toBe(false)
      expect(store.isConnected).toBe(false)
      expect(store.error).toBe('Connection failed')
    })
  })

  describe('connectGithub', () => {
    it('should call connect_github with correct parameters', async () => {
      const store = useConnectionStore()
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockResolvedValueOnce({
        success: true,
        message: 'Connected',
        storage_type: 'github',
        root_path: '/'
      })

      const config: GitHubConnectionConfig = {
        type: 'github',
        repoUrl: 'git@github.com:test/repo.git',
        username: 'git',
        sshKeyContent: 'dGVzdCBrZXk=',
        branch: 'main',
        localPath: '/tmp/repo'
      }

      const result = await store.connectGithub(config)
      
      expect(result).toBe(true)
      expect(mockInvoke).toHaveBeenCalledWith('connect_github', {
        request: {
          repo_url: 'git@github.com:test/repo.git',
          username: 'git',
          ssh_key_content: 'dGVzdCBrZXk=',
          branch: 'main',
          local_path: '/tmp/repo'
        }
      })
      expect(store.isConnected).toBe(true)
      expect(store.storageType).toBe('github')
    })
  })

  describe('disconnect', () => {
    it('should clear connection state', async () => {
      const store = useConnectionStore()
      const mockInvoke = vi.mocked(invoke)
      
      store.isConnected = true
      store.storageType = 'ec2'
      store.files = [{ name: 'test', path: '/test', size: 100, isDir: false }]
      
      await store.disconnect()
      
      expect(mockInvoke).toHaveBeenCalledWith('disconnect')
      expect(store.isConnected).toBe(false)
      expect(store.storageType).toBeNull()
      expect(store.files).toEqual([])
    })
  })

  describe('localStorage', () => {
    it('should save config to localStorage on successful EC2 connection', async () => {
      const store = useConnectionStore()
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockResolvedValueOnce({
        success: true,
        message: 'Connected',
        storage_type: 'ec2',
        root_path: '/home/ubuntu'
      })

      const config: Ec2ConnectionConfig = {
        type: 'ec2',
        host: '192.168.1.1',
        username: 'ubuntu',
        pemContent: 'dGVzdA==',
        port: 22
      }

      await store.connectEc2(config)
      
      const saved = JSON.parse(localStorage.getItem('image-connection') || '{}')
      expect(saved.type).toBe('ec2')
      expect(saved.host).toBe('192.168.1.1')
    })

    it('should remove config from localStorage on disconnect', async () => {
      const store = useConnectionStore()
      localStorage.setItem('image-connection', JSON.stringify({ type: 'ec2', host: 'test' }))
      
      await store.disconnect()
      
      expect(localStorage.getItem('image-connection')).toBeNull()
    })
  })
})
