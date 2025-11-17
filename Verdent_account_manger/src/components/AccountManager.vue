<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AccountCard from './AccountCard.vue'
import Toast from './Toast.vue'
import ConfirmDialog from './ConfirmDialog.vue'

interface Account {
  id: string
  email: string
  password: string
  register_time: string
  expire_time?: string      // 订阅到期时间 (从 currentPeriodEnd 转换)
  status: string
  token?: string
  quota_remaining?: string
  quota_used?: string
  quota_total?: string
  subscription_type?: string
  trial_days?: number
  current_period_end?: number  // Unix时间戳(秒)
  auto_renew?: boolean
  token_expire_time?: string  // 新增: Token 过期时间 (从 JWT exp 解析)
  last_updated?: string
}

interface AutoRegisterRequest {
  count: number
  password: string
  max_workers: number
  headless: boolean
  use_random_password: boolean
}

const accounts = ref<Account[]>([])
const currentAccountId = ref<string>('') // 当前使用的账号ID
const loading = ref(false)
const showRegisterDialog = ref(false)
const showImportDialog = ref(false)
const importMode = ref<'token' | 'account' | 'json'>('token')
const importTokens = ref('')
const importAccounts = ref('')
const importJson = ref('')
const showExportDialog = ref(false)
const exportFormat = ref<'json' | 'txt' | 'csv'>('json')

const editingAccount = ref<Account | null>(null)

// 批量操作相关
const selectedAccounts = ref<Set<string>>(new Set())
const showBatchActions = ref(false)

// 邮箱隐私模式
const emailPrivacyMode = ref(true)

// 筛选相关
const filterType = ref<string>('')  // 当前筛选类型：'', 'full', 'partial', 'empty', 'abnormal'

// 过滤后的账号列表
const filteredAccounts = computed(() => {
  if (!filterType.value) {
    return accounts.value
  }

  return accounts.value.filter(account => {
    const remaining = parseFloat(account.quota_remaining || '0')
    const used = parseFloat(account.quota_used || '0')
    const total = parseFloat(account.quota_total || '0')

    switch (filterType.value) {
      case 'full':
        return used === 0 && total > 0  // 满额度（未使用）
      case 'partial':
        return remaining > 0 && used > 0  // 已使用但还有剩余
      case 'empty':
        return total === 0  // 0额度
      case 'abnormal':
        return remaining === 0 && total > 0  // 额度用尽
      default:
        return true
    }
  })
})

// 设置筛选类型
async function setFilter(type: string) {
  // 如果点击同一个筛选项，则清除筛选
  filterType.value = filterType.value === type ? '' : type
  // 保存到持久化存储
  try {
    await invoke('update_filter_type', { filterType: filterType.value || null })
  } catch (error) {
    console.error('保存筛选类型失败:', error)
  }
}

// 获取筛选标签
function getFilterLabel(type: string): string {
  const labels: Record<string, string> = {
    full: '满额度账号',
    partial: '已消耗账号',
    empty: '0额度账号',
    abnormal: '异常账号（额度用尽）'
  }
  return labels[type] || '全部账号'
}

// 账号统计计算属性
const accountStats = computed(() => {
  const stats = {
    total: accounts.value.length,
    fullQuota: 0,        // 满额度（未使用）
    partialQuota: 0,     // 部分使用
    emptyQuota: 0,       // 0额度
    abnormal: 0,         // 异常账号（剩余为0但总额不为0）
    totalQuota: 0,       // 所有账号的总额度
    totalRemaining: 0,   // 所有账号的剩余额度
    totalUsed: 0         // 所有账号的已用额度
  }

  accounts.value.forEach(account => {
    // 解析额度数值
    const remaining = parseFloat(account.quota_remaining || '0')
    const used = parseFloat(account.quota_used || '0')
    const total = parseFloat(account.quota_total || '0')

    // 累计总额
    stats.totalQuota += total
    stats.totalRemaining += remaining
    stats.totalUsed += used

    // 分类统计
    if (total === 0) {
      stats.emptyQuota++
    } else if (remaining === 0 && total > 0) {
      stats.abnormal++  // 总额不为0但剩余为0，视为异常
    } else if (used === 0 && total > 0) {
      stats.fullQuota++  // 未使用过的满额度账号
    } else if (remaining > 0) {
      stats.partialQuota++  // 已使用但还有剩余
    }
  })

  return stats
})

// 注册配置
const registerConfig = ref({
  count: 1,
  password: 'VerdentAI@2024',
  max_workers: 2,
  headless: false,  // 默认关闭无头模式，显示浏览器窗口
  use_random_password: false
})

// Toast 通知状态
const toast = ref({
  show: false,
  message: '',
  type: 'success' as 'success' | 'error' | 'warning' | 'info'
})

// 确认对话框状态
const confirmDialog = ref({
  show: false,
  title: '',
  message: '',
  confirmText: '确认',
  cancelText: '取消',
  type: 'warning' as 'danger' | 'warning' | 'info',
  onConfirm: () => {}
})

// Toast 辅助函数
function showToast(message: string, type: 'success' | 'error' | 'warning' | 'info' = 'success') {
  toast.value = { show: true, message, type }
}

function closeToast() {
  toast.value.show = false
}

// 手动关闭加载状态
function handleCloseLoading() {
  loading.value = false
}

// 确认对话框辅助函数
function showConfirm(
  message: string,
  onConfirm: () => void,
  options?: {
    title?: string
    confirmText?: string
    cancelText?: string
    type?: 'danger' | 'warning' | 'info'
  }
) {
  confirmDialog.value = {
    show: true,
    message,
    title: options?.title || '确认操作',
    confirmText: options?.confirmText || '确认',
    cancelText: options?.cancelText || '取消',
    type: options?.type || 'warning',
    onConfirm
  }
}

function handleConfirm() {
  confirmDialog.value.onConfirm()
  confirmDialog.value.show = false
}

function handleCancel() {
  confirmDialog.value.show = false
}

// 邮箱隐私处理函数
function maskEmail(email: string): string {
  if (!emailPrivacyMode.value) return email
  
  const parts = email.split('@')
  if (parts.length !== 2) return email
  
  const [localPart, domain] = parts
  const visibleLength = Math.min(3, Math.floor(localPart.length / 3))
  
  if (localPart.length <= 6) {
    return `${localPart.substring(0, 2)}***@${domain.substring(0, 1)}***`
  }
  
  // 例如: lucas_brown162179 -> lucas***
  const visiblePart = localPart.substring(0, visibleLength)
  return `${visiblePart}***@${domain.substring(0, 1)}***`
}

// 邮箱隐私开关
async function toggleEmailPrivacy() {
  emailPrivacyMode.value = !emailPrivacyMode.value
  // 保存到持久化存储
  try {
    await invoke('update_email_privacy', { privacyMode: emailPrivacyMode.value })
  } catch (error) {
    console.error('保存隐私模式失败:', error)
  }
  showToast(emailPrivacyMode.value ? '已开启邮箱隐私模式' : '已关闭邮箱隐私模式', 'info')
}

// 选择/取消选择账户
async function toggleAccountSelection(accountId: string) {
  if (selectedAccounts.value.has(accountId)) {
    selectedAccounts.value.delete(accountId)
  } else {
    selectedAccounts.value.add(accountId)
  }
  // 保存到持久化存储
  try {
    await invoke('update_selected_accounts', { selected: Array.from(selectedAccounts.value) })
  } catch (error) {
    console.error('保存选中账户失败:', error)
  }
}

// 设置当前使用的账号（点击登录按钮时自动调用）
async function setCurrentAccount(accountId: string) {
  // 直接设置为当前账号，不再是切换逻辑
  currentAccountId.value = accountId
  const account = accounts.value.find(a => a.id === accountId)
  if (account) {
    showToast(`当前使用账号：${account.email}`, 'info')
  }
  // 保存到持久化存储
  try {
    await invoke('update_current_account', { accountId })
  } catch (error) {
    console.error('保存当前账号失败:', error)
  }
}

// 全选/取消全选
async function toggleSelectAll() {
  if (selectedAccounts.value.size === filteredAccounts.value.length) {
    selectedAccounts.value.clear()
  } else {
    filteredAccounts.value.forEach(account => {
      selectedAccounts.value.add(account.id)
    })
  }
  // 保存到持久化存储
  try {
    await invoke('update_selected_accounts', { selected: Array.from(selectedAccounts.value) })
  } catch (error) {
    console.error('保存选中账户失败:', error)
  }
}

