<template>
  <div class="settings-tab">
    <h1>Settings</h1>
    <hr />

    <!-- Common Section -->
    <h2>Common</h2>
    <div class="mb-3 form-check form-switch">
      <input
          type="checkbox"
          class="form-check-input"
          id="common-password-enabled"
          v-model="settings.common.password_enabled"
          role="switch"
      />
      <label class="form-check-label" for="common-password-enabled">
        Password Login enabled
      </label>
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

    <!-- OIDC Section -->
    <h2>OIDC</h2>
    <div class="mb-3">
      <label for="oidc-id" class="form-label">Client ID</label>
      <input
          id="oidc-id"
          v-model="settings.oidc.id"
          type="text"
          class="form-control"
      />
    </div>
    <div class="mb-3">
      <label for="oidc-secret" class="form-label">Client Secret</label>
      <input
          id="oidc-secret"
          v-model="settings.oidc.secret"
          type="password"
          class="form-control"
      />
    </div>
    <div class="mb-3">
      <label for="oidc-auth-url" class="form-label">Authorization URL</label>
      <input
          id="oidc-auth-url"
          v-model="settings.oidc.auth_url"
          type="text"
          class="form-control"
      />
    </div>
    <div class="mb-3">
      <label for="oidc-callback-url" class="form-label">Callback URL</label>
      <input
          id="oidc-callback-url"
          v-model="settings.oidc.callback_url"
          type="text"
          class="form-control"
      />
    </div>

    <!-- Change Password Section -->
    <h2>Security</h2>
    <button class="btn btn-warning mt-3" @click="showPasswordDialog = true">
      Change Password
    </button>

    <!-- Change Password Dialog -->
    <div v-if="showPasswordDialog" class="password-dialog">
      <div class="dialog-content">
        <h3>Change Password</h3>
        <div v-if="authStore.password_auth" class="mb-3">
          <label for="old-password" class="form-label">Old Password</label>
          <input
              id="old-password"
              v-model="changePasswordReq.oldPassword"
              type="password"
              class="form-control"
          />
        </div>
        <div class="mb-3">
          <label for="new-password" class="form-label">New Password</label>
          <input
              id="new-password"
              v-model="changePasswordReq.newPassword"
              type="password"
              class="form-control"
          />
        </div>
        <div class="mb-3">
          <label for="confirm-password" class="form-label">Confirm New Password</label>
          <input
              id="confirm-password"
              v-model="confirmPassword"
              type="password"
              class="form-control"
          />
        </div>
        <button
            class="btn btn-primary"
            :disabled="!canChangePassword"
            @click="changePassword"
        >
          Change Password
        </button>
        <button class="btn btn-secondary" @click="showPasswordDialog = false">Cancel</button>
      </div>
    </div>

    <!-- Save Button -->
    <button class="btn btn-primary mt-3" @click="saveSettings">Save</button>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref, onMounted } from 'vue';
import { useSettingseStore } from '@/stores/settings';
import { useAuthStore } from '@/stores/auth';

export default defineComponent({
  name: 'SettingsTab',
  setup() {
    const settingsStore = useSettingseStore();
    const authStore = useAuthStore();

    const settings = computed(() => settingsStore.settings);
    const showPasswordDialog = ref(false);
    const changePasswordReq = ref({ oldPassword: '', newPassword: '' });
    const confirmPassword = ref('');

    const canChangePassword = computed(() =>
        changePasswordReq.value.newPassword === confirmPassword.value &&
        changePasswordReq.value.newPassword.length > 0
    );

    const changePassword = async () => {
      const req = {
        old_password: authStore.password_auth ? changePasswordReq.value.oldPassword : null,
        new_password: changePasswordReq.value.newPassword,
      };
      await authStore.change_password(req);
      showPasswordDialog.value = false;
      changePasswordReq.value = { oldPassword: '', newPassword: '' };
      confirmPassword.value = '';
    };

    onMounted(async () => {
      await settingsStore.fetchSettings();
    });

    return {
      settings,
      saveSettings: settingsStore.saveSettings,
      showPasswordDialog,
      changePasswordReq,
      confirmPassword,
      canChangePassword,
      changePassword,
      authStore,
    };
  },
});
</script>

<style scoped>
.password-dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: white;
  padding: 20px;
  box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);
  border-radius: 8px;
}
.dialog-content {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
</style>
