<script setup lang="ts">
const props = defineProps({
  info: Object
})

defineEmits(['next', 'back'])
</script>

<template>
  <div class="step-container animate-fade-in">
    <h2>Information</h2>
    <p>Please read the following important information before continuing with Setup.</p>
    
    <div class="info-content-box">
      <div class="item"><strong>Package:</strong> {{ info?.display_name || info?.name }}</div>
      <div class="item"><strong>Version:</strong> {{ info?.version }}</div>
      <div class="item"><strong>Author:</strong> {{ info?.author }}</div>
      <div class="item"><strong>License:</strong> {{ info?.license }}</div>
      <hr />
      <div class="description-box">
        {{ info?.description || 'No additional information provided.' }}
      </div>
    </div>
    
    <p class="footer-text">When you are ready to continue with Setup, click Next.</p>
  </div>
</template>

<style scoped>
.step-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding-bottom: 0.5rem;
}

.info-content-box {
  flex: 1;
  background: white;
  border: 1px solid #ccc;
  padding: 1rem;
  margin-bottom: 1rem;
  overflow-y: auto;
  font-size: 0.875rem;
  min-height: 150px;
}

.item {
  margin-bottom: 0.5rem;
}

hr {
  border: 0;
  border-top: 1px solid #eee;
  margin: 1rem 0;
}

.description-box {
  white-space: pre-wrap;
  color: #333;
}

.footer-text {
  margin-top: auto;
}

h2 {
  color: #003399;
  font-weight: normal;
  margin-bottom: 0.5rem;
}

p {
  margin-bottom: 0.75rem;
}
</style>
