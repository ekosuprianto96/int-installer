<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps({
  info: Object
})

const emit = defineEmits(['update:shouldLaunch'])

const shouldLaunch = ref(props.info?.auto_launch || false)

// Pre-sync once on mount
watch(shouldLaunch, (val) => {
  emit('update:shouldLaunch', val)
}, { immediate: true })
</script>

<template>
  <div class="step-container animate-fade-in">
    <h2>Completing the INT Engine Setup Wizard</h2>
    <p>Setup has finished installing <strong class="text-primary">{{ info?.display_name || info?.name }}</strong> on your computer.</p>
    <p>The application may be launched by selecting the installed shortcuts.</p>
    <p>Click Finish to exit Setup.</p>

    <div v-if="info?.auto_launch" class="launch-option">
      <label class="checkbox-container">
        <input type="checkbox" v-model="shouldLaunch" />
        Launch {{ info?.display_name || info?.name }} now
      </label>
    </div>
  </div>
</template>

<style scoped>
.step-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

h2 {
  color: #003399;
  font-weight: normal;
  margin-bottom: 1.5rem;
  font-size: 1.25rem;
}

p {
  margin-bottom: 1rem;
  line-height: 1.5;
}

.text-primary {
  color: #003399;
}

.launch-option {
  margin-top: 0rem;
  padding-top: 1.5rem;
  border-top: 1px solid #e0e0e0;
}

.checkbox-container {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  cursor: pointer;
  font-size: 0.9rem;
  user-select: none;
}

.checkbox-container input {
  width: 1rem;
  height: 1rem;
  cursor: pointer;
}
</style>