// 批量刷新
async function handleBatchRefresh() {
  const ids = Array.from(selectedAccounts.value)
  if (ids.length === 0) {
    showToast('请先选择要刷新的账户', 'warning')
    return
  }

  showConfirm(
    `确定要刷新 ${ids.length} 个账户的信息吗？`,
    async () => {
      loading.value = true
      let successCount = 0
      let failCount = 0

      try {
        for (const id of ids) {
          try {
            const result = await invoke<{ success: boolean; error?: string }>('refresh_account_info', {
              accountId: id
            })
            if (result.success) {
              successCount++
            } else {
              failCount++
            }
          } catch {
            failCount++
          }
        }

        if (failCount === 0) {
          showToast(`成功刷新 ${successCount} 个账户`, 'success')
        } else {
          showToast(`刷新完成：成功 ${successCount} 个，失败 ${failCount} 个`, 'warning')
        }
        
        await loadAccounts()
        selectedAccounts.value.clear()
      } finally {
        loading.value = false
      }
    },
    { title: '批量刷新', confirmText: '刷新', type: 'info' }
  )
}

// 批量删除
async function handleBatchDelete() {
  const ids = Array.from(selectedAccounts.value)
  if (ids.length === 0) {
    showToast('请先选择要删除的账户', 'warning')
    return
  }

  showConfirm(
    `确定要删除 ${ids.length} 个账户吗？此操作不可恢复！`,
    async () => {
      loading.value = true
      let successCount = 0
      let failCount = 0

      try {
        for (const id of ids) {
          try {
            const result = await invoke<{ success: boolean; error?: string }>('delete_account', {
              accountId: id
            })
            if (result.success) {
              successCount++
            } else {
              failCount++
            }
          } catch {
            failCount++
          }
        }

        if (failCount === 0) {
          showToast(`成功删除 ${successCount} 个账户`, 'success')
        } else {
          showToast(`删除完成：成功 ${successCount} 个，失败 ${failCount} 个`, 'warning')
        }
        
        await loadAccounts()
        selectedAccounts.value.clear()
      } finally {
        loading.value = false
      }
    },
    { title: '批量删除', confirmText: '删除', type: 'error' }
  )
}

// 导出账户
async function handleExport() {
  const accountsToExport = selectedAccounts.value.size > 0
    ? accounts.value.filter(acc => selectedAccounts.value.has(acc.id))
    : accounts.value

  if (accountsToExport.length === 0) {
    showToast('没有可导出的账户', 'warning')
    return
  }

  let content = ''
  let filename = `verdent_accounts_${new Date().toISOString().split('T')[0]}`

  switch (exportFormat.value) {
    case 'json':
      content = JSON.stringify(accountsToExport, null, 2)
      filename += '.json'
      break
    case 'csv':
      const headers = ['email', 'password', 'token', 'quota_remaining', 'quota_total', 'subscription_type']
      const csvRows = [headers.join(',')]
      accountsToExport.forEach(acc => {
        const row = headers.map(h => {
          const value = acc[h as keyof Account] || ''
          return `"${value}"`
        })
        csvRows.push(row.join(','))
      })
      content = csvRows.join('\n')
      filename += '.csv'
      break
    case 'txt':
      const txtLines = accountsToExport.map(acc => {
        return `${acc.email}:${acc.password}${acc.token ? ':' + acc.token : ''}`
      })
      content = txtLines.join('\n')
      filename += '.txt'
      break
  }

  // 创建下载链接
  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.click()
  URL.revokeObjectURL(url)

  showToast(`成功导出 ${accountsToExport.length} 个账户`, 'success')
  showExportDialog.value = false
  selectedAccounts.value.clear()
}

// 处理JSON文件上传
function handleJsonFileUpload(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  
  if (!file) return
  
  const reader = new FileReader()
  reader.onload = (e) => {
    importJson.value = e.target?.result as string
    showToast('JSON文件已加载', 'info')
  }
  reader.onerror = () => {
    showToast('文件读取失败', 'error')
  }
  reader.readAsText(file)
}

// JSON导入处理
async function handleJsonImport() {
  const jsonText = importJson.value.trim()
  
  if (!jsonText) {
    showToast('请输入JSON内容', 'warning')
    return
  }

  try {
    const data = JSON.parse(jsonText)
    const items = Array.isArray(data) ? data : [data]
    
    const accountsToImport: Array<{ token?: string; email?: string; password?: string }> = []
    
    // 智能识别JSON中的账户信息
    items.forEach(item => {
      if (typeof item === 'object') {
        // 查找token
        const token = item.token || item.access_token || item.accessToken || item.jwt || null
        // 查找email
        const email = item.email || item.mail || item.username || item.user || null
        // 查找password
        const password = item.password || item.pass || item.pwd || null
        
        if (token || (email && password)) {
          accountsToImport.push({ token, email, password })
        }
      }
    })
    
    if (accountsToImport.length === 0) {
      showToast('未在JSON中找到有效的账户信息', 'warning')
      return
    }
    
    // 关闭对话框
    showImportDialog.value = false
    importJson.value = ''
    
    // 开始导入
    loading.value = true
    let successCount = 0
    let failCount = 0
    
    try {
      for (const account of accountsToImport) {
        if (account.token) {
          // Token导入
          try {
            const result = await invoke<{ success: boolean; error?: string }>('import_account_by_token', {
              token: account.token
            })
            if (result.success) {
              successCount++
            } else {
              failCount++
            }
          } catch {
            failCount++
          }
        } else if (account.email && account.password) {
          // 账号密码导入
          try {
            const result = await invoke<{ success: boolean; error?: string }>('import_account_by_credentials', {
              email: account.email,
              password: account.password
            })
            if (result.success) {
              successCount++
            } else {
              failCount++
            }
          } catch {
            failCount++
          }
        }
      }
      
      if (failCount === 0) {
        showToast(`成功导入 ${successCount} 个账户`, 'success')
      } else {
        showToast(`导入完成：成功 ${successCount} 个，失败 ${failCount} 个`, 'warning')
      }
      
      await loadAccounts()
    } finally {
      loading.value = false
    }
    
  } catch (error) {
    showToast('JSON格式错误：' + error, 'error')
  }
}

// 处理键盘快捷键
function handleKeydown(event: KeyboardEvent) {
  // ESC 键关闭加载覆盖层
  if (event.key === 'Escape' && loading.value) {
    handleCloseLoading()
  }
  
  // Ctrl+A 或 Cmd+A 全选
  if ((event.ctrlKey || event.metaKey) && event.key === 'a' && !event.shiftKey) {
    event.preventDefault()
    if (filteredAccounts.value.length > 0) {
      if (selectedAccounts.value.size === filteredAccounts.value.length) {
        selectedAccounts.value.clear()
      } else {
        filteredAccounts.value.forEach(account => {
          selectedAccounts.value.add(account.id)
        })
      }
    }
  }
}

onMounted(async () => {
  // 先加载用户设置
  await loadUserSettings()
  // 再加载账户
  await loadAccounts()
  // 添加键盘事件监听
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  // 保存设置
  saveUserSettings()
  // 清理键盘事件监听
  document.removeEventListener('keydown', handleKeydown)
})

async function loadAccounts() {
  try {
    loading.value = true
    accounts.value = await invoke<Account[]>('get_all_accounts')
  } catch (error) {
    console.error('加载账户失败:', error)
    showToast('加载账户失败: ' + error, 'error')
  } finally {
    loading.value = false
  }
}

// 加载用户设置
async function loadUserSettings() {
  try {
    const settings = await invoke<{
      current_account_id: string | null
      email_privacy_mode: boolean
      selected_accounts: string[]
      filter_type: string | null
      import_mode: string
      export_format: string
      register_count: number
      register_max_workers: number
      register_password: string
      register_use_random_password: boolean
      register_headless: boolean
    }>('get_user_settings')
    
    // 恢复设置
    if (settings.current_account_id) {
      currentAccountId.value = settings.current_account_id
    }
    emailPrivacyMode.value = settings.email_privacy_mode
    selectedAccounts.value = new Set(settings.selected_accounts)
    filterType.value = settings.filter_type || ''
    importMode.value = settings.import_mode as 'token' | 'account' | 'json'
    exportFormat.value = settings.export_format as 'json' | 'txt' | 'csv'
    
    // 恢复自动注册配置
    registerConfig.value.count = settings.register_count
    registerConfig.value.max_workers = settings.register_max_workers
    registerConfig.value.password = settings.register_password
    registerConfig.value.use_random_password = settings.register_use_random_password
    registerConfig.value.headless = settings.register_headless
  } catch (error) {
    console.error('加载用户设置失败:', error)
  }
}

