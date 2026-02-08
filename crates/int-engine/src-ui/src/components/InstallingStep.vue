<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  progress: Object
})

const percent = computed(() => {
  if (!props.progress?.total) return 0
  return Math.round((props.progress.current / props.progress.total) * 100)
})
</script>

<template>
  <div class="step-container animate-fade-in">
    <h2>Installing</h2>
    <p>Please wait while Setup installs the application on your computer.</p>
    
    <div class="install-box">
      <p class="status-detail">{{ progress?.status || 'Extracting files...' }}</p>
      <div class="progress-bar-container">
        <div class="progress-fill" :style="{ width: percent + '%' }"></div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.step-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.install-box {
  margin-top: 2rem;
}

.status-detail {
  font-size: 0.875rem;
  margin-bottom: 0.5rem;
}

.progress-bar-container {
  height: 18px;
  background: #eee;
  border: 1px solid #999;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--primary);
  transition: width 0.2s linear;
}

h2 {
  color: #003399;
  font-weight: normal;
  margin-bottom: 1rem;
}
</style>
