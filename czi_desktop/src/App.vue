<template>
  <div id="app">
    <header class="app-header">
      <div class="container">
        <h1>CodeZombiesInvestigator</h1>
        <p class="subtitle">Advanced Static Analysis for Dead Code Detection</p>
      </div>
    </header>

    <div class="main-content">
      <nav class="sidebar">
        <div class="nav-menu">
          <ul class="nav-list">
            <li v-for="section in sections" :key="section.id">
              <button
                class="nav-btn"
                :class="{ active: currentSection === section.id }"
                @click="switchSection(section.id)"
              >
                <span class="nav-icon">{{ section.icon }}</span>
                <span class="nav-text">{{ section.name }}</span>
              </button>
            </li>
          </ul>
        </div>
      </nav>

      <main class="content-area">
        <!-- Global Alert Container -->
        <div id="global-alert-container" class="alert-container">
          <div
            v-for="alert in alerts"
            :key="alert.id"
            class="alert"
            :class="`alert-${alert.type}`"
          >
            <span class="alert-icon">{{ getAlertIcon(alert.type) }}</span>
            <span class="alert-message">{{ alert.message }}</span>
            <button
              class="alert-close"
              @click="dismissAlert(alert.id)"
            >×</button>
          </div>
        </div>

        <!-- Repositories Section -->
        <section v-show="currentSection === 'repositories'" class="section">
          <h2>Repository Configuration</h2>

          <RepositoryConfig
            @repository-added="handleRepositoryAdded"
            @repository-validated="handleRepositoryValidated"
          />

          <RepositoryList
            @repository-removed="handleRepositoryRemoved"
            @repository-updated="handleRepositoryUpdated"
            @repository-synced="handleRepositorySynced"
          />
        </section>

        <!-- Analysis Section -->
        <section v-show="currentSection === 'analysis'" class="section">
          <h2>Code Analysis</h2>
          <p class="section-description">Configure and run zombie code analysis on your repositories.</p>

          <AnalysisPanel
            :repositories="repositories"
            @analysis-started="handleAnalysisStarted"
            @analysis-completed="handleAnalysisCompleted"
          />
        </section>

        <!-- Results Section -->
        <section v-show="currentSection === 'results'" class="section">
          <h2>Analysis Results</h2>
          <p class="section-description">Review zombie code analysis results and explore dependencies.</p>

          <div class="placeholder-content">
            <p>Results viewer will be implemented in User Story 3.</p>
          </div>

          <!-- ResultsViewer
            :results="analysisResults"
            @results-exported="handleResultsExported"
          /> -->
        </section>

        <!-- Settings Section -->
        <section v-show="currentSection === 'settings'" class="section">
          <h2>Settings</h2>
          <p class="section-description">Configure application preferences and analysis options.</p>

          <SettingsPanel
            :settings="appSettings"
            @settings-updated="handleSettingsUpdated"
          />
        </section>
      </main>
    </div>
  </div>
</template>

<script>
import { ref, reactive, onMounted } from 'vue'
import RepositoryConfig from './components/repository/RepositoryConfig.vue'
import RepositoryList from './components/repository/RepositoryList.vue'
import AnalysisPanel from './components/analysis/AnalysisPanel.vue'
// import ResultsViewer from './components/results/ResultsViewer.vue' // To be implemented in US3
// import SettingsPanel from './components/settings/SettingsPanel.vue' // Already exists

