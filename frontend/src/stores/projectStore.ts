import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/api'
import type { Project } from '@/types'
import { toast } from '@/utils/toast'

export const useProjectStore = defineStore('project', () => {
  const projects = ref<Project[]>([])
  const currentProject = ref<Project | null>(null)
  const loading = ref(false)

  const fetchProjects = async () => {
    loading.value = true
    try {
      const list = await api.getProjects()
      projects.value = list
      if (!currentProject.value && list.length > 0) {
        await selectProject(list[0].id)
      }
    } catch (err: any) {
      toast.error('获取项目列表失败: ' + (err.message || '网络错误'))
    } finally {
      loading.value = false
    }
  }

  const selectProject = async (id: string) => {
    loading.value = true
    try {
      const detail = await api.getProjectDetail(id)
      currentProject.value = detail
    } catch (err: any) {
      toast.error('加载项目详情失败: ' + (err.message || '网络错误'))
    } finally {
      loading.value = false
    }
  }

  const createProject = async (name: string, description?: string) => {
    try {
      const created = await api.createProject({ name, description })
      toast.success('创建项目成功')
      await fetchProjects()
      await selectProject(created.id)
      return created
    } catch (err: any) {
      toast.error('创建项目失败: ' + (err.message || '网络错误'))
      throw err
    }
  }

  const updateProject = async (id: string, data: { name?: string; description?: string }) => {
    try {
      const updated = await api.updateProject(id, data)
      toast.success('更新项目成功')
      if (currentProject.value && currentProject.value.id === id) {
        currentProject.value.name = updated.name
        currentProject.value.description = updated.description
      }
      await fetchProjects()
    } catch (err: any) {
      toast.error('更新项目失败: ' + (err.message || '网络错误'))
      throw err
    }
  }

  const deleteProject = async (id: string) => {
    try {
      await api.deleteProject(id)
      toast.success('删除项目成功')
      currentProject.value = null
      await fetchProjects()
    } catch (err: any) {
      toast.error('删除项目失败: ' + (err.message || '网络错误'))
      throw err
    }
  }

  return {
    projects,
    currentProject,
    loading,
    fetchProjects,
    selectProject,
    createProject,
    updateProject,
    deleteProject,
  }
})
