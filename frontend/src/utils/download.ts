import axios from 'axios'
import { toast } from '@/utils/toast'

/**
 * Extract filename from Content-Disposition header
 */
function extractFileName(contentDisposition?: string, defaultName: string = 'download'): string {
  if (!contentDisposition) return defaultName

  // Try RFC 5987: filename*=UTF-8''encoded_name
  const rfc5987Match = contentDisposition.match(/filename\*=(?:UTF-8|utf-8)''([^;]+)/i)
  if (rfc5987Match && rfc5987Match[1]) {
    try {
      return decodeURIComponent(rfc5987Match[1].trim())
    } catch {
      return rfc5987Match[1].trim()
    }
  }

  // Try standard: filename="name" or filename=name
  const standardMatch = contentDisposition.match(/filename="?([^";]+)"?/i)
  if (standardMatch && standardMatch[1]) {
    try {
      return decodeURIComponent(standardMatch[1].trim())
    } catch {
      return standardMatch[1].trim()
    }
  }

  return defaultName
}

/**
 * Download a file from an API endpoint as a Blob using the unified Axios client
 */
export async function downloadFileStream(
  url: string,
  fallbackFileName: string = 'download'
): Promise<void> {
  const toastId = toast.loading('正在准备导出文件，请稍候...')

  try {
    const response = await axios.get(url, {
      baseURL: '/api',
      responseType: 'blob',
      timeout: 60000,
    })

    // If backend returned error JSON in a Blob
    if (response.data?.type === 'application/json') {
      const text = await response.data.text()
      try {
        const json = JSON.parse(text)
        throw new Error(json.detail || json.message || '导出失败')
      } catch (err: any) {
        if (err.message && err.message !== '导出失败') throw err
      }
    }

    const disposition =
      response.headers['content-disposition'] || response.headers['Content-Disposition']
    const fileName = extractFileName(disposition, fallbackFileName)

    const blob = new Blob([response.data], {
      type: response.headers['content-type'] || 'application/octet-stream',
    })

    const blobUrl = window.URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.style.display = 'none'
    link.href = blobUrl
    link.setAttribute('download', fileName)
    document.body.appendChild(link)
    link.click()

    setTimeout(() => {
      document.body.removeChild(link)
      window.URL.revokeObjectURL(blobUrl)
    }, 300)

    toast.success('文件导出成功', { id: toastId })
  } catch (error: any) {
    const errMsg = error.response?.data?.detail || error.message || '导出文件时发生网络异常'
    toast.error('导出失败: ' + errMsg, { id: toastId })
    throw error
  }
}

/**
 * Download project JSON backup snapshot
 */
export async function downloadJsonFile(
  url: string,
  fallbackFileName: string = 'backup.json'
): Promise<void> {
  const toastId = toast.loading('正在生成 JSON 备份快照...')

  try {
    const response = await axios.get(url, {
      baseURL: '/api',
      timeout: 60000,
    })

    const jsonStr =
      typeof response.data === 'string' ? response.data : JSON.stringify(response.data, null, 2)
    const blob = new Blob([jsonStr], { type: 'application/json;charset=utf-8' })
    const blobUrl = window.URL.createObjectURL(blob)

    const link = document.createElement('a')
    link.style.display = 'none'
    link.href = blobUrl
    link.setAttribute('download', fallbackFileName)
    document.body.appendChild(link)
    link.click()

    setTimeout(() => {
      document.body.removeChild(link)
      window.URL.revokeObjectURL(blobUrl)
    }, 300)

    toast.success('JSON 备份快照下载成功', { id: toastId })
  } catch (error: any) {
    const errMsg = error.response?.data?.detail || error.message || '导出 JSON 备份失败'
    toast.error('导出失败: ' + errMsg, { id: toastId })
    throw error
  }
}
