<template>
  <div class="d-flex">
    <!-- Sidebar -->
    <Sidebar
        :currentTab="currentTab"
        @change-tab="setTab"
    />

    <!-- Main Content -->
    <div class="container mt-4 flex-grow-1">

      <!-- Conditional Tab Rendering -->
      <OverviewTab v-if="currentTab === 'Overview'" />
      <GenerateTab v-if="currentTab === 'Generate'" />
      <SettingsTab v-if="currentTab === 'Settings'" />
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import Sidebar from './components/Sidebar.vue';
import OverviewTab from './components/OverviewTab.vue';
import GenerateTab from './components/GenerateTab.vue';
import SettingsTab from './components/SettingsTab.vue';

export default defineComponent({
  name: 'App',
  components: {
    Sidebar,
    OverviewTab,
    GenerateTab,
    SettingsTab,
  },

  setup() {
    // Current tab state
    const currentTab = ref('Overview');

    // Tab switcher
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
  margin-left: 50px; /* Adjust based on your Sidebar width */
}
</style>
