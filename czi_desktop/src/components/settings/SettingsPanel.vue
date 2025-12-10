<template>
  <div class="settings-panel">
    <h3>Application Settings</h3>
    <p class="settings-description">
      Configure application preferences and analysis options.
    </p>

    <div class="settings-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="tab-button"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        {{ tab.name }}
      </button>
    </div>

    <div class="settings-content">
      <!-- General Settings -->
      <div v-show="activeTab === 'general'" class="settings-section">
        <h4>General Settings</h4>

        <div class="form-group">
          <label class="checkbox-group">
            <input
              type="checkbox"
              v-model="settings.general.autoSave"
            >
            <span>Auto-save settings</span>
          </label>
          <div class="help-text">Automatically save settings changes</div>
        </div>

        <div class="form-group">
          <label class="checkbox-group">
            <input
              type="checkbox"
              v-model="settings.general.autoUpdate"
            >
            <span>Check for updates automatically</span>
          </label>
          <div class="help-text">Check for application updates on startup</div>
        </div>

        <div class="form-group">
          <label for="cache-dir">Cache Directory</label>
          <div class="path-input-group">
            <input
              type="text"
              id="cache-dir"
              v-model="settings.general.cacheDir"
              class="form-control"
              placeholder="./cache"
            >
            <button
              type="button"
              class="btn btn-secondary"
              @click="selectCacheDir"
            >
              Browse
            </button>
          </div>
          <div class="help-text">Directory for storing cached repository data</div>
        </div>

        <div class="form-group">
          <label for="log-level">Log Level</label>
          <select
            id="log-level"
            v-model="settings.general.logLevel"
            class="form-control"
          >
            <option value="error">Error</option>
            <option value="warn">Warning</option>
            <option value="info">Info</option>
            <option value="debug">Debug</option>
            <option value="trace">Trace</option>
          </select>
          <div class="help-text">Application logging verbosity</div>
        </div>
      </div>

      <!-- Analysis Settings -->
      <div v-show="activeTab === 'analysis'" class="settings-section">
        <h4>Analysis Settings</h4>

        <div class="form-group">
          <label for="max-depth">Default Maximum Analysis Depth</label>
          <input
            type="number"
            id="max-depth"
            v-model.number="settings.analysis.maxDepth"
            class="form-control"
            min="1"
            max="50"
          >
          <div class="help-text">Maximum depth for dependency analysis (1-50)</div>
        </div>

        <div class="form-group">
          <label for="timeout">Analysis Timeout (seconds)</label>
          <input
            type="number"
            id="timeout"
            v-model.number="settings.analysis.timeout"
            class="form-control"
            min="30"
            max="3600"
          >
          <div class="help-text">Maximum time allowed for analysis (30-3600 seconds)</div>
        </div>

        <div class="form-group">
          <label class="checkbox-group">
            <input
              type="checkbox"
              v-model="settings.analysis.incrementalAnalysis"
            >
            <span>Enable incremental analysis</span>
          </label>
          <div class="help-text">Only analyze changed files when possible</div>
        </div>

        <div class="form-group">
          <label class="checkbox-group">
            <input
              type="checkbox"
              v-model="settings.analysis.parallelProcessing"
            >
            <span>Enable parallel processing</span>
          </label>
          <div class="help-text">Use multiple CPU cores for faster analysis</div>
        </div>

        <div class="form-group">
          <label for="default-language">Default Language</label>
          <select
            id="default-language"
            v-model="settings.analysis.defaultLanguage"
            class="form-control"
          >
            <option value="auto">Auto-detect</option>
            <option value="rust">Rust</option>
            <option value="javascript">JavaScript</option>
            <option value="typescript">TypeScript</option>
            <option value="python">Python</option>
            <option value="go">Go</option>
            <option value="java">Java</option>
            <option value="cpp">C++</option>
          </select>
          <div class="help-text">Default language for analysis</div>
        </div>

        <div class="form-group">
          <label for="ignore-patterns">Default Ignore Patterns</label>
          <textarea
            id="ignore-patterns"
            v-model="ignorePatternsText"
            class="form-control"
            rows="4"
            placeholder="*.min.js\nnode_modules/**\ntest/fixtures/**\ndist/**"
          ></textarea>
          <div class="help-text">One pattern per line. Supports glob patterns.</div>
        </div>
      </div>

      <!-- UI Settings -->
      <div v-show="activeTab === 'ui'" class="settings-section">
        <h4>User Interface Settings</h4>

        <div class="form-group">
          <label for="theme">Theme</label>
          <select
            id="theme"
            v-model="settings.ui.theme"
            class="form-control"
          >
            <option value="light">Light</option>
            <option value="dark">Dark</option>
            <option value="auto">Auto (System)</option>
          </select>
          <div class="help-text">Application color theme</div>
        </div>

        <div class="form-group">
          <label class="checkbox-group">
            <input
              type="checkbox"
              v-model="settings.ui.showLineNumbers"
            >
            <span>Show line numbers in code preview</span>
          </label>
          <div class="help-text">Display line numbers in code previews</div>
        </div>

        <div class="form-group">
          <label class="checkbox-group">
            <input
              type="checkbox"
              v-model="settings.ui.syntaxHighlighting"
            >
            <span>Enable syntax highlighting</span>
          </label>
          <div class="help-text">Enable syntax highlighting in code previews</div>
        </div>

        <div class="form-group">
          <label for="font-size">Font Size</label>
          <select
            id="font-size"
            v-model="settings.ui.fontSize"
            class="form-control"
          >
            <option value="small">Small</option>
            <option value="medium">Medium</option>
            <option value="large">Large</option>
          </select>
          <div class="help-text">Interface font size</div>
        </div>

        <div class="form-group">
          <label for="results-per-page">Results Per Page</label>
          <input
            type="number"
            id="results-per-page"
            v-model.number="settings.ui.resultsPerPage"
            class="form-control"
            min="10"
            max="200"
          >
          <div class="help-text">Number of results to display per page (10-200)</div>
        </div>
      </div>

      <!-- Advanced Settings -->
      <div v-show="activeTab === 'advanced'" class="settings-section">
        <h4>Advanced Settings</h4>

        <div class="form-group">
          <label for="worker-threads">Worker Threads</label>
          <input
            type="number"
            id="worker-threads"
            v-model.number="settings.advanced.workerThreads"
            class="form-control"
            min="1"
            max="16"
          >
          <div class="help-text">Number of worker threads for analysis (1-16)</div>
        </div>

        <div class="form-group">
          <label for="memory-limit">Memory Limit (MB)</label>
          <input
            type="number"
            id="memory-limit"
            v-model.number="settings.advanced.memoryLimit"
            class="form-control"
            min="512"
            max="8192"
          >
          <div class="help-text">Maximum memory usage for analysis (512-8192 MB)</div>
        </div>

        <div class="form-group">
          <label class="checkbox-group">
            <input
              type="checkbox"
              v-model="settings.advanced.enableTelemetry"
            >
            <span>Enable telemetry</span>
          </label>
          <div class="help-text">Send anonymous usage statistics</div>
        </div>

        <div class="form-group">
          <label class="checkbox-group">
            <input
              type="checkbox"
              v-model="settings.advanced.enableBetaFeatures"
            >
            <span>Enable beta features</span>
          </label>
          <div class="help-text">Enable experimental features</div>
        </div>

        <div class="form-group">
          <label for="api-endpoint">API Endpoint</label>
          <input
            type="text"
            id="api-endpoint"
            v-model="settings.advanced.apiEndpoint"
            class="form-control"
            placeholder="https://api.example.com"
          >
          <div class="help-text">Custom API endpoint (if applicable)</div>
        </div>
      </div>
    </div>

    <div class="settings-actions">
      <button
        class="btn btn-primary"
        @click="saveSettings"
        :disabled="isSaving"
      >
        {{ isSaving ? 'Saving...' : 'Save Settings' }}
      </button>
      <button
        class="btn btn-secondary"
        @click="resetSettings"
      >
        Reset to Defaults
      </button>
      <button
        class="btn btn-secondary"
        @click="exportSettings"
      >
        Export Settings
      </button>
      <button
        class="btn btn-secondary"
        @click="importSettings"
      >
        Import Settings
      </button>
    </div>

    <!-- Import/Export File Input -->
    <input
      ref="fileInput"
      type="file"
      accept=".json"
      style="display: none"
      @change="handleFileImport"
    >
  </div>
