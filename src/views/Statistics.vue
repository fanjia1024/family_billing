<template>
  <div class="statistics-page">
    <h1>统计分析</h1>
    
    <div class="date-range">
      <van-field
        v-model="startDate"
        label="开始日期"
        placeholder="选择开始日期"
        is-link
        readonly
        @click="showStartDatePicker = true"
      />
      <van-field
        v-model="endDate"
        label="结束日期"
        placeholder="选择结束日期"
        is-link
        readonly
        @click="showEndDatePicker = true"
      />
      <van-button type="primary" @click="loadStatistics">查询</van-button>
      <van-button @click="resetDateRange">重置</van-button>
    </div>

    <!-- Start Date Picker -->
    <van-popup v-model:show="showStartDatePicker" position="bottom">
      <van-date-picker
        v-model="startDatePickerValue"
        @confirm="onStartDateConfirm"
        @cancel="showStartDatePicker = false"
      />
    </van-popup>

    <!-- End Date Picker -->
    <van-popup v-model:show="showEndDatePicker" position="bottom">
      <van-date-picker
        v-model="endDatePickerValue"
        @confirm="onEndDateConfirm"
        @cancel="showEndDatePicker = false"
      />
    </van-popup>

    <div class="summary">
      <van-card>
        <template #title>
          <h2>统计汇总</h2>
        </template>
        <template #desc>
          <p>总收入: ¥{{ statistics.total_income.toFixed(2) }}</p>
          <p>总支出: ¥{{ statistics.total_expense.toFixed(2) }}</p>
          <p>结余: ¥{{ statistics.balance.toFixed(2) }}</p>
        </template>
      </van-card>
    </div>

    <div class="charts">
      <div class="chart-item">
        <h3>月度收支趋势</h3>
        <div ref="trendChart" style="width: 100%; height: 400px;"></div>
      </div>
      
      <div class="chart-item">
        <h3>支出分类占比</h3>
        <div ref="pieChart" style="width: 100%; height: 400px;"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { statisticsApi } from '../api/tauri'
import * as echarts from 'echarts'
import type { Statistics } from '../types'

const startDate = ref('')
const endDate = ref('')
const showStartDatePicker = ref(false)
const showEndDatePicker = ref(false)
// Vant 4 DatePicker 需要字符串数组格式 ['2024', '01', '05']
const today = new Date()
const startDatePickerValue = ref<string[]>([
  String(today.getFullYear()),
  String(today.getMonth() + 1).padStart(2, '0'),
  String(today.getDate()).padStart(2, '0')
])
const endDatePickerValue = ref<string[]>([
  String(today.getFullYear()),
  String(today.getMonth() + 1).padStart(2, '0'),
  String(today.getDate()).padStart(2, '0')
])

const statistics = ref<Statistics>({
  total_income: 0,
  total_expense: 0,
  balance: 0,
  monthly_data: [],
  category_data: []
})

const trendChart = ref<HTMLElement | null>(null)
const pieChart = ref<HTMLElement | null>(null)
let trendChartInstance: echarts.ECharts | null = null
let pieChartInstance: echarts.ECharts | null = null

// Vant 4 DatePicker 返回 { selectedValues: ['2024', '01', '05'] }
const onStartDateConfirm = ({ selectedValues }: { selectedValues: string[] }) => {
  startDate.value = selectedValues.join('-')
  startDatePickerValue.value = selectedValues
  showStartDatePicker.value = false
}

const onEndDateConfirm = ({ selectedValues }: { selectedValues: string[] }) => {
  endDate.value = selectedValues.join('-')
  endDatePickerValue.value = selectedValues
  showEndDatePicker.value = false
}

const resetDateRange = () => {
  startDate.value = ''
  endDate.value = ''
  loadStatistics()
}

const loadStatistics = async () => {
  try {
    const data = await statisticsApi.getStatistics(
      startDate.value || undefined,
      endDate.value || undefined
    )
    statistics.value = data
    
    // Update trend chart
    if (trendChart.value && trendChartInstance) {
      const option = {
        title: {
          text: '月度收支趋势'
        },
        tooltip: {
          trigger: 'axis'
        },
        legend: {
          data: ['收入', '支出']
        },
        xAxis: {
          type: 'category',
          data: data.monthly_data.map(m => m.month)
        },
        yAxis: {
          type: 'value'
        },
        series: [
          {
            name: '收入',
            type: 'line',
            data: data.monthly_data.map(m => m.income)
          },
          {
            name: '支出',
            type: 'line',
            data: data.monthly_data.map(m => m.expense)
          }
        ]
      }
      trendChartInstance.setOption(option)
    }
    
    // Update pie chart
    if (pieChart.value && pieChartInstance) {
      const option = {
        title: {
          text: '支出分类占比'
        },
        tooltip: {
          trigger: 'item',
          formatter: '{a} <br/>{b}: ¥{c} ({d}%)'
        },
        series: [
          {
            name: '支出分类',
            type: 'pie',
            radius: '50%',
            data: data.category_data.map(c => ({
              value: c.amount,
              name: c.category_name
            }))
          }
        ]
      }
      pieChartInstance.setOption(option)
    }
  } catch (error) {
    console.error('Failed to load statistics:', error)
  }
}

onMounted(async () => {
  if (trendChart.value) {
    trendChartInstance = echarts.init(trendChart.value)
    // 响应式调整
    window.addEventListener('resize', () => {
      trendChartInstance?.resize()
    })
  }
  if (pieChart.value) {
    pieChartInstance = echarts.init(pieChart.value)
    // 响应式调整
    window.addEventListener('resize', () => {
      pieChartInstance?.resize()
    })
  }
  
  await loadStatistics()
})

onUnmounted(() => {
  window.removeEventListener('resize', () => {})
  if (trendChartInstance) {
    trendChartInstance.dispose()
  }
  if (pieChartInstance) {
    pieChartInstance.dispose()
  }
})
</script>

<style scoped>
.statistics-page {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.date-range {
  display: flex;
  gap: 10px;
  margin: 20px 0;
  flex-wrap: wrap;
  align-items: center;
}

.summary {
  margin: 20px 0;
}

.charts {
  margin-top: 30px;
}

.chart-item {
  margin: 30px 0;
}

.chart-item h3 {
  margin-bottom: 15px;
}
</style>
