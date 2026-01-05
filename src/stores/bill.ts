import { defineStore } from 'pinia'
import { ref } from 'vue'
import { billApi, categoryApi } from '../api/tauri'
import { formatErrorMessage } from '../utils/errorHandler'
import { showToast } from 'vant'
import type { Bill, Category, BillFilters } from '../types'

export const useBillStore = defineStore('bill', () => {
  const bills = ref<Bill[]>([])
  const categories = ref<Category[]>([])
  const loading = ref(false)

  const loadBills = async (filters?: BillFilters) => {
    loading.value = true
    try {
      bills.value = await billApi.getBills(filters)
    } catch (error) {
      console.error('Failed to load bills:', error)
      showToast(formatErrorMessage(error, '加载账单失败'))
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
      categories.value = await categoryApi.getCategories()
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

