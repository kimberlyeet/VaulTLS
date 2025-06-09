<template>
  <div class="settings-tab">
    <h1>Settings</h1>
    <hr />
    <!-- Application Section -->
    <div v-if="isAdmin" class="mb-3">
      <!-- Common Section -->
      <h3>Common</h3>
      <div class="card mt-3">
        <div class="card-body">
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
        </div>
      </div>

      <!-- Mail Section -->
      <h3>Mail</h3>
      <div class="card mt-3">
        <div class="card-body">
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
        </div>
      </div>

      <!-- OIDC Section -->
      <h3>OIDC</h3>
      <div class="card mt-3">
        <div class="card-body">
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
        </div>
      </div>
    </div>

    <h2>User</h2>
    <div class="card mt-5">
      <div class="card-body">
        <h4 class="card-header">Change Password</h4>
        <form v-if="authStore.password_auth" @submit.prevent="changePassword">
          <div v-if="authStore.current_user?.has_password" class="mb-3">
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
          <div v-if="password_error" class="alert alert-danger mt-3">
            {{ password_error }}
          </div>

          <button
              type="submit"
              class="btn btn-primary"
              :disabled="!canChangePassword"
          >
            Change Password
          </button>
        </form>
      </div>
      <div v-if="editableUser" class="card-body">
        <h4 class="card-header">Profile</h4>
        <div class="mb-3">
          <label for="user_name" class="form-label">Username</label>
          <input
              id="user_name"
              v-model="editableUser.name"
              type="text"
              class="form-control"
          />
        </div>
        <div class="mb-3">
          <label for="user_email" class="form-label">E-Mail</label>
          <input
              id="user_email"
              v-model="editableUser.email"
              type="email"
              class="form-control"
          />
        </div>
      </div>
    </div>

    <!-- Error Messages -->
    <div v-if="settings_error" class="alert alert-danger mt-3">
      {{ settings_error }}
    </div>
    <div v-if="user_error" class="alert alert-danger mt-3">
      {{ user_error }}
    </div>

    <!-- Save Button -->
    <button class="btn btn-primary mt-3" @click="saveSettings">Save</button>
  </div>
</template>

<script lang="ts">
import {computed, defineComponent, ref, onMounted} from 'vue';
import { useSettingseStore } from '@/stores/settings';
import { useAuthStore } from '@/stores/auth';
import {type User, UserRole} from "@/types/User.ts";
import {useUserStore} from "@/stores/users.ts";

export default defineComponent({
  name: 'SettingsTab',
  setup() {
    const settingsStore = useSettingseStore();
    const authStore = useAuthStore();
    const userStore = useUserStore();

    const settings = computed(() => settingsStore.settings);
    const showPasswordDialog = ref(false);
    const changePasswordReq = ref({ oldPassword: '', newPassword: '' });
    const confirmPassword = ref('');
    const current_user = computed(() => authStore.current_user);
    const editableUser = ref<User | null>(null);
    const settings_error = computed(() => settingsStore.error);
    const user_error = computed(() => userStore.error);
    const password_error = computed(() => authStore.error);

    const canChangePassword = computed(() =>
        changePasswordReq.value.newPassword === confirmPassword.value &&
        changePasswordReq.value.newPassword.length > 0
    );

    const isAdmin = computed(() => authStore.current_user?.role === UserRole.Admin);

    const changePassword = async () => {
      const req = {
        old_password: changePasswordReq.value.oldPassword,
        new_password: changePasswordReq.value.newPassword
      };
      await authStore.change_password(req);
      showPasswordDialog.value = false;
      changePasswordReq.value = { oldPassword: '', newPassword: '' };
      confirmPassword.value = '';
    };

    const saveSettings = async () => {
      if (current_user.value?.role == UserRole.Admin) {
        await settingsStore.saveSettings();
      }

      if (editableUser.value) {
        await userStore.updateUser(editableUser.value);
        await authStore.fetchCurrentUser();
      }
    }

    onMounted(async () => {
      if (isAdmin.value) {
        await settingsStore.fetchSettings();
      }
      if (current_user.value) {
        editableUser.value = { ...current_user.value };
      }
    });

    return {
      settings,
      editableUser,
      saveSettings,
      showPasswordDialog,
      changePasswordReq,
      confirmPassword,
      canChangePassword,
      changePassword,
      authStore,
      isAdmin,
      settings_error,
      user_error,
      password_error
    };
  },
});
</script>

<style scoped>
</style>
