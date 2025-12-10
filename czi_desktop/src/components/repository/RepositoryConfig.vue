<template>
  <div class="repository-config">
    <h3>Add New Repository</h3>
    <form @submit.prevent="handleSubmit" class="repository-form">
      <div class="form-row">
        <div class="form-group">
          <label for="repo-name">Repository Name</label>
          <input
            type="text"
            id="repo-name"
            v-model="formData.name"
            class="form-control"
            placeholder="e.g., backend-service"
            required
            @input="updateLocalPath"
          >
        </div>
        <div class="form-group">
          <label for="repo-url">Git URL</label>
          <input
            type="url"
            id="repo-url"
            v-model="formData.url"
            class="form-control"
            placeholder="https://github.com/user/repo.git"
            required
          >
        </div>
      </div>

      <div class="form-row">
        <div class="form-group">
          <label for="repo-branch">Branch</label>
          <input
            type="text"
            id="repo-branch"
            v-model="formData.branch"
            class="form-control"
            placeholder="main"
          >
        </div>
        <div class="form-group">
          <label for="repo-auth-type">Authentication</label>
          <select
            id="repo-auth-type"
            v-model="formData.authType"
            class="form-control"
            @change="onAuthTypeChange"
          >
            <option value="none">None (Public Repository)</option>
            <option value="token">Personal Access Token</option>
            <option value="ssh_key">SSH Key</option>
            <option value="basic">Username/Password</option>
          </select>
        </div>
      </div>

      <!-- Authentication Configuration -->
      <AuthConfig
        v-if="formData.authType !== 'none'"
        :auth-type="formData.authType"
        v-model:auth-config="formData.authConfig"
      />

      <div class="form-group">
        <label for="repo-local-path">Local Cache Path</label>
        <div class="path-input-group">
          <input
            type="text"
            id="repo-local-path"
            v-model="formData.localPath"
            class="form-control"
            placeholder="./cache/repository-name"
          >
          <button
            type="button"
            class="btn btn-secondary"
            @click="selectDirectory"
          >
            Browse
          </button>
        </div>
      </div>

      <div class="form-actions">
        <button type="submit" class="btn btn-primary" :disabled="isSubmitting">
          {{ isSubmitting ? 'Adding...' : 'Add Repository' }}
        </button>
        <button
          type="button"
          class="btn btn-secondary"
          @click="validateRepository"
          :disabled="isValidating"
        >
          {{ isValidating ? 'Testing...' : 'Test Connection' }}
        </button>
        <button
          type="button"
          class="btn btn-secondary"
          @click="resetForm"
        >
          Clear Form
        </button>
      </div>
    </form>

    <!-- Validation Results -->
    <RepositoryValidator
      v-if="validationResult"
      :result="validationResult"
      @close="validationResult = null"
    />
  </div>
</template>

<script>
import { ref, reactive, watch } from 'vue'
import AuthConfig from './AuthConfig.vue'
import RepositoryValidator from './RepositoryValidator.vue'

