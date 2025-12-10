<template>
  <div class="auth-config">
    <div v-if="authType === 'token'" class="auth-section">
      <div class="form-group">
        <label for="auth-token">Personal Access Token</label>
        <input
          type="password"
          id="auth-token"
          v-model="token"
          class="form-control"
          placeholder="Enter your personal access token"
          @input="updateConfig"
        >
        <div class="help-text">
          <p>Enter your GitHub/GitLab personal access token. </p>
          <p>Make sure it has the necessary permissions to read repositories.</p>
        </div>
      </div>
    </div>

    <div v-else-if="authType === 'ssh_key'" class="auth-section">
      <div class="form-group">
        <label for="ssh-key-path">SSH Key Path</label>
        <div class="path-input-group">
          <input
            type="text"
            id="ssh-key-path"
            v-model="sshKeyPath"
            class="form-control"
            placeholder="/home/user/.ssh/id_rsa"
            @input="updateConfig"
          >
          <button
            type="button"
            class="btn btn-secondary"
            @click="selectSSHKey"
          >
            Browse
          </button>
        </div>
      </div>

      <div class="form-group">
        <label for="ssh-passphrase">SSH Key Passphrase (Optional)</label>
        <input
          type="password"
          id="ssh-passphrase"
          v-model="sshPassphrase"
          class="form-control"
          placeholder="Enter SSH key passphrase (if applicable)"
          @input="updateConfig"
        >
      </div>

      <div class="help-text">
        <p>Select your SSH private key file (usually id_rsa, id_ed25519, etc.)</p>
        <p>Leave passphrase empty if your key is not encrypted.</p>
      </div>
    </div>

    <div v-else-if="authType === 'basic'" class="auth-section">
      <div class="form-row">
        <div class="form-group">
          <label for="basic-username">Username</label>
          <input
            type="text"
            id="basic-username"
            v-model="basicUsername"
            class="form-control"
            placeholder="Enter your username"
            @input="updateConfig"
          >
        </div>
        <div class="form-group">
          <label for="basic-password">Password</label>
          <input
            type="password"
            id="basic-password"
            v-model="basicPassword"
            class="form-control"
            placeholder="Enter your password"
            @input="updateConfig"
          >
        </div>
      </div>

      <div class="help-text">
        <p>Enter your Git service username and password.</p>
        <p>Consider using personal access tokens instead for better security.</p>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, reactive, watch, computed } from 'vue'

export default {
  name: 'AuthConfig',
  props: {
    authType: {
      type: String,
      required: true,
      validator: (value) => ['token', 'ssh_key', 'basic', 'none'].includes(value)
    },
    authConfig: {
      type: Object,
      default: () => ({})
    }
  },
  emits: ['update:authConfig'],
  setup(props, { emit }) {
    // Local state for different auth types
    const token = ref('')
    const sshKeyPath = ref('')
    const sshPassphrase = ref('')
    const basicUsername = ref('')
    const basicPassword = ref('')

    // Initialize from props
    const initializeFromProps = () => {
      if (props.authConfig.token) {
        token.value = props.authConfig.token
      }
      if (props.authConfig.ssh_key_path) {
        sshKeyPath.value = props.authConfig.ssh_key_path
      }
      if (props.authConfig.ssh_passphrase) {
        sshPassphrase.value = props.authConfig.ssh_passphrase
      }
      if (props.authConfig.username) {
        basicUsername.value = props.authConfig.username
      }
      if (props.authConfig.password) {
        basicPassword.value = props.authConfig.password
      }
    }

    // Watch for auth type changes and reset fields
    watch(() => props.authType, (newType, oldType) => {
      if (newType !== oldType) {
        // Reset all fields when auth type changes
        token.value = ''
        sshKeyPath.value = ''
        sshPassphrase.value = ''
        basicUsername.value = ''
        basicPassword.value = ''
        updateConfig()
      }
    })

    // Watch for prop changes
    watch(() => props.authConfig, initializeFromProps, { immediate: true })

    // Computed auth config based on current type
    const currentAuthConfig = computed(() => {
      switch (props.authType) {
        case 'token':
          return { token: token.value }
        case 'ssh_key':
          return {
            ssh_key_path: sshKeyPath.value,
            ssh_passphrase: sshPassphrase.value
          }
        case 'basic':
          return {
            username: basicUsername.value,
            password: basicPassword.value
          }
        default:
          return {}
      }
    })

    // Update parent component
    const updateConfig = () => {
      emit('update:authConfig', currentAuthConfig.value)
    }

    // Select SSH key file
    const selectSSHKey = async () => {
      try {
        const { dialog } = window.__TAURI__.dialog
        const selected = await dialog.open({
          multiple: false,
          filters: [
            {
              name: 'SSH Keys',
              extensions: ['', 'rsa', 'ed25519', 'pem']
            }
          ]
        })

        if (selected) {
          sshKeyPath.value = selected
          updateConfig()
        }
      } catch (error) {
        console.error('Failed to open file dialog:', error)
        // Fallback to manual input
        alert('Failed to open file dialog. Please enter the path manually.')
      }
    }

    return {
      // State
      token,
      sshKeyPath,
      sshPassphrase,
      basicUsername,
      basicPassword,
      // Methods
      updateConfig,
      selectSSHKey
    }
  }
}
</script>

<style scoped>
.auth-config {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  padding: 1.5rem;
  margin-top: 1rem;
}

.auth-section {
  animation: fadeIn 0.3s ease-in-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
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

.btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  font-size: 0.9rem;
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

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background: #5a6268;
  transform: translateY(-1px);
}

.help-text {
  margin-top: 0.5rem;
  padding: 0.75rem;
  background: #e9ecef;
  border-radius: 4px;
  font-size: 0.875rem;
  color: #6c757d;
}

.help-text p {
  margin: 0.25rem 0;
}

.help-text p:first-child {
  margin-top: 0;
}

.help-text p:last-child {
  margin-bottom: 0;
}

@media (max-width: 768px) {
  .form-row {
    grid-template-columns: 1fr;
  }
}
</style>

<style>
/* Scoped styles don't work with v-html, so we need global styles */
.auth-config :deep(.help-text) {
  margin-top: 0.5rem;
}
</style>

<style scoped>
/* Scoped styles for component-specific styling */
.auth-config {
  background: #f8f9fa;
  border-left: 4px solid #667eea;
}
</style>

<style>
/* Global styles for auth config */
.auth-config {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  padding: 1.5rem;
  margin-top: 1rem;
}
</style>

<style>
/* Component styles */
.auth-config {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  padding: 1.5rem;
  margin-top: 1rem;
  transition: all 0.3s ease;
}

.auth-config:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}
</style>

<style>
/* Final scoped styles */
.auth-config {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  padding: 1.5rem;
  margin-top: 1rem;
  transition: all 0.3s ease;
}
</style>