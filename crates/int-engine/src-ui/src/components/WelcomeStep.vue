<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'

const emit = defineEmits(['next', 'select-package'])

const handleBrowse = async () => {
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'INT Package',
                extensions: ['int', 'gz']
            }]
        })
        
        if (selected) {
            emit('select-package', selected as string)
        }
    } catch (err) {
        console.error('Failed to open dialog:', err)
    }
}
</script>

<template>
  <div class="step-container animate-fade-in">
    <h2>Welcome to the INT Engine Setup Wizard</h2>
    <p>This wizard will guide you through the installation of your INT package.</p>
    <p>It is recommended that you close all other applications before starting Setup. This will make it possible to update relevant system files without having to reboot your computer.</p>
    <p>Click Next to continue, or Cancel to exit Setup.</p>
    
    <div class="browse-section">
      <button class="btn" @click="handleBrowse">Browse Package...</button>
    </div>
  </div>
</template>

<style scoped>
.step-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.browse-section {
  margin-top: 0rem;
  margin-bottom: 2rem;
}

h2 {
  color: #003399;
  font-weight: normal;
  margin-bottom: 1rem;
}

p {
  margin-bottom: 1rem;
}
</style>
