<template>
  <van-card
    v-if="bill"
    :title="bill.description || '无描述'"
    :desc="formatBillInfo(bill)"
    :thumb="billImageUrl"
  >
    <template #tags>
      <van-tag
        :type="bill.type === 'income' ? 'success' : 'danger'"
        style="margin-right: 8px"
      >
        {{ bill.type === 'income' ? '收入' : '支出' }}
      </van-tag>
      <van-tag v-if="bill.category" plain>
        {{ bill.category.icon }} {{ bill.category.name }}
      </van-tag>
    </template>
    <template #footer>
      <div class="bill-footer">
        <span class="bill-date">{{ formatDate(bill.bill_date) }}</span>
        <div class="bill-actions">
          <van-button
            size="mini"
            type="primary"
            @click="$emit('edit', bill)"
          >
            编辑
          </van-button>
          <van-button
            size="mini"
            type="danger"
            @click="$emit('delete', bill.id)"
          >
            删除
          </van-button>
        </div>
      </div>
    </template>
  </van-card>
  <div v-else class="invalid-bill">无效账单数据</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Bill } from '../types'
import dayjs from 'dayjs'

const props = defineProps<{
  bill: Bill | null | undefined
}>()

defineEmits<{
  edit: [bill: Bill]
  delete: [id: number]
}>()

const billImageUrl = computed(() => {
  if (!props.bill) return undefined
  if (props.bill.images && props.bill.images.length > 0) {
    // Return image URL if available
    return props.bill.images[0].image_path
  }
  return undefined
})

const formatBillInfo = (bill: Bill | null | undefined) => {
  if (!bill) return '无效账单'
  const amount = `¥${(bill.amount || 0).toFixed(2)}`
  const member = bill.member ? ` - ${bill.member.name}` : ''
  return `${amount}${member}`
}

const formatDate = (date: string | undefined) => {
  if (!date) return '未知日期'
  return dayjs(date).format('YYYY-MM-DD')
}
</script>

<style scoped>
.bill-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 10px;
}

.bill-date {
  font-size: 12px;
  color: #999;
}

.bill-actions {
  display: flex;
  gap: 8px;
}
</style>

