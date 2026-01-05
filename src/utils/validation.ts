/**
 * 数据验证工具函数
 */

export interface ValidationResult {
  valid: boolean
  message?: string
}

/**
 * 验证金额
 */
export function validateAmount(amount: number | string): ValidationResult {
  const num = typeof amount === 'string' ? parseFloat(amount) : amount
  
  if (isNaN(num)) {
    return { valid: false, message: '金额必须是数字' }
  }
  
  if (num <= 0) {
    return { valid: false, message: '金额必须大于0' }
  }
  
  if (num > 99999999) {
    return { valid: false, message: '金额过大' }
  }
  
  return { valid: true }
}

/**
 * 验证日期格式
 */
export function validateDate(date: string): ValidationResult {
  if (!date) {
    return { valid: false, message: '日期不能为空' }
  }
  
  const dateRegex = /^\d{4}-\d{2}-\d{2}$/
  if (!dateRegex.test(date)) {
    return { valid: false, message: '日期格式不正确，应为 YYYY-MM-DD' }
  }
  
  const dateObj = new Date(date)
  if (isNaN(dateObj.getTime())) {
    return { valid: false, message: '无效的日期' }
  }
  
  // 检查日期不能是未来
  const today = new Date()
  today.setHours(23, 59, 59, 999)
  if (dateObj > today) {
    return { valid: false, message: '日期不能是未来日期' }
  }
  
  return { valid: true }
}

/**
 * 验证成员ID
 */
export function validateMemberId(memberId: number): ValidationResult {
  if (!memberId || memberId <= 0) {
    return { valid: false, message: '请选择成员' }
  }
  
  return { valid: true }
}

/**
 * 验证分类ID
 */
export function validateCategoryId(categoryId: number): ValidationResult {
  if (!categoryId || categoryId <= 0) {
    return { valid: false, message: '请选择分类' }
  }
  
  return { valid: true }
}

/**
 * 验证账单描述
 */
export function validateDescription(description: string, maxLength: number = 200): ValidationResult {
  if (description && description.length > maxLength) {
    return { valid: false, message: `描述不能超过${maxLength}个字符` }
  }
  
  return { valid: true }
}

/**
 * 验证账单对象
 */
export function validateBill(bill: {
  member_id: number
  category_id: number
  amount: number
  bill_date: string
  description?: string
}): ValidationResult {
  const memberCheck = validateMemberId(bill.member_id)
  if (!memberCheck.valid) {
    return memberCheck
  }
  
  const categoryCheck = validateCategoryId(bill.category_id)
  if (!categoryCheck.valid) {
    return categoryCheck
  }
  
  const amountCheck = validateAmount(bill.amount)
  if (!amountCheck.valid) {
    return amountCheck
  }
  
  const dateCheck = validateDate(bill.bill_date)
  if (!dateCheck.valid) {
    return dateCheck
  }
  
  if (bill.description) {
    const descCheck = validateDescription(bill.description)
    if (!descCheck.valid) {
      return descCheck
    }
  }
  
  return { valid: true }
}

