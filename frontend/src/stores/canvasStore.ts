import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Node, Edge, Connection } from '@vue-flow/core'
import dagre from 'dagre'
import { api } from '@/api'
import type { TableMeta, FieldMapping, CanvasView, CanvasTableMeta } from '@/types'
import { ElMessage } from 'element-plus'

export type HistoryAction =
  | {
      type: 'ADD_MAPPINGS'
      projectId: string
      mappings: Array<{
        id: string
        sourceTableId: string
        sourceColumnId: string
        targetTableId: string
        targetColumnId: string
        edge: Edge
      }>
    }
  | {
      type: 'REMOVE_MAPPINGS'
      projectId: string
      mappings: Array<{
        id: string
        sourceTableId: string
        sourceColumnId: string
        targetTableId: string
        targetColumnId: string
        edge: Edge
      }>
    }
  | {
      type: 'MOVE_NODE'
      nodeId: string
      tableId: string
      prevPos: { x: number; y: number }
      newPos: { x: number; y: number }
    }
  | {
      type: 'LAYOUT_NODES'
      prevPositions: Array<{ id: string; nodeId: string; pos: { x: number; y: number } }>
      newPositions: Array<{ id: string; nodeId: string; pos: { x: number; y: number } }>
    }

export const useCanvasStore = defineStore('canvas', () => {
  const nodes = ref<Node[]>([])
  const edges = ref<Edge[]>([])
  const selectedEdgeId = ref<string | null>(null)
  const highlightedNodeIds = ref<Set<string>>(new Set())
  const highlightedEdgeIds = ref<Set<string>>(new Set())

  // Undo / Redo History Stack (max 30 steps)
  const historyStack = ref<HistoryAction[]>([])
  const redoStack = ref<HistoryAction[]>([])
  const canUndo = computed(() => historyStack.value.length > 0)
  const canRedo = computed(() => redoStack.value.length > 0)
  const dragStartPosMap = new Map<string, { x: number; y: number }>()

  const pushHistory = (action: HistoryAction) => {
    redoStack.value = []
    historyStack.value.push(action)
    if (historyStack.value.length > 30) {
      historyStack.value.shift()
    }
  }

  // Multi-canvas / Subject View State
  const canvases = ref<CanvasView[]>([])
  const currentCanvasId = ref<string | null>(null)
  const currentCanvas = computed(() => canvases.value.find(c => c.id === currentCanvasId.value) || null)
  const loadingCanvas = ref(false)

  // Focused Table state for quick search
  const focusedTableId = ref<string | null>(null)
  let focusTimer: any = null

  // Active field and its counterpart columns for breathing animation & line visibility
  const activeColumnId = ref<string | null>(null)
  const associatedColumnIds = ref<Set<string>>(new Set())

  let savePositionTimer: any = null

  const persistPositions = async (updates: Array<{ id: string; pos_x: number; pos_y: number }>) => {
    if (updates.length === 0) return
    try {
      if (currentCanvasId.value) {
        await api.updateCanvasPositions(currentCanvasId.value, updates)
      } else {
        await api.updateTablePositions(updates)
      }
    } catch (e) {
      console.error('保存节点坐标失败', e)
    }
  }

  // Dynamically adapt edge handles: always connect via the closest handles
  const updateEdgeHandles = () => {
    const nodeMap = new Map(nodes.value.map(n => [n.id, n]))
    edges.value.forEach(edge => {
      const srcNode = nodeMap.get(edge.source)
      const tgtNode = nodeMap.get(edge.target)
      if (srcNode && tgtNode) {
        const srcColId = edge.data?.sourceColumnId
        const tgtColId = edge.data?.targetColumnId

        // If source node is to the left of target node:
        // Use source node's RIGHT handle (source_*) and target node's LEFT handle (target_*)
        if (srcNode.position.x <= tgtNode.position.x) {
          edge.sourceHandle = `source_${srcColId}`
          edge.targetHandle = `target_${tgtColId}`
        } else {
          // If source node is to the right of target node:
          // Use source node's LEFT handle (target_*) and target node's RIGHT handle (source_*)
          edge.sourceHandle = `target_${srcColId}`
          edge.targetHandle = `source_${tgtColId}`
        }
      }
    })
  }

  // Transform tables & mappings from project into Vue Flow format
  const syncFromProject = (tables: TableMeta[] = [], mappings: FieldMapping[] = []) => {
    // Clear active state
    activeColumnId.value = null
    associatedColumnIds.value.clear()

    // 1. Convert tables to nodes
    nodes.value = tables.map(t => ({
      id: `table_${t.id}`,
      type: 'tableCard',
      position: { x: t.pos_x, y: t.pos_y },
      data: {
        tableId: t.id,
        tableName: t.table_name,
        tableComment: t.table_comment || '',
        columns: t.columns || [],
        colorTag: (t as any).color_tag || 'blue',
      },
    }))

    // 2. Convert mappings to edges (Strictly hidden by default)
    const tableMap = new Map(tables.map(t => [t.id, t]))
    const colMap = new Map()
    for (const t of tables) {
      for (const c of t.columns) {
        colMap.set(c.id, c)
      }
    }

    edges.value = mappings.map(m => {
      const srcTable = tableMap.get(m.source_table_id)
      const srcCol = colMap.get(m.source_column_id)
      const tgtTable = tableMap.get(m.target_table_id)
      const tgtCol = colMap.get(m.target_column_id)

      const label = `${srcTable?.table_name || ''}.${srcCol?.column_name || ''} → ${tgtTable?.table_name || ''}.${tgtCol?.column_name || ''}`

      return {
        id: `edge_${m.id}`,
        source: `table_${m.source_table_id}`,
        sourceHandle: `source_${m.source_column_id}`,
        target: `table_${m.target_table_id}`,
        targetHandle: `target_${m.target_column_id}`,
        type: 'smoothstep',
        animated: false,
        hidden: true, // Hidden by default, only shown when field is activated
        data: {
          mappingId: m.id,
          sourceTableId: m.source_table_id,
          sourceColumnId: m.source_column_id,
          targetTableId: m.target_table_id,
          targetColumnId: m.target_column_id,
          label,
        },
        style: {
          stroke: '#2563eb',
          strokeWidth: 2.5,
        },
      }
    })

    // Compute initial shortest path handles
    updateEdgeHandles()
  }

  // Handle node dragging: dynamically update handles in real-time
  const onNodeDragStart = (e: any) => {
    if (e?.node) {
      dragStartPosMap.set(e.node.id, { x: e.node.position.x, y: e.node.position.y })
    }
  }

  const onNodeDrag = () => {
    updateEdgeHandles()
  }

  // Handle node drag end - debounce position saving to backend
  const onNodeDragStop = (e?: any) => {
    updateEdgeHandles()

    if (e?.node) {
      const startPos = dragStartPosMap.get(e.node.id)
      if (startPos) {
        const dx = Math.abs(startPos.x - e.node.position.x)
        const dy = Math.abs(startPos.y - e.node.position.y)
        if (dx > 2 || dy > 2) {
          pushHistory({
            type: 'MOVE_NODE',
            nodeId: e.node.id,
            tableId: e.node.data.tableId,
            prevPos: { ...startPos },
            newPos: { x: Math.round(e.node.position.x), y: Math.round(e.node.position.y) },
          })
        }
        dragStartPosMap.delete(e.node.id)
      }
    }

    if (savePositionTimer) {
      clearTimeout(savePositionTimer)
    }
    savePositionTimer = setTimeout(async () => {
      const updates = nodes.value.map(n => ({
        id: n.data.tableId,
        pos_x: Math.round(n.position.x),
        pos_y: Math.round(n.position.y),
      }))
      if (updates.length > 0) {
        await persistPositions(updates)
      }
    }, 400)
  }

  // Select a field to activate: highlight field pair + show edges + auto-scroll
  const selectColumn = (columnId: string | null) => {
    // If clicking same column or null, toggle off
    if (!columnId || activeColumnId.value === columnId) {
      activeColumnId.value = null
      associatedColumnIds.value.clear()
      edges.value.forEach(e => {
        e.hidden = true
        e.animated = false
      })
      return
    }

    // Find all edges that involve this column
    const matchingEdges = edges.value.filter(e =>
      e.data?.sourceColumnId === columnId || e.data?.targetColumnId === columnId
    )

    if (matchingEdges.length === 0) {
      // Field has no mappings: clear active state and hide lines
      activeColumnId.value = null
      associatedColumnIds.value.clear()
      edges.value.forEach(e => {
        e.hidden = true
        e.animated = false
      })
      return
    }

    activeColumnId.value = columnId
    const matchedCols = new Set<string>()

    // Make matching edges visible and animated
    edges.value.forEach(e => {
      const isMatch = e.data?.sourceColumnId === columnId || e.data?.targetColumnId === columnId
      if (isMatch) {
        e.hidden = false
        e.animated = true
        e.style = {
          stroke: '#2563eb',
          strokeWidth: 3,
        }
        if (e.data?.sourceColumnId) matchedCols.add(e.data.sourceColumnId)
        if (e.data?.targetColumnId) matchedCols.add(e.data.targetColumnId)
      } else {
        e.hidden = true
        e.animated = false
      }
    })

    associatedColumnIds.value = matchedCols
    updateEdgeHandles()

    // Auto-scroll associated table cards so the matched column is smoothly brought into view
    setTimeout(() => {
      matchedCols.forEach(cId => {
        if (cId !== columnId) {
          const el = document.getElementById(`col_item_${cId}`)
          if (el) {
            el.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
          }
        }
      })
    }, 60)
  }

  // Create connection from field handle drag
  const onConnect = async (params: Connection, projectId: string) => {
    const sourceTableId = params.source.replace('table_', '')
    const targetTableId = params.target.replace('table_', '')
    const sourceColumnId = (params.sourceHandle || '').replace('source_', '').replace('target_', '')
    const targetColumnId = (params.targetHandle || '').replace('target_', '').replace('source_', '')

    if (!sourceColumnId || !targetColumnId) {
      ElMessage.warning('未能识别字段端点')
      return
    }

    if (sourceTableId === targetTableId && sourceColumnId === targetColumnId) {
      ElMessage.warning('不能将字段映射到其自身')
      return
    }

    try {
      const created = await api.createMapping(projectId, {
        source_table_id: sourceTableId,
        source_column_id: sourceColumnId,
        target_table_id: targetTableId,
        target_column_id: targetColumnId,
      })

      // Check if edge already displayed
      const edgeId = `edge_${created.id}`
      if (edges.value.some(e => e.id === edgeId)) {
        ElMessage.info('该映射已存在')
        return
      }

      const newEdge: Edge = {
        id: edgeId,
        source: params.source,
        sourceHandle: params.sourceHandle,
        target: params.target,
        targetHandle: params.targetHandle,
        type: 'smoothstep',
        animated: true,
        hidden: false, // Show immediately upon creation
        data: {
          mappingId: created.id,
          sourceTableId,
          sourceColumnId,
          targetTableId,
          targetColumnId,
        },
        style: {
          stroke: '#2563eb',
          strokeWidth: 3,
        },
      }

      edges.value.push(newEdge)
      updateEdgeHandles()
      selectColumn(sourceColumnId)

      pushHistory({
        type: 'ADD_MAPPINGS',
        projectId,
        mappings: [{
          id: created.id,
          sourceTableId,
          sourceColumnId,
          targetTableId,
          targetColumnId,
          edge: newEdge,
        }],
      })

      ElMessage.success('建立字段映射成功')
    } catch (err: any) {
      ElMessage.error('创建映射失败: ' + (err.message || '网络错误'))
    }
  }

  // Batch auto mapping
  const createBatchMappings = async (
    projectId: string,
    pairs: Array<{ sourceTableId: string; sourceColumnId: string; targetTableId: string; targetColumnId: string }>
  ) => {
    if (pairs.length === 0) return 0
    try {
      const created = await api.batchCreateMappings(projectId, pairs.map(p => ({
        source_table_id: p.sourceTableId,
        source_column_id: p.sourceColumnId,
        target_table_id: p.targetTableId,
        target_column_id: p.targetColumnId,
      })))

      const historyMappings: any[] = []

      for (const m of created) {
        const edgeId = `edge_${m.id}`
        if (edges.value.some(e => e.id === edgeId)) continue

        const edge: Edge = {
          id: edgeId,
          source: `table_${m.source_table_id}`,
          sourceHandle: `source_${m.source_column_id}`,
          target: `table_${m.target_table_id}`,
          targetHandle: `target_${m.target_column_id}`,
          type: 'smoothstep',
          animated: true,
          hidden: false,
          data: {
            mappingId: m.id,
            sourceTableId: m.source_table_id,
            sourceColumnId: m.source_column_id,
            targetTableId: m.target_table_id,
            targetColumnId: m.target_column_id,
          },
          style: {
            stroke: '#2563eb',
            strokeWidth: 3,
          },
        }
        edges.value.push(edge)
        historyMappings.push({
          id: m.id,
          sourceTableId: m.source_table_id,
          sourceColumnId: m.source_column_id,
          targetTableId: m.target_table_id,
          targetColumnId: m.target_column_id,
          edge,
        })
      }

      updateEdgeHandles()

      if (historyMappings.length > 0) {
        pushHistory({
          type: 'ADD_MAPPINGS',
          projectId,
          mappings: historyMappings,
        })
      }

      return created.length
    } catch (e: any) {
      ElMessage.error('批量创建映射失败: ' + (e.message || '网络错误'))
      return 0
    }
  }

  // Delete edge
  const removeEdge = async (edgeId: string, projectId?: string) => {
    const edge = edges.value.find(e => e.id === edgeId)
    if (!edge || !edge.data?.mappingId) {
      edges.value = edges.value.filter(e => e.id !== edgeId)
      return
    }

    try {
      await api.deleteMapping(edge.data.mappingId)
      edges.value = edges.value.filter(e => e.id !== edgeId)

      pushHistory({
        type: 'REMOVE_MAPPINGS',
        projectId: projectId || '',
        mappings: [{
          id: edge.data.mappingId,
          sourceTableId: edge.data.sourceTableId,
          sourceColumnId: edge.data.sourceColumnId,
          targetTableId: edge.data.targetTableId,
          targetColumnId: edge.data.targetColumnId,
          edge: { ...edge },
        }],
      })

      // If deleted edge was active, re-evaluate selection
      if (activeColumnId.value) {
        selectColumn(activeColumnId.value)
      }
      ElMessage.success('删除映射成功')
    } catch (err: any) {
      ElMessage.error('删除映射失败: ' + (err.message || '网络错误'))
    }
  }

  // Dagre auto layout
  const applyAutoLayout = async (direction: 'LR' | 'TB' = 'LR') => {
    if (nodes.value.length === 0) return

    const prevPositions = nodes.value.map(n => ({
      id: n.data.tableId,
      nodeId: n.id,
      pos: { ...n.position },
    }))

    const dagreGraph = new dagre.graphlib.Graph()
    dagreGraph.setDefaultEdgeLabel(() => ({}))
    dagreGraph.setGraph({
      rankdir: direction,
      align: 'UL',
      nodesep: 80,
      ranksep: 220,
      marginx: 50,
      marginy: 50,
    })

    const nodeWidth = 330
    nodes.value.forEach(node => {
      const colCount = node.data.columns ? node.data.columns.length : 0
      const nodeHeight = Math.min(Math.max(120 + colCount * 34, 180), 600)
      dagreGraph.setNode(node.id, { width: nodeWidth, height: nodeHeight })
    })

    edges.value.forEach(edge => {
      dagreGraph.setEdge(edge.source, edge.target)
    })

    dagre.layout(dagreGraph)

    const updates = nodes.value.map(node => {
      const nodeWithPosition = dagreGraph.node(node.id)
      const colCount = node.data.columns ? node.data.columns.length : 0
      const nodeHeight = Math.min(Math.max(120 + colCount * 34, 180), 600)

      const posX = Math.round(nodeWithPosition.x - nodeWidth / 2)
      const posY = Math.round(nodeWithPosition.y - nodeHeight / 2)

      node.position = { x: posX, y: posY }

      return {
        id: node.data.tableId,
        pos_x: posX,
        pos_y: posY,
      }
    })

    const newPositions = nodes.value.map(n => ({
      id: n.data.tableId,
      nodeId: n.id,
      pos: { ...n.position },
    }))

    pushHistory({
      type: 'LAYOUT_NODES',
      prevPositions,
      newPositions,
    })

    updateEdgeHandles()

    // Batch persist to backend
    try {
      await persistPositions(updates)
      ElMessage.success('智能排版布局完成')
    } catch (e) {
      console.error('保存自动排版坐标失败', e)
    }
  }

  // Undo & Redo execution
  const undo = async () => {
    if (historyStack.value.length === 0) return
    const action = historyStack.value.pop()!

    try {
      if (action.type === 'ADD_MAPPINGS') {
        const ids = action.mappings.map(m => m.id)
        await api.batchDeleteMappings(ids)
        edges.value = edges.value.filter(e => !ids.includes(e.data?.mappingId))
        updateEdgeHandles()
        redoStack.value.push(action)
        ElMessage.info('已撤销新建连线')
      } else if (action.type === 'REMOVE_MAPPINGS') {
        const created = await api.batchCreateMappings(
          action.projectId,
          action.mappings.map(m => ({
            source_table_id: m.sourceTableId,
            source_column_id: m.sourceColumnId,
            target_table_id: m.targetTableId,
            target_column_id: m.targetColumnId,
          }))
        )
        for (let i = 0; i < created.length; i++) {
          const c = created[i]
          const orig = action.mappings[i]
          orig.id = c.id
          edges.value.push({
            ...orig.edge,
            id: `edge_${c.id}`,
            data: {
              ...orig.edge.data,
              mappingId: c.id,
            },
          })
        }
        updateEdgeHandles()
        redoStack.value.push(action)
        ElMessage.info('已撤销删除连线')
      } else if (action.type === 'MOVE_NODE') {
        const node = nodes.value.find(n => n.id === action.nodeId)
        if (node) {
          node.position = { ...action.prevPos }
          await persistPositions([{ id: action.tableId, pos_x: action.prevPos.x, pos_y: action.prevPos.y }])
          updateEdgeHandles()
        }
        redoStack.value.push(action)
        ElMessage.info('已撤销卡片移动')
      } else if (action.type === 'LAYOUT_NODES') {
        for (const item of action.prevPositions) {
          const node = nodes.value.find(n => n.id === item.nodeId)
          if (node) {
            node.position = { ...item.pos }
          }
        }
        await persistPositions(action.prevPositions.map(p => ({ id: p.id, pos_x: p.pos.x, pos_y: p.pos.y })))
        updateEdgeHandles()
        redoStack.value.push(action)
        ElMessage.info('已撤销自动排版')
      }
    } catch (e: any) {
      ElMessage.error('撤销失败: ' + (e.message || '网络错误'))
    }
  }

  const redo = async () => {
    if (redoStack.value.length === 0) return
    const action = redoStack.value.pop()!

    try {
      if (action.type === 'ADD_MAPPINGS') {
        const created = await api.batchCreateMappings(
          action.projectId,
          action.mappings.map(m => ({
            source_table_id: m.sourceTableId,
            source_column_id: m.sourceColumnId,
            target_table_id: m.targetTableId,
            target_column_id: m.targetColumnId,
          }))
        )
        for (let i = 0; i < created.length; i++) {
          const c = created[i]
          const orig = action.mappings[i]
          orig.id = c.id
          edges.value.push({
            ...orig.edge,
            id: `edge_${c.id}`,
            data: {
              ...orig.edge.data,
              mappingId: c.id,
            },
          })
        }
        updateEdgeHandles()
        historyStack.value.push(action)
        ElMessage.info('已重做新建连线')
      } else if (action.type === 'REMOVE_MAPPINGS') {
        const ids = action.mappings.map(m => m.id)
        await api.batchDeleteMappings(ids)
        edges.value = edges.value.filter(e => !ids.includes(e.data?.mappingId))
        updateEdgeHandles()
        historyStack.value.push(action)
        ElMessage.info('已重做删除连线')
      } else if (action.type === 'MOVE_NODE') {
        const node = nodes.value.find(n => n.id === action.nodeId)
        if (node) {
          node.position = { ...action.newPos }
          await persistPositions([{ id: action.tableId, pos_x: action.newPos.x, pos_y: action.newPos.y }])
          updateEdgeHandles()
        }
        historyStack.value.push(action)
        ElMessage.info('已重做卡片移动')
      } else if (action.type === 'LAYOUT_NODES') {
        for (const item of action.newPositions) {
          const node = nodes.value.find(n => n.id === item.nodeId)
          if (node) {
            node.position = { ...item.pos }
          }
        }
        await persistPositions(action.newPositions.map(p => ({ id: p.id, pos_x: p.pos.x, pos_y: p.pos.y })))
        updateEdgeHandles()
        historyStack.value.push(action)
        ElMessage.info('已重做自动排版')
      }
    } catch (e: any) {
      ElMessage.error('重做失败: ' + (e.message || '网络错误'))
    }
  }

  // Highlight lineage for a selected table or edge
  const highlightLineage = (tableId?: string) => {
    if (!tableId) {
      highlightedNodeIds.value.clear()
      highlightedEdgeIds.value.clear()
      return
    }

    const startNodeId = `table_${tableId}`
    const relatedNodes = new Set<string>([startNodeId])
    const relatedEdges = new Set<string>()

    // Traverse upstream
    const findUpstream = (currNode: string) => {
      edges.value.forEach(edge => {
        if (edge.target === currNode) {
          relatedEdges.add(edge.id)
          if (!relatedNodes.has(edge.source)) {
            relatedNodes.add(edge.source)
            findUpstream(edge.source)
          }
        }
      })
    }

    // Traverse downstream
    const findDownstream = (currNode: string) => {
      edges.value.forEach(edge => {
        if (edge.source === currNode) {
          relatedEdges.add(edge.id)
          if (!relatedNodes.has(edge.target)) {
            relatedNodes.add(edge.target)
            findDownstream(edge.target)
          }
        }
      })
    }

    findUpstream(startNodeId)
    findDownstream(startNodeId)

    highlightedNodeIds.value = relatedNodes
    highlightedEdgeIds.value = relatedEdges
  }

  const clearHighlight = () => {
    highlightedNodeIds.value.clear()
    highlightedEdgeIds.value.clear()
  }

  // Focus table for quick search
  const focusTable = (tableId: string) => {
    focusedTableId.value = tableId
    if (focusTimer) clearTimeout(focusTimer)
    focusTimer = setTimeout(() => {
      focusedTableId.value = null
    }, 2500)
  }

  // Multi-canvas operations
  const fetchCanvases = async (projectId: string, autoSelectFirst = true) => {
    try {
      const list = await api.getCanvases(projectId)
      canvases.value = list
      if (list.length > 0 && (autoSelectFirst || !currentCanvasId.value || !list.some(c => c.id === currentCanvasId.value))) {
        await switchCanvas(list[0].id)
      }
    } catch (e) {
      console.error('加载画布列表失败', e)
    }
  }

  const switchCanvas = async (canvasId: string) => {
    loadingCanvas.value = true
    currentCanvasId.value = canvasId
    try {
      const detail = await api.getCanvasDetail(canvasId)
      syncFromProject(detail.tables, detail.mappings)
    } catch (e) {
      console.error('加载画布详情失败', e)
    } finally {
      loadingCanvas.value = false
    }
  }

  const createCanvas = async (projectId: string, name: string) => {
    try {
      const created = await api.createCanvas(projectId, { name })
      ElMessage.success('新建业务视图成功')
      await fetchCanvases(projectId, false)
      await switchCanvas(created.id)
      return created
    } catch (e: any) {
      ElMessage.error('新建画布失败: ' + (e.message || '网络错误'))
      throw e
    }
  }

  const updateCanvasName = async (canvasId: string, name: string) => {
    try {
      const updated = await api.updateCanvas(canvasId, { name })
      const item = canvases.value.find(c => c.id === canvasId)
      if (item) item.name = updated.name
      ElMessage.success('画布已重命名')
    } catch (e: any) {
      ElMessage.error('重命名画布失败: ' + (e.message || '网络错误'))
    }
  }

  const deleteCanvas = async (canvasId: string, projectId: string) => {
    try {
      await api.deleteCanvas(canvasId)
      ElMessage.success('画布已删除')
      currentCanvasId.value = null
      await fetchCanvases(projectId, true)
    } catch (e: any) {
      ElMessage.error('删除画布失败: ' + (e.message || '网络错误'))
    }
  }

  const addTablesToCurrentCanvas = async (tableIds: string[]) => {
    if (!currentCanvasId.value) return
    try {
      await api.addTablesToCanvas(currentCanvasId.value, tableIds)
      await switchCanvas(currentCanvasId.value)
      ElMessage.success(`成功引入 ${tableIds.length} 张表到当前画布`)
    } catch (e: any) {
      ElMessage.error('引入表失败: ' + (e.message || '网络错误'))
    }
  }

  const removeTableFromCurrentCanvas = async (tableId: string) => {
    if (!currentCanvasId.value) return
    try {
      await api.removeTableFromCanvas(currentCanvasId.value, tableId)
      nodes.value = nodes.value.filter(n => n.data.tableId !== tableId)
      edges.value = edges.value.filter(e => e.data.sourceTableId !== tableId && e.data.targetTableId !== tableId)
      updateEdgeHandles()
      ElMessage.success('已从当前画布移出')
    } catch (e: any) {
      ElMessage.error('移出表失败: ' + (e.message || '网络错误'))
    }
  }

  const updateTableColor = async (tableId: string, colorTag: string) => {
    const node = nodes.value.find(n => n.data.tableId === tableId)
    if (node) {
      node.data.colorTag = colorTag
    }
    if (currentCanvasId.value) {
      try {
        await api.updateCanvasTableColor(currentCanvasId.value, tableId, colorTag)
      } catch (e) {
        console.error('更新表分层颜色失败', e)
      }
    }
  }

  return {
    nodes,
    edges,
    selectedEdgeId,
    highlightedNodeIds,
    highlightedEdgeIds,
    activeColumnId,
    associatedColumnIds,
    canvases,
    currentCanvasId,
    currentCanvas,
    loadingCanvas,
    focusedTableId,
    focusTable,
    fetchCanvases,
    switchCanvas,
    createCanvas,
    updateCanvasName,
    deleteCanvas,
    addTablesToCurrentCanvas,
    removeTableFromCurrentCanvas,
    updateTableColor,
    selectColumn,
    updateEdgeHandles,
    syncFromProject,
    historyStack,
    redoStack,
    canUndo,
    canRedo,
    undo,
    redo,
    onNodeDragStart,
    createBatchMappings,
    onNodeDrag,
    onNodeDragStop,
    onConnect,
    removeEdge,
    applyAutoLayout,
    highlightLineage,
    clearHighlight,
  }
})