</template>

<script>
import { ref, reactive, watch, computed } from 'vue'

export default {
  name: 'SettingsPanel',
  props: {
    settings: {
      type: Object,
      default: () => ({
        general: {
          autoSave: true,
          autoUpdate: false,
          cacheDir: './cache',
          logLevel: 'info'
        },
        analysis: {
          maxDepth: 10,
          timeout: 300,
          incrementalAnalysis: true,
          parallelProcessing: true,
          defaultLanguage: 'auto',
          ignorePatterns: []
        },
        ui: {
          theme: 'light',
          showLineNumbers: true,
          syntaxHighlighting: true,
          fontSize: 'medium',
          resultsPerPage: 50
        },
        advanced: {
          workerThreads: 4,
          memoryLimit: 2048,
          enableTelemetry: false,
          enableBetaFeatures: false,
          apiEndpoint: ''
        }
      })
    }
  },
  emits: ['settings-updated'],
  setup(props, { emit }) {
    // State
    const activeTab = ref('general')
    const isSaving = ref(false)
    const fileInput = ref(null)

    // Local settings state
    const settings = reactive({
      general: { ...props.settings.general },
      analysis: { ...props.settings.analysis },
      ui: { ...props.settings.ui },
      advanced: { ...props.settings.advanced }
    })

    // Ignore patterns text
    const ignorePatternsText = computed({
      get: () => settings.analysis.ignorePatterns.join('\n'),
      set: (value) => {
        settings.analysis.ignorePatterns = value
          .split('\n')
          .map(pattern => pattern.trim())
          .filter(pattern => pattern.length > 0)
      }
    })

    // Tabs configuration
    const tabs = [
      { id: 'general', name: 'General' },
      { id: 'analysis', name: 'Analysis' },
      { id: 'ui', name: 'User Interface' },
      { id: 'advanced', name: 'Advanced' }
    ]

    // Watch for prop changes
    watch(() => props.settings, (newSettings) => {
      Object.assign(settings, {
        general: { ...newSettings.general },
        analysis: { ...newSettings.analysis },
        ui: { ...newSettings.ui },
        advanced: { ...newSettings.advanced }
      })
    }, { deep: true })

    // Methods
    const saveSettings = async () => {
      isSaving.value = true
      try {
        if (window.__TAURI__) {
          const { invoke } = window.__TAURI__.tauri
          await invoke('save_settings', { settings: settings })
        } else {
          // Fallback for development
          localStorage.setItem('app_settings', JSON.stringify(settings))
        }
        emit('settings-updated', settings)
      } catch (error) {
        console.error('Failed to save settings:', error)
      } finally {
        isSaving.value = false
      }
    }

    const resetSettings = async () => {
      if (confirm('Are you sure you want to reset all settings to defaults?')) {
        try {
          if (window.__TAURI__) {
            const { invoke } = window.__TAURI__.tauri
            const defaultSettings = await invoke('get_default_settings')
            Object.assign(settings, defaultSettings)
          } else {
            // Fallback defaults
            const defaults = {
              general: {
                autoSave: true,
                autoUpdate: false,
                cacheDir: './cache',
                logLevel: 'info'
              },
              analysis: {
                maxDepth: 10,
                timeout: 300,
                incrementalAnalysis: true,
                parallelProcessing: true,
                defaultLanguage: 'auto',
                ignorePatterns: []
              },
              ui: {
                theme: 'light',
                showLineNumbers: true,
                syntaxHighlighting: true,
                fontSize: 'medium',
                resultsPerPage: 50
              },
              advanced: {
                workerThreads: 4,
                memoryLimit: 2048,
                enableTelemetry: false,
                enableBetaFeatures: false,
                apiEndpoint: ''
              }
            }
            Object.assign(settings, defaults)
          }
          await saveSettings()
        } catch (error) {
          console.error('Failed to reset settings:', error)
        }
      }
    }

    const exportSettings = async () => {
      try {
        const data = JSON.stringify(settings, null, 2)
        const blob = new Blob([data], { type: 'application/json' })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = 'settings.json'
        a.click()
        URL.revokeObjectURL(url)
      } catch (error) {
        console.error('Failed to export settings:', error)
      }
    }

    const importSettings = () => {
      fileInput.value?.click()
    }

    const handleFileImport = async (event) => {
      const file = event.target.files?.[0]
      if (!file) return

      try {
        const text = await file.text()
        const importedSettings = JSON.parse(text)

        // Validate imported settings
        if (validateSettings(importedSettings)) {
          Object.assign(settings, importedSettings)
          await saveSettings()
        } else {
          alert('Invalid settings file format')
        }
      } catch (error) {
        console.error('Failed to import settings:', error)
        alert('Failed to import settings file')
      }

      // Reset file input
      event.target.value = ''
    }

    const selectCacheDir = async () => {
      try {
        if (window.__TAURI__) {
          const { dialog } = window.__TAURI__.dialog
          const selected = await dialog.open({
            directory: true,
            multiple: false
          })

          if (selected) {
            settings.general.cacheDir = selected
          }
        } else {
          // Fallback for development
          const dir = prompt('Enter cache directory path:', settings.general.cacheDir)
          if (dir) {
            settings.general.cacheDir = dir
          }
        }
      } catch (error) {
        console.error('Failed to select directory:', error)
      }
    }

    const validateSettings = (settings) => {
      // Basic validation - check if all required sections exist
      const requiredSections = ['general', 'analysis', 'ui', 'advanced']
      return requiredSections.every(section => settings.hasOwnProperty(section))
    }

    return {
      // State
      activeTab,
      isSaving,
      settings,
      ignorePatternsText,
      tabs,
      fileInput,
      // Methods
      saveSettings,
      resetSettings,
      exportSettings,
      importSettings,
      handleFileImport,
      selectCacheDir
    }
  }
}
</script>

