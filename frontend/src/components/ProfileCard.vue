<template>
  <div class="card text-center shadow-sm">
    <div class="card-body">
      <img
          :src="profileImage"
          alt="User Photo"
          class="rounded-circle img-thumbnail mb-3"
          style="width: 100px; height: 100px;"
      />
      <h5 class="card-title">Triton</h5>
      <p class="card-text text-muted">@hymalia</p>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import profileImage from '@/assets/profile.png';

export default defineComponent({
  name: 'ProfileCard',
  data() {
    return {
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
</style>