export default {
  name: 'App',
  components: {
    RepositoryConfig,
    RepositoryList,
    AnalysisPanel,
    // ResultsViewer, // To be implemented in US3
    SettingsPanel
  },
  setup() {
    // Application state
    const currentSection = ref('repositories')
    const repositories = ref([])
    const analysisResults = ref(null)
    const appSettings = ref({})
    const alerts = ref([])
    let alertIdCounter = 0

    // Navigation sections
    const sections = ref([
      { id: 'repositories', name: 'Repositories', icon: '📁' },
      { id: 'analysis', name: 'Analysis', icon: '🔍' },
      { id: 'results', name: 'Results', icon: '📊' },
      { id: 'settings', name: 'Settings', icon: '⚙️' }
    ])

    // Initialize app
    onMounted(() => {
      initializeApp()
      loadInitialData()
    })

    // Initialize Tauri API
    const initializeApp = async () => {
      try {
        // Check if Tauri is available
        if (window.__TAURI__) {
          console.log('Tauri API initialized successfully')
          showAlert('Application initialized successfully', 'success')
        } else {
          console.warn('Tauri API not available - running in development mode')
          showAlert('Running in development mode', 'info')
        }
      } catch (error) {
        console.error('Failed to initialize app:', error)
        showAlert('Failed to initialize application', 'error')
      }
    }

    // Load initial data
    const loadInitialData = async () => {
      try {
        // Load repositories
        await loadRepositories()

        // Load settings
        await loadSettings()

        // Load any existing analysis results
        await loadAnalysisResults()
      } catch (error) {
        console.error('Failed to load initial data:', error)
        showAlert('Failed to load initial data', 'error')
      }
    }

    // Navigation
    const switchSection = (sectionId) => {
      currentSection.value = sectionId

      // Load section-specific data if needed
      switch (sectionId) {
        case 'analysis':
          // Ensure repositories are loaded for analysis
          if (repositories.value.length === 0) {
            loadRepositories()
          }
          break
        case 'results':
          // Load latest results if not already loaded
          if (!analysisResults.value) {
            loadAnalysisResults()
          }
          break
      }
    }

    // Load repositories
    const loadRepositories = async () => {
      try {
        if (window.__TAURI__) {
          const { invoke } = window.__TAURI__.tauri
          repositories.value = await invoke('list_repositories')
        } else {
          // Mock data for development
          repositories.value = [
            {
              id: '1',
              name: 'example-repo',
              url: 'https://github.com/example/repo.git',
              branch: 'main',
              local_path: './cache/example-repo',
              sync_status: 'completed',
              last_sync: new Date().toISOString()
            }
          ]
        }
      } catch (error) {
        console.error('Failed to load repositories:', error)
        showAlert('Failed to load repositories', 'error')
      }
    }

    // Load settings
    const loadSettings = async () => {
      try {
        if (window.__TAURI__) {
          const { invoke } = window.__TAURI__.tauri
          appSettings.value = await invoke('get_settings')
        } else {
          // Default settings for development
          appSettings.value = {
            theme: 'light',
            autoSave: true,
            maxAnalysisDepth: 10,
            incrementalAnalysis: true
          }
        }
      } catch (error) {
        console.error('Failed to load settings:', error)
        showAlert('Failed to load settings', 'error')
      }
    }

    // Load analysis results
    const loadAnalysisResults = async () => {
      try {
        if (window.__TAURI__) {
          const { invoke } = window.__TAURI__.tauri
          const analyses = await invoke('list_analyses', { repository_ids: [] })

          if (analyses.length > 0) {
            const latestAnalysis = analyses
              .filter(a => a.status === 'completed')
              .sort((a, b) => new Date(b.completed_at) - new Date(a.completed_at))[0]

            if (latestAnalysis) {
              analysisResults.value = await invoke('get_analysis_results', { id: latestAnalysis.id })
            }
          }
        }
      } catch (error) {
        console.error('Failed to load analysis results:', error)
        // Don't show alert for this - results might not exist yet
      }
    }

    // Alert management
    const showAlert = (message, type = 'info', duration = 5000) => {
      const id = ++alertIdCounter
      const alert = {
        id,
        message,
        type,
        icon: getAlertIcon(type)
      }

      alerts.value.push(alert)

      // Auto-dismiss after duration
      if (duration > 0) {
        setTimeout(() => {
          dismissAlert(id)
        }, duration)
      }
    }

    const dismissAlert = (id) => {
      const index = alerts.value.findIndex(a => a.id === id)
      if (index > -1) {
        alerts.value.splice(index, 1)
      }
    }

    const getAlertIcon = (type) => {
      const icons = {
        success: '✅',
        error: '❌',
        warning: '⚠️',
        info: 'ℹ️'
      }
      return icons[type] || 'ℹ️'
    }

    // Event handlers
    const handleRepositoryAdded = (repository) => {
      showAlert(`Repository "${repository.name}" added successfully`, 'success')
      loadRepositories()
    }

    const handleRepositoryValidated = (result) => {
      if (result.accessible) {
        showAlert('Repository validation successful', 'success')
      } else {
        showAlert(`Repository validation failed: ${result.error}`, 'error')
      }
    }

    const handleRepositoryRemoved = (repository) => {
      showAlert(`Repository "${repository.name}" removed`, 'info')
      loadRepositories()
    }

    const handleRepositoryUpdated = (repository) => {
      showAlert(`Repository "${repository.name}" updated`, 'success')
      loadRepositories()
    }

    const handleRepositorySynced = (repository) => {
      showAlert(`Repository "${repository.name}" synchronized`, 'success')
    }

    const handleAnalysisStarted = (analysisId) => {
      showAlert('Analysis started', 'info')
      // Switch to analysis section to show progress
      currentSection.value = 'analysis'
    }

    const handleAnalysisCompleted = (results) => {
      showAlert('Analysis completed successfully', 'success')
      analysisResults.value = results
      // Switch to results section
      currentSection.value = 'results'
    }

    const handleResultsExported = (format) => {
      showAlert(`Results exported as ${format.toUpperCase()}`, 'success')
    }

    const handleSettingsUpdated = (settings) => {
      showAlert('Settings updated', 'success')
      appSettings.value = settings
    }

    return {
      // State
      currentSection,
      sections,
      repositories,
      analysisResults,
      appSettings,
      alerts,
      // Methods
      switchSection,
      loadRepositories,
      loadSettings,
      loadAnalysisResults,
      showAlert,
      dismissAlert,
      getAlertIcon,
      // Event handlers
      handleRepositoryAdded,
      handleRepositoryValidated,
      handleRepositoryRemoved,
      handleRepositoryUpdated,
      handleRepositorySynced,
      handleAnalysisStarted,
      handleAnalysisCompleted,
      handleResultsExported,
      handleSettingsUpdated
    }
  }
}
</script>

