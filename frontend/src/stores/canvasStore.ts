import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Node, Edge, Connection } from '@vue-flow/core'
import { computeDagLayeredLayout } from '@/utils/dagLayout'
import { api } from '@/api'
import type { TableMeta, FieldMapping, CanvasView, CanvasTableMeta, RelationPath } from '@/types'
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
  const activeRelationPath = ref<RelationPath | null>(null)
  const pickingTableRole = ref<'source' | 'target' | null>(null)

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
  let updateNodeInternalsFn: ((nodeIds: string | string[]) => void) | null = null

  const registerUpdateNodeInternals = (fn: (nodeIds: string | string[]) => void) => {
    updateNodeInternalsFn = fn
  }

  const triggerUpdateNodeInternals = (nodeIds?: string | string[]) => {
    if (updateNodeInternalsFn) {
      if (nodeIds) {
        updateNodeInternalsFn(nodeIds)
      } else {
        updateNodeInternalsFn(nodes.value.map(n => n.id))
      }
    }
  }

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
    const sw = 320
    const tw = 320

    const parseColId = (h: string | null | undefined): string => {
      if (!h) return ''
      return h.replace(/^(source|target)_/, '').replace(/_(left|right)$/, '')
    }

    edges.value.forEach(edge => {
      const srcNode = nodeMap.get(edge.source)
      const tgtNode = nodeMap.get(edge.target)
      if (srcNode && tgtNode) {
        const srcColId = parseColId(edge.sourceHandle) || edge.data?.sourceColumnId
        const tgtColId = parseColId(edge.targetHandle) || edge.data?.targetColumnId

        if (!srcColId || !tgtColId) return

        const sx = srcNode.position.x
        const tx = tgtNode.position.x
        const srcW = (srcNode as any).dimensions?.width || sw
        const tgtW = (tgtNode as any).dimensions?.width || tw

        const srcCenterX = sx + srcW / 2
        const tgtCenterX = tx + tgtW / 2
        const dx = tgtCenterX - srcCenterX

        let newSrcSide: 'left' | 'right' = 'right'
        let newTgtSide: 'left' | 'right' = 'left'

        // If target node is clearly to the right of source node
        if (dx > 30) {
          newSrcSide = 'right'
          newTgtSide = 'left'
        }
        // If target node is clearly to the left of source node
        else if (dx < -30) {
          newSrcSide = 'left'
          newTgtSide = 'right'
        }
        // Vertically stacked in the same column (clean right-side C-curve)
        else {
          newSrcSide = 'right'
          newTgtSide = 'right'
        }

        const newSrcHandle = `source_${srcColId}_${newSrcSide}`
        const newTgtHandle = `target_${tgtColId}_${newTgtSide}`

        // Only assign when handle value actually changes to avoid triggering reactive proxy cascades
        if (edge.sourceHandle !== newSrcHandle) {
          edge.sourceHandle = newSrcHandle
        }
        if (edge.targetHandle !== newTgtHandle) {
          edge.targetHandle = newTgtHandle
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
        sourceHandle: `source_${m.source_column_id}_right`,
        target: `table_${m.target_table_id}`,
        targetHandle: `target_${m.target_column_id}_left`,
        type: 'smartObstacle',
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
          stroke: '#3b82f6',
          strokeWidth: 2,
        },
      }
    })

    // Compute initial shortest path handles
    updateEdgeHandles()
    triggerUpdateNodeInternals()
    setTimeout(() => {
      triggerUpdateNodeInternals()
    }, 100)
  }

  // Handle node dragging: dynamically update handles in real-time throttled by rAF
  let nodeDragRafId: number | null = null

  const onNodeDragStart = (e: any) => {
    if (e?.node) {
      dragStartPosMap.set(e.node.id, { x: e.node.position.x, y: e.node.position.y })
    }
  }

  const onNodeDrag = () => {
    if (nodeDragRafId !== null) return
    nodeDragRafId = requestAnimationFrame(() => {
      updateEdgeHandles()
    triggerUpdateNodeInternals()
      nodeDragRafId = null
    })
  }

  // Handle node drag end - debounce position saving to backend
  const onNodeDragStop = (e?: any) => {
    if (nodeDragRafId !== null) {
      cancelAnimationFrame(nodeDragRafId)
      nodeDragRafId = null
    }
    updateEdgeHandles()
    triggerUpdateNodeInternals()

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
        e.type = 'smartObstacle'
        e.style = {
          stroke: '#3b82f6',
          strokeWidth: 2,
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
    triggerUpdateNodeInternals()

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
      triggerUpdateNodeInternals()
    }, 60)

    // Re-sync edge positions once smooth scroll settles
    setTimeout(() => {
      updateEdgeHandles()
    triggerUpdateNodeInternals()
      triggerUpdateNodeInternals()
    }, 350)
  }

  // Create connection from field handle drag
  const onConnect = async (params: Connection, projectId: string) => {
    const sourceTableId = params.source.replace('table_', '')
    const targetTableId = params.target.replace('table_', '')
    const parseColId = (h: string | null | undefined): string => {
      if (!h) return ''
      return h.replace(/^(source|target)_/, '').replace(/_(left|right)$/, '')
    }
    const sourceColumnId = parseColId(params.sourceHandle)
    const targetColumnId = parseColId(params.targetHandle)

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

      const srcNode = nodes.value.find(n => n.id === params.source)
      const tgtNode = nodes.value.find(n => n.id === params.target)
      const srcCol = srcNode?.data?.columns?.find((c: any) => c.id === sourceColumnId)
      const tgtCol = tgtNode?.data?.columns?.find((c: any) => c.id === targetColumnId)
      const label = `${srcNode?.data?.tableName || ''}.${srcCol?.column_name || ''} → ${tgtNode?.data?.tableName || ''}.${tgtCol?.column_name || ''}`

      const newEdge: Edge = {
        id: edgeId,
        source: params.source,
        sourceHandle: params.sourceHandle,
        target: params.target,
        targetHandle: params.targetHandle,
        type: 'smartObstacle',
        animated: true,
        hidden: false, // Show immediately upon creation
        data: {
          mappingId: created.id,
          sourceTableId,
          sourceColumnId,
          targetTableId,
          targetColumnId,
          label,
        },
        style: {
          stroke: '#3b82f6',
          strokeWidth: 2,
        },
      }

      edges.value.push(newEdge)
      updateEdgeHandles()
    triggerUpdateNodeInternals()
      triggerUpdateNodeInternals([params.source, params.target])
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
          sourceHandle: `source_${m.source_column_id}_right`,
          target: `table_${m.target_table_id}`,
          targetHandle: `target_${m.target_column_id}_left`,
          type: 'smartObstacle',
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
            stroke: '#3b82f6',
            strokeWidth: 2,
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
    triggerUpdateNodeInternals()

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

  // Smoothly interpolate node coordinates using requestAnimationFrame
  const animateNodePositions = (
    targets: Array<{ nodeId: string; targetX: number; targetY: number }>,
    duration = 450
  ): Promise<void> => {
    return new Promise(resolve => {
      const nodeMap = new Map(nodes.value.map(n => [n.id, n]))
      const startItems = targets
        .map(t => {
          const node = nodeMap.get(t.nodeId)
          if (!node) return null
          return {
            node,
            startX: node.position.x,
            startY: node.position.y,
            targetX: t.targetX,
            targetY: t.targetY,
          }
        })
        .filter((item): item is NonNullable<typeof item> => item !== null)

      if (startItems.length === 0) {
        resolve()
        return
      }

      const startTime = performance.now()

      const tick = (currentTime: number) => {
        const elapsed = currentTime - startTime
        const progress = Math.min(elapsed / duration, 1.0)
        // Deceleration ease-out cubic curve
        const ease = 1 - Math.pow(1 - progress, 3)

        startItems.forEach(item => {
          item.node.position = {
            x: Math.round(item.startX + (item.targetX - item.startX) * ease),
            y: Math.round(item.startY + (item.targetY - item.startY) * ease),
          }
        })

        updateEdgeHandles()
    triggerUpdateNodeInternals()

        if (progress < 1.0) {
          requestAnimationFrame(tick)
        } else {
          resolve()
        }
      }

      requestAnimationFrame(tick)
    })
  }

  // DAG Directed Layered Flow Layout (Left-to-Right)
  const applyAutoLayout = async (_direction: 'LR' | 'TB' = 'LR') => {
    if (nodes.value.length === 0) return

    // 1. Snapshot previous positions for undo history
    const prevPositions = nodes.value.map(n => ({
      id: n.data.tableId,
      nodeId: n.id,
      pos: { ...n.position },
    }))

    // 2. Prepare nodes and edges for DAG computation
    const dagNodes = nodes.value.map(n => ({
      id: n.id,
      columnsCount: n.data.columns ? n.data.columns.length : 0,
      data: n.data,
    }))

    const dagEdges = edges.value.map(e => ({
      source: e.source,
      target: e.target,
    }))

    // 3. Compute Left-to-Right Layered Layout
    const { nodePositions } = computeDagLayeredLayout(dagNodes, dagEdges, {
      columnSpacing: 460, // 320px card + 140px corridor for smooth S-curves
      rowSpacing: 48,
      startX: 80,
      startY: 80,
    })

    const targetList: Array<{ nodeId: string; targetX: number; targetY: number }> = []
    const updates: Array<{ id: string; pos_x: number; pos_y: number }> = []

    nodes.value.forEach(node => {
      const targetPos = nodePositions.get(node.id)
      if (targetPos) {
        targetList.push({
          nodeId: node.id,
          targetX: targetPos.x,
          targetY: targetPos.y,
        })
        updates.push({
          id: node.data.tableId,
          pos_x: targetPos.x,
          pos_y: targetPos.y,
        })
      }
    })

    // 4. Smooth gliding animation
    await animateNodePositions(targetList, 450)

    const newPositions = nodes.value.map(n => ({
      id: n.data.tableId,
      nodeId: n.id,
      pos: { ...n.position },
    }))

    // 5. Push history for Ctrl+Z undo
    pushHistory({
      type: 'LAYOUT_NODES',
      prevPositions,
      newPositions,
    })

    updateEdgeHandles()
    triggerUpdateNodeInternals()

    // 6. Batch persist to backend
    try {
      await persistPositions(updates)
      ElMessage.success('DAG 有向拓扑分层排版完成')
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
    triggerUpdateNodeInternals()
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
    triggerUpdateNodeInternals()
        redoStack.value.push(action)
        ElMessage.info('已撤销删除连线')
      } else if (action.type === 'MOVE_NODE') {
        const node = nodes.value.find(n => n.id === action.nodeId)
        if (node) {
          node.position = { ...action.prevPos }
          await persistPositions([{ id: action.tableId, pos_x: action.prevPos.x, pos_y: action.prevPos.y }])
          updateEdgeHandles()
    triggerUpdateNodeInternals()
        }
        redoStack.value.push(action)
        ElMessage.info('已撤销卡片移动')
      } else if (action.type === 'LAYOUT_NODES') {
        const targets = action.prevPositions.map(p => ({
          nodeId: p.nodeId,
          targetX: p.pos.x,
          targetY: p.pos.y,
        }))
        await animateNodePositions(targets, 350)
        await persistPositions(action.prevPositions.map(p => ({ id: p.id, pos_x: p.pos.x, pos_y: p.pos.y })))
        updateEdgeHandles()
    triggerUpdateNodeInternals()
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
    triggerUpdateNodeInternals()
        historyStack.value.push(action)
        ElMessage.info('已重做新建连线')
      } else if (action.type === 'REMOVE_MAPPINGS') {
        const ids = action.mappings.map(m => m.id)
        await api.batchDeleteMappings(ids)
        edges.value = edges.value.filter(e => !ids.includes(e.data?.mappingId))
        updateEdgeHandles()
    triggerUpdateNodeInternals()
        historyStack.value.push(action)
        ElMessage.info('已重做删除连线')
      } else if (action.type === 'MOVE_NODE') {
        const node = nodes.value.find(n => n.id === action.nodeId)
        if (node) {
          node.position = { ...action.newPos }
          await persistPositions([{ id: action.tableId, pos_x: action.newPos.x, pos_y: action.newPos.y }])
          updateEdgeHandles()
    triggerUpdateNodeInternals()
        }
        historyStack.value.push(action)
        ElMessage.info('已重做卡片移动')
      } else if (action.type === 'LAYOUT_NODES') {
        const targets = action.newPositions.map(p => ({
          nodeId: p.nodeId,
          targetX: p.pos.x,
          targetY: p.pos.y,
        }))
        await animateNodePositions(targets, 350)
        await persistPositions(action.newPositions.map(p => ({ id: p.id, pos_x: p.pos.x, pos_y: p.pos.y })))
        updateEdgeHandles()
    triggerUpdateNodeInternals()
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

  // Relation Path Discovery highlights
  const highlightRelationPath = (path: RelationPath | null) => {
    if (!path) {
      clearRelationPathHighlight()
      return
    }

    activeRelationPath.value = path
    const targetNodeIds = new Set(path.involved_table_ids.map(id => `table_${id}`))
    highlightedNodeIds.value = targetNodeIds

    const targetEdgeIds = new Set<string>()
    path.steps.forEach(step => {
      const fromTableId = step.from_table.id
      const fromColId = step.from_column.id
      const toTableId = step.to_table.id
      const toColId = step.to_column.id

      edges.value.forEach(e => {
        const d = e.data
        if (!d) return
        const matchForward = (d.sourceTableId === fromTableId && d.sourceColumnId === fromColId && d.targetTableId === toTableId && d.targetColumnId === toColId)
        const matchBackward = (d.sourceTableId === toTableId && d.sourceColumnId === toColId && d.targetTableId === fromTableId && d.targetColumnId === fromColId)
        if (matchForward || matchBackward) {
          targetEdgeIds.add(e.id)
          e.hidden = false
          e.animated = true
          e.style = {
            stroke: '#2563eb',
            strokeWidth: 3,
          }
        }
      })
    })

    highlightedEdgeIds.value = targetEdgeIds
  }

  const clearRelationPathHighlight = () => {
    activeRelationPath.value = null
    highlightedNodeIds.value.clear()
    highlightedEdgeIds.value.clear()
    if (!activeColumnId.value) {
      edges.value.forEach(e => {
        e.hidden = true
        e.animated = false
      })
    }
  }

  let onTablePickedCallback: ((role: 'source' | 'target', tableId: string, tableName: string) => void) | null = null

  const startPickingTable = (role: 'source' | 'target', callback: (role: 'source' | 'target', tableId: string, tableName: string) => void) => {
    pickingTableRole.value = role
    onTablePickedCallback = callback
  }

  const cancelPickingTable = () => {
    pickingTableRole.value = null
    onTablePickedCallback = null
  }

  const triggerTablePicked = (tableId: string, tableName: string) => {
    if (pickingTableRole.value && onTablePickedCallback) {
      const role = pickingTableRole.value
      const cb = onTablePickedCallback
      cancelPickingTable()
      cb(role, tableId, tableName)
    }
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
    triggerUpdateNodeInternals()
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
    activeRelationPath,
    pickingTableRole,
    startPickingTable,
    cancelPickingTable,
    triggerTablePicked,
    registerUpdateNodeInternals,
    triggerUpdateNodeInternals,
    highlightRelationPath,
    clearRelationPathHighlight,
  }
})
