<template>
  <div class="d-flex">
    <!-- Sidebar -->
    <Sidebar
        :currentTab="currentTab"
        @change-tab="setTab"
    />

    <!-- Main Content -->
    <div class="container mt-4 flex-grow-1">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import Sidebar from '@/components/Sidebar.vue';

const currentTab = ref('Overview');
const setTab = (tab: string) => {
  currentTab.value = tab;
};
</script>

<style scoped>
.container {
  margin-left: 275px;
}
</style>
