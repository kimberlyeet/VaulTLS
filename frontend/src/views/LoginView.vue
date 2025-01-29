<template>
  <div class="container d-flex justify-content-center align-items-center vh-100">
    <div class="card p-4 shadow" style="max-width: 400px; width: 100%;">
      <h1 class="text-center mb-4">Login</h1>
      <form @submit.prevent="submitLogin">
        <div class="mb-3">
          <label for="password" class="form-label">Password</label>
          <input
              id="password"
              type="password"
              v-model="password"
              class="form-control"
              autocomplete="current-password"
              required
          />
        </div>
        <button type="submit" class="btn btn-primary w-100">Login</button>
        <p v-if="loginError" class="text-danger mt-3">
          {{ loginError }}
        </p>
      </form>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { useAuthStore } from '../stores/auth';
import router from "@/router/router.ts";

export default defineComponent({
  name: 'LoginView',
  setup() {
    const password = ref('');
    const loginError = ref('');
    const authStore = useAuthStore();

    const submitLogin = async () => {
      loginError.value = '';
      const success = await authStore.login(password.value);
      if (!success) {
        loginError.value = 'Login failed. Please try again.';
      } else {
        // On success, redirect to the main page
        await router.push("Overview");
      }
    };

    return {
      password,
      loginError,
      submitLogin,
    };
  },
});
</script>
