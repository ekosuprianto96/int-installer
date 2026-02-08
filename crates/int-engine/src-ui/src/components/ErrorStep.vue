<script setup lang="ts">
defineProps({
  message: String
})

defineEmits(['back'])
</script>

<template>
  <div class="step-container animate-fade-in">
    <h2 class="error-title">Installation Error</h2>
    <p>Setup encountered an error and cannot continue with the installation.</p>
    
    <div class="error-box">
      <strong>Error Detail:</strong>
      <p>{{ message }}</p>
    </div>
    
    <p>Click Back to try again, or Cancel to exit Setup.</p>
  </div>
</template>

<style scoped>
.step-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.error-title {
  color: #cc0000 !important;
}

.error-box {
  background: #fffafa;
  border: 1px solid #ffcccc;
  padding: 1rem;
  margin-bottom: 1rem;
  flex: 1;
  overflow-y: auto;
  font-size: 0.875rem;
  color: #cc0000;
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