<style scoped>
.settings-panel {
  background: white;
  border-radius: 8px;
  padding: 2rem;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.settings-description {
  color: #6c757d;
  margin-bottom: 2rem;
  font-size: 1.1rem;
}

.settings-tabs {
  display: flex;
  border-bottom: 2px solid #e9ecef;
  margin-bottom: 2rem;
}

.tab-button {
  padding: 0.75rem 1.5rem;
  background: none;
  border: none;
  border-bottom: 3px solid transparent;
  color: #6c757d;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tab-button:hover {
  color: #495057;
  background: #f8f9fa;
}

.tab-button.active {
  color: #667eea;
  border-bottom-color: #667eea;
  background: #f8f9fa;
}

.settings-section {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.settings-section h4 {
  margin: 0 0 1.5rem 0;
  color: #495057;
  font-size: 1.1rem;
  font-weight: 600;
}

.form-group {
  margin-bottom: 1.5rem;
}

.form-group:last-child {
  margin-bottom: 0;
}

.checkbox-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-weight: 500;
  color: #495057;
}

.checkbox-group input[type="checkbox"] {
  width: 18px;
  height: 18px;
  accent-color: #667eea;
}

.form-group label {
  display: block;
  font-weight: 500;
  color: #495057;
  margin-bottom: 0.5rem;
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

.help-text {
  font-size: 0.875rem;
  color: #6c757d;
  margin-top: 0.5rem;
}

.settings-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 2rem;
  padding-top: 2rem;
  border-top: 1px solid #e9ecef;
  flex-wrap: wrap;
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
  white-space: nowrap;
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
  .settings-tabs {
    flex-wrap: wrap;
  }

  .tab-button {
    flex: 1;
    min-width: 120px;
  }

  .detail-controls {
    flex-direction: column;
  }

  .settings-actions {
    flex-direction: column;
  }

  .btn {
    width: 100%;
  }
}
</style>