<style>
/* Global styles for the Vue app */
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  background-color: #f8f9fa;
  color: #333;
  line-height: 1.6;
}

#app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

/* Header styles */
.app-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 2rem 0;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

.app-header h1 {
  text-align: center;
  font-size: 2.5rem;
  font-weight: 300;
  margin: 0;
}

.subtitle {
  text-align: center;
  margin-top: 0.5rem;
  opacity: 0.9;
  font-size: 1.1rem;
}

/* Main content layout */
.main-content {
  flex: 1;
  display: flex;
  max-width: 1400px;
  margin: 0 auto;
  width: 100%;
  background: white;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
}

/* Sidebar navigation */
.sidebar {
  width: 250px;
  background: #f8f9fa;
  border-right: 1px solid #dee2e6;
  padding: 2rem 0;
}

.nav-menu {
  height: 100%;
}

.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.nav-list li {
  margin: 0;
}

.nav-btn {
  width: 100%;
  padding: 1rem 1.5rem;
  border: none;
  background: none;
  color: #6c757d;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  text-align: left;
}

.nav-btn:hover {
  background: #e9ecef;
  color: #495057;
}

.nav-btn.active {
  background: #667eea;
  color: white;
  box-shadow: 0 2px 4px rgba(102, 126, 234, 0.3);
}

.nav-icon {
  font-size: 1.2rem;
  width: 24px;
  text-align: center;
}

/* Content area */
.content-area {
  flex: 1;
  padding: 2rem;
  overflow-y: auto;
}

.section {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.section h2 {
  color: #495057;
  margin-bottom: 1.5rem;
  font-size: 1.75rem;
  font-weight: 600;
}

.section-description {
  color: #6c757d;
  margin-bottom: 2rem;
  font-size: 1.1rem;
}

/* Alert system */
.alert-container {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 1000;
  max-width: 400px;
}

.alert {
  background: white;
  border-radius: 8px;
  padding: 1rem 1.5rem;
  margin-bottom: 0.5rem;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: center;
  gap: 0.75rem;
  animation: slideInRight 0.3s ease;
}

@keyframes slideInRight {
  from {
    opacity: 0;
    transform: translateX(100%);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.alert-success {
  border-left: 4px solid #28a745;
}

.alert-error {
  border-left: 4px solid #dc3545;
}

.alert-warning {
  border-left: 4px solid #ffc107;
}

.alert-info {
  border-left: 4px solid #17a2b8;
}

.alert-icon {
  font-size: 1.2rem;
  flex-shrink: 0;
}

.alert-message {
  flex: 1;
  font-size: 0.95rem;
}

.alert-close {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  color: #6c757d;
  padding: 0;
  margin-left: 0.5rem;
  line-height: 1;
}

.alert-close:hover {
  color: #495057;
}

/* Responsive design */
@media (max-width: 768px) {
  .main-content {
    flex-direction: column;
  }

  .sidebar {
    width: 100%;
    order: 2;
  }

  .content-area {
    order: 1;
  }

  .nav-btn {
    padding: 0.75rem 1rem;
    font-size: 0.9rem;
  }

  .alert-container {
    left: 20px;
    right: 20px;
    max-width: none;
  }
}

@media (max-width: 480px) {
  .app-header h1 {
    font-size: 2rem;
  }

  .subtitle {
    font-size: 1rem;
  }

  .content-area {
    padding: 1rem;
  }
}

.placeholder-content {
  background: #f8f9fa;
  border: 2px dashed #dee2e6;
  border-radius: 8px;
  padding: 3rem;
  text-align: center;
  color: #6c757d;
}
</style>