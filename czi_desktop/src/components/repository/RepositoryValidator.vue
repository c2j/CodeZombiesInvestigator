<template>
  <div v-if="result" class="repository-validator modal-overlay" @click="$emit('close')">
    <div class="validator-content modal-content" @click.stop>
      <div class="validator-header">
        <h3>Repository Validation Results</h3>
        <button class="close-btn" @click="$emit('close')" title="Close">×</button>
      </div>

      <div class="validator-body">
        <!-- Success State -->
        <div v-if="result.accessible" class="validation-success">
          <div class="success-icon">✅</div>
          <h4>Connection Successful!</h4>
          <p>The repository is accessible and ready for analysis.</p>

          <div class="validation-details">
            <div class="detail-section">
              <h5>Repository Information</h5>
              <div class="detail-grid">
                <div class="detail-item">
                  <span class="detail-label">Repository Type:</span>
                  <span class="detail-value">{{ result.repository_type || 'Unknown' }}</span>
                </div>
                <div v-if="result.default_branch" class="detail-item">
                  <span class="detail-label">Default Branch:</span>
                  <span class="detail-value">{{ result.default_branch }}</span>
                </div>
                <div v-if="result.branches && result.branches.length" class="detail-item">
                  <span class="detail-label">Available Branches:</span>
                  <span class="detail-value">{{ result.branches.length }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">Authentication Method:</span>
                  <span class="detail-value">{{ formatAuthMethod(result.auth_method) }}</span>
                </div>
              </div>
            </div>

            <div v-if="result.branches && result.branches.length" class="detail-section">
              <h5>Available Branches</h5>
              <div class="branches-list">
                <div
                  v-for="branch in result.branches.slice(0, 10)"
                  :key="branch"
                  class="branch-item"
                >
                  <span class="branch-name">{{ branch }}</span>
                  <span v-if="branch === result.default_branch" class="default-badge">Default</span>
                </div>
                <div v-if="result.branches.length > 10" class="more-branches">
                  ... and {{ result.branches.length - 10 }} more branches
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Error State -->
        <div v-else class="validation-error">
          <div class="error-icon">⚠️</div>
          <h4>Connection Failed</h4>
          <p>Unable to access the repository. Please check the details and try again.</p>

          <div v-if="result.error" class="error-details">
            <h5>Error Details</h5>
            <div class="error-message">
              <pre>{{ result.error }}</pre>
            </div>
          </div>

          <div class="error-suggestions">
            <h5>Troubleshooting Suggestions</h5>
            <ul>
              <li v-if="result.auth_method === 'none'">
                Verify that the repository URL is correct and publicly accessible
              </li>
              <li v-if="['token', 'basic'].includes(result.auth_method)">
                Check your authentication credentials and ensure they have the necessary permissions
              </li>
              <li v-if="result.auth_method === 'ssh_key'">
                Verify that your SSH key is correctly configured and has the right permissions
              </li>
              <li>Ensure you have network connectivity to the Git service</li>
              <li>Check if the repository exists and you have access to it</li>
            </ul>
          </div>
        </div>

        <!-- Tested URL -->
        <div class="tested-url">
          <strong>Tested URL:</strong> {{ result.tested_url || 'Not available' }}
        </div>
      </div>

      <div class="validator-actions">
        <button class="btn btn-primary" @click="$emit('close')">
          Close
        </button>
        <button
          v-if="!result.accessible"
          class="btn btn-secondary"
          @click="retryValidation"
        >
          Retry
        </button>
      </div>
    </div>
  </div>
</template>

<script>
import { computed } from 'vue'

export default {
  name: 'RepositoryValidator',
  props: {
    result: {
      type: Object,
      required: true,
      validator: (value) => {
        return value && typeof value.accessible === 'boolean'
      }
    }
  },
  emits: ['close', 'retry'],
  setup(props, { emit }) {
    // Format authentication method for display
    const formatAuthMethod = (method) => {
      const authMethodMap = {
        'none': 'No Authentication',
        'token': 'Personal Access Token',
        'ssh_key': 'SSH Key',
        'basic': 'Username/Password'
      }
      return authMethodMap[method] || method
    }

    // Retry validation
    const retryValidation = () => {
      emit('retry')
    }

    return {
      formatAuthMethod,
      retryValidation
    }
  }
}
</script>

<style scoped>
.repository-validator {
  /* Inherit modal styles from parent */
}

.validator-content {
  max-width: 600px;
  width: 90%;
}

.validator-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid #e9ecef;
}

