import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Node, Edge, Connection } from '@vue-flow/core'
import dagre from 'dagre'
import { api } from '@/api'
import type { TableMeta, FieldMapping } from '@/types'
import { ElMessage } from 'element-plus'

export const useCanvasStore = defineStore('canvas', () => {
  const nodes = ref<Node[]>([])
  const edges = ref<Edge[]>([])
  const selectedEdgeId = ref<string | null>(null)
  const highlightedNodeIds = ref<Set<string>>(new Set())
  const highlightedEdgeIds = ref<Set<string>>(new Set())

  // Active field and its counterpart columns for breathing animation & line visibility
  const activeColumnId = ref<string | null>(null)
  const associatedColumnIds = ref<Set<string>>(new Set())

  let savePositionTimer: any = null

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

      const label = `${srcTable?.table_name || ''}.${srcCol?.column_name || ''} ➔ ${tgtTable?.table_name || ''}.${tgtCol?.column_name || ''}`

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
  const onNodeDrag = () => {
    updateEdgeHandles()
  }

  // Handle node drag end - debounce position saving to backend
  const onNodeDragStop = () => {
    updateEdgeHandles()

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
        try {
          await api.updateTablePositions(updates)
        } catch (e) {
          console.error('保存节点坐标失败', e)
        }
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

      edges.value.push({
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
      })

      updateEdgeHandles()
      selectColumn(sourceColumnId)
      ElMessage.success('建立字段映射成功')
    } catch (err: any) {
      ElMessage.error('创建映射失败: ' + (err.message || '网络错误'))
    }
  }

  // Delete edge
  const removeEdge = async (edgeId: string) => {
    const edge = edges.value.find(e => e.id === edgeId)
    if (!edge || !edge.data?.mappingId) {
      edges.value = edges.value.filter(e => e.id !== edgeId)
      return
    }

    try {
      await api.deleteMapping(edge.data.mappingId)
      edges.value = edges.value.filter(e => e.id !== edgeId)
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

    updateEdgeHandles()

    // Batch persist to backend
    try {
      await api.updateTablePositions(updates)
      ElMessage.success('智能排版布局完成')
    } catch (e) {
      console.error('保存自动排版坐标失败', e)
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

  return {
    nodes,
    edges,
    selectedEdgeId,
    highlightedNodeIds,
    highlightedEdgeIds,
    activeColumnId,
    associatedColumnIds,
    selectColumn,
    updateEdgeHandles,
    syncFromProject,
    onNodeDrag,
    onNodeDragStop,
    onConnect,
    removeEdge,
    applyAutoLayout,
    highlightLineage,
    clearHighlight,
  }
})
