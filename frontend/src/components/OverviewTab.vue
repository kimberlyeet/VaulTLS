<template>
  <div>
    <h1>Certificates</h1>
    <hr />
    <div class="table-responsive">
      <table class="table table-striped">
        <thead>
          <tr>
            <th v-if="isAdmin">User</th>
            <th>Name</th>
            <th>Created on</th>
            <th>Valid until</th>
            <th>Password</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="cert in certificates.values()" :key="cert.id">
            <td v-if="isAdmin">{{ userStore.idToName(cert.user_id) }}</td>
            <td>{{ cert.name }}</td>
            <td>{{ new Date(cert.created_on).toLocaleDateString() }}</td>
            <td>{{ new Date(cert.valid_until).toLocaleDateString() }}</td>
            <td class="password-cell">
              <template v-if="shownCerts.has(cert.id)">
                <input
                    type="text"
                    :value="cert.pkcs12_password"
                    readonly
                    class="input-container form-control form-control-sm me-2"
                    style="font-family: monospace;"
                    @mousedown="(e) => (e.target as HTMLInputElement).select()"
                />
              </template>
              <template v-else>
                <span style="display: inline-block;">•••••••</span>
              </template>
              <img
                  :src="shownCerts.has(cert.id) ?  '/images/eye-open.png' : '/images/eye-hidden.png'"
                  alt="Logo"
                  class="eye-icon d-block mx-auto mb-4"
                  @click="togglePasswordShown(cert)"
              />
            </td>
            <td>
              <button class="btn btn-primary btn-sm" style="margin-right: 1%; width: 30%" @click="downloadCertificate(cert.id)">
                Download
              </button>
              <button v-if="isAdmin" class="btn btn-danger btn-sm" style="width: 30%" @click="confirmDeletion(cert)">
                Delete
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <button
        v-if="isAdmin"
        class="btn btn-primary mb-3"
        @click="showGenerateModal"
    >
      Create New Certificate
    </button>

    <div v-if="loading" class="text-center mt-3">Loading certificates...</div>
    <div v-if="error" class="alert alert-danger mt-3">{{ error }}</div>

    <!-- Generate Certificate Modal -->
    <div
        v-if="isGenerateModalVisible"
        class="modal show d-block"
        tabindex="-1"
        style="background: rgba(0, 0, 0, 0.5)"
    >
      <div class="modal-dialog">
        <div class="modal-content">
          <div class="modal-header">
            <h5 class="modal-title">Generate New Certificate</h5>
            <button type="button" class="btn-close" @click="closeGenerateModal"></button>
          </div>
          <div class="modal-body">
            <div class="mb-3">
              <label for="certName" class="form-label">Certificate Name</label>
              <input
                  id="certName"
                  v-model="certReq.cert_name"
                  type="text"
                  class="form-control"
                  placeholder="Enter certificate name"
              />
            </div>
            <div class="mb-3">
              <label for="userId" class="form-label">User</label>
              <select
                  id="userId"
                  v-model="certReq.user_id"
                  class="form-control"
              >
                <option value="" disabled>Select a user</option>
                <option v-for="user in userStore.users" :key="user.id" :value="user.id">
                  {{ user.name }}
                </option>
              </select>
            </div>
            <div class="mb-3">
              <label for="validity" class="form-label">Validity (years)</label>
              <input
                  id="validity"
                  v-model.number="certReq.validity_in_years"
                  type="number"
                  class="form-control"
                  min="1"
                  placeholder="Enter validity period"
              />
            </div>
            <div class="mb-3 form-check form-switch">
              <input
                  type="checkbox"
                  class="form-check-input"
                  id="systemGeneratedPassword"
                  v-model="certReq.system_generated_password"
                  :disabled="passwordRule == PasswordRule.System"
                  role="switch"
              />
              <label class="form-check-label" for="system_generated_password">
                System Generated Password
              </label>
            </div>
            <div class="mb-3" v-if="!certReq.system_generated_password">
              <label for="certPassword" class="form-label">Password</label>
              <input
                  id="certPassword"
                  v-model="certReq.pkcs12_password"
                  type="text"
                  class="form-control"
                  placeholder="Enter password"
              />
            </div>
            <div v-if="isMailValid" class="mb-3 form-check form-switch">
              <input
                  type="checkbox"
                  class="form-check-input"
                  id="notify-user"
                  v-model="certReq.notify_user"
                  role="switch"
              />
              <label class="form-check-label" for="notify-user">
                Notify User
              </label>
            </div>
          </div>
          <div class="modal-footer">
            <button type="button" class="btn btn-secondary" @click="closeGenerateModal">
              Cancel
            </button>
            <button
                type="button"
                class="btn btn-primary"
                :disabled="loading || ((!certReq.system_generated_password && certReq.pkcs12_password.length == 0) && passwordRule == PasswordRule.Required)"
                @click="createCertificate"
            >
              <span v-if="loading">Creating...</span>
              <span v-else>Create Certificate</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div
        v-if="isDeleteModalVisible"
        class="modal show d-block"
        tabindex="-1"
        style="background: rgba(0, 0, 0, 0.5)"
    >
      <div class="modal-dialog">
        <div class="modal-content">
          <div class="modal-header">
            <h5 class="modal-title">Delete Certificate</h5>
            <button type="button" class="btn-close" @click="closeDeleteModal"></button>
          </div>
          <div class="modal-body">
            <p>
              Are you sure you want to delete the certificate
              <strong>{{ certToDelete?.name }}</strong>?
            </p>
            <p class="text-warning">
              <small>
                Disclaimer: Deleting the certificate will not revoke it. The certificate will remain
                valid until its expiration date.
              </small>
            </p>
          </div>
          <div class="modal-footer">
            <button type="button" class="btn btn-secondary" @click="closeDeleteModal">
              Cancel
            </button>
            <button type="button" class="btn btn-danger" @click="deleteCertificate">
              Delete
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue';
import { useCertificateStore } from '@/stores/certificates';
import type { Certificate } from "@/types/Certificate";
import type { CertificateRequirements } from "@/types/CertificateRequirements";
import { useAuthStore } from "@/stores/auth.ts";
import { UserRole } from "@/types/User.ts";
import { useUserStore } from "@/stores/users.ts";
import { useSettingsStore } from "@/stores/settings.ts";
import { PasswordRule } from "@/types/Settings.ts";

