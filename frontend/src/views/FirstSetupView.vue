<template>
  <div class="container d-flex justify-content-center align-items-center vh-100">
    <div class="card p-4 shadow" style="max-width: 400px; width: 100%;">
      <h1 class="text-center mb-4">Hello</h1>
      <form @submit.prevent="setupPassword">
        <div class="mb-3">
          <label for="username" class="form-label">Username</label>
          <input
              id="username"
              type="text"
              v-model="username"
              class="form-control"
              required
          />
        </div>
        <div class="mb-3">
          <label for="ca_name" class="form-label">Name of CA entity</label>
          <input
              id="ca_name"
              type="text"
              v-model="ca_name"
              class="form-control"
              required
          />
        </div>
        <div class="mb-3">
          <label for="ca_validity_in_years" class="form-label">Validity of CA in years</label>
          <input
              id="ca_validity_in_years"
              type="text"
              v-model="ca_validity_in_years"
              class="form-control"
              required
          />
        </div>
        <div class="mb-3">
          <label for="password" class="form-label">Password</label>
          <input
              id="password"
              type="password"
              v-model="password"
              class="form-control"
              autocomplete="new-password"
              required
          />
        </div>
        <button type="submit" class="btn btn-primary w-100">Set Password</button>
        <p v-if="errorMessage" class="text-danger mt-3">
          {{ errorMessage }}
        </p>
      </form>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import router from '../router/router';
import { setup } from "@/api/auth.ts";

export default defineComponent({
  name: 'FirstSetupView',
  setup() {
    const username = ref('');
    const ca_name = ref('');
    const ca_validity_in_years = ref(10);
    const password = ref('');
    const errorMessage = ref('');

    const setupPassword = async () => {
      try {
        await setup({
          name: username.value,
          ca_name: ca_name.value,
          ca_validity_in_years: ca_validity_in_years.value,
          password: password.value
        });
        await router.replace({ name: 'Login' });
      } catch (err) {
        errorMessage.value = 'Failed to set password.';
      }
    };

    return {
      username,
      ca_name,
      ca_validity_in_years,
      password,
      errorMessage,
      setupPassword,
    };
  },
});
</script>
