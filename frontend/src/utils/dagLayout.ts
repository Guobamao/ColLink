/**
 * Enhanced DAG Layered Layout Algorithm (Sugiyama-style with Barycenter Vertical Alignment)
 *
 * Steps:
 * 1. Cycle Breaking via Depth-First Search (Back-edge elimination).
 * 2. Topological Layer Assignment (Longest-path layering, roots at Layer 0).
 * 3. Crossing Reduction using 3-Pass Alternating Barycenter Heuristic (Forward -> Backward -> Forward).
 * 4. Precise Node Height Estimation matching DOM table card geometries.
 * 5. Barycenter-weighted Vertical Coordinate Alignment (placing target/intermediate cards
 *    adjacent to the vertical centers of their connected partner tables, eliminating steep diagonals).
 * 6. Generous horizontal column corridor (140px gap) providing ample space for smooth Bézier curves.
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
  columnSpacing?: number // Horizontal gap between column layer origins (default: 460 = 320px node + 140px corridor)
  rowSpacing?: number    // Vertical gap between cards in the same column (default: 48)
  startX?: number        // Origin X (default: 80)
  startY?: number        // Origin Y (default: 80)
}

/**
 * Accurately calculate the visual height of a table node based on its DOM structure:
 * - Card header: 42px (or 60px if has comment subtitle)
 * - Filter input & actions: 34px
 * - Column items: 27px per item, bounded by max-h-[300px] (~11 visible rows)
 */
export function getNodeHeight(n: DagNode | undefined): number {
  if (!n) return 240
  if (n.height && n.height > 100) return n.height
  if (n.data?.dimensions?.height && n.data.dimensions.height > 100) {
    return n.data.dimensions.height
  }

  const cols = n.columnsCount ?? (n.data?.columns ? n.data.columns.length : 6)
  const hasComment = Boolean(n.data?.tableComment)
  const headerH = hasComment ? 58 : 42
  const searchH = 34
  const colListH = Math.min(cols * 27 + 6, 300)

  return Math.round(headerH + searchH + colListH)
}

