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
        <div v-if="statistics.category_data.length > 0" ref="pieChart" style="width: 100%; height: 400px;"></div>
        <van-empty v-else description="暂无支出分类数据" />
      </div>
      
      <!-- 支出分类详情列表 -->
      <div class="chart-item" v-if="statistics.category_data.length > 0">
        <h3>支出分类详情</h3>
        <van-cell-group>
          <van-cell
            v-for="(item, index) in statistics.category_data"
            :key="item.category_id"
            :title="item.category_name"
            :value="`¥${item.amount.toFixed(2)}`"
          >
            <template #label>
              <div class="category-item-label">
                <span>占比: {{ item.percentage.toFixed(1) }}%</span>
                <div class="category-bar">
                  <div 
                    class="category-bar-fill" 
                    :style="{ width: `${item.percentage}%` }"
                  ></div>
                </div>
              </div>
            </template>
          </van-cell>
        </van-cell-group>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { statisticsApi } from '../api/tauri'
import * as echarts from 'echarts'
import type { Statistics } from '../types'
import { showToast } from 'vant'

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

const updateTrendChart = (data: Statistics) => {
  if (!trendChart.value || !trendChartInstance) return
  
  const option = {
    title: {
      text: '月度收支趋势',
      left: 'center'
    },
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        let result = params[0].axisValue + '<br/>'
        params.forEach((item: any) => {
          result += `${item.marker}${item.seriesName}: ¥${item.value.toFixed(2)}<br/>`
        })
        return result
      }
    },
    legend: {
      data: ['收入', '支出'],
      bottom: 0
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '15%',
      containLabel: true
    },
    xAxis: {
      type: 'category',
      data: data.monthly_data.length > 0 ? data.monthly_data.map(m => m.month) : ['暂无数据'],
      boundaryGap: false
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        formatter: (value: number) => `¥${value.toFixed(0)}`
      }
    },
    series: [
      {
        name: '收入',
        type: 'line',
        data: data.monthly_data.length > 0 ? data.monthly_data.map(m => m.income) : [0],
        itemStyle: { color: '#07c160' },
        smooth: true
      },
      {
        name: '支出',
        type: 'line',
        data: data.monthly_data.length > 0 ? data.monthly_data.map(m => m.expense) : [0],
        itemStyle: { color: '#ee0a24' },
        smooth: true
      }
    ]
  }
  trendChartInstance.setOption(option, true)
  trendChartInstance.resize()
}

const updatePieChart = (data: Statistics) => {
  if (!pieChart.value || !pieChartInstance) return
  
  if (data.category_data.length === 0) {
    // 显示空状态
    const option = {
      title: {
        text: '暂无支出分类数据',
        left: 'center',
        top: 'center',
        textStyle: {
          fontSize: 14,
          color: '#999'
        }
      }
    }
    pieChartInstance.setOption(option, true)
    return
  }
  
  const option = {
    title: {
      text: '支出分类占比',
      left: 'center'
    },
    tooltip: {
      trigger: 'item',
      formatter: (params: any) => {
        return `${params.name}<br/>金额: ¥${params.value.toFixed(2)}<br/>占比: ${params.percent}%`
      }
    },
    legend: {
      orient: 'vertical',
      left: 'left',
      bottom: 0,
      formatter: (name: string) => {
        const item = data.category_data.find(c => c.category_name === name)
        return item ? `${name} (${item.percentage.toFixed(1)}%)` : name
      }
    },
    series: [
      {
        name: '支出分类',
        type: 'pie',
        radius: ['40%', '70%'],
        avoidLabelOverlap: false,
        itemStyle: {
          borderRadius: 10,
          borderColor: '#fff',
          borderWidth: 2
        },
        label: {
          show: true,
          formatter: '{b}\n¥{c}\n({d}%)'
        },
        emphasis: {
          label: {
            show: true,
            fontSize: 16,
            fontWeight: 'bold'
          }
        },
        data: data.category_data.map(c => ({
          value: c.amount,
          name: c.category_name
        }))
      }
    ]
  }
  pieChartInstance.setOption(option, true)
  pieChartInstance.resize()
}

const loadStatistics = async () => {
  try {
    const data = await statisticsApi.getStatistics(
      startDate.value || undefined,
      endDate.value || undefined
    )
    statistics.value = data
    
    // 使用 nextTick 确保 DOM 更新后再更新图表
    await nextTick()
    
    updateTrendChart(data)
    updatePieChart(data)
  } catch (error) {
    console.error('Failed to load statistics:', error)
    showToast('加载统计数据失败')
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
  font-size: 18px;
  font-weight: 600;
  color: #323233;
}

.category-item-label {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 4px;
}

.category-bar {
  width: 100%;
  height: 6px;
  background: #ebedf0;
  border-radius: 3px;
  overflow: hidden;
}

.category-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #1989fa 0%, #07c160 100%);
  transition: width 0.3s ease;
}
</style>
