<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import WelcomeStep from './components/WelcomeStep.vue'
import InfoStep from './components/InfoStep.vue'
import PathStep from './components/PathStep.vue'
import InstallingStep from './components/InstallingStep.vue'
import CompleteStep from './components/CompleteStep.vue'
import ErrorStep from './components/ErrorStep.vue'

import { getCurrentWindow } from '@tauri-apps/api/window'

const currentStep = ref('welcome')
const packageInfo = ref<any>(null)
const packagePath = ref('')
const installPath = ref('')
const error = ref('')
const progress = ref({ current: 0, total: 0, status: '' })
const pathStepRef = ref<any>(null)
const shouldLaunchNow = ref(false)

// Get package path from CLI args (passed by Tauri)
onMounted(async () => {
  // Check for launch arguments (e.g. double clicked file)
  try {
    const launchArgs = await invoke('get_launch_args') as string | null
    if (launchArgs) {
      console.log('Launch args detected:', launchArgs)
      await setPackage(launchArgs)
    }
  } catch (e) {
    console.error('Failed to get launch args:', e)
  }
})

const handleNext = async () => {
  if (currentStep.value === 'welcome') {
    currentStep.value = 'info'
  } else if (currentStep.value === 'info') {
    currentStep.value = 'path'
  } else if (currentStep.value === 'path') {
    if (pathStepRef.value) {
      installPath.value = pathStepRef.value.path
    }
    startInstallation()
  }
}

const handleBack = () => {
  if (currentStep.value === 'info') currentStep.value = 'welcome'
  else if (currentStep.value === 'path') currentStep.value = 'info'
  else if (currentStep.value === 'error') currentStep.value = 'welcome'
}

const startInstallation = async () => {
  currentStep.value = 'installing'
  try {
    await invoke('install_package', {
      path: packagePath.value,
      installPath: installPath.value,
      startService: true,
      scope: packageInfo.value?.install_scope?.toLowerCase() || 'user'
    })
    currentStep.value = 'complete'
  } catch (e: any) {
    error.value = e.toString()
    currentStep.value = 'error'
  }
}

const setPackage = async (path: string) => {
  try {
    packagePath.value = path
    const info = await invoke('validate_package', { path }) as any
    packageInfo.value = info
    currentStep.value = 'info'
  } catch (e: any) {
    error.value = e.toString()
    currentStep.value = 'error'
  }
}

const handleCancel = async () => {
  if (currentStep.value === 'complete' && shouldLaunchNow.value && packageInfo.value) {
    try {
      const launchCmd = packageInfo.value.launch_command || packageInfo.value.entry
      if (launchCmd) {
        await invoke('launch_app', { 
          command: launchCmd, 
          installPath: installPath.value 
        })
      }
    } catch (e) {
      console.error('Failed to auto-launch:', e)
    }
  }
  await invoke('exit_app')
}

const logs = ref<string[]>([])

// Installation has multiple phases, assign progress percentage to each
// Total phases: extracting -> copying -> permissions -> script -> service -> desktop -> complete
// Assign percentages: extracting(0-30%), copying(30-60%), other phases(60-95%), complete(100%)
listen('install-progress-extracting', (event: any) => {
  // Extraction: 0% to 30% based on files extracted
  const extractPercent = event.payload.total > 0 
    ? Math.round((event.payload.current / event.payload.total) * 30) 
    : 0
  progress.value = { current: extractPercent, total: 100, status: 'Extracting files...' }
})
listen('install-progress-copying', (event: any) => {
  // Copying: 30% to 60% based on files copied
  const copyPercent = event.payload.total > 0 
    ? 30 + Math.round((event.payload.current / event.payload.total) * 30)
    : 30
  progress.value = { current: copyPercent, total: 100, status: 'Copying files...' }
})
listen('install-progress-permissions', () => {
  progress.value = { current: 65, total: 100, status: 'Setting permissions...' }
})
listen('install-progress-script', () => {
  progress.value = { current: 75, total: 100, status: 'Running post-install script...' }
})
listen('install-progress-service', () => {
  progress.value = { current: 85, total: 100, status: 'Registering system service...' }
})
listen('install-progress-desktop', () => {
  progress.value = { current: 92, total: 100, status: 'Creating desktop entry...' }
})
listen('install-log', (event: any) => {
  logs.value.push(event.payload.message)
})
listen('install-progress-completed', () => {
  progress.value = { current: 100, total: 100, status: 'Installation complete!' }
  currentStep.value = 'complete'
})
</script>

<template>
  <div class="wizard-container">
    <aside class="wizard-sidebar">
      <div class="sidebar-logo">📦</div>
      <div class="sidebar-text">
        <h3>INT Engine</h3>
        <p>Setup Wizard</p>
      </div>
    </aside>

    <div class="wizard-main-wrapper">
      <main class="wizard-content animate-fade-in">
        <WelcomeStep 
          v-if="currentStep === 'welcome'" 
          @next="handleNext" 
          @select-package="setPackage" 
        />
        
        <InfoStep 
          v-if="currentStep === 'info'" 
          :info="packageInfo" 
          @next="handleNext" 
          @back="handleBack" 
        />
        
        <PathStep 
          ref="pathStepRef"
          v-if="currentStep === 'path'" 
          :defaultPath="packageInfo?.install_path"
          @next="handleNext" 
          @back="handleBack" 
        />
        
        <InstallingStep 
          v-if="currentStep === 'installing'" 
          :progress="progress" 
          :logs="logs"
        />
        
        <CompleteStep 
          v-if="currentStep === 'complete'" 
          :info="packageInfo" 
          v-model:shouldLaunch="shouldLaunchNow"
        />
        
        <ErrorStep 
          v-if="currentStep === 'error'" 
          :message="error" 
          @back="handleBack" 
        />
      </main>

      <footer class="wizard-footer">
        <button 
          class="btn" 
          v-if="['info', 'path', 'error'].includes(currentStep)"
          @click="handleBack"
        >
          < Back
        </button>
        <button 
          class="btn" 
          v-if="currentStep === 'welcome'"
          disabled
        >
          < Back
        </button>

        <button 
          class="btn btn-primary" 
          v-if="['welcome', 'info', 'path'].includes(currentStep)"
          :disabled="currentStep === 'welcome' && !packagePath"
          @click="handleNext"
        >
          Next >
        </button>
        <button 
          class="btn btn-primary" 
          v-if="currentStep === 'complete'"
          @click="handleCancel"
        >
          Finish
        </button>

        <button class="btn" @click="handleCancel">Cancel</button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.wizard-main-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: white;
}

.sidebar-logo {
  font-size: 2.5rem;
  margin-bottom: 0.75rem;
}

.sidebar-text h3 {
  font-size: 0.9rem;
  font-weight: 600;
  margin-bottom: 0.2rem;
}

.sidebar-text p {
  font-size: 0.7rem;
  opacity: 0.7;
}

.wizard-content {
  flex: 1;
  overflow-y: auto;
  padding: 2.5rem;
}

.wizard-footer {
  height: 60px;
  background-color: #f0f0f0;
  border-top: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 0 1rem;
  gap: 0.5rem;
}
</style>