// 保存用户设置
async function saveUserSettings() {
  try {
    await invoke('save_user_settings', {
      settings: {
        current_account_id: currentAccountId.value || null,
        email_privacy_mode: emailPrivacyMode.value,
        selected_accounts: Array.from(selectedAccounts.value),
        filter_type: filterType.value || null,
        import_mode: importMode.value,
        export_format: exportFormat.value,
        register_count: registerConfig.value.count,
        register_max_workers: registerConfig.value.max_workers,
        register_password: registerConfig.value.password,
        register_use_random_password: registerConfig.value.use_random_password,
        register_headless: registerConfig.value.headless,
        last_updated: new Date().toISOString()
      }
    })
  } catch (error) {
    console.error('保存用户设置失败:', error)
  }
}

// 设置导入模式
async function setImportMode(mode: 'token' | 'account' | 'json') {
  importMode.value = mode
  await saveUserSettings()
}

// 设置导出格式
async function setExportFormat(format: 'json' | 'txt' | 'csv') {
  exportFormat.value = format
  await saveUserSettings()
}

// 更新自动注册配置
async function updateRegisterConfig() {
  try {
    await invoke('update_register_config', {
      count: registerConfig.value.count,
      maxWorkers: registerConfig.value.max_workers,
      password: registerConfig.value.password,
      useRandomPassword: registerConfig.value.use_random_password,
      headless: registerConfig.value.headless
    })
  } catch (error) {
    console.error('保存自动注册配置失败:', error)
  }
}

async function handleAutoRegister() {
  if (registerConfig.value.count < 1 || registerConfig.value.count > 10) {
    showToast('注册数量必须在 1-10 之间', 'warning')
    return
  }

  // 先关闭对话框
  showRegisterDialog.value = false

  // 等待一小段时间确保对话框关闭动画完成,然后显示loading
  await new Promise(resolve => setTimeout(resolve, 100))
  loading.value = true

  try {
    const request: AutoRegisterRequest = {
      count: registerConfig.value.count,
      password: registerConfig.value.password,
      max_workers: registerConfig.value.max_workers,
      headless: registerConfig.value.headless,
      use_random_password: registerConfig.value.use_random_password
    }

    const response = await invoke<{ 
      success: boolean; 
      registered_count: number; 
      failed_count: number;
      total_count: number;
      accounts: Account[];
      error?: string;
    }>('auto_register_accounts', { request })

    // 根据结果显示不同的提示
    if (response.error) {
      // 有错误信息，显示错误
      showToast(`注册失败: ${response.error}`, 'error')
    } else if (response.failed_count === 0) {
      // 全部成功
      showToast(`🎉 注册完成！成功 ${response.registered_count}/${response.total_count} 个账号`, 'success')
    } else if (response.registered_count === 0) {
      // 全部失败
      showToast(`❌ 注册失败！0/${response.total_count} 个账号成功`, 'error')
    } else {
      // 部分成功
      showToast(`⚠️ 注册部分完成！成功 ${response.registered_count}/${response.total_count} 个账号，失败 ${response.failed_count} 个`, 'warning')
    }
    
    await loadAccounts()
  } catch (error) {
    console.error('自动注册失败:', error)
    showToast('自动注册失败: ' + error, 'error')
  } finally {
    loading.value = false
  }
}

async function handleRefresh(id: string) {
  try {
    console.log('刷新账户信息:', id)

    // 调用后端命令刷新账户信息
    const result = await invoke<{ success: boolean; account?: Account; error?: string }>('refresh_account_info', {
      accountId: id
    })

    if (result.success) {
      console.log('✅ 账户信息刷新成功')
      showToast('刷新成功', 'success')
      // 重新加载账户列表以显示最新数据
      await loadAccounts()
    } else {
      console.error('❌ 刷新失败:', result.error)
      showToast(`刷新失败: ${result.error || '未知错误'}`, 'error')
    }
  } catch (error) {
    console.error('刷新失败:', error)
    showToast(`刷新失败: ${error}`, 'error')
  }
}

async function handleRefreshAll() {
  if (accounts.value.length === 0) {
    return
  }

  // 使用确认对话框
  showConfirm(
    `确定要刷新所有 ${accounts.value.length} 个账户的信息吗?\n\n将并发刷新所有账户,这可能需要一些时间。`,
    async () => {
      // 设置loading状态并等待DOM更新
      loading.value = true
      await new Promise(resolve => setTimeout(resolve, 100))

      const totalCount = accounts.value.length
      console.log(`开始并发刷新 ${totalCount} 个账户...`)

      try {
        // 使用 Promise.allSettled 并发刷新所有账户
        const refreshPromises = accounts.value.map(account =>
          invoke<{ success: boolean; account?: Account; error?: string }>('refresh_account_info', {
            accountId: account.id
          })
            .then(result => ({ account, result, error: null }))
            .catch(error => ({ account, result: null, error }))
        )

        const results = await Promise.allSettled(refreshPromises)

        // 统计结果
        let successCount = 0
        let failCount = 0
        const errors: string[] = []

        results.forEach((promiseResult, index) => {
          if (promiseResult.status === 'fulfilled') {
            const { account, result, error } = promiseResult.value
            if (result?.success) {
              successCount++
              console.log(`✅ [${index + 1}/${totalCount}] 刷新成功: ${account.email}`)
            } else {
              failCount++
              const errorMsg = `${account.email}: ${result?.error || error || '未知错误'}`
              errors.push(errorMsg)
              console.error(`❌ [${index + 1}/${totalCount}] 刷新失败: ${errorMsg}`)
            }
          } else {
            failCount++
            const account = accounts.value[index]
            const errorMsg = `${account.email}: ${promiseResult.reason}`
            errors.push(errorMsg)
            console.error(`❌ [${index + 1}/${totalCount}] 刷新失败: ${errorMsg}`)
          }
        })

        // 刷新完成后重新加载账户列表
        await loadAccounts()

        // 显示结果
        if (failCount === 0) {
          showToast(`刷新完成! 成功刷新 ${successCount} 个账户`, 'success')
        } else {
          let message = `刷新完成!\n\n成功: ${successCount} 个\n失败: ${failCount} 个`
          if (errors.length > 0) {
            message += `\n\n失败详情:\n${errors.join('\n')}`
          }
          alert(message)
        }

      } catch (error) {
        console.error('批量刷新失败:', error)
        showToast(`批量刷新失败: ${error}`, 'error')
      } finally {
        loading.value = false
      }
    },
    {
      title: '确认刷新',
      confirmText: '开始刷新',
      type: 'info'
    }
  )
}

function handleEdit(account: Account) {
  editingAccount.value = { ...account }
  
  // 处理过期时间格式，转换为 datetime-local 需要的格式
  if (editingAccount.value.expire_time) {
    // 将 ISO 格式或其他格式转换为 datetime-local 需要的 YYYY-MM-DDTHH:mm 格式
    const date = new Date(editingAccount.value.expire_time)
    if (!isNaN(date.getTime())) {
      // 获取本地时间并格式化
      const year = date.getFullYear()
      const month = String(date.getMonth() + 1).padStart(2, '0')
      const day = String(date.getDate()).padStart(2, '0')
      const hours = String(date.getHours()).padStart(2, '0')
      const minutes = String(date.getMinutes()).padStart(2, '0')
      editingAccount.value.expire_time = `${year}-${month}-${day}T${hours}:${minutes}`
    }
  }
}

async function handleDelete(id: string) {
  const account = accounts.value.find(a => a.id === id)
  const email = account?.email || '该账户'

  showConfirm(
    `确定要删除账户 "${email}" 吗?\n\n此操作不可撤销!`,
    async () => {
      try {
        await invoke('delete_account', { id })
        showToast('删除成功', 'success')
        await loadAccounts()
      } catch (error) {
        console.error('删除失败:', error)
        showToast('删除失败: ' + error, 'error')
      }
    },
    {
      title: '确认删除',
      confirmText: '删除',
      type: 'danger'
    }
  )
}

