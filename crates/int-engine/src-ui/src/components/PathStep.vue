<script setup lang="ts">
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'

const props = defineProps({
  defaultPath: String
})

const emit = defineEmits(['next', 'back'])
const path = ref(props.defaultPath || '')

const handleBrowse = async () => {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath: path.value || undefined
        })
        
        if (selected) {
            path.value = selected as string
        }
    } catch (err) {
        console.error('Failed to open directory dialog:', err)
    }
}

// Expose path for parent (App.vue)
defineExpose({ path })
</script>

<template>
  <div class="step-container animate-fade-in">
    <h2>Select Destination Location</h2>
    <p>Where should the application be installed?</p>
    <p>Setup will install the application into the following folder.</p>
    <p>To continue, click Next. If you would like to select a different folder, click Browse.</p>
    
    <div class="path-box">
      <input type="text" v-model="path" />
      <button class="btn" @click="handleBrowse">Browse...</button>
    </div>
  </div>
</template>

<style scoped>
.step-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.path-box {
  background: white;
  border: 1px solid #ccc;
  padding: 1.5rem;
  display: flex;
  gap: 0.5rem;
  align-items: center;
  margin-top: 1rem;
}

input {
  flex: 1;
  padding: 0.4rem;
  border: 1px solid #ccc;
  font-family: inherit;
}

h2 {
  color: #003399;
  font-weight: normal;
  margin-bottom: 1rem;
}

p {
  margin-bottom: 0.5rem;
}
</style>