export function computeDagLayeredLayout(
  nodes: DagNode[],
  edges: DagEdge[],
  options: DagLayoutOptions = {}
): DagLayoutResult {
  const {
    columnSpacing = 460, // 320px node width + 140px breathing corridor for smooth Bézier S-curves
    rowSpacing = 48,
    startX = 80,
    startY = 80,
  } = options

  if (nodes.length === 0) {
    return { nodePositions: new Map(), layers: [] }
  }

  const nodeIds = new Set(nodes.map(n => n.id))

  // 1. Build Adjacency List & Degrees (deduplicate multiple edges between same tables)
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

  // 2. Cycle Breaking via Depth-First Search
  const visitState = new Map<string, number>() // 0=unvisited, 1=visiting, 2=visited
  for (const n of nodes) visitState.set(n.id, 0)

  const acyclicAdj = new Map<string, Set<string>>()
  const acyclicRevAdj = new Map<string, Set<string>>()
  for (const n of nodes) {
    acyclicAdj.set(n.id, new Set())
    acyclicRevAdj.set(n.id, new Set())
  }

  function dfsCycleBreak(u: string) {
    visitState.set(u, 1)
    const neighbors = adj.get(u) || new Set()
    for (const v of neighbors) {
      const state = visitState.get(v)
      if (state === 1) {
        // Back-edge detected: break cycle
        continue
      } else {
        acyclicAdj.get(u)!.add(v)
        acyclicRevAdj.get(v)!.add(u)
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

  const queue: string[] = []
  const tempInDegree = new Map(acyclicInDegree)

  for (const n of nodes) {
    if (tempInDegree.get(n.id) === 0) {
      queue.push(n.id)
    }
  }

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

  for (const n of nodes) {
    if (!topoOrder.includes(n.id)) {
      topoOrder.push(n.id)
      nodeLayer.set(n.id, 0)
    }
  }

  // Separate completely isolated nodes (no relations at all)
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

  // 4. Crossing Reduction: 3-Pass Alternating Barycenter Heuristic
  // Pass 1: Forward sweep (Layer 1 -> L-1)
  for (let l = 1; l < layers.length; l++) {
    sortLayerByBarycenter(layers[l], layers[l - 1], acyclicRevAdj)
  }
  // Pass 2: Backward sweep (Layer L-2 -> 0)
  for (let l = layers.length - 2; l >= 0; l--) {
    sortLayerByBarycenter(layers[l], layers[l + 1], acyclicAdj)
  }
  // Pass 3: Final Forward sweep (Layer 1 -> L-1)
  for (let l = 1; l < layers.length; l++) {
    sortLayerByBarycenter(layers[l], layers[l - 1], acyclicRevAdj)
  }

  function sortLayerByBarycenter(
    currLayer: string[],
    referenceLayer: string[],
    adjRelation: Map<string, Set<string>>
  ) {
    const refPosMap = new Map<string, number>()
    referenceLayer.forEach((id, idx) => refPosMap.set(id, idx))

    const barycenters = new Map<string, number>()
    for (const v of currLayer) {
      const neighbors = Array.from(adjRelation.get(v) || []).filter(p => refPosMap.has(p))
      if (neighbors.length > 0) {
        const sum = neighbors.reduce((acc, p) => acc + refPosMap.get(p)!, 0)
        barycenters.set(v, sum / neighbors.length)
      } else {
        barycenters.set(v, 999)
      }
    }

    currLayer.sort((a, b) => (barycenters.get(a) || 0) - (barycenters.get(b) || 0))
  }

  // Append isolated nodes cleanly if any
  if (isolatedNodes.length > 0) {
    if (layers.length === 0) {
      const perCol = 4
      for (let i = 0; i < isolatedNodes.length; i += perCol) {
        layers.push(isolatedNodes.slice(i, i + perCol).map(n => n.id))
      }
    } else {
      layers[0].push(...isolatedNodes.map(n => n.id))
    }
  }

  // 5. Barycenter-weighted Vertical Coordinate Assignment
  const nodeMap = new Map(nodes.map(n => [n.id, n]))
  const nodePositions = new Map<string, { x: number; y: number }>()

  // Position Layer 0 (initial column layout)
  if (layers.length > 0) {
    const layer0 = layers[0]
    const x0 = startX
    let currY = startY

    layer0.forEach(nodeId => {
      const n = nodeMap.get(nodeId)
      const h = getNodeHeight(n)
      nodePositions.set(nodeId, { x: x0, y: Math.round(currY) })
      currY += h + rowSpacing
    })
  }

  // Position Subsequent Layers with Vertical Barycenter Alignment
  for (let l = 1; l < layers.length; l++) {
    const layer = layers[l]
    const x = startX + l * columnSpacing

    // Calculate ideal Y-center for each node based on its connected parents/neighbors in previous layers
    const idealCenters = new Map<string, number>()

    layer.forEach((nodeId, idx) => {
      const parentIds = Array.from(revAdj.get(nodeId) || []).filter(pid => nodePositions.has(pid))
      const childIds = Array.from(adj.get(nodeId) || []).filter(cid => nodePositions.has(cid))
      const connectedPlacedIds = [...parentIds, ...childIds]

      if (connectedPlacedIds.length > 0) {
        const sumCenterY = connectedPlacedIds.reduce((acc, id) => {
          const pos = nodePositions.get(id)!
          const n = nodeMap.get(id)
          const h = getNodeHeight(n)
          return acc + (pos.y + h / 2)
        }, 0)
        idealCenters.set(nodeId, sumCenterY / connectedPlacedIds.length)
      } else {
        // Fallback: estimate from neighbor or layer order
        idealCenters.set(nodeId, startY + idx * 320 + 150)
      }
    })

    // Compute ideal top Y positions
    const items: Array<{ id: string; height: number; idealTop: number; assignedTop: number }> = layer.map(nodeId => {
      const n = nodeMap.get(nodeId)
      const h = getNodeHeight(n)
      const idealCenter = idealCenters.get(nodeId) || (startY + 200)
      const idealTop = Math.max(startY, Math.round(idealCenter - h / 2))
      return { id: nodeId, height: h, idealTop, assignedTop: idealTop }
    })

    // Forward relaxation: ensure minimum row spacing between consecutive nodes
    if (items.length > 0) {
      items[0].assignedTop = Math.max(startY, items[0].idealTop)
      for (let i = 1; i < items.length; i++) {
        const prev = items[i - 1]
        const curr = items[i]
        const minTop = prev.assignedTop + prev.height + rowSpacing
        curr.assignedTop = Math.max(minTop, curr.idealTop)
      }
    }

    // Backward relaxation: if the last element was pushed too low or can be pulled up closer to its ideal
    for (let i = items.length - 2; i >= 0; i--) {
      const next = items[i + 1]
      const curr = items[i]
      const maxTop = next.assignedTop - curr.height - rowSpacing
      if (curr.assignedTop > maxTop) {
        curr.assignedTop = maxTop
      }
    }

    // Ensure all assigned tops are at least startY
    let shiftDown = 0
    if (items.length > 0 && items[0].assignedTop < startY) {
      shiftDown = startY - items[0].assignedTop
    }
    items.forEach(it => {
      it.assignedTop += shiftDown
      nodePositions.set(it.id, {
        x: Math.round(x),
        y: Math.round(it.assignedTop),
      })
    })
  }

  // 6. Global Vertical Balance Sweep for Layers with very few nodes
  // If a middle layer has only 1 or 2 nodes compared to a tall 4-node layer,
  // align the vertical center of the layer with the vertical center of its direct connections
  for (let l = 1; l < layers.length; l++) {
    const layer = layers[l]
    if (layer.length === 0) continue

    const allConnectedParents = layer.flatMap(id => Array.from(revAdj.get(id) || []))
      .filter(pid => nodePositions.has(pid))

    if (allConnectedParents.length > 0) {
      const parentCenters = allConnectedParents.map(pid => {
        const pPos = nodePositions.get(pid)!
        const pNode = nodeMap.get(pid)
        return pPos.y + getNodeHeight(pNode) / 2
      })
      const avgParentCenter = parentCenters.reduce((a, b) => a + b, 0) / parentCenters.length

      const layerTops = layer.map(id => nodePositions.get(id)!.y)
      const layerBottoms = layer.map(id => {
        const pos = nodePositions.get(id)!
        const n = nodeMap.get(id)
        return pos.y + getNodeHeight(n)
      })
      const layerMinY = Math.min(...layerTops)
      const layerMaxY = Math.max(...layerBottoms)
      const currentLayerCenter = (layerMinY + layerMaxY) / 2

      const offset = Math.round(avgParentCenter - currentLayerCenter)
      // Only gently adjust if offset is significant and doesn't violate startY
      if (Math.abs(offset) > 20 && layerMinY + offset >= startY) {
        layer.forEach(id => {
          const pos = nodePositions.get(id)!
          nodePositions.set(id, { x: pos.x, y: pos.y + offset })
        })
      }
    }
  }

  return { nodePositions, layers }
}