async function saveEdit() {
  if (!editingAccount.value) return

  try {
    // 将 datetime-local 格式转换回 ISO 格式保存
    const accountToSave = { ...editingAccount.value }
    if (accountToSave.expire_time) {
      // 如果用户输入了过期时间，转换为 ISO 格式
      const date = new Date(accountToSave.expire_time)
      if (!isNaN(date.getTime())) {
        accountToSave.expire_time = date.toISOString()
      }
    }
    
    await invoke('update_account', {
      id: accountToSave.id,
      account: accountToSave
    })
    showToast('更新成功', 'success')
    editingAccount.value = null
    await loadAccounts()
  } catch (error) {
    console.error('更新失败:', error)
    showToast(`更新失败: ${error}`, 'error')
  }
}

function cancelEdit() {
  editingAccount.value = null
}

async function handleImportAccounts() {
  if (importMode.value === 'json') {
    // JSON导入模式
    await handleJsonImport()
    return
  } else if (importMode.value === 'token') {
    // Token 导入模式
    const tokens = importTokens.value
      .split('\n')
      .map(t => t.trim())
      .filter(t => t.length > 0)

    if (tokens.length === 0) {
      showToast('请输入至少一个 Token', 'warning')
      return
    }

    // 先关闭对话框
    showImportDialog.value = false
    importTokens.value = ''

    // 等待一小段时间确保对话框关闭动画完成,然后显示loading
    await new Promise(resolve => setTimeout(resolve, 100))
    loading.value = true

    let successCount = 0
    let failCount = 0
    const errors: string[] = []

    try {
      console.log(`开始导入 ${tokens.length} 个 Token...`)

      // 逐个导入 Token
      for (let i = 0; i < tokens.length; i++) {
        const token = tokens[i]
        console.log(`[${i + 1}/${tokens.length}] 导入 Token: ${token.substring(0, 20)}...`)

        try {
          const result = await invoke<{ success: boolean; account?: Account; error?: string }>('import_account_by_token', {
            token
          })

          if (result.success) {
            successCount++
            console.log(`✅ [${i + 1}/${tokens.length}] 导入成功: ${result.account?.email || '未知'}`)
          } else {
            failCount++
            const errorMsg = result.error || '未知错误'
            errors.push(`Token ${i + 1}: ${errorMsg}`)
            console.error(`❌ [${i + 1}/${tokens.length}] 导入失败: ${errorMsg}`)
          }
        } catch (error) {
          failCount++
          const errorMsg = `${error}`
          errors.push(`Token ${i + 1}: ${errorMsg}`)
          console.error(`❌ [${i + 1}/${tokens.length}] 导入失败: ${errorMsg}`)
        }

        // 添加小延迟避免请求过快
        if (i < tokens.length - 1) {
          await new Promise(resolve => setTimeout(resolve, 500))
        }
      }

      // 显示结果
      if (failCount === 0) {
        showToast(`导入完成! 成功导入 ${successCount} 个账户`, 'success')
      } else {
        let message = `导入完成!\n\n成功: ${successCount} 个\n失败: ${failCount} 个`
        if (errors.length > 0) {
          message += `\n\n失败详情:\n${errors.join('\n')}`
        }
        alert(message)
      }

    } catch (error) {
      console.error('批量导入失败:', error)
      showToast(`批量导入失败: ${error}`, 'error')
    } finally {
      loading.value = false
    }

    // 导入完成后重新加载账户列表
    await loadAccounts()
    
  } else {
    // 账号密码导入模式
    const accountLines = importAccounts.value
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0)

    if (accountLines.length === 0) {
      showToast('请输入至少一个账号', 'warning')
      return
    }

    // 解析账号密码
    const accounts: Array<{ email: string; password: string }> = []
    for (const line of accountLines) {
      const parts = line.split(/[:|\t|,]/)
      if (parts.length >= 2) {
        accounts.push({
          email: parts[0].trim(),
          password: parts[1].trim()
        })
      } else {
        showToast(`格式错误: ${line}`, 'warning')
        return
      }
    }

    // 关闭对话框
    showImportDialog.value = false
    importAccounts.value = ''

    // 等待动画完成
    await new Promise(resolve => setTimeout(resolve, 100))
    loading.value = true

    let successCount = 0
    let failCount = 0
    const errors: string[] = []

    try {
      console.log(`开始导入 ${accounts.length} 个账户...`)

      // 逐个处理账户
      for (let i = 0; i < accounts.length; i++) {
        const account = accounts[i]
        console.log(`[${i + 1}/${accounts.length}] 导入账户: ${account.email}`)

        try {
          // 尝试登录并获取Token
          const result = await invoke<{ success: boolean; token?: string; account?: Account; error?: string }>('import_account_by_credentials', {
            email: account.email,
            password: account.password
          })

          if (result.success) {
            successCount++
            console.log(`✅ [${i + 1}/${accounts.length}] 导入成功: ${account.email}`)
          } else {
            failCount++
            const errorMsg = result.error || '未知错误'
            errors.push(`${account.email}: ${errorMsg}`)
            console.error(`❌ [${i + 1}/${accounts.length}] 导入失败: ${errorMsg}`)
          }
        } catch (error) {
          failCount++
          const errorMsg = `${error}`
          errors.push(`${account.email}: ${errorMsg}`)
          console.error(`❌ [${i + 1}/${accounts.length}] 导入失败: ${errorMsg}`)
        }

        // 添加延迟避免过快
        if (i < accounts.length - 1) {
          await new Promise(resolve => setTimeout(resolve, 1000))
        }
      }

      // 显示结果
      if (failCount === 0) {
        showToast(`导入完成! 成功导入 ${successCount} 个账户`, 'success')
      } else {
        let message = `导入完成!\n\n成功: ${successCount} 个\n失败: ${failCount} 个`
        if (errors.length > 0) {
          message += `\n\n失败详情:\n${errors.join('\n')}`
        }
        alert(message)
      }

    } catch (error) {
      console.error('批量导入失败:', error)
      showToast(`批量导入失败: ${error}`, 'error')
    } finally {
      loading.value = false
    }

    // 重新加载账户列表
    await loadAccounts()
  }
}
</script>

