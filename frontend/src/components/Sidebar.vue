<template>
  <div class="sidebar bg-light vh-100 p-3 shadow-lg rounded-end" style="width: 250px;">
    <ProfileCard />

    <nav class="mt-4">
      <ul class="nav flex-column">
        <li class="nav-item mb-2">
          <a
              href="#"
              class="nav-link d-flex align-items-center gap-2"
              :class="{ active: currentTab === 'Overview' }"
              @click.prevent="$emit('change-tab', 'Overview')"
          >
            <i class="bi bi-house-door-fill"></i>
            Overview
          </a>
        </li>
        <li class="nav-item mb-2">
          <a
              href="#"
              class="nav-link d-flex align-items-center gap-2"
              :class="{ active: currentTab === 'Generate' }"
              @click.prevent="$emit('change-tab', 'Generate')"
          >
            <i class="bi bi-tools"></i>
            Generate
          </a>
        </li>
        <li class="nav-item">
          <a
              href="#"
              class="nav-link d-flex align-items-center gap-2"
              :class="{ active: currentTab === 'Settings' }"
              @click.prevent="$emit('change-tab', 'Settings')"
          >
            <i class="bi bi-gear-fill"></i>
            Settings
          </a>
        </li>
      </ul>
    </nav>
  </div>
</template>

<script lang="ts">
import { defineComponent, type PropType } from 'vue';
import ProfileCard from './ProfileCard.vue';

export default defineComponent({
  name: 'Sidebar',
  components: {
    ProfileCard,
  },
  props: {
    currentTab: {
      type: String as PropType<string>,
      required: true,
    },
  },
});
</script>

<style scoped>
.nav-link {
  color: #000;
  text-decoration: none;
}

.nav-link.active {
  font-weight: bold;
  background-color: #e7e7e7;
  border-radius: 4px;
}
</style>
