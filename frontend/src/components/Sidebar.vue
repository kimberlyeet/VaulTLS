<template>
  <div class="sidebar bg-light vh-100 p-3 shadow-lg rounded-end" style="width: 250px;">
    <ProfileCard />

    <nav class="mt-4">
      <ul class="nav flex-column">
        <li class="nav-item mb-2">
          <a
              href="#"
              class="nav-link d-flex align-items-center gap-2"
              :class="{ active: activeRouteName === 'Overview' }"
              @click.prevent="goToRoute('Overview')"
          >
            <i class="bi bi-house-door-fill"></i>
            Overview
          </a>
        </li>
        <li v-if="isAdmin" class="nav-item mb-2">
          <a
              href="#"
              class="nav-link d-flex align-items-center gap-2"
              :class="{ active: activeRouteName === 'Users' }"
              @click.prevent="goToRoute('Users')"
          >
            <i class="bi bi-tools"></i>
            Users
          </a>
        </li>
        <li v-if="isAdmin" class="nav-item">
          <a
              href="#"
              class="nav-link d-flex align-items-center gap-2"
              :class="{ active: activeRouteName === 'Settings' }"
              @click.prevent="goToRoute('Settings')"
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
import { defineComponent, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import ProfileCard from './ProfileCard.vue';
import {UserRole} from "@/types/User.ts";
import {useAuthStore} from "@/stores/auth.ts";

export default defineComponent({
  name: 'Sidebar',
  components: { ProfileCard },
  setup() {
    const route = useRoute();
    const router = useRouter();
    const authStore = useAuthStore();

    const activeRouteName = computed(() => route.name);
    const isAdmin = computed(() => authStore.current_user?.role == UserRole.Admin);

    // A helper method to navigate to a given route name
    const goToRoute = (name: string) => {
      router.push({ name });
    };

    return {
      activeRouteName,
      isAdmin,
      goToRoute,
    };
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