<template>
  <div class="account-manager">
    <div class="header">
      <h2>
        <img src="/账户管理.svg" alt="账户管理" class="header-icon" />
        账户管理
      </h2>
      <div class="header-actions">
        <!-- 邮箱隐私开关 -->
        <button 
          class="btn-secondary privacy-btn"
          @click="toggleEmailPrivacy"
          :title="emailPrivacyMode ? '关闭邮箱隐私' : '开启邮箱隐私'"
        >
          <img :src="emailPrivacyMode ? '/隐私.svg' : '/显示.svg'" alt="隐私" class="btn-action-icon" />
          <span class="btn-text">{{ emailPrivacyMode ? '隐私模式' : '显示邮箱' }}</span>
        </button>
        
        <button class="btn-secondary" @click="showExportDialog = true" title="导出账户">
          <img src="/导出.svg" alt="导出" class="btn-action-icon" />
          <span class="btn-text">导出</span>
        </button>
        
        <button
          class="btn-secondary refresh-all-btn"
          @click="handleRefreshAll"
          :disabled="loading || accounts.length === 0"
          title="刷新所有账户信息"
        >
          <img src="/刷新.svg" alt="刷新" class="btn-action-icon refresh-all-icon" />
          <span class="btn-text">全部刷新</span>
        </button>
        
        <button class="btn-import" @click="showImportDialog = true" title="导入账户">
          <img src="/导入.svg" alt="导入" class="btn-action-icon" />
          <span class="btn-text">导入账户</span>
        </button>
        
        <button class="btn-primary" @click="showRegisterDialog = true" title="自动注册">
          <img src="/注册.svg" alt="注册" class="btn-action-icon" />
          <span class="btn-text">自动注册</span>
        </button>
      </div>
    </div>

    <!-- 统计面板 -->
    <div v-if="accounts.length > 0" class="stats-panel">
      <div class="stats-header">
        <div class="stats-title">
          <img src="/统计.svg" alt="统计" class="stats-icon" />
          <h3>账号统计</h3>
        </div>
        <div class="stats-subtitle">
          实时更新 · {{ accounts.length }} 个账号
        </div>
      </div>
      <div class="stats-grid">
        <div 
          class="stat-card clickable" 
          :class="{ active: filterType === '' }"
          @click="setFilter('')"
          title="显示全部账号"
        >
          <div class="stat-value">{{ accountStats.total }}</div>
          <div class="stat-label">总账号数</div>
        </div>
        <div 
          class="stat-card success clickable" 
          :class="{ active: filterType === 'full' }"
          @click="setFilter('full')"
          title="筛选满额度账号"
        >
          <div class="stat-value">{{ accountStats.fullQuota }}</div>
          <div class="stat-label">满额度账号</div>
        </div>
        <div 
          class="stat-card warning clickable" 
          :class="{ active: filterType === 'partial' }"
          @click="setFilter('partial')"
          title="筛选已消耗账号"
        >
          <div class="stat-value">{{ accountStats.partialQuota }}</div>
          <div class="stat-label">已消耗账号</div>
        </div>
        <div 
          class="stat-card danger clickable" 
          :class="{ active: filterType === 'empty' }"
          @click="setFilter('empty')"
          title="筛选0额度账号"
        >
          <div class="stat-value">{{ accountStats.emptyQuota }}</div>
          <div class="stat-label">0额度账号</div>
        </div>
        <div 
          class="stat-card error clickable" 
          :class="{ active: filterType === 'abnormal' }"
          @click="setFilter('abnormal')"
          title="筛选异常账号"
        >
          <div class="stat-value">{{ accountStats.abnormal }}</div>
          <div class="stat-label">异常账号</div>
          <div class="stat-hint">额度用尽</div>
        </div>
        <div class="stat-card info">
          <div class="stat-value">${{ accountStats.totalQuota.toFixed(2) }}</div>
          <div class="stat-label">总额度</div>
        </div>
        <div class="stat-card primary">
          <div class="stat-value">${{ accountStats.totalRemaining.toFixed(2) }}</div>
          <div class="stat-label">剩余额度</div>
        </div>
        <div class="stat-card secondary">
          <div class="stat-value">${{ accountStats.totalUsed.toFixed(2) }}</div>
          <div class="stat-label">已用额度</div>
        </div>
      </div>
    </div>

    <!-- 筛选提示 -->
    <div v-if="filterType" class="filter-hint">
      <span>正在筛选：{{ getFilterLabel(filterType) }}</span>
      <button class="clear-filter" @click="filterType = ''">
        ✕ 清除筛选
      </button>
    </div>

    <!-- 账号列表始终显示 -->
    <div class="accounts-grid">
      <!-- 全选和批量操作栏 -->
      <div v-if="accounts.length > 0" class="select-all-card">
        <div class="select-all-left">
          <button 
            @click="toggleSelectAll"
            class="select-all-btn"
            title="也可以使用 Ctrl+A 或 Cmd+A"
          >
            {{ selectedAccounts.size === filteredAccounts.length && filteredAccounts.length > 0 ? '取消全选' : '全选' }}
          </button>
          <span class="select-hint">
            💡 点击账户卡片空白处选择，Ctrl+A 全选
          </span>
        </div>
        
        <!-- 批量操作按钮 -->
        <div v-if="selectedAccounts.size > 0" class="batch-actions-inline">
          <span class="selected-count">
            <span class="count-badge">{{ selectedAccounts.size }}</span>
            个已选择
          </span>
          <button class="btn-batch-action refresh" @click="handleBatchRefresh" title="批量刷新选中的账户">
            <img src="/刷新.svg" alt="刷新" class="batch-icon" />
            <span>刷新</span>
          </button>
          <button class="btn-batch-action delete" @click="handleBatchDelete" title="批量删除选中的账户">
            <img src="/删除 .svg" alt="删除" class="batch-icon" />
            <span>删除</span>
          </button>
          <button class="btn-batch-action cancel" @click="selectedAccounts.clear()" title="取消所有选择">
            ✕ 取消
          </button>
        </div>
      </div>
      
      <AccountCard
        v-for="account in filteredAccounts"
        :key="account.id"
        :account="{ ...account, email: maskEmail(account.email) }"
        :original-email="account.email"
        :is-selected="selectedAccounts.has(account.id)"
        :is-current="currentAccountId === account.id"
        @refresh="handleRefresh"
        @edit="handleEdit"
        @delete="handleDelete"
        @show-toast="showToast"
        @toggle-selection="toggleAccountSelection"
        @set-current="setCurrentAccount"
      />
      
      <div v-if="filteredAccounts.length === 0" class="empty-state">
        <p v-if="filterType">没有符合条件的账号</p>
        <p v-else>暂无账户</p>
        <p v-if="filterType" class="hint">
          当前筛选：{{ getFilterLabel(filterType) }}
        </p>
        <p v-else class="hint">点击右上角"自动注册"按钮开始</p>
      </div>
    </div>

    <!-- 加载覆盖层 -->
    <div v-if="loading" class="loading-overlay" @click="handleCloseLoading">
      <div class="loading-content" @click.stop>
        <button class="loading-close" @click="handleCloseLoading" title="关闭">
          <img src="/icon-close.svg" alt="关闭" class="close-icon" />
        </button>
        <div class="spinner"></div>
        <p>处理中...</p>
        <p class="loading-hint">点击背景或关闭按钮隐藏提示</p>
        <p class="loading-hint">（操作仍会在后台继续）</p>
      </div>
    </div>

    <div v-if="showRegisterDialog" class="modal-overlay" @click="showRegisterDialog = false">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>
            <img src="/注册.svg" alt="注册" class="modal-header-icon" />
            自动注册账号
          </h3>
          <button class="modal-close" @click="showRegisterDialog = false">
            <img src="/icon-close.svg" alt="关闭" class="close-icon" />
          </button>
        </div>
        
        <div class="modal-body">
          <div class="alert alert-info" style="margin-bottom: 16px; padding: 12px; background: #e3f2fd; border-radius: 8px; font-size: 13px; color: #1976d2;">
            <strong>✨ 免费临时邮箱</strong><br>
            使用 tempmail.plus 免费服务,无需 API Key,自动生成随机邮箱地址
          </div>

          <div class="form-row">
            <div class="form-group">
              <label>注册数量</label>
              <input
                v-model.number="registerConfig.count"
                @change="updateRegisterConfig"
                type="number"
                min="1"
                max="10"
                placeholder="1-10"
              />
            </div>

            <div class="form-group">
              <label>并发数</label>
              <input
                v-model.number="registerConfig.max_workers"
                @change="updateRegisterConfig"
                type="number"
                min="1"
                max="5"
                placeholder="1-5"
              />
            </div>
          </div>

          <div class="form-group">
            <label>密码</label>
            <input
              v-model="registerConfig.password"
              @change="updateRegisterConfig"
              type="text"
              placeholder="VerdentAI@2024"
              :disabled="registerConfig.use_random_password"
            />
            <p v-if="registerConfig.use_random_password" class="help-text" style="color: #667eea; margin-top: 4px;">
              🎲 已启用随机密码,每个账户将生成唯一的 12-16 位强密码
            </p>
          </div>

          <div class="form-group checkbox-group">
            <input
              id="use-random-password"
              v-model="registerConfig.use_random_password"
              @change="updateRegisterConfig"
              type="checkbox"
            />
            <label for="use-random-password">🎲 使用随机密码 (每个账户生成唯一密码)</label>
          </div>

          <div class="form-group checkbox-group">
            <input
              id="headless"
              v-model="registerConfig.headless"
              @change="updateRegisterConfig"
              type="checkbox"
            />
            <label for="headless">无头模式 (后台运行)</label>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn-secondary" @click="showRegisterDialog = false">
            取消
          </button>
          <button class="btn-primary" @click="handleAutoRegister">
            开始注册
          </button>
        </div>
      </div>
    </div>

    <div v-if="showImportDialog" class="modal-overlay" @click="showImportDialog = false">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>
            <img src="/导入.svg" alt="导入" class="modal-header-icon" />
            导入账户
          </h3>
          <button class="modal-close" @click="showImportDialog = false">
            <img src="/icon-close.svg" alt="关闭" class="close-icon" />
          </button>
        </div>

        <div class="modal-body">
          <!-- 导入模式选择 -->
          <div class="import-mode-tabs">
            <button 
              class="mode-tab" 
              :class="{ active: importMode === 'token' }"
              @click="setImportMode('token')"
            >
              🎫 Token 导入
            </button>
            <button 
              class="mode-tab" 
              :class="{ active: importMode === 'account' }"
              @click="setImportMode('account')"
            >
              🔑 账号密码导入
            </button>
            <button 
              class="mode-tab" 
              :class="{ active: importMode === 'json' }"
              @click="setImportMode('json')"
            >
              📄 JSON 导入
            </button>
          </div>

          <!-- Token 导入模式 -->
          <div v-if="importMode === 'token'">
            <div class="alert alert-info" style="margin-bottom: 16px; padding: 12px; background: #e3f2fd; border-radius: 8px; font-size: 13px; color: #1976d2;">
              <strong>🎫 Token 导入说明</strong><br>
              • 每行粘贴一个 Token<br>
              • 支持批量导入多个 Token<br>
              • 如果邮箱已存在,将更新账户信息<br>
              • 导入的账户将自动获取额度、订阅等信息
            </div>

            <div class="form-group">
              <label>Token 列表 (每行一个)</label>
              <textarea
                v-model="importTokens"
                rows="10"
                placeholder="粘贴 Token,每行一个&#10;例如:&#10;eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...&#10;eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
                style="font-family: monospace; font-size: 12px;"
              ></textarea>
            </div>
          </div>

          <!-- 账号密码导入模式 -->
          <div v-if="importMode === 'account'">
            <div class="alert alert-info" style="margin-bottom: 16px; padding: 12px; background: #fff3e0; border-radius: 8px; font-size: 13px; color: #e65100;">
              <strong>🔑 账号密码导入说明</strong><br>
              • 每行输入一对账号密码<br>
              • 格式: 邮箱:密码 或 邮箱,密码<br>
              • 支持分隔符: 英文冒号(:)、逗号(,)、Tab键<br>
              • 将自动登录并获取Token
            </div>

            <div class="form-group">
              <label>账号列表 (每行一对)</label>
              <textarea
                v-model="importAccounts"
                rows="10"
                placeholder="粘贴账号密码,每行一对&#10;例如:&#10;user1@example.com:password123&#10;user2@example.com,password456&#10;user3@example.com	password789"
                style="font-family: monospace; font-size: 12px;"
              ></textarea>
            </div>
          </div>
          
          <!-- JSON 导入模式 -->
          <div v-if="importMode === 'json'">
            <div class="alert alert-info" style="margin-bottom: 16px; padding: 12px; background: #e8f5e9; border-radius: 8px; font-size: 13px; color: #2e7d32;">
              <strong>📄 JSON 导入说明</strong><br>
              • 支持导入包含账户信息的JSON格式<br>
              • 智能识别 token、email、password 等字段<br>
              • 支持单个对象或数组格式<br>
              • 自动识别常见的字段名变体
            </div>

            <div class="form-group">
              <label>JSON 数据</label>
              <textarea
                v-model="importJson"
                rows="10"
                placeholder='粘贴JSON数据,支持以下格式:&#10;[&#10;  {&#10;    "email": "user@example.com",&#10;    "password": "pass123",&#10;    "token": "eyJ..."&#10;  }&#10;]&#10;或者任何包含账户信息的JSON格式'
                style="font-family: monospace; font-size: 12px;"
              ></textarea>
            </div>
            
            <div class="form-group">
              <label for="json-file" class="file-upload-btn">
                <img src="/文件.svg" alt="文件" class="file-icon" />
                选择JSON文件
                <input 
                  id="json-file"
                  type="file"
                  accept=".json"
                  @change="handleJsonFileUpload"
                  style="display: none;"
                />
              </label>
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn-secondary" @click="showImportDialog = false">
            取消
          </button>
          <button class="btn-primary" @click="handleImportAccounts">
            开始导入
          </button>
        </div>
      </div>
    </div>

    <!-- 导出对话框 -->
    <div v-if="showExportDialog" class="modal-overlay" @click="showExportDialog = false">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>
            <img src="/导出.svg" alt="导出" class="modal-header-icon" />
            导出账户
          </h3>
          <button class="modal-close" @click="showExportDialog = false">
            <img src="/icon-close.svg" alt="关闭" class="close-icon" />
          </button>
        </div>

        <div class="modal-body">
          <div class="alert alert-info" style="margin-bottom: 16px; padding: 12px; background: #e3f2fd; border-radius: 8px; font-size: 13px; color: #1976d2;">
            <strong>📥 导出说明</strong><br>
            • {{ selectedAccounts.size > 0 ? `已选择 ${selectedAccounts.size} 个账户` : '将导出所有账户' }}<br>
            • 选择导出格式后点击"开始导出"<br>
            • 文件将自动下载到默认下载目录
          </div>

          <div class="form-group">
            <label>导出格式</label>
            <div class="export-format-options">
              <label class="format-option">
                <input type="radio" :checked="exportFormat === 'json'" @change="setExportFormat('json')" />
                <span>JSON格式</span>
                <small>完整账户信息，包含所有字段</small>
              </label>
              <label class="format-option">
                <input type="radio" :checked="exportFormat === 'csv'" @change="setExportFormat('csv')" />
                <span>CSV格式</span>
                <small>表格格式，方便在Excel中打开</small>
              </label>
              <label class="format-option">
                <input type="radio" :checked="exportFormat === 'txt'" @change="setExportFormat('txt')" />
                <span>TXT格式</span>
                <small>邮箱:密码:token的简单格式</small>
              </label>
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn-secondary" @click="showExportDialog = false">
            取消
          </button>
          <button class="btn-primary" @click="handleExport">
            开始导出
          </button>
        </div>
      </div>
    </div>

    <div v-if="editingAccount" class="modal-overlay" @click="cancelEdit">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>
            <img src="/编辑.svg" alt="编辑" class="modal-header-icon" />
            编辑账户
          </h3>
          <button class="modal-close" @click="cancelEdit">
            <img src="/icon-close.svg" alt="关闭" class="close-icon" />
          </button>
        </div>
        
        <div class="modal-body">
          <div class="form-group">
            <label>邮箱</label>
            <input v-model="editingAccount.email" type="text" readonly />
          </div>

          <div class="form-group">
            <label>密码</label>
            <input v-model="editingAccount.password" type="text" />
          </div>

          <div class="form-group">
            <label>Token</label>
            <textarea v-model="editingAccount.token" rows="3"></textarea>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label>剩余额度</label>
              <input v-model="editingAccount.quota_remaining" type="text" placeholder="例如: 98.89" />
            </div>

            <div class="form-group">
              <label>已消耗额度</label>
              <input v-model="editingAccount.quota_used" type="text" placeholder="例如: 1.11" />
            </div>

            <div class="form-group">
              <label>总额度</label>
              <input v-model="editingAccount.quota_total" type="text" placeholder="例如: 100.00" />
            </div>
          </div>

          <div class="form-group">
            <label>过期时间</label>
            <input v-model="editingAccount.expire_time" type="datetime-local" />
          </div>

          <div class="form-group">
            <label>状态</label>
            <select v-model="editingAccount.status">
              <option value="active">正常</option>
              <option value="expired">已过期</option>
              <option value="suspended">已暂停</option>
            </select>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn-secondary" @click="cancelEdit">取消</button>
          <button class="btn-primary" @click="saveEdit">保存</button>
        </div>
      </div>
    </div>

    <!-- Toast 通知 -->
    <Toast
      :show="toast.show"
      :message="toast.message"
      :type="toast.type"
      @close="closeToast"
    />

    <!-- 确认对话框 -->
    <ConfirmDialog
      :show="confirmDialog.show"
      :title="confirmDialog.title"
      :message="confirmDialog.message"
      :confirmText="confirmDialog.confirmText"
      :cancelText="confirmDialog.cancelText"
      :type="confirmDialog.type"
      @confirm="handleConfirm"
      @cancel="handleCancel"
    />
  </div>
