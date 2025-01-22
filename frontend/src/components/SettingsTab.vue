<template>
  <div class="settings-tab">
    <h1>Settings</h1>
    <hr />

    <!-- Common Section -->
    <h2>Common</h2>
    <div class="mb-3">
      <label for="common-username" class="form-label">Username</label>
      <input
          id="common-username"
          v-model="settings.common.username"
          type="text"
          class="form-control"
      />
    </div>

    <!-- Mail Section -->
    <h2>Mail</h2>
    <div class="mb-3">
      <label for="mail-address" class="form-label">Address</label>
      <input
          id="mail-address"
          v-model="settings.mail.address"
          type="email"
          class="form-control"
      />
    </div>
    <div class="mb-3">
      <label for="mail-username" class="form-label">Username</label>
      <input
          id="mail-username"
          v-model="settings.mail.username"
          type="text"
          class="form-control"
      />
    </div>
    <div class="mb-3">
      <label for="mail-password" class="form-label">Password</label>
      <input
          id="mail-password"
          v-model="settings.mail.password"
          type="password"
          class="form-control"
      />
    </div>
    <div class="mb-3">
      <label for="mail-from" class="form-label">From</label>
      <input
          id="mail-from"
          v-model="settings.mail.from"
          type="email"
          class="form-control"
      />
    </div>
    <div class="mb-3">
      <label for="mail-to" class="form-label">To</label>
      <input
          id="mail-to"
          v-model="settings.mail.to"
          type="email"
          class="form-control"
      />
    </div>

    <!-- Save Button -->
    <button class="btn btn-primary mt-3" @click="saveSettings">Save</button>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, onMounted} from 'vue';
import { useSettingseStore } from '@/stores/settings'; // Update the path as needed

export default defineComponent({
  name: 'SettingsTab',

  setup() {
    const settingsStore = useSettingseStore();

    const settings = computed(() => settingsStore.settings);

    // Fetch settings when the component is mounted
    onMounted(async () => {
      await settingsStore.fetchSettings();
    });

    return {
      settings,
      saveSettings: settingsStore.saveSettings,
    };
  },
});
</script>