export default {
  name: 'RepositoryConfig',
  components: {
    AuthConfig,
    RepositoryValidator
  },
  emits: ['repository-added', 'repository-validated'],
  setup(props, { emit }) {
    const isSubmitting = ref(false)
    const isValidating = ref(false)
    const validationResult = ref(null)

    const formData = reactive({
      name: '',
      url: '',
      branch: 'main',
      authType: 'none',
      authConfig: {},
      localPath: ''
    })

    // Watch for auth type changes and reset auth config
    const onAuthTypeChange = () => {
      formData.authConfig = {}
    }

    // Update local path based on repository name
    const updateLocalPath = () => {
      if (formData.name && !formData.localPath) {
        formData.localPath = `./cache/${formData.name}`
      }
    }

    // Handle form submission
    const handleSubmit = async () => {
      if (isSubmitting.value) return

      isSubmitting.value = true
      try {
        const { invoke } = window.__TAURI__.tauri

        const repositoryData = {
          name: formData.name,
          url: formData.url,
          branch: formData.branch,
          auth_config: {
            auth_type: formData.authType,
            ...formData.authConfig
          },
          local_path: formData.localPath
        }

        await invoke('add_repository', { repository: repositoryData })

        emit('repository-added', repositoryData)
        showAlert('Repository added successfully!', 'success')
        resetForm()
      } catch (error) {
        console.error('Failed to add repository:', error)
        showAlert(`Failed to add repository: ${error.message}`, 'error')
      } finally {
        isSubmitting.value = false
      }
    }

    // Validate repository connection
    const validateRepository = async () => {
      if (isValidating.value) return

      // Basic validation
      if (!formData.url) {
        showAlert('Please enter a repository URL', 'warning')
        return
      }

      isValidating.value = true
      try {
        const { invoke } = window.__TAURI__.tauri

        const testData = {
          url: formData.url,
          auth_config: {
            auth_type: formData.authType,
            ...formData.authConfig
          }
        }

        const result = await invoke('test_repository_access', {
          url: formData.url,
          auth_config: testData.auth_config,
          temp_dir: './temp_test'
        })

        validationResult.value = result
        emit('repository-validated', result)

        if (result.accessible) {
          showAlert('Repository connection successful!', 'success')
        } else {
          showAlert(`Connection failed: ${result.error}`, 'error')
        }
      } catch (error) {
        console.error('Failed to validate repository:', error)
        showAlert(`Validation failed: ${error.message}`, 'error')
      } finally {
        isValidating.value = false
      }
    }

    // Select directory using Tauri dialog
    const selectDirectory = async () => {
      try {
        const { dialog } = window.__TAURI__.dialog
        const selected = await dialog.open({
          directory: true,
          multiple: false
        })

        if (selected) {
          formData.localPath = selected
        }
      } catch (error) {
        console.error('Failed to open directory dialog:', error)
        showAlert('Failed to open directory dialog', 'error')
      }
    }

    // Reset form to initial state
    const resetForm = () => {
      formData.name = ''
      formData.url = ''
      formData.branch = 'main'
      formData.authType = 'none'
      formData.authConfig = {}
      formData.localPath = ''
      validationResult.value = null
    }

    // Show alert message
    const showAlert = (message, type) => {
      // This would integrate with a global alert system
      console.log(`[${type.toUpperCase()}] ${message}`)
      // Could emit to parent component or use global state
    }

    return {
      formData,
      isSubmitting,
      isValidating,
      validationResult,
      onAuthTypeChange,
      updateLocalPath,
      handleSubmit,
      validateRepository,
      selectDirectory,
      resetForm
    }
  }
}
</script>

<style scoped>
.repository-config {
  background: white;
  border-radius: 8px;
  padding: 2rem;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  margin-bottom: 2rem;
}

.repository-form {
  max-width: 800px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  margin-bottom: 1rem;
}

.form-group {
  display: flex;
  flex-direction: column;
}

.form-group label {
  font-weight: 500;
  margin-bottom: 0.5rem;
  color: #495057;
}

.form-control {
  padding: 0.75rem;
  border: 1px solid #ced4da;
  border-radius: 4px;
  font-size: 1rem;
  transition: border-color 0.15s ease-in-out, box-shadow 0.15s ease-in-out;
}

.form-control:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 0.2rem rgba(102, 126, 234, 0.25);
}

.path-input-group {
  display: flex;
  gap: 0.5rem;
}

.path-input-group .form-control {
  flex: 1;
}

.form-actions {
  display: flex;
  gap: 1rem;
  margin-top: 1.5rem;
}

.btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 4px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease-in-out;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background: #5a6268;
  transform: translateY(-1px);
}

@media (max-width: 768px) {
  .form-row {
    grid-template-columns: 1fr;
  }

  .form-actions {
    flex-direction: column;
  }
}
</style>