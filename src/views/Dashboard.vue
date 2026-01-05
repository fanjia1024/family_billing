<template>
  <div class="dashboard-page">
    <h1>首页仪表盘</h1>
    
    <!-- 空状态引导 -->
    <van-empty 
      v-if="!hasData && !billStore.loading" 
      description="还没有数据，开始记录你的账单吧！"
      image="network"
    >
      <div class="empty-actions">
        <van-button type="primary" @click="goToFamily">创建家庭</van-button>
        <van-button type="success" @click="goToBills">添加账单</van-button>
      </div>
    </van-empty>

    <!-- 有数据时显示统计 -->
    <template v-else>
      <div class="stats-cards">
        <van-card>
          <template #title>
            <h2>本月收支</h2>
          </template>
          <template #desc>
            <div class="stats-content">
              <p class="income">收入: ¥{{ statistics.total_income.toFixed(2) }}</p>
              <p class="expense">支出: ¥{{ statistics.total_expense.toFixed(2) }}</p>
              <p :class="statistics.balance >= 0 ? 'balance-positive' : 'balance-negative'">
                结余: ¥{{ statistics.balance.toFixed(2) }}
              </p>
            </div>
          </template>
        </van-card>
      </div>

      <div class="quick-actions">
        <van-button type="primary" @click="goToBills">添加账单</van-button>
        <van-button type="success" @click="goToOcr">OCR识别</van-button>
        <van-button @click="goToStatistics">查看统计</van-button>
      </div>

      <div class="chart-section" v-if="statistics.monthly_data.length > 0">
        <h2>收支趋势</h2>
        <div ref="chartContainer" style="width: 100%; height: 400px;"></div>
      </div>
      <van-empty v-else description="暂无趋势数据" />

      <div class="recent-bills">
        <div class="section-header">
          <h2>最近账单</h2>
          <van-button size="small" type="primary" plain @click="goToBills">查看全部</van-button>
        </div>
        <van-list v-if="recentBills.length > 0">
          <van-cell
            v-for="bill in recentBills"
            :key="bill.id"
            :title="bill.description || '无描述'"
            :label="`${bill.type === 'income' ? '收入' : '支出'} - ¥${bill.amount.toFixed(2)}`"
            is-link
            @click="goToBills"
          />
        </van-list>
        <van-empty v-else description="暂无账单" />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useBillStore } from '../stores/bill'
import { useFamilyStore } from '../stores/family'
import { statisticsApi } from '../api/tauri'
import * as echarts from 'echarts'
import type { Statistics } from '../types'

const router = useRouter()
const billStore = useBillStore()
const familyStore = useFamilyStore()
const chartContainer = ref<HTMLElement | null>(null)
let chartInstance: echarts.ECharts | null = null

const statistics = ref<Statistics>({
  total_income: 0,
  total_expense: 0,
  balance: 0,
  monthly_data: [],
  category_data: []
})

const recentBills = computed(() => billStore.bills.slice(0, 5))

const hasData = computed(() => {
  return billStore.bills.length > 0 || statistics.value.monthly_data.length > 0
})

const goToFamily = () => {
  router.push('/family')
}

const goToBills = () => {
  router.push('/bills')
}

const goToOcr = () => {
  router.push('/bills')
  // 可以添加一个事件来触发 OCR 对话框
}

const goToStatistics = () => {
  router.push('/statistics')
}

const loadStatistics = async () => {
  try {
    const data = await statisticsApi.getStatistics()
    statistics.value = data
    
    if (chartContainer.value && chartInstance) {
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
      chartInstance.setOption(option)
    }
  } catch (error) {
    console.error('Failed to load statistics:', error)
  }
}

onMounted(async () => {
  await familyStore.loadFamily()
  await familyStore.loadMembers()
  await billStore.loadBills()
  
  if (chartContainer.value) {
    chartInstance = echarts.init(chartContainer.value)
    // 响应式调整
    window.addEventListener('resize', () => {
      chartInstance?.resize()
    })
  }
  
  await loadStatistics()
})

onUnmounted(() => {
  window.removeEventListener('resize', () => {})
  if (chartInstance) {
    chartInstance.dispose()
  }
})
</script>

<style scoped>
.dashboard-page {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.empty-actions {
  display: flex;
  gap: 10px;
  justify-content: center;
  margin-top: 20px;
}

.stats-cards {
  margin: 20px 0;
}

.stats-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.stats-content .income {
  color: #07c160;
  font-weight: 500;
}

.stats-content .expense {
  color: #ee0a24;
  font-weight: 500;
}

.stats-content .balance-positive {
  color: #07c160;
  font-weight: 600;
  font-size: 16px;
}

.stats-content .balance-negative {
  color: #ee0a24;
  font-weight: 600;
  font-size: 16px;
}

.quick-actions {
  display: flex;
  gap: 10px;
  margin: 20px 0;
  flex-wrap: wrap;
}

.chart-section {
  margin: 30px 0;
}

.recent-bills {
  margin-top: 30px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.section-header h2 {
  margin: 0;
}
</style>
