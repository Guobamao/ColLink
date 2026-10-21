/**
 * Standard DAG Layered Layout Algorithm (Sugiyama-style Left-to-Right Flow)
 *
 * Steps:
 * 1. Cycle Breaking via Depth-First Search (Back-edge elimination).
 * 2. Topological Layer Assignment (Longest-path layering, pure roots at Layer 0).
 * 3. Crossing Reduction using 2-Pass Barycenter Heuristic within layers.
 * 4. Coordinate Assignment with configurable column/row spacing & gentle vertical centering.
 */

export interface DagNode {
  id: string
  width?: number
  height?: number
  columnsCount?: number
  data?: any
}

export interface DagEdge {
  source: string
  target: string
}

export interface DagLayoutResult {
  nodePositions: Map<string, { x: number; y: number }>
  layers: string[][]
}

export interface DagLayoutOptions {
  columnSpacing?: number // Horizontal gap between column layers
  rowSpacing?: number    // Vertical gap between cards in the same column
  startX?: number        // Origin X
  startY?: number        // Origin Y
}

export function computeDagLayeredLayout(
  nodes: DagNode[],
  edges: DagEdge[],
  options: DagLayoutOptions = {}
): DagLayoutResult {
  const {
    columnSpacing = 360,
    rowSpacing = 60,
    startX = 80,
    startY = 80,
  } = options

  if (nodes.length === 0) {
    return { nodePositions: new Map(), layers: [] }
  }

  const nodeIds = new Set(nodes.map(n => n.id))

  // 1. Build Adjacency List & In-degrees (deduplicate multiple edges between same tables)
  const adj = new Map<string, Set<string>>()
  const revAdj = new Map<string, Set<string>>()
  const inDegree = new Map<string, number>()
  const outDegree = new Map<string, number>()

  for (const n of nodes) {
    adj.set(n.id, new Set())
    revAdj.set(n.id, new Set())
    inDegree.set(n.id, 0)
    outDegree.set(n.id, 0)
  }

  for (const e of edges) {
    if (nodeIds.has(e.source) && nodeIds.has(e.target) && e.source !== e.target) {
      if (!adj.get(e.source)!.has(e.target)) {
        adj.get(e.source)!.add(e.target)
        revAdj.get(e.target)!.add(e.source)
        inDegree.set(e.target, (inDegree.get(e.target) || 0) + 1)
        outDegree.set(e.source, (outDegree.get(e.source) || 0) + 1)
      }
    }
  }

  // 2. Cycle Breaking via DFS
  // States: 0 = unvisited, 1 = visiting (in stack), 2 = visited
  const visitState = new Map<string, number>()
  for (const n of nodes) visitState.set(n.id, 0)

  const acyclicAdj = new Map<string, Set<string>>()
  for (const n of nodes) acyclicAdj.set(n.id, new Set())

  function dfsCycleBreak(u: string) {
    visitState.set(u, 1)
    const neighbors = adj.get(u) || new Set()
    for (const v of neighbors) {
      const state = visitState.get(v)
      if (state === 1) {
        // Back-edge detected (cycle) -> Ignore back-edge for layering
        continue
      } else {
        acyclicAdj.get(u)!.add(v)
        if (state === 0) {
          dfsCycleBreak(v)
        }
      }
    }
    visitState.set(u, 2)
  }

  for (const n of nodes) {
    if (visitState.get(n.id) === 0) {
      dfsCycleBreak(n.id)
    }
  }

  // Recompute in-degrees on the acyclic graph
  const acyclicInDegree = new Map<string, number>()
  for (const n of nodes) acyclicInDegree.set(n.id, 0)
  for (const [, targets] of acyclicAdj.entries()) {
    for (const v of targets) {
      acyclicInDegree.set(v, (acyclicInDegree.get(v) || 0) + 1)
    }
  }

  // 3. Layer Assignment (Longest Path in DAG)
  const nodeLayer = new Map<string, number>()
  for (const n of nodes) nodeLayer.set(n.id, 0)

  // Topological sort queue
  const queue: string[] = []
  const tempInDegree = new Map(acyclicInDegree)

  for (const n of nodes) {
    if (tempInDegree.get(n.id) === 0) {
      queue.push(n.id)
    }
  }

  // Handle graph with all nodes in cycles (fallback)
  if (queue.length === 0 && nodes.length > 0) {
    queue.push(nodes[0].id)
  }

  const topoOrder: string[] = []
  while (queue.length > 0) {
    const u = queue.shift()!
    topoOrder.push(u)
    const currentLayer = nodeLayer.get(u)!

    const neighbors = acyclicAdj.get(u) || new Set()
    for (const v of neighbors) {
      nodeLayer.set(v, Math.max(nodeLayer.get(v)!, currentLayer + 1))
      const deg = (tempInDegree.get(v) || 1) - 1
      tempInDegree.set(v, deg)
      if (deg === 0) {
        queue.push(v)
      }
    }
  }

  // Add any unvisited nodes (e.g., disconnected components)
  for (const n of nodes) {
    if (!topoOrder.includes(n.id)) {
      topoOrder.push(n.id)
      nodeLayer.set(n.id, 0)
    }
  }

  // Separate completely isolated nodes (no in or out edges) to keep core pipeline clean
  const isolatedNodes = nodes.filter(n => (inDegree.get(n.id) || 0) === 0 && (outDegree.get(n.id) || 0) === 0)
  const connectedNodes = nodes.filter(n => (inDegree.get(n.id) || 0) > 0 || (outDegree.get(n.id) || 0) > 0)

  // Group connected nodes by layer
  const layerMap = new Map<number, string[]>()
  for (const n of connectedNodes) {
    const layer = nodeLayer.get(n.id) || 0
    if (!layerMap.has(layer)) {
      layerMap.set(layer, [])
    }
    layerMap.get(layer)!.push(n.id)
  }

  const sortedLayerIndices = Array.from(layerMap.keys()).sort((a, b) => a - b)
  const layers: string[][] = sortedLayerIndices.map(idx => layerMap.get(idx)!)

  // 4. Crossing Reduction: Barycenter Heuristic (2 Passes)
  for (let l = 1; l < layers.length; l++) {
    const prevLayer = layers[l - 1]
    const prevPosMap = new Map<string, number>()
    prevLayer.forEach((id, idx) => prevPosMap.set(id, idx))

    const currentLayer = layers[l]
    const barycenters = new Map<string, number>()

    for (const v of currentLayer) {
      const parents = Array.from(revAdj.get(v) || []).filter(p => prevPosMap.has(p))
      if (parents.length > 0) {
        const sum = parents.reduce((acc, p) => acc + prevPosMap.get(p)!, 0)
        barycenters.set(v, sum / parents.length)
      } else {
        barycenters.set(v, 999) // Keep at end if no direct parent in previous layer
      }
    }

    currentLayer.sort((a, b) => (barycenters.get(a) || 0) - (barycenters.get(b) || 0))
  }

  // If there are isolated nodes, append them as a dedicated bottom cluster or last layer
  if (isolatedNodes.length > 0) {
    // If no connected nodes, place isolated nodes in a neat grid (e.g., max 3 per column)
    if (layers.length === 0) {
      const perCol = 4
      const isolatedLayers: string[][] = []
      for (let i = 0; i < isolatedNodes.length; i += perCol) {
        isolatedLayers.push(isolatedNodes.slice(i, i + perCol).map(n => n.id))
      }
      layers.push(...isolatedLayers)
    } else {
      // Append isolated nodes to Layer 0 at the bottom
      layers[0].push(...isolatedNodes.map(n => n.id))
    }
  }

  // 5. Coordinate Assignment
  const nodeMap = new Map(nodes.map(n => [n.id, n]))
  const nodePositions = new Map<string, { x: number; y: number }>()

  // Approximate node heights
  const getNodeHeight = (n: DagNode | undefined): number => {
    if (!n) return 240
    if (n.height && n.height > 100) return n.height
    const cols = n.columnsCount ?? (n.data?.columns ? n.data.columns.length : 6)
    // Table header ~90px, search input ~36px, each field ~32px, max 400px
    return Math.min(Math.max(120 + cols * 32, 180), 400)
  }

  // Calculate column heights for vertical centering
  const colHeights: number[] = layers.map(layer => {
    let h = 0
    layer.forEach((nodeId, idx) => {
      const node = nodeMap.get(nodeId)
      h += getNodeHeight(node)
      if (idx < layer.length - 1) h += rowSpacing
    })
    return h
  })

  const maxColHeight = Math.max(...colHeights, 0)

  // Assign (x, y) coordinates
  layers.forEach((layer, colIdx) => {
    const x = startX + colIdx * columnSpacing
    const colHeight = colHeights[colIdx] || 0
    // Subtle vertical balance
    const colOffsetY = Math.max(0, Math.round((maxColHeight - colHeight) * 0.25))

    let currentY = startY + colOffsetY

    layer.forEach(nodeId => {
      const node = nodeMap.get(nodeId)
      const h = getNodeHeight(node)
      nodePositions.set(nodeId, {
        x: Math.round(x),
        y: Math.round(currentY),
      })
      currentY += h + rowSpacing
    })
  })

  return { nodePositions, layers }
}
