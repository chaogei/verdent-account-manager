<template>
  <div v-if="show" class="modal-overlay" @click.self="handleCancel">
    <div class="update-dialog">
      <div class="dialog-header">
        <h2>{{ title }}</h2>
        <button class="close-btn" @click="handleCancel">&times;</button>
      </div>
      
      <div class="dialog-body">
        <div v-if="loading" class="loading-container">
          <div class="spinner"></div>
          <p>正在检查更新...</p>
        </div>
        
        <div v-else-if="error" class="error-container">
          <p class="error-text">{{ error }}</p>
        </div>
        
        <div v-else-if="updateInfo && updateInfo.has_update" class="update-info">
          <div class="version-info">
            <div class="version-item">
              <span class="label">当前版本：</span>
              <span class="value">{{ updateInfo.current_version }}</span>
            </div>
            <div class="version-item">
              <span class="label">最新版本：</span>
              <span class="value new-version">{{ updateInfo.latest_version }}</span>
            </div>
            <div v-if="updateInfo.published_at" class="version-item">
              <span class="label">发布时间：</span>
              <span class="value">{{ formatDate(updateInfo.published_at) }}</span>
            </div>
          </div>
          
          <div v-if="updateInfo.release_notes" class="release-notes">
            <h3>更新内容：</h3>
            <div class="notes-content" v-html="formatReleaseNotes(updateInfo.release_notes)"></div>
          </div>
        </div>
        
        <div v-else class="no-update">
          <p>🎉 当前已是最新版本！</p>
        </div>
      </div>
      
      <div class="dialog-footer">
        <button 
          v-if="updateInfo && updateInfo.has_update" 
          class="btn btn-primary" 
          @click="handleUpdate"
        >
          立即更新
        </button>
        <button 
          v-if="updateInfo && updateInfo.has_update && !isManualCheck" 
          class="btn btn-secondary" 
          @click="handleSkip"
        >
          跳过此版本
        </button>
        <button class="btn btn-cancel" @click="handleCancel">
          {{ updateInfo && updateInfo.has_update ? '稍后提醒' : '关闭' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { open as shellOpen } from '@tauri-apps/plugin-shell'

interface UpdateInfo {
  has_update: boolean
  current_version: string
  latest_version?: string
  release_name?: string
  release_notes?: string
  download_url?: string
  published_at?: string
}

interface Props {
  show: boolean
  updateInfo: UpdateInfo | null
  loading?: boolean
  error?: string
  isManualCheck?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  error: '',
  isManualCheck: false
})

const emit = defineEmits<{
  close: []
  update: []
  skip: []
}>()

const title = ref('软件更新')

watch(() => props.updateInfo, (newVal) => {
  if (newVal && newVal.has_update) {
    title.value = '发现新版本'
  } else if (newVal && !newVal.has_update) {
    title.value = '已是最新版本'
  }
})

const handleUpdate = () => {
  if (props.updateInfo?.download_url) {
    shellOpen(props.updateInfo.download_url)
  }
  emit('update')
}

const handleSkip = () => {
  emit('skip')
}

const handleCancel = () => {
  emit('close')
}

const formatDate = (dateStr: string): string => {
  try {
    const date = new Date(dateStr)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return dateStr
  }
}

const formatReleaseNotes = (notes: string): string => {
  return notes
    .replace(/^### /gm, '<h4>')
    .replace(/^## /gm, '<h3>')
    .replace(/^# /gm, '<h2>')
    .replace(/\n/g, '<br>')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/`(.+?)`/g, '<code>$1</code>')
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(4px);
}

.update-dialog {
  background: white;
  border-radius: 12px;
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(-20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.dialog-header {
  padding: 20px 24px;
  border-bottom: 1px solid #e5e7eb;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.dialog-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
}

.close-btn {
  background: none;
  border: none;
  font-size: 28px;
  color: #9ca3af;
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #f3f4f6;
  color: #1f2937;
}

.dialog-body {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}

.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 0;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f4f6;
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-container p {
  margin-top: 16px;
  color: #6b7280;
}

.error-container {
  padding: 20px;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 8px;
}

.error-text {
  color: #dc2626;
  margin: 0;
}

.update-info {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.version-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: #f9fafb;
  border-radius: 8px;
}

.version-item {
  display: flex;
  align-items: center;
}

.version-item .label {
  font-weight: 500;
  color: #6b7280;
  width: 90px;
}

.version-item .value {
  color: #1f2937;
}

.version-item .new-version {
  color: #3b82f6;
  font-weight: 600;
}

.release-notes {
  padding: 16px;
  background: #f9fafb;
  border-radius: 8px;
}

.release-notes h3 {
  margin: 0 0 12px 0;
  font-size: 15px;
  font-weight: 600;
  color: #1f2937;
}

.notes-content {
  color: #4b5563;
  line-height: 1.6;
  font-size: 14px;
}

.notes-content :deep(h2),
.notes-content :deep(h3),
.notes-content :deep(h4) {
  margin: 12px 0 8px 0;
  font-weight: 600;
}

.notes-content :deep(code) {
  background: #e5e7eb;
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
  font-size: 13px;
}

.no-update {
  text-align: center;
  padding: 40px 0;
}

.no-update p {
  font-size: 16px;
  color: #6b7280;
  margin: 0;
}

.dialog-footer {
  padding: 16px 24px;
  border-top: 1px solid #e5e7eb;
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn-primary {
  background: #3b82f6;
  color: white;
}

.btn-primary:hover {
  background: #2563eb;
}

.btn-secondary {
  background: #f3f4f6;
  color: #6b7280;
}

.btn-secondary:hover {
  background: #e5e7eb;
  color: #1f2937;
}

.btn-cancel {
  background: white;
  color: #6b7280;
  border: 1px solid #e5e7eb;
}

.btn-cancel:hover {
  background: #f9fafb;
  color: #1f2937;
}
</style>
