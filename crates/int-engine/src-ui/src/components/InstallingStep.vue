<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue'

const props = defineProps({
  progress: Object,
  logs: Array as () => string[]
})

const logContainer = ref<HTMLElement | null>(null)

const percent = computed(() => {
  if (!props.progress?.total) return 0
  return Math.round((props.progress.current / props.progress.total) * 100)
})

watch(() => props.logs?.length, async () => {
  await nextTick()
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
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

      <div class="log-viewer" ref="logContainer">
        <div v-for="(log, index) in logs" :key="index" class="log-line">
          {{ log }}
        </div>
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
  margin-top: 1.5rem;
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0; /* Important for flex child overflow */
}

.status-detail {
  font-size: 0.875rem;
  margin-bottom: 0.5rem;
  color: #333;
}

.progress-bar-container {
  height: 22px;
  background: linear-gradient(to bottom, #e0e0e0, #f5f5f5);
  border: 1px solid #999;
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 1rem;
  flex-shrink: 0;
  box-shadow: inset 0 1px 2px rgba(0,0,0,0.1);
}

.progress-fill {
  height: 100%;
  background: #03e307e0;
  transition: width 0.2s linear;
}

.log-viewer {
  flex: 1;
  min-height: 180px;
  background: #1e1e1e;
  border: 1px solid #333;
  border-radius: 4px;
  padding: 0.75rem;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 0.8rem;
  color: #d4d4d4;
  overflow-y: auto;
  line-height: 1.5;
}

.log-line {
  white-space: pre-wrap;
  word-break: break-all;
  margin-bottom: 3px;
  padding: 2px 0;
}

.log-line:nth-child(odd) {
  background: rgba(255,255,255,0.02);
}

h2 {
  color: #003399;
  font-weight: normal;
  margin-bottom: 1rem;
}
</style>
