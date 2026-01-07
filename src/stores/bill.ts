import { defineStore } from 'pinia'
import { ref } from 'vue'
import { billApi, categoryApi } from '../api/tauri'
import { formatErrorMessage } from '../utils/errorHandler'
import { showToast } from 'vant'
import type { Bill, Category, BillFilters } from '../types'
import { useFamilyStore } from './family'

export const useBillStore = defineStore('bill', () => {
  const bills = ref<Bill[]>([])
  const categories = ref<Category[]>([])
  const loading = ref(false)

  const loadBills = async (filters?: BillFilters) => {
    loading.value = true
    try {
      console.log('[loadBills] 开始加载账单，filters:', filters)
      const loadedBills = await billApi.getBills(filters)
      console.log('[loadBills] 从API获取的账单数量:', loadedBills?.length || 0)
      
      // 过滤掉 null 或 undefined 的账单
      const validBills = (loadedBills || []).filter(bill => bill != null && typeof bill === 'object')
      console.log('[loadBills] 有效账单数量:', validBills.length)
      
      // 确保 familyStore 和 categories 已加载
      const familyStore = useFamilyStore()
      if (familyStore.members.length === 0) {
        console.log('[loadBills] 成员列表为空，开始加载成员')
        await familyStore.loadMembers()
      }
      console.log('[loadBills] 成员数量:', familyStore.members.length)
      
      if (categories.value.length === 0) {
        console.log('[loadBills] 分类列表为空，开始加载分类')
        await loadCategories()
      }
      console.log('[loadBills] 分类数量:', categories.value.length)
      
      // 填充 member 和 category 数据，添加安全检查
      bills.value = validBills.map(bill => {
        try {
          if (!bill || typeof bill !== 'object') {
            console.warn('[loadBills] 无效的账单对象:', bill)
            return null
          }
          
          if (typeof bill.member_id === 'undefined' || bill.member_id === null) {
            console.warn('[loadBills] 账单缺少 member_id:', bill)
            return null
          }
          
          if (typeof bill.category_id === 'undefined' || bill.category_id === null) {
            console.warn('[loadBills] 账单缺少 category_id:', bill)
            return null
          }
          
          const member = familyStore.members.find(m => m.id === bill.member_id)
          const category = categories.value.find(c => c.id === bill.category_id)
          
          return {
            ...bill,
            member: member || undefined,
            category: category || undefined
          }
        } catch (err) {
          console.error('[loadBills] 处理账单时出错:', err, bill)
          return null
        }
      }).filter(bill => bill != null) as Bill[]
      
      console.log('[loadBills] 最终账单数量:', bills.value.length)
    } catch (error) {
      console.error('[loadBills] 加载账单失败:', error)
      showToast(formatErrorMessage(error, '加载账单失败'))
      // 即使出错也设置空数组，避免页面崩溃
      bills.value = []
    } finally {
      loading.value = false
    }
  }

  const createBill = async (bill: {
    member_id: number
    category_id: number
    type: 'income' | 'expense'
    amount: number
    description: string
    source: 'wechat' | 'alipay' | 'manual'
    bill_date: string
    bill_month?: string
  }) => {
    loading.value = true
    try {
      await billApi.createBill(bill)
      await loadBills()
    } catch (error) {
      console.error('Failed to create bill:', error)
      const message = formatErrorMessage(error, '创建账单失败')
      showToast(message)
      throw error
    } finally {
      loading.value = false
    }
  }

  const updateBill = async (id: number, bill: Partial<{
    member_id: number
    category_id: number
    type: 'income' | 'expense'
    amount: number
    description: string
    bill_date: string
    bill_month?: string
  }>) => {
    loading.value = true
    try {
      await billApi.updateBill(id, bill)
      await loadBills()
    } catch (error) {
      console.error('Failed to update bill:', error)
      const message = formatErrorMessage(error, '更新账单失败')
      showToast(message)
      throw error
    } finally {
      loading.value = false
    }
  }

  const deleteBill = async (id: number) => {
    loading.value = true
    try {
      await billApi.deleteBill(id)
      await loadBills()
    } catch (error) {
      console.error('Failed to delete bill:', error)
      const message = formatErrorMessage(error, '删除账单失败')
      showToast(message)
      throw error
    } finally {
      loading.value = false
    }
  }

  const loadCategories = async () => {
    loading.value = true
    try {
      const loadedCategories = await categoryApi.getCategories()
      // 确保类型正确
      categories.value = loadedCategories.map(cat => ({
        ...cat,
        type: (cat.type === 'income' || cat.type === 'expense') ? cat.type : 'expense' as 'income' | 'expense'
      }))
    } catch (error) {
      console.error('Failed to load categories:', error)
    } finally {
      loading.value = false
    }
  }

  const createCategory = async (name: string, type: 'income' | 'expense', icon?: string) => {
    loading.value = true
    try {
      await categoryApi.createCategory(name, type, icon)
      await loadCategories()
    } catch (error) {
      console.error('Failed to create category:', error)
      const message = formatErrorMessage(error, '创建分类失败')
      showToast(message)
      throw error
    } finally {
      loading.value = false
    }
  }

  const updateCategory = async (id: number, name: string, icon?: string) => {
    loading.value = true
    try {
      await categoryApi.updateCategory(id, name, icon)
      await loadCategories()
    } catch (error) {
      console.error('Failed to update category:', error)
      const message = formatErrorMessage(error, '更新分类失败')
      showToast(message)
      throw error
    } finally {
      loading.value = false
    }
  }

  const deleteCategory = async (id: number) => {
    loading.value = true
    try {
      await categoryApi.deleteCategory(id)
      await loadCategories()
    } catch (error) {
      console.error('Failed to delete category:', error)
      const message = formatErrorMessage(error, '删除分类失败')
      showToast(message)
      throw error
    } finally {
      loading.value = false
    }
  }

  return {
    bills,
    categories,
    loading,
    loadBills,
    createBill,
    updateBill,
    deleteBill,
    loadCategories,
    createCategory,
    updateCategory,
    deleteCategory
  }
})

