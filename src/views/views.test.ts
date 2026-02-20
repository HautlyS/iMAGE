import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createRouter, createWebHistory } from 'vue-router'
import { createPinia, setActivePinia } from 'pinia'
import LoginView from '../views/LoginView.vue'
import GalleryView from '../views/GalleryView.vue'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('LoginView', () => {
  let router: ReturnType<typeof createRouter>

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    localStorage.clear()

    router = createRouter({
      history: createWebHistory(),
      routes: [
        { path: '/', name: 'login', component: LoginView },
        { path: '/gallery', name: 'gallery', component: { template: '<div />' } }
      ]
    })
  })

  it('should render storage type selector', async () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [createPinia(), router]
      }
    })

    expect(wrapper.find('.storage-type-selector').exists()).toBe(true)
    expect(wrapper.findAll('.type-btn')).toHaveLength(2)
  })

  it('should show EC2 form by default', async () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [createPinia(), router]
      }
    })

    expect(wrapper.find('#host').exists()).toBe(true)
    expect(wrapper.find('#username').exists()).toBe(true)
    expect(wrapper.find('#pem').exists()).toBe(true)
  })

  it('should switch to GitHub form when clicking GitHub button', async () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [createPinia(), router]
      }
    })

    const githubBtn = wrapper.findAll('.type-btn')[1]
    await githubBtn.trigger('click')

    expect(wrapper.find('#repoUrl').exists()).toBe(true)
    expect(wrapper.find('#sshKey').exists()).toBe(true)
  })

  it('should disable connect button when form is empty', () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [createPinia(), router]
      }
    })

    const connectBtn = wrapper.find('.connect-btn')
    expect(connectBtn.attributes('disabled')).toBeDefined()
  })

  it('should show info box for GitHub option', async () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [createPinia(), router]
      }
    })

    const githubBtn = wrapper.findAll('.type-btn')[1]
    await githubBtn.trigger('click')

    expect(wrapper.find('.info-box').exists()).toBe(true)
  })

  it('should have correct form labels', async () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [createPinia(), router]
      }
    })

    expect(wrapper.find('label[for="host"]').text()).toBe('IP Address / Host')
    expect(wrapper.find('label[for="username"]').text()).toBe('Username')
    expect(wrapper.find('label[for="port"]').text()).toBe('Port')
    expect(wrapper.find('label[for="pem"]').text()).toBe('SSH Key (.pem file)')
  })
})

describe('GalleryView', () => {
  let router: ReturnType<typeof createRouter>

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()

    router = createRouter({
      history: createWebHistory(),
      routes: [
        { path: '/', name: 'login', component: { template: '<div />' } },
        { path: '/gallery', name: 'gallery', component: GalleryView },
        { path: '/viewer', name: 'viewer', component: { template: '<div />' } }
      ]
    })
  })

  it('should render gallery header', async () => {
    const wrapper = mount(GalleryView, {
      global: {
        plugins: [createPinia(), router],
        stubs: {
          AlbumCard: true,
          MediaCard: true
        }
      }
    })

    expect(wrapper.find('.gallery-header').exists()).toBe(true)
    expect(wrapper.find('h1').text()).toBe('iMAGE')
  })

  it('should show empty state when no files', async () => {
    const wrapper = mount(GalleryView, {
      global: {
        plugins: [createPinia(), router],
        stubs: {
          AlbumCard: true,
          MediaCard: true
        }
      }
    })

    expect(wrapper.find('.empty').exists()).toBe(true)
  })

  it('should have refresh button', () => {
    const wrapper = mount(GalleryView, {
      global: {
        plugins: [createPinia(), router],
        stubs: {
          AlbumCard: true,
          MediaCard: true
        }
      }
    })

    expect(wrapper.find('.icon-btn').exists()).toBe(true)
  })

  it('should have breadcrumb navigation', () => {
    const wrapper = mount(GalleryView, {
      global: {
        plugins: [createPinia(), router],
        stubs: {
          AlbumCard: true,
          MediaCard: true
        }
      }
    })

    expect(wrapper.find('.breadcrumb').exists()).toBe(true)
    expect(wrapper.find('.breadcrumb-item.root').exists()).toBe(true)
  })
})
