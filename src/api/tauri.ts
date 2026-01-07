import { invoke } from '@tauri-apps/api/core'

// Check if running in Tauri environment
const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

// Safe invoke that handles browser environment
async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) {
    console.warn(`Tauri not available, cannot invoke: ${cmd}`)
    throw new Error('此功能需要在桌面应用中运行')
  }
  return invoke<T>(cmd, args)
}

// Family commands
export const familyApi = {
  getFamily: () => safeInvoke<{ id: number; name: string; created_at: string } | null>('get_family'),
  createFamily: (name: string) => safeInvoke<number>('create_family', { name }),
  updateFamily: (id: number, name: string) => safeInvoke<void>('update_family', { id, name })
}

// Member commands
export const memberApi = {
  getMembers: () => safeInvoke<Array<{ id: number; family_id: number; name: string; avatar?: string; role: string; created_at: string }>>('get_members'),
  createMember: (name: string, role: 'admin' | 'member', avatar?: string) => 
    safeInvoke<number>('create_member', { name, role, avatar }),
  updateMember: (id: number, name: string, role: 'admin' | 'member', avatar?: string) =>
    safeInvoke<void>('update_member', { id, name, role, avatar }),
  deleteMember: (id: number) => safeInvoke<void>('delete_member', { id })
}

// Bill commands
export const billApi = {
  getBills: (filters?: { member_id?: number; category_id?: number; start_date?: string; end_date?: string }) =>
    safeInvoke<Array<any>>('get_bills', { filters }),
  createBill: (bill: { member_id: number; category_id: number; type: 'income' | 'expense'; amount: number; description: string; source: 'wechat' | 'alipay' | 'manual'; bill_date: string; bill_month?: string }) =>
    safeInvoke<number>('create_bill', { bill }),
  updateBill: (id: number, bill: Partial<{ member_id: number; category_id: number; type: 'income' | 'expense'; amount: number; description: string; bill_date: string }>) =>
    safeInvoke<void>('update_bill', { id, bill }),
  deleteBill: (id: number) => safeInvoke<void>('delete_bill', { id })
}

// Category commands
export const categoryApi = {
  getCategories: () => safeInvoke<Array<{ id: number; name: string; type: string; icon?: string }>>('get_categories'),
  createCategory: (name: string, type: 'income' | 'expense', icon?: string) =>
    safeInvoke<number>('create_category', { name, type, icon }),
  updateCategory: (id: number, name: string, icon?: string) =>
    safeInvoke<void>('update_category', { id, name, icon }),
  deleteCategory: (id: number) => safeInvoke<void>('delete_category', { id })
}

// OCR 批量识别结果类型
export interface OcrBillItem {
  category: string
  amount: number
  percentage: number | null
  bill_type: 'income' | 'expense'
}

export interface OcrBatchResult {
  items: OcrBillItem[]
  total_amount: number
  bill_date: string
  bill_month?: string
  raw_text: string
}

// OCR commands
export const ocrApi = {
  recognizeImage: (imagePath: string) =>
    safeInvoke<{ type: 'income' | 'expense'; amount: number; description: string; bill_date: string; raw_text: string }>('ocr_recognize', { imagePath }),
  recognizeBatch: (imagePath: string) =>
    safeInvoke<OcrBatchResult>('ocr_recognize_batch', { imagePath }),
  saveBillWithImage: (bill: { member_id: number; category_id: number; type: 'income' | 'expense'; amount: number; description: string; source: 'wechat' | 'alipay' | 'manual'; bill_date: string; bill_month?: string }, imagePath: string) =>
    safeInvoke<number>('save_bill_with_image', { bill, imagePath })
}

// Export/Import commands
export const exportApi = {
  exportToJson: (filePath: string) => safeInvoke<void>('export_to_json', { filePath }),
  exportToExcel: (filePath: string) => safeInvoke<void>('export_to_excel', { filePath }),
  importFromJson: (filePath: string) => safeInvoke<void>('import_from_json', { filePath })
}

// Statistics commands
export const statisticsApi = {
  getStatistics: (startDate?: string, endDate?: string) =>
    safeInvoke<{ total_income: number; total_expense: number; balance: number; monthly_data: Array<any>; category_data: Array<any> }>('get_statistics', { startDate, endDate })
}

// Image commands
export const imageApi = {
  saveUploadedImage: (imageData: number[], filename: string) =>
    safeInvoke<string>('save_uploaded_image', { imageData, filename }),
  getImagePath: (imagePath: string) =>
    safeInvoke<string>('get_image_path', { imagePath })
}

// Export isTauri for checking environment
export { isTauri }

