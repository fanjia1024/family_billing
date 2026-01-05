<template>
  <van-tabbar v-model="active" fixed placeholder safe-area-inset-bottom>
    <van-tabbar-item icon="home-o" name="/">首页</van-tabbar-item>
    <van-tabbar-item icon="orders-o" name="/bills">账单</van-tabbar-item>
    <van-tabbar-item icon="chart-trending-o" name="/statistics">统计</van-tabbar-item>
    <van-tabbar-item icon="friends-o" name="/family">家庭</van-tabbar-item>
    <van-tabbar-item icon="setting-o" name="/settings">设置</van-tabbar-item>
  </van-tabbar>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

// 使用 ref 存储当前活动的 tab
const active = ref(route.path)

// 监听路由变化，更新 active
watch(() => route.path, (newPath) => {
  active.value = newPath
}, { immediate: true })

// 监听 active 变化，导航到对应路由
watch(active, (newActive) => {
  if (newActive !== route.path) {
    router.push(newActive)
  }
})
</script>

<style scoped>
/* NavBar styles */
</style>