.validator-header h3 {
  margin: 0;
  color: #495057;
  font-size: 1.25rem;
}

.close-btn {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  color: #6c757d;
  padding: 0.25rem;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.close-btn:hover {
  color: #495057;
  background: #f8f9fa;
}

.validation-success {
  text-align: center;
  animation: successPulse 0.6s ease;
}

@keyframes successPulse {
  0% { transform: scale(0.95); }
  50% { transform: scale(1.05); }
  100% { transform: scale(1); }
}

.success-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
  animation: bounce 0.6s ease;
}

@keyframes bounce {
  0%, 20%, 50%, 80%, 100% { transform: translateY(0); }
  40% { transform: translateY(-10px); }
  60% { transform: translateY(-5px); }
}

.validation-success h4 {
  color: #28a745;
  margin-bottom: 0.5rem;
  font-size: 1.25rem;
}

.validation-success p {
  color: #6c757d;
  margin-bottom: 1.5rem;
}

.validation-details {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 1.5rem;
  margin-top: 1.5rem;
  text-align: left;
}

.detail-section {
  margin-bottom: 1.5rem;
}

.detail-section:last-child {
  margin-bottom: 0;
}

.detail-section h5 {
  margin: 0 0 1rem 0;
  color: #495057;
  font-size: 1rem;
  font-weight: 600;
}

.detail-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.detail-label {
  font-weight: 500;
  color: #6c757d;
  font-size: 0.875rem;
}

.detail-value {
  color: #495057;
  font-weight: 500;
}

.branches-list {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid #dee2e6;
  border-radius: 4px;
  padding: 0.5rem;
}

.branch-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem;
  border-radius: 4px;
  transition: background-color 0.15s ease;
}

.branch-item:hover {
  background: #f8f9fa;
}

.branch-name {
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
}

.default-badge {
  background: #28a745;
  color: white;
  padding: 0.25rem 0.5rem;
  border-radius: 12px;
  font-size: 0.75rem;
  font-weight: 500;
}

.more-branches {
  text-align: center;
  color: #6c757d;
  font-style: italic;
  padding: 0.5rem;
  font-size: 0.875rem;
}

.validation-error {
  text-align: center;
}

.error-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
  animation: shake 0.5s ease;
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-5px); }
  75% { transform: translateX(5px); }
}

.validation-error h4 {
  color: #dc3545;
  margin-bottom: 0.5rem;
  font-size: 1.25rem;
}

.validation-error p {
  color: #6c757d;
  margin-bottom: 1.5rem;
}

.error-details {
  background: #f8d7da;
  border: 1px solid #f5c6cb;
  border-radius: 8px;
  padding: 1.5rem;
  margin-top: 1.5rem;
  text-align: left;
}

.error-details h5 {
  margin: 0 0 1rem 0;
  color: #721c24;
  font-size: 1rem;
  font-weight: 600;
}

.error-message {
  background: #ffffff;
  border: 1px solid #f5c6cb;
  border-radius: 4px;
  padding: 1rem;
}

.error-message pre {
  margin: 0;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  color: #721c24;
  white-space: pre-wrap;
  word-break: break-word;
}

.error-suggestions {
  background: #fff3cd;
  border: 1px solid #ffeaa7;
  border-radius: 8px;
  padding: 1.5rem;
  margin-top: 1.5rem;
  text-align: left;
}

.error-suggestions h5 {
  margin: 0 0 1rem 0;
  color: #856404;
  font-size: 1rem;
  font-weight: 600;
}

.error-suggestions ul {
  margin: 0;
  padding-left: 1.5rem;
  color: #856404;
}

.error-suggestions li {
  margin-bottom: 0.5rem;
  line-height: 1.5;
}

.error-suggestions li:last-child {
  margin-bottom: 0;
}

.tested-url {
  background: #e9ecef;
  border-radius: 4px;
  padding: 0.75rem;
  margin-top: 1.5rem;
  font-size: 0.875rem;
  color: #495057;
  text-align: left;
}

.tested-url strong {
  color: #212529;
}

.validator-actions {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
  margin-top: 1.5rem;
  padding-top: 1.5rem;
  border-top: 1px solid #e9ecef;
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
  .validator-content {
    width: 95%;
    margin: 1rem;
  }

  .detail-grid {
    grid-template-columns: 1fr;
  }

  .validator-actions {
    flex-direction: column;
  }

  .branches-list {
    max-height: 150px;
  }
}
</style>