</template>

<style scoped>
.account-manager {
  padding: 10px;
  width: 100%;
  max-width: 100%;
  margin: 0 auto;
  box-sizing: border-box;
  overflow-y: auto;
  overflow-x: hidden;
  height: 100vh;
}

/* 响应式padding调整 */
@media (max-width: 768px) {
  .account-manager {
    padding: 10px;
  }
}

@media (min-width: 1920px) {
  .account-manager {
    padding: 10px;
  }
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
  gap: 8px;
}

.header-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: nowrap;
  justify-content: flex-end;
  flex-shrink: 1; /* 允许适度收缩 */
  margin-left: auto;
}

/* 中等屏幕时隐藏所有按钮文字，统一显示图标 */
@media (max-width: 1100px) {
  .header-actions .btn-text {
    display: none;
  }
  
  .header-actions .btn-secondary,
  .header-actions .btn-import,
  .header-actions .btn-primary {
    padding: 8px;
    min-width: auto;
    width: 36px;
    height: 36px;
  }
  
  .header-actions {
    gap: 6px;
  }
}

.header h2 {
  font-size: 20px;
  font-weight: 700;
  color: #1f2937;
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  flex-shrink: 0;
  min-width: 140px;
}

/* 响应式布局 - 小屏幕，只在真正需要时才换行 */
@media (max-width: 650px) {
  .header {
    flex-direction: column;
    align-items: stretch;
    gap: 10px;
  }
  
  .header h2 {
    margin-bottom: 0;
    justify-content: center;
  }
  
  .header-actions {
    justify-content: center;
    flex-wrap: wrap;
    margin-left: 0;
    gap: 6px;
  }
  
  /* 确保按钮在小屏幕时保持一致的尺寸 */
  .header-actions .btn-secondary,
  .header-actions .btn-import,
  .header-actions .btn-primary {
    padding: 8px;
    min-width: auto;
    width: 36px;
    height: 36px;
  }
  
  .header-actions .btn-text {
    display: none;
  }
}