// stores
const certificateStore = useCertificateStore();
const authStore = useAuthStore();
const userStore = useUserStore();
const settingStore = useSettingsStore();

// local state
const shownCerts = ref(new Set<number>());

const certificates = computed(() => certificateStore.certificates);
const settings = computed(() => settingStore.settings);
const loading = computed(() => certificateStore.loading);
const error = computed(() => certificateStore.error);

const isDeleteModalVisible = ref(false);
const isGenerateModalVisible = ref(false);
const certToDelete = ref<Certificate | null>(null);

const passwordRule = computed(() => {
  return settings.value?.common.password_rule ?? PasswordRule.Optional;
});

const certReq = reactive<CertificateRequirements>({
  cert_name: '',
  user_id: 0,
  validity_in_years: 1,
  system_generated_password: passwordRule.value == PasswordRule.System,
  pkcs12_password: '',
  notify_user: false,
});

const isAdmin = computed(() => {
  return authStore.current_user !== null && authStore.current_user.role === UserRole.Admin;
});

const isMailValid = computed(() => {
  return (settings.value?.mail.smtp_host.length ?? 0) > 0 && (settings.value?.mail.smtp_port ?? 0) > 0;
});

watch(passwordRule, (newVal) => {
  certReq.system_generated_password = (newVal === PasswordRule.System);
}, { immediate: true });

onMounted(async () => {
  await certificateStore.fetchCertificates();
  await settingStore.fetchSettings();
  if (isAdmin.value) {
    await userStore.fetchUsers();
  }
});

const showGenerateModal = async () => {
  await userStore.fetchUsers();
  isGenerateModalVisible.value = true;
};

const closeGenerateModal = () => {
  isGenerateModalVisible.value = false;
  certReq.cert_name = '';
  certReq.user_id = 0;
  certReq.validity_in_years = 1;
  certReq.pkcs12_password = '';
  certReq.notify_user = false;
};

const createCertificate = async () => {
    await certificateStore.createCertificate(certReq);
    closeGenerateModal();
};

const confirmDeletion = (cert: Certificate) => {
  certToDelete.value = cert;
  isDeleteModalVisible.value = true;
};

const closeDeleteModal = () => {
  certToDelete.value = null;
  isDeleteModalVisible.value = false;
};

const downloadCertificate = async (certId: number) => {
  await certificateStore.downloadCertificate(certId);
}

const deleteCertificate = async () => {
  if (certToDelete.value) {
    await certificateStore.deleteCertificate(certToDelete.value.id);
    closeDeleteModal();
  }
};

const togglePasswordShown = async (cert: Certificate) => {
  if (!cert.pkcs12_password) {
    await certificateStore.fetchCertificatePassword(cert.id);
  }

  if (shownCerts.value.has(cert.id)) {
    shownCerts.value.delete(cert.id);
  } else {
    shownCerts.value.add(cert.id);
  }
};
</script>


<style scoped>
.modal {
  z-index: 1050;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* When multiple modals are present, we want to stack them properly */
.modal + .modal {
  z-index: 1051;
}

.password-cell {
  width: 250px;
  position: relative; 
  padding-right: 25px;
}

.input-container {
  width: 200px;
  padding-right: 25px; 
}

.password-cell .eye-icon {
  position: absolute;
  cursor: pointer;
  right: 5px; 
  top: 50%; 
  transform: translateY(-50%);
  width: 25px; 
}
</style>