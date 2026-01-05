/**
 * 错误处理工具
 */

export function handleError(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }
  
  if (typeof error === 'string') {
    return error
  }
  
  if (typeof error === 'object' && error !== null) {
    const err = error as Record<string, unknown>
    if (err.message) {
      return String(err.message)
    }
    if (err.error) {
      return String(err.error)
    }
  }
  
  return '未知错误'
}

export function formatErrorMessage(error: unknown, defaultMessage: string = '操作失败'): string {
  const errorMessage = handleError(error)
  
  // 常见错误消息映射
  const errorMap: Record<string, string> = {
    'Failed to open database connection': '数据库连接失败',
    'Failed to load': '加载失败',
    'Failed to create': '创建失败',
    'Failed to update': '更新失败',
    'Failed to delete': '删除失败',
    'Failed to save': '保存失败',
    'not found': '未找到',
    'already exists': '已存在',
    'invalid': '无效',
    'required': '必填',
    'permission denied': '权限不足',
  }
  
  // 查找匹配的错误消息
  for (const [key, value] of Object.entries(errorMap)) {
    if (errorMessage.toLowerCase().includes(key.toLowerCase())) {
      return value
    }
  }
  
  return errorMessage || defaultMessage
}