.header-icon {
  width: 24px;
  height: 24px;
  object-fit: contain;
}

/* 统计面板样式 */
.stats-panel {
  margin-bottom: 10px;
  padding: 16px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  border: 1px solid #e5e5e7;
}

/* 小屏幕下减少padding */
@media (max-width: 768px) {
  .stats-panel {
    padding: 16px;
    margin-bottom: 16px;
  }
}

.stats-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e5e5e7;
}

.stats-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.stats-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #1d1d1f;
}

.stats-subtitle {
  font-size: 12px;
  color: #8e8e93;
  font-weight: 500;
}

.stats-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(125px, 1fr));
  gap: 10px;
}

.stat-card {
  padding: 12px 10px;
  background: white;
  border: 1px solid #e5e5e7;
  border-radius: 8px;
  text-align: center;
  transition: all 0.2s;
  position: relative;
  min-width: 0; /* 防止内容溢出 */
  overflow: visible; /* 允许提示框显示 */
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-height: 70px;
}

.stat-card.clickable {
  cursor: pointer;
  user-select: none;
}

.stat-card.clickable:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
}

.stat-card.clickable:active {
  transform: translateY(0);
}

.stat-card.active {
  box-shadow: 0 0 0 2px #007aff;
  background: white;
}

/* 筛选提示样式 */
.filter-hint {
  background: #e3f2fd;
  border-radius: 8px;
  padding: 10px 16px;
  margin-bottom: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 14px;
  color: #1976d2;
}

.clear-filter {
  background: white;
  border: 1px solid #1976d2;
  color: #1976d2;
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.clear-filter:hover {
  background: #1976d2;
  color: white;
}

.stat-value {
  font-size: clamp(16px, 2.2vw, 22px); /* 响应式字体大小 */
  font-weight: 700;
  color: #1d1d1f;
  margin-bottom: 4px;
  line-height: 1.2;
  word-break: break-word;
  overflow-wrap: break-word;
  hyphens: auto;
}

.stat-label {
  font-size: 11px;
  color: #8e8e93;
  font-weight: 500;
  line-height: 1.2;
  margin-top: 2px;
}

.stat-hint {
  font-size: 10px;
  color: #999;
  margin-top: 2px;
}

/* 统计卡片颜色主题 */
.stat-card.success {
  background: #e8f5e8;
  border-color: #34c759;
}

.stat-card.success .stat-value {
  color: #34c759;
}

.stat-card.warning {
  background: #fff3e0;
  border-color: #ff9500;
}

.stat-card.warning .stat-value {
  color: #ff9500;
}

.stat-card.danger {
  background: #ffebe9;
  border-color: #ff3b30;
}

.stat-card.danger .stat-value {
  color: #ff3b30;
}

.stat-card.error {
  background: #ffe6e6;
  border-color: #d70015;
}

.stat-card.error .stat-value {
  color: #d70015;
}

.stat-card.info {
  background: #e3f2fd;
  border-color: #007aff;
}

.stat-card.info .stat-value {
  color: #007aff;
}

.stat-card.primary {
  background: #f0efff;
  border-color: #5856d6;
}

.stat-card.primary .stat-value {
  color: #5856d6;
}

.stat-card.secondary {
  background: #fafafa;
  border-color: #8e8e93;
}

.stat-card.secondary .stat-value {
  color: #6b7280;
}

/* 金额卡片特殊处理 */
.stat-card.info .stat-value,
.stat-card.primary .stat-value,
.stat-card.secondary .stat-value {
  font-size: clamp(14px, 1.8vw, 20px); /* 金额卡片使用更小的字体 */
  font-feature-settings: 'tnum'; /* 等宽数字 */
  letter-spacing: -0.02em; /* 减小字间距 */
}

.accounts-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 12px;
  padding: 0 2px;
  justify-content: start; /* 左对齐网格项目 */
}

/* 响应式布局 - 超大屏幕 (>1920px) */
@media (min-width: 1920px) {
  .accounts-grid {
    grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
    gap: 14px;
  }
  .stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  }
  .stat-value {
    font-size: 22px;
  }
  .stat-card {
    padding: 16px 14px;
    min-height: 80px;
  }
}

/* 响应式布局 - 大屏幕 (1440-1920px) */
@media (min-width: 1440px) and (max-width: 1919px) {
  .accounts-grid {
    grid-template-columns: repeat(auto-fill, minmax(330px, 1fr));
  }
  .stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(135px, 1fr));
  }
  .stat-value {
    font-size: 20px;
  }
}

/* 响应式布局 - 中等屏幕 (1024-1439px) */
@media (min-width: 1024px) and (max-width: 1439px) {
  .accounts-grid {
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  }
  .stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(125px, 1fr));
  }
  .stat-value {
    font-size: 18px;
  }
}

/* 响应式布局 - 小屏幕 (768-1023px) */
@media (min-width: 768px) and (max-width: 1023px) {
  .accounts-grid {
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 10px;
  }
  .stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(115px, 1fr));
    gap: 8px;
  }
  .stat-value {
    font-size: 16px;
  }
  .stat-card {
    padding: 10px 8px;
  }
}

/* 响应式布局 - 移动设备 (<768px) */
@media (max-width: 767px) {
  .accounts-grid {
    grid-template-columns: 1fr;
    gap: 10px;
    padding: 0;
  }
  .stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    gap: 6px;
  }
  .stat-value {
    font-size: 14px;
  }
  .stat-label {
    font-size: 10px;
  }
  .stat-card {
    padding: 8px 6px;
    min-height: 60px;
  }
  /* 移动设备上金额卡片更小的字体 */
  .stat-card.info .stat-value,
  .stat-card.primary .stat-value,
  .stat-card.secondary .stat-value {
    font-size: 12px;
  }
}

.empty-state {
  grid-column: 1 / -1;
  text-align: center;
  padding: 60px 20px;
  color: #6b7280;
}

.empty-state p {
  font-size: 18px;
  margin: 8px 0;
}

.empty-state .hint {
  font-size: 14px;
  opacity: 0.7;
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.loading-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 30px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
  position: relative;
  min-width: 200px;
}

.loading-content p {
  margin-top: 12px;
  color: #666;
  font-size: 14px;
  animation: none; /* 确保文字不旋转 */
  transform: none; /* 确保文字不应用任何变换 */
}

.loading-hint {
  font-size: 12px !important;
  color: #999 !important;
  margin-top: 8px !important;
}

.loading-close {
  position: absolute;
  top: 10px;
  right: 10px;
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 4px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.2s;
}

.loading-close:hover {
  background: #f3f4f6;
}

.loading-close .close-icon {
  width: 12px;
  height: 12px;
  object-fit: contain;
}

.spinner {
  width: 48px;
  height: 48px;
  border: 4px solid #e5e7eb;
  border-top-color: #667eea;
  border-radius: 50%;
  animation: spin 0.8s linear infinite; /* 只有spinner旋转 */
  margin-bottom: 16px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

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
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 16px;
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  overflow-y: auto;
  overflow-x: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

/* 组件内模态框的滚动条样式 */
.modal-content::-webkit-scrollbar {
  width: 8px;
}

.modal-content::-webkit-scrollbar-track {
  background: transparent;
  border-radius: 8px;
}

.modal-content::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.15);
  border-radius: 8px;
  border: 1px solid transparent;
  background-clip: padding-box;
}

.modal-content::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.25);
  background-clip: padding-box;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid #e5e5e7;
}

.modal-header h3 {
  font-size: 20px;
  font-weight: 700;
  color: #1f2937;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-header-icon {
  width: 24px;
  height: 24px;
  object-fit: contain;
}

.modal-close {
  background: transparent;
  border: none;
  font-size: 18px;
  cursor: pointer;
  color: #6b7280;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.2s;
  flex-shrink: 0;
}

