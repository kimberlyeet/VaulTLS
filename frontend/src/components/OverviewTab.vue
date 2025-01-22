<template>
  <div>
    <h1>Certificates</h1>
    <hr />
    <table class="table table-bordered">
      <thead>
      <tr>
        <th>Name</th>
        <th>Created on</th>
        <th>Valid until</th>
        <th>Actions</th>
      </tr>
      </thead>
      <tbody>
      <tr v-for="cert in certificates" :key="cert.id">
        <td>{{ cert.name }}</td>
        <td>{{ new Date(cert.created_on).toLocaleDateString() }}</td>
        <td>{{ new Date(cert.valid_until).toLocaleDateString() }}</td>
        <td>
          <button class="btn btn-primary btn-sm" @click="downloadCertificate(cert.id)">
            Download
          </button>
          <button class="btn btn-danger btn-sm ms-2" @click="confirmDeletion(cert)">
            Delete
          </button>
        </td>
      </tr>
      </tbody>
    </table>

    <div v-if="loading" class="text-center mt-3">Loading certificates...</div>
    <div v-if="error" class="alert alert-danger mt-3">{{ error }}</div>

    <!-- Disclaimer Modal -->
    <div
        v-if="isModalVisible"
        class="modal show d-block"
        tabindex="-1"
        style="background: rgba(0, 0, 0, 0.5)"
    >
      <div class="modal-dialog">
        <div class="modal-content">
          <div class="modal-header">
            <h5 class="modal-title">Delete Certificate</h5>
            <button type="button" class="btn-close" @click="closeModal"></button>
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
            <button type="button" class="btn btn-secondary" @click="closeModal">
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

<script lang="ts">
import { computed, defineComponent, ref, onMounted } from 'vue';
import { useCertificateStore } from '@/stores/certificates';

export default defineComponent({
  name: 'OverviewTab',

  setup() {
    const certificateStore = useCertificateStore();

    const certificates = computed(() => certificateStore.certificates);
    const loading = computed(() => certificateStore.loading);
    const error = computed(() => certificateStore.error);

    // Local state for the modal
    const isModalVisible = ref(false);
    const certToDelete = ref(null);

    // Fetch certificates when the component is mounted
    onMounted(() => {
      certificateStore.fetchCertificates();
    });

    const confirmDeletion = (cert: any) => {
      certToDelete.value = cert;
      isModalVisible.value = true;
    };

    const closeModal = () => {
      certToDelete.value = null;
      isModalVisible.value = false;
    };

    const deleteCertificate = async () => {
      if (certToDelete.value) {
        try {
          await certificateStore.deleteCertificate(certToDelete.value.id);
        } catch (error) {
          console.error(error);
        } finally {
          closeModal();
        }
      }
    };

    return {
      certificates,
      loading,
      error,
      downloadCertificate: certificateStore.downloadCertificate,
      confirmDeletion,
      closeModal,
      deleteCertificate,
      isModalVisible,
      certToDelete,
    };
  },
});
</script>

<style scoped>
.modal {
  z-index: 1050;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
