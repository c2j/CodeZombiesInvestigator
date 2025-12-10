<template>
  <div class="repository-list">
    <div class="list-header">
      <h3>Configured Repositories</h3>
      <div class="list-actions">
        <button
          class="btn btn-secondary"
          @click="refreshList"
          :disabled="isLoading"
        >
          <span v-if="isLoading">⟳ Refreshing...</span>
          <span v-else>⟳ Refresh</span>
        </button>
      </div>
    </div>

    <div v-if="isLoading" class="loading-state">
      <div class="spinner"></div>
      <p>Loading repositories...</p>
    </div>

    <div v-else-if="repositories.length === 0" class="empty-state">
      <div class="empty-icon">📁</div>
      <h4>No repositories configured</h4>
      <p>Add your first repository to start analyzing code.</p>
    </div>

    <div v-else class="repository-items">
      <div
        v-for="repo in repositories"
        :key="repo.id"
        class="repository-item"
        :class="{ 'syncing': repo.sync_status === 'syncing' }"
      >
        <div class="repository-info">
          <div class="repository-header">
            <h4>{{ repo.name }}</h4>
            <div class="repository-status" :class="getStatusClass(repo)">
              <span class="status-indicator">{{ getStatusIcon(repo) }}</span>
              <span class="status-text">{{ getStatusText(repo) }}</span>
            </div>
          </div>

          <div class="repository-details">
            <div class="detail-item">
              <span class="detail-label">URL:</span>
              <span class="detail-value url">{{ repo.url }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Branch:</span>
              <span class="detail-value">{{ repo.branch }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Local Path:</span>
              <span class="detail-value path">{{ repo.local_path }}</span>
            </div>
            <div v-if="repo.last_sync" class="detail-item">
              <span class="detail-label">Last Sync:</span>
              <span class="detail-value">{{ formatDate(repo.last_sync) }}</span>
            </div>
          </div>

          <div v-if="repo.error" class="repository-error">
            <span class="error-icon">⚠️</span>
            <span class="error-text">{{ repo.error }}</span>
          </div>
        </div>

        <div class="repository-actions">
          <button
            class="btn btn-primary"
            @click="syncRepository(repo)"
            :disabled="repo.sync_status === 'syncing'"
            title="Synchronize repository"
          >
            <span v-if="repo.sync_status === 'syncing'">⟳ Syncing...</span>
            <span v-else>⟳ Sync</span>
          </button>

          <button
            class="btn btn-secondary"
            @click="validateRepository(repo)"
            :disabled="repo.sync_status === 'syncing'"
            title="Test repository access"
          >
            ✓ Validate
          </button>

          <button
            class="btn btn-secondary"
            @click="editRepository(repo)"
            title="Edit repository configuration"
          >
            ✏️ Edit
          </button>

          <button
            class="btn btn-danger"
            @click="confirmRemoveRepository(repo)"
            :disabled="repo.sync_status === 'syncing'"
            title="Remove repository"
          >
            🗑️ Remove
          </button>
        </div>
      </div>
    </div>

    <!-- Confirmation Modal -->
    <div v-if="showRemoveConfirm" class="modal-overlay" @click="cancelRemove">
      <div class="modal-content" @click.stop>
        <h3>Confirm Removal</h3>
        <p>Are you sure you want to remove the repository "{{ repoToRemove?.name }}"?</p>
        <p class="warning-text">This action cannot be undone.</p>
        <div class="modal-actions">
          <button class="btn btn-danger" @click="removeRepository">
            Remove
          </button>
          <button class="btn btn-secondary" @click="cancelRemove">
            Cancel
          </button>
        </div>
      </div>
    </div>

    <!-- Edit Modal -->
    <div v-if="showEditModal" class="modal-overlay" @click="cancelEdit">
      <div class="modal-content" @click.stop>
        <h3>Edit Repository</h3>
        <RepositoryConfig
          v-if="repoToEdit"
          :initial-data="repoToEdit"
          @repository-updated="handleRepositoryUpdated"
          @cancel="cancelEdit"
        />
      </div>
    </div>
  </div>
</template>

<script>
import { ref, reactive, computed, onMounted } from 'vue'
import RepositoryConfig from './RepositoryConfig.vue'

export default {
  name: 'RepositoryList',
  components: {
    RepositoryConfig
  },
  emits: ['repository-removed', 'repository-updated', 'repository-synced'],
  setup(props, { emit }) {
    const repositories = ref([])
    const isLoading = ref(true)
    const showRemoveConfirm = ref(false)
    const showEditModal = ref(false)
    const repoToRemove = ref(null)
    const repoToEdit = ref(null)

    // Load repositories on mount
    onMounted(() => {
      loadRepositories()
    })

    // Load repositories from backend
    const loadRepositories = async () => {
      isLoading.value = true
      try {
        const { invoke } = window.__TAURI__.tauri
        repositories.value = await invoke('list_repositories')
      } catch (error) {
        console.error('Failed to load repositories:', error)
        showAlert('Failed to load repositories', 'error')
      } finally {
        isLoading.value = false
      }
    }

    // Refresh repository list
    const refreshList = async () => {
      await loadRepositories()
    }

    // Sync repository
    const syncRepository = async (repo) => {
      if (repo.sync_status === 'syncing') return

      repo.sync_status = 'syncing'
      try {
        const { invoke } = window.__TAURI__.tauri
        await invoke('sync_repository', { repository_id: repo.id })

        showAlert('Repository synchronized successfully!', 'success')
        emit('repository-synced', repo)

        // Reload to get updated status
        await loadRepositories()
      } catch (error) {
        console.error('Failed to sync repository:', error)
        repo.error = error.message
        showAlert(`Failed to sync repository: ${error.message}`, 'error')
      } finally {
        repo.sync_status = 'idle'
      }
    }

    // Validate repository
    const validateRepository = async (repo) => {
      try {
        const { invoke } = window.__TAURI__.tauri
        const result = await invoke('test_repository_access', {
          url: repo.url,
          auth_config: repo.auth_config,
          temp_dir: './temp_test'
        })

        if (result.accessible) {
          showAlert('Repository validation successful!', 'success')
          repo.error = null
        } else {
          showAlert(`Validation failed: ${result.error}`, 'error')
          repo.error = result.error
        }
      } catch (error) {
        console.error('Failed to validate repository:', error)
        showAlert(`Validation failed: ${error.message}`, 'error')
        repo.error = error.message
      }
    }

    // Edit repository
    const editRepository = (repo) => {
      repoToEdit.value = { ...repo }
      showEditModal.value = true
    }

    // Handle repository updated
    const handleRepositoryUpdated = async (updatedRepo) => {
      try {
        const { invoke } = window.__TAURI__.tauri
        await invoke('update_repository', { repository: updatedRepo })

        showAlert('Repository updated successfully!', 'success')
        emit('repository-updated', updatedRepo)

        // Reload list
        await loadRepositories()
        cancelEdit()
      } catch (error) {
        console.error('Failed to update repository:', error)
        showAlert(`Failed to update repository: ${error.message}`, 'error')
      }
    }

    // Confirm remove repository
    const confirmRemoveRepository = (repo) => {
      repoToRemove.value = repo
      showRemoveConfirm.value = true
    }

    // Remove repository
    const removeRepository = async () => {
      if (!repoToRemove.value) return

      try {
        const { invoke } = window.__TAURI__.tauri
        await invoke('remove_repository', { repository_id: repoToRemove.value.id })

        showAlert('Repository removed successfully!', 'success')
        emit('repository-removed', repoToRemove.value)

        // Reload list
        await loadRepositories()
        cancelRemove()
      } catch (error) {
        console.error('Failed to remove repository:', error)
        showAlert(`Failed to remove repository: ${error.message}`, 'error')
      }
    }

    // Cancel remove
    const cancelRemove = () => {
      showRemoveConfirm.value = false
      repoToRemove.value = null
    }

    // Cancel edit
    const cancelEdit = () => {
      showEditModal.value = false
      repoToEdit.value = null
    }

    // Get status class for styling
    const getStatusClass = (repo) => {
      return {
        'status-success': repo.sync_status === 'completed' && !repo.error,
        'status-warning': repo.sync_status === 'syncing',
        'status-error': repo.error || repo.sync_status === 'failed'
      }
    }

    // Get status icon
    const getStatusIcon = (repo) => {
      if (repo.sync_status === 'syncing') return '⟳'
      if (repo.error) return '⚠️'
      if (repo.sync_status === 'completed') return '✅'
      return '⏸️'
    }

    // Get status text
    const getStatusText = (repo) => {
      if (repo.sync_status === 'syncing') return 'Syncing...'
      if (repo.error) return 'Error'
      if (repo.sync_status === 'completed') return 'Ready'
      return 'Not synced'
    }

    // Format date
    const formatDate = (dateString) => {
      if (!dateString) return 'Never'
      return new Date(dateString).toLocaleString()
    }

    // Show alert (this would integrate with global alert system)
    const showAlert = (message, type) => {
      console.log(`[${type.toUpperCase()}] ${message}`)
      // Could integrate with global alert system
    }

    return {
      // State
      repositories,
      isLoading,
      showRemoveConfirm,
      showEditModal,
      repoToRemove,
      repoToEdit,
      // Methods
      loadRepositories,
      refreshList,
      syncRepository,
      validateRepository,
      editRepository,
      handleRepositoryUpdated,
      confirmRemoveRepository,
      removeRepository,
      cancelRemove,
      cancelEdit,
      getStatusClass,
      getStatusIcon,
      getStatusText,
      formatDate
    }
  }
}
</script>

<style scoped>
.repository-list {
  background: white;
  border-radius: 8px;
  padding: 2rem;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
  padding-bottom: 1rem;
  border-bottom: 2px solid #e9ecef;
}

.list-header h3 {
  margin: 0;
  color: #495057;
  font-size: 1.25rem;
  font-weight: 600;
}

.list-actions {
  display: flex;
  gap: 0.5rem;
}

.loading-state,
.empty-state {
  text-align: center;
  padding: 3rem 2rem;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #e9ecef;
  border-top: 4px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 1rem;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.empty-state h4 {
  color: #6c757d;
  margin-bottom: 0.5rem;
}

.empty-state p {
  color: #6c757d;
  margin: 0;
}

.repository-items {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.repository-item {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  padding: 1.5rem;
  transition: all 0.3s ease;
}

.repository-item:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.repository-item.syncing {
  opacity: 0.7;
  pointer-events: none;
}

.repository-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 1rem;
}

.repository-header h4 {
  margin: 0;
  color: #495057;
  font-size: 1.1rem;
  font-weight: 600;
}

.repository-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.25rem 0.75rem;
  border-radius: 20px;
  font-size: 0.875rem;
  font-weight: 500;
}

.repository-status.status-success {
  background: #d4edda;
  color: #155724;
  border: 1px solid #c3e6cb;
}

.repository-status.status-warning {
  background: #fff3cd;
  color: #856404;
  border: 1px solid #ffeaa7;
}

.repository-status.status-error {
  background: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
}

.status-indicator {
  font-size: 1rem;
}

.repository-details {
  margin-bottom: 1rem;
}

.detail-item {
  display: flex;
  margin-bottom: 0.5rem;
  font-size: 0.9rem;
}

.detail-label {
  font-weight: 500;
  color: #6c757d;
  min-width: 100px;
  margin-right: 1rem;
}

.detail-value {
  color: #495057;
  flex: 1;
}

.detail-value.url {
  font-family: 'Courier New', monospace;
  font-size: 0.85rem;
  word-break: break-all;
}

.detail-value.path {
  font-family: 'Courier New', monospace;
  font-size: 0.85rem;
}

.repository-error {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem;
  background: #f8d7da;
  border: 1px solid #f5c6cb;
  border-radius: 4px;
  margin-bottom: 1rem;
  color: #721c24;
  font-size: 0.875rem;
}

.error-icon {
  font-size: 1rem;
}

.repository-actions {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease-in-out;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  white-space: nowrap;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: #007bff;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #0056b3;
  transform: translateY(-1px);
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background: #5a6268;
  transform: translateY(-1px);
}

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: #c82333;
  transform: translateY(-1px);
}

/* Modal styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.3s ease;
}

.modal-content {
  background: white;
  border-radius: 8px;
  padding: 2rem;
  max-width: 500px;
  width: 90%;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  animation: slideUp 0.3s ease;
}

.modal-content h3 {
  margin-top: 0;
  margin-bottom: 1rem;
  color: #495057;
}

.modal-actions {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
  margin-top: 1.5rem;
}

.warning-text {
  color: #dc3545;
  font-weight: 500;
  margin: 1rem 0;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 768px) {
  .repository-header {
    flex-direction: column;
    gap: 1rem;
  }

  .repository-actions {
    justify-content: flex-start;
  }

  .modal-content {
    width: 95%;
    padding: 1.5rem;
  }
}
</style>