.modal-close:hover {
  background: #f3f4f6;
  color: #374151;
}

.close-icon {
  width: 14px;
  height: 14px;
  object-fit: contain;
}

.modal-body {
  padding: 24px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid #e5e5e7;
}

.modal-footer button {
  min-width: 120px;
  white-space: nowrap;
  padding: 10px 24px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
  color: #374151;
}

.form-group input,
.form-group select,
.form-group textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 14px;
  transition: border-color 0.2s;
}

.form-group input:focus,
.form-group select:focus,
.form-group textarea:focus {
  outline: none;
  border-color: #667eea;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.checkbox-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 导入模式选择标签 */
.import-mode-tabs {
  display: flex;
  gap: 0;
  border-bottom: 2px solid #e5e5e7;
  margin-bottom: 20px;
}

.mode-tab {
  flex: 1;
  padding: 12px;
  background: none;
  border: none;
  color: #8e8e93;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s;
  position: relative;
}

.mode-tab:hover {
  color: #1d1d1f;
  background: #fafafa;
}

.mode-tab.active {
  color: #007aff;
  background: white;
}

.mode-tab.active::after {
  content: '';
  position: absolute;
  bottom: -2px;
  left: 0;
  right: 0;
  height: 2px;
  background: #007aff;
}

.checkbox-group input[type="checkbox"] {
  width: auto;
  margin: 0;
}

.checkbox-group label {
  margin: 0;
  cursor: pointer;
}

.btn-primary,
.btn-secondary,
.btn-import {
  padding: 8px 14px;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  line-height: 1.5;
  height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

/* 主要按钮默认有最小宽度 */
.btn-primary,
.btn-import {
  min-width: 100px;
}

/* 在中等屏幕时移除最小宽度限制 */
@media (max-width: 1100px) {
  .btn-primary,
  .btn-import {
    min-width: auto;
  }
}

/* 次要按钮更紧凑 */
.btn-secondary {
  padding: 8px 12px;
}

.btn-action-icon {
  width: 16px;
  height: 16px;
  object-fit: contain;
}

/* 刷新按钮旋转动画 */
@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.refresh-all-icon {
  transition: transform 0.3s ease;
}

.refresh-all-btn:hover .refresh-all-icon {
  animation: rotate 0.6s ease;
}

.btn-primary {
  background: #007aff;
  color: white;
}

.btn-primary .btn-action-icon {
  filter: brightness(0) invert(1);
}

.btn-primary:hover {
  background: #0051d5;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 122, 255, 0.2);
}

.btn-import {
  background: #34c759;
  color: white;
}

.btn-import .btn-action-icon {
  filter: brightness(0) invert(1);
}

.btn-import:hover {
  background: #30a14e;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(52, 199, 89, 0.2);
}


.btn-secondary {
  background: #f5f5f7;
  color: #1d1d1f;
  border: 1px solid #d2d2d7;
}

.btn-secondary:hover:not(:disabled) {
  background: #e8e8ed;
  border-color: #c6c6c9;
}

.btn-primary:disabled,
.btn-secondary:disabled,
.btn-import:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.btn-primary:disabled:hover,
.btn-secondary:disabled:hover,
.btn-import:disabled:hover {
  transform: none;
  box-shadow: none;
}

/* 批量操作样式 */
.batch-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 6px 8px;
  background: linear-gradient(to right, #f8fff9, #f0fff2);
  border: 1px solid #34c759;
  border-radius: 8px;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.batch-actions .selected-count {
  font-size: 13px;
  font-weight: 500;
  color: #1d7a1d;
  margin-right: 8px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.batch-actions .count-badge {
  background: #34c759;
  color: white;
  padding: 2px 8px;
  border-radius: 12px;
  font-weight: 600;
  font-size: 14px;
}

.batch-actions .btn-secondary {
  padding: 6px 12px;
  height: 32px;
  font-size: 13px;
}

.batch-actions .delete-btn {
  background: #ffe6e6;
  color: #d70015;
  border-color: #ffb3b3;
}

.batch-actions .delete-btn:hover {
  background: #ff3b30;
  color: white;
  border-color: #ff3b30;
}

.batch-actions .delete-btn:hover .btn-action-icon {
  filter: brightness(0) invert(1);
}

.batch-actions .cancel-btn {
  background: white;
  border-color: #d2d2d7;
  color: #8e8e93;
  padding: 6px 10px;
}

.batch-actions .cancel-btn:hover {
  background: #f5f5f7;
  border-color: #8e8e93;
  color: #1d1d1f;
}

/* 全选和批量操作栏样式 */
.select-all-card {
  grid-column: 1 / -1;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  margin-bottom: 8px;
  background: linear-gradient(135deg, #f8fff9, #ffffff);
  border: 1px solid #e5e5e7;
  border-radius: 12px;
}

.select-all-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.select-all-btn {
  padding: 6px 20px;
  background: white;
  border: 1px solid #d2d2d7;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  color: #1d1d1f;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  min-width: 100px;
}

.select-all-btn:hover {
  background: #f0fff2;
  border-color: #34c759;
  color: #34c759;
}

.select-all-btn:active {
  transform: scale(0.98);
}

.select-hint {
  font-size: 12px;
  color: #8e8e93;
  font-style: italic;
}

/* 行内批量操作样式 */
.batch-actions-inline {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: white;
  border: 1px solid #34c759;
  border-radius: 8px;
  animation: slideIn 0.3s ease;
}

.batch-actions-inline .selected-count {
  font-size: 13px;
  font-weight: 500;
  color: #1d7a1d;
  display: flex;
  align-items: center;
  gap: 4px;
  margin-right: 8px;
}

.batch-actions-inline .count-badge {
  background: #34c759;
  color: white;
  padding: 2px 8px;
  border-radius: 12px;
  font-weight: 600;
  font-size: 14px;
}

/* 批量操作按钮 */
.btn-batch-action {
  padding: 6px 14px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: inline-flex;
  flex-direction: row;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 32px;
  white-space: nowrap;
  min-width: 70px;
}

.btn-batch-action span {
  display: inline-block;
  line-height: 1;
}

.btn-batch-action.refresh {
  background: #e5f2ff;
  color: #007aff;
}

.btn-batch-action.refresh:hover {
  background: #007aff;
  color: white;
}

.btn-batch-action.delete {
  background: #ffebeb;
  color: #ff3b30;
}

.btn-batch-action.delete:hover {
  background: #ff3b30;
  color: white;
}

.btn-batch-action.cancel {
  background: #f5f5f7;
  color: #8e8e93;
}

.btn-batch-action.cancel:hover {
  background: #e8e8ed;
  color: #1d1d1f;
}

.batch-icon {
  width: 16px;
  height: 16px;
  object-fit: contain;
  flex-shrink: 0;
}

.btn-batch-action:hover .batch-icon {
  filter: brightness(0) invert(1);
}

/* 邮箱隐私按钮 */
.privacy-btn {
  background: #f0efff;
  border-color: #5856d6;
}

.privacy-btn:hover {
  background: #5856d6;
  color: white;
}

.privacy-btn:hover .btn-action-icon {
  filter: brightness(0) invert(1);
}

/* 文件上传按钮 */
.file-upload-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 16px;
  background: #f5f5f7;
  color: #1d1d1f;
  border: 1px solid #d2d2d7;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
  font-weight: 500;
}

.file-upload-btn:hover {
  background: #e8e8ed;
  border-color: #007aff;
  color: #007aff;
  transform: translateY(-1px);
}

.file-icon {
  width: 18px;
  height: 18px;
  object-fit: contain;
  vertical-align: middle;
  margin-top: -2px; /* 微调垂直对齐 */
}

.file-upload-btn:hover .file-icon {
  filter: hue-rotate(200deg) saturate(2); /* 悬停时图标变蓝 */
}

/* 导出格式选项 */
.export-format-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 8px;
}

.format-option {
  display: flex;
  flex-direction: column;
  padding: 12px;
  background: #f5f5f7;
  border: 1px solid #d2d2d7;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.format-option:hover {
  background: white;
  border-color: #007aff;
}

.format-option input[type="radio"] {
  margin-right: 8px;
  width: auto;
}

.format-option span {
  font-weight: 500;
  font-size: 14px;
  margin-left: 24px;
}

.format-option small {
  font-size: 12px;
  color: #8e8e93;
  margin-left: 24px;
  margin-top: 4px;
}
</style>
