// src/components/UserTab.vue
<template>
  <div class="user-tab">
    <h1>Users</h1>
    <hr />
    <!-- Loading and Error states -->
    <div v-if="userStore.loading" class="alert alert-info">
      Loading...
    </div>
    <div v-if="userStore.error" class="alert alert-danger">
      {{ userStore.error }}
    </div>

    <!-- Users Table -->
    <div class="table-responsive">
      <table class="table table-striped">
        <thead>
          <tr>
            <th>Username</th>
            <th>E-Mail</th>
            <th>Role</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in userStore.users" :key="user.id">
            <td>{{ user.name }}</td>
            <td>{{ user.email }}</td>
            <td>{{ user.role }}</td>
            <td>
              <button 
                class="btn btn-danger btn-sm"
                @click="handleDeleteUser(user.id)"
              >
                Delete
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Create User Button -->
    <button 
      class="btn btn-primary mb-3"
      @click="showCreateModal = true"
    >
      Create New User
    </button>

    <!-- Create User Modal -->
    <div 
      class="modal fade" 
      :class="{ 'show d-block': showCreateModal }"
      tabindex="-1"
      v-if="showCreateModal"
    >
      <div class="modal-dialog">
        <div class="modal-content">
          <div class="modal-header">
            <h5 class="modal-title">Create New User</h5>
            <button 
              type="button" 
              class="btn-close" 
              @click="showCreateModal = false"
            ></button>
          </div>
          <div class="modal-body">
            <form @submit.prevent="handleCreateUser">
              <div class="mb-3">
                <label for="user_name" class="form-label">Username</label>
                <input 
                  type="text" 
                  class="form-control" 
                  id="user_name"
                  v-model="newUser.user_name"
                  required
                >
              </div>
              <div class="mb-3">
                <label for="user_email" class="form-label">E-Mail</label>
                <input
                    type="text"
                    class="form-control"
                    id="user_email"
                    v-model="newUser.user_email"
                    required
                >
              </div>
              <div class="mb-3">
                <label for="password" class="form-label">Password</label>
                <input 
                  type="password" 
                  class="form-control" 
                  id="password"
                  v-model="newUser.password"
                >
              </div>
              <div class="mb-3">
                <label for="user_role" class="form-label">Role</label>
                <select
                    class="form-select"
                    id="user_role"
                    v-model="newUser.role"
                    required
                >
                  <option :value="UserRole.User">User</option>
                  <option :value="UserRole.Admin">Admin</option>
                </select>
              </div>

              <div class="modal-footer">
                <button 
                  type="button" 
                  class="btn btn-secondary" 
                  @click="showCreateModal = false"
                >
                  Cancel
                </button>
                <button type="submit" class="btn btn-primary">
                  Create User
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
    <!-- Modal Backdrop -->
    <div 
      class="modal-backdrop fade show" 
      v-if="showCreateModal"
    ></div>
  </div>
</template>

<script lang="ts">
import {defineComponent, onMounted, ref} from 'vue'
import {type CreateUserRequest, UserRole} from '@/types/User'
import {useUserStore} from "@/stores/users.ts";

export default defineComponent({
  name: 'UserTab',
  computed: {
    UserRole() {
      return UserRole
    }
  },
  setup() {
    const userStore = useUserStore()
    const showCreateModal = ref(false)
    const newUser = ref<CreateUserRequest>({
      user_name: '',
      user_email: '',
      password: '',
      role: UserRole.User
    })

    onMounted(async () => {
      await userStore.fetchUsers()
    })

    const handleCreateUser = async () => {
      await userStore.createUser(newUser.value)
      showCreateModal.value = false
      // Reset form
      newUser.value = {
        user_name: '',
        user_email: '',
        password: '',
        role: UserRole.User
      }
    }

    const handleDeleteUser = async (id: number) => {
      if (confirm('Are you sure you want to delete this user?')) {
        await userStore.deleteUser(id)
      }
    }

    return {
      userStore,
      showCreateModal,
      newUser,
      handleCreateUser,
      handleDeleteUser
    }
  }
})
</script>

<style scoped>
.user-tab {
  padding: 20px;
}

/* Modal overlay styles when modal is shown */
:deep(.modal.show) {
  background-color: rgba(0, 0, 0, 0.5);
}
</style>