<template>
  <div>
    <h1>Generate</h1>
    <hr />
    <div class="mb-3">
      <label for="certName" class="form-label">Name</label>
      <input
          id="certName"
          v-model="certReq.name"
          type="text"
          class="form-control"
          placeholder="Enter certificate name"
      />
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
    <button
        class="btn btn-primary"
        :disabled="loading"
        @click="createCertificate"
    >
      <span v-if="loading">Creating...</span>
      <span v-else>Create Certificate</span>
    </button>
    <div v-if="error" class="alert alert-danger mt-3">{{ error }}</div>
  </div>
</template>

<script lang="ts">
import { defineComponent, reactive } from 'vue';
import { useCertificateStore } from '@/stores/certificates';
import type {CertificateRequirements} from "@/types/CertificateRequirements.ts";

export default defineComponent({
  name: 'GenerateTab',

  setup() {
    const certificateStore = useCertificateStore();

    // Reactive form state
    const certReq = reactive<CertificateRequirements>({
      name: '',
      validity_in_years: 1,
    });

    // Create a new certificate and reset the form
    const createCertificate = async () => {
      try {
        await certificateStore.createCertificate(certReq);
        certReq.name = '';
        certReq.validity_in_years = 1;
      } catch (error) {
        console.error('Error creating certificate:', error);
      }
    };

    return {
      certReq,
      createCertificate,
      loading: certificateStore.loading,
      error: certificateStore.error,
    };
  },
});
</script>
