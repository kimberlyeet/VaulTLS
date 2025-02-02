<template>
  <div class="container d-flex justify-content-center align-items-center vh-100">
    <div class="card p-4 shadow" style="max-width: 400px; width: 100%;">
      <h1 class="text-center mb-4">Login</h1>

      <form v-if="authStore.password_auth" @submit.prevent="submitLogin">
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

      <p v-else class="text-center text-warning">
        Password authentication is not set up.
      </p>

      <div v-if="authStore.oidc_url" class="mt-3">
        <button @click="redirectToOIDC" class="btn btn-outline-primary w-100">
          <i class="bi bi-box-arrow-in-right me-2"></i> Login with OAuth
        </button>
      </div>
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
        await router.push("Overview");
      }
    };

    const redirectToOIDC = () => {
      if (authStore.oidc_url) {
        window.location.href = `${window.location.origin}/api/auth/oidc/login`
      }
    };

    return {
      password,
      loginError,
      submitLogin,
      authStore,
      redirectToOIDC,
    };
  },
});
</script>
