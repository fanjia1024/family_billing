import { defineStore } from 'pinia'
import { ref } from 'vue'
import { familyApi, memberApi } from '../api/tauri'
import { formatErrorMessage } from '../utils/errorHandler'
import { showToast } from 'vant'
import type { Family, Member } from '../types'

export const useFamilyStore = defineStore('family', () => {
  const family = ref<Family | null>(null)
  const members = ref<Member[]>([])
  const loading = ref(false)

  const loadFamily = async () => {
    loading.value = true
    try {
      const data = await familyApi.getFamily()
      family.value = data || null
    } catch (error) {
      console.error('Failed to load family:', error)
      showToast(formatErrorMessage(error, '加载家庭信息失败'))
    } finally {
      loading.value = false
    }
  }

  const createFamily = async (name: string) => {
    loading.value = true
    try {
      await familyApi.createFamily(name)
      await loadFamily()
    } catch (error) {
      console.error('Failed to create family:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  const updateFamily = async (id: number, name: string) => {
    loading.value = true
    try {
      await familyApi.updateFamily(id, name)
      await loadFamily()
    } catch (error) {
      console.error('Failed to update family:', error)
      showToast(formatErrorMessage(error, '更新家庭信息失败'))
      throw error
    } finally {
      loading.value = false
    }
  }

  const loadMembers = async () => {
    loading.value = true
    try {
      const loadedMembers = await memberApi.getMembers()
      // 确保 role 类型正确
      members.value = loadedMembers.map(member => ({
        ...member,
        role: (member.role === 'admin' || member.role === 'member') ? member.role : 'member' as 'admin' | 'member'
      }))
    } catch (error) {
      console.error('Failed to load members:', error)
    } finally {
      loading.value = false
    }
  }

  const createMember = async (name: string, role: 'admin' | 'member' = 'member', avatar?: string) => {
    loading.value = true
    try {
      await memberApi.createMember(name, role, avatar)
      await loadMembers()
    } catch (error) {
      console.error('Failed to create member:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  const updateMember = async (id: number, name: string, role: 'admin' | 'member', avatar?: string) => {
    loading.value = true
    try {
      await memberApi.updateMember(id, name, role, avatar)
      await loadMembers()
    } catch (error) {
      console.error('Failed to update member:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  const deleteMember = async (id: number) => {
    loading.value = true
    try {
      await memberApi.deleteMember(id)
      await loadMembers()
    } catch (error) {
      console.error('Failed to delete member:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  return {
    family,
    members,
    loading,
    loadFamily,
    createFamily,
    updateFamily,
    loadMembers,
    createMember,
    updateMember,
    deleteMember
  }
})

