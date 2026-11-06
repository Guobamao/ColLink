import { toast } from 'vue-sonner'
import { reactive } from 'vue'

export { toast }

export interface ConfirmOptions {
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  type?: 'primary' | 'danger' | 'warning'
}

export const confirmState = reactive<{
  open: boolean
  title: string
  message: string
  confirmText: string
  cancelText: string
  type: 'primary' | 'danger' | 'warning'
  resolve?: (value: boolean) => void
}>({
  open: false,
  title: '确认提示',
  message: '',
  confirmText: '确定',
  cancelText: '取消',
  type: 'primary',
})

export function confirmDialog(options: ConfirmOptions | string): Promise<boolean> {
  return new Promise((resolve) => {
    if (typeof options === 'string') {
      confirmState.message = options
      confirmState.title = '操作确认'
      confirmState.confirmText = '确定'
      confirmState.cancelText = '取消'
      confirmState.type = 'primary'
    } else {
      confirmState.title = options.title || '操作确认'
      confirmState.message = options.message
      confirmState.confirmText = options.confirmText || '确定'
      confirmState.cancelText = options.cancelText || '取消'
      confirmState.type = options.type || 'primary'
    }
    confirmState.resolve = resolve
    confirmState.open = true
  })
}

export interface PromptOptions {
  title?: string
  message: string
  defaultValue?: string
  placeholder?: string
  confirmText?: string
  cancelText?: string
}

export const promptState = reactive<{
  open: boolean
  title: string
  message: string
  inputValue: string
  placeholder: string
  confirmText: string
  cancelText: string
  resolve?: (value: string | null) => void
}>({
  open: false,
  title: '输入提示',
  message: '',
  inputValue: '',
  placeholder: '',
  confirmText: '确定',
  cancelText: '取消',
})

export function promptDialog(options: PromptOptions | string): Promise<string | null> {
  return new Promise((resolve) => {
    if (typeof options === 'string') {
      promptState.message = options
      promptState.title = '请输入'
      promptState.inputValue = ''
      promptState.placeholder = ''
      promptState.confirmText = '确定'
      promptState.cancelText = '取消'
    } else {
      promptState.title = options.title || '请输入'
      promptState.message = options.message
      promptState.inputValue = options.defaultValue || ''
      promptState.placeholder = options.placeholder || ''
      promptState.confirmText = options.confirmText || '确定'
      promptState.cancelText = options.cancelText || '取消'
    }
    promptState.resolve = resolve
    promptState.open = true
  })
}

// Backwards compatibility alias for ElMessageBox
export const ElMessageBox = {
  confirm: async (message: string, title?: string, options?: any) => {
    const ok = await confirmDialog({
      title: title || '操作确认',
      message,
      confirmText: options?.confirmButtonText || '确定',
      cancelText: options?.cancelButtonText || '取消',
      type: options?.type === 'warning' ? 'warning' : options?.type === 'danger' ? 'danger' : 'primary',
    })
    if (!ok) {
      throw new Error('cancel')
    }
    return 'confirm'
  },
  prompt: async (message: string, title?: string, options?: any) => {
    const res = await promptDialog({
      title: title || '请输入',
      message,
      defaultValue: options?.inputValue || '',
      confirmText: options?.confirmButtonText || '确定',
      cancelText: options?.cancelButtonText || '取消',
    })
    if (res === null) {
      throw new Error('cancel')
    }
    return { value: res, action: 'confirm' }
  },
}

// Backwards compatibility alias for ElMessage
export const ElMessage = {
  success: (msg: string) => toast.success(msg),
  error: (msg: string) => toast.error(msg),
  warning: (msg: string) => toast.warning(msg),
  info: (msg: string) => toast.info(msg),
}

export const ElNotification = {
  success: (opts: { title?: string; message?: string }) =>
    toast.success(opts.title || '成功', { description: opts.message }),
  error: (opts: { title?: string; message?: string }) =>
    toast.error(opts.title || '错误', { description: opts.message }),
  warning: (opts: { title?: string; message?: string }) =>
    toast.warning(opts.title || '警告', { description: opts.message }),
  info: (opts: { title?: string; message?: string }) =>
    toast.info(opts.title || '提示', { description: opts.message }),
}
