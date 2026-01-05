<template>
  <div class="settings-page">
    <h1>设置</h1>
    
    <van-cell-group>
      <van-cell title="数据导出" is-link @click="handleExport">
        <template #icon>
          <van-icon name="down" />
        </template>
      </van-cell>
      <van-cell title="数据导入" is-link @click="handleImport">
        <template #icon>
          <van-icon name="up" />
        </template>
      </van-cell>
    </van-cell-group>
  </div>
</template>

<script setup lang="ts">
import { exportApi } from '../api/tauri'
import { showToast } from 'vant'
import { save, open } from '@tauri-apps/plugin-dialog'
import { formatErrorMessage } from '../utils/errorHandler'

const handleExport = async () => {
  try {
    const filePath = await save({
      defaultPath: 'household-billing-export.json',
      filters: [{
        name: 'JSON',
        extensions: ['json']
      }]
    })
    
    if (!filePath) {
      // 用户取消了文件选择
      return
    }
    
    // 确保文件路径是字符串（save 函数返回 string | null）
    const path = filePath as string
    console.log('导出文件路径:', path)
    
    await exportApi.exportToJson(path)
    showToast('导出成功')
  } catch (error) {
    console.error('Export failed:', error)
    const errorMessage = formatErrorMessage(error, '导出失败')
    showToast(errorMessage)
  }
}

const handleImport = async () => {
  try {
    const filePath = await open({
      directory: false,
      multiple: false,
      filters: [{
        name: 'JSON',
        extensions: ['json']
      }]
    })
    
    if (!filePath) {
      // 用户取消了文件选择
      return
    }
    
    // 处理可能的数组返回值（multiple: false 时应该是单个字符串）
    let path: string
    if (Array.isArray(filePath)) {
      if (filePath.length === 0) {
        return
      }
      path = filePath[0]
    } else {
      path = filePath
    }
    
    console.log('导入文件路径:', path)
    
    await exportApi.importFromJson(path)
    showToast('导入成功')
  } catch (error) {
    console.error('Import failed:', error)
    const errorMessage = formatErrorMessage(error, '导入失败')
    showToast(errorMessage)
  }
}
</script>

<style scoped>
.settings-page {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}
</style>
