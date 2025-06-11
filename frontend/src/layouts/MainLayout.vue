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

<script lang="ts">
import { defineComponent, ref } from 'vue';
import Sidebar from '@/components/Sidebar.vue';

export default defineComponent({
  name: 'MainLayout',
  components: { Sidebar },
  setup() {
    const currentTab = ref('Overview');
    const setTab = (tab: string) => {
      currentTab.value = tab;
    };

    return {
      currentTab,
      setTab,
    };
  },
});
</script>

<style scoped>
.container {
  margin-left: 275px;
}
</style>
