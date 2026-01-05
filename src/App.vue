<template>
  <div class="app">
    <router-view />
    <NavBar />
  </div>
</template>

<script setup lang="ts">
import NavBar from './components/NavBar.vue'
</script>

<style>
.app {
  min-height: 100vh;
  padding-bottom: 60px; /* Space for tabbar */
}
</style>

