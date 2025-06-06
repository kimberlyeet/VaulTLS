<template>
  <div class="card text-center shadow-sm">
    <div class="card-body">
      <img
          :src="profileImage"
          alt="User Photo"
          class="rounded-circle img-thumbnail mb-3"
          style="width: 100px; height: 100px;"
      />
      <h5 class="card-title">{{ authStore.current_user?.name }}</h5>
      <p class="card-text text-muted email">{{ formatEmail(authStore.current_user?.email) }}</p>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import profileImage from '@/assets/profile.png';
import {useAuthStore} from "@/stores/auth.ts";

export default defineComponent({
  name: 'ProfileCard',
  setup() {
    const formatEmail = (email?: string) => {
      return email?.replace('@', '\u200B@');
    };


    const authStore = useAuthStore();
    return {
      formatEmail,
      authStore,
      profileImage,
    };
  },
});
</script>

<style scoped>
.card {
  max-width: 300px;
  margin: auto;
}
.email {
}
</style>
