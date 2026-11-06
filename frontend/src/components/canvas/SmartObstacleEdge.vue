<script setup lang="ts">
import { computed } from 'vue'
import { Position, useVueFlow } from '@vue-flow/core'
import { useCanvasStore } from '@/stores/canvasStore'
import { useThemeStore } from '@/stores/themeStore'

interface Props {
  id: string
  source: string
  target: string
  sourceX: number
  sourceY: number
  targetX: number
  targetY: number
  sourcePosition: Position
  targetPosition: Position
  data?: any
  style?: Record<string, any>
  markerEnd?: string
}

const props = defineProps<Props>()
const { getNodes, edges } = useVueFlow()
const canvasStore = useCanvasStore()
const themeStore = useThemeStore()

interface Rect {
  id: string
  left: number
  right: number
  top: number
  bottom: number
  centerX: number
  centerY: number
}

// Compute whether this edge or its fields are currently active/focused
const isLineActive = computed(() => {
  if (canvasStore.selectedEdgeId === props.id) return true
  if (canvasStore.highlightedEdgeIds.has(props.id)) return true
  if (canvasStore.activeColumnId) {
    const srcCol = props.data?.sourceColumnId
    const tgtCol = props.data?.targetColumnId
    if (srcCol === canvasStore.activeColumnId || tgtCol === canvasStore.activeColumnId) {
      return true
    }
  }
  return false
})

// Whether user is focusing on any specific lineage path or field
const hasGlobalFocus = computed(() => {
  return Boolean(
    canvasStore.activeColumnId ||
    canvasStore.selectedEdgeId ||
    canvasStore.highlightedEdgeIds.size > 0 ||
    canvasStore.highlightedNodeIds.size > 0
  )
})

const edgePath = computed(() => {
  if (props.sourceX == null || props.targetX == null || isNaN(props.sourceX) || isNaN(props.targetX)) {
    return ''
  }

  // Fast path: if edge is hidden in store and not in active export/view, skip computation
  const currentEdge = edges.value.find(e => e.id === props.id)
  if (currentEdge && currentEdge.hidden) {
    return ''
  }

  const sx = props.sourceX
  const sy = props.sourceY
  const tx = props.targetX
  const ty = props.targetY
  const dx = tx - sx
  const dy = ty - sy

  // Collect potential obstacle nodes between source and target
  const allNodes = getNodes.value || []
  const obstacles: Rect[] = []
  const PADDING = 14

  for (let i = 0; i < allNodes.length; i++) {
    const n = allNodes[i]
    if (n.id === props.source || n.id === props.target || n.hidden) continue
    const nx = n.position.x
    const ny = n.position.y
    const nw = (n as any).dimensions?.width || 320
    const nh = (n as any).dimensions?.height || 280

    obstacles.push({
      id: n.id,
      left: nx - PADDING,
      right: nx + nw + PADDING,
      top: ny - PADDING,
      bottom: ny + nh + PADDING,
      centerX: nx + nw / 2,
      centerY: ny + nh / 2,
    })
  }

  // Anti-bundling curvature modulation for adjacent parallel field lines
  const visibleEdges = edges.value.filter(e => !e.hidden)
  const tgtHandle = (props as any).targetHandleId || currentEdge?.targetHandle
  const srcHandle = (props as any).sourceHandleId || currentEdge?.sourceHandle

  // Handle fan-out: When multiple edges connect to the same target handle or source handle,
  // distribute their landing Y offsets slightly so they do not collapse into a single indistinguishable bar.
  const sharingTargetEdges = tgtHandle
    ? visibleEdges.filter(e => (e.targetHandle === tgtHandle || (e as any).targetHandleId === tgtHandle) && e.target === props.target)
    : []
  const targetK = sharingTargetEdges.findIndex(e => e.id === props.id)
  const targetFanY = sharingTargetEdges.length > 1 && targetK >= 0
    ? Math.max(-9, Math.min(9, (targetK - (sharingTargetEdges.length - 1) / 2) * 4))
    : 0

  const sharingSourceEdges = srcHandle
    ? visibleEdges.filter(e => (e.sourceHandle === srcHandle || (e as any).sourceHandleId === srcHandle) && e.source === props.source)
    : []
  const sourceK = sharingSourceEdges.findIndex(e => e.id === props.id)
  const sourceFanY = sharingSourceEdges.length > 1 && sourceK >= 0
    ? Math.max(-9, Math.min(9, (sourceK - (sharingSourceEdges.length - 1) / 2) * 4))
    : 0

  const startY = sy + sourceFanY
  const endY = ty + targetFanY

  const edgeIdx = visibleEdges.findIndex(e => e.id === props.id)
  const idx = edgeIdx >= 0 ? edgeIdx : Math.abs(props.id.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0))
  // subtle tangent modulation factor (-12px ~ +12px)
  const fanOffset = ((idx % 7) - 3) * 4

  // Adaptive tangent calculation avoiding exaggerated ballooning on large dy
  const calcTangent = (distX: number, distY: number, maxT = 180) => {
    const absX = Math.abs(distX)
    const absY = Math.abs(distY)
    const ratio = Math.max(0.32, Math.min(0.48, 0.45 - (absY / 1000) * 0.12))
    return Math.max(45, Math.min(maxT, absX * ratio))
  }

  // Find the optimal horizontal passage (open inter-card gap or outer margin) through a column
  const findBestPassageY = (
    colCards: Rect[],
    expectedY: number,
    endY: number,
    fanOffset: number
  ): number => {
    if (colCards.length === 0) return expectedY

    const sorted = [...colCards].sort((a, b) => a.top - b.top)
    interface Passage {
      y: number
      minY: number
      maxY: number
      isGap: boolean
    }
    const passages: Passage[] = []

    // 1. Passage above the topmost card
    const topCard = sorted[0]
    passages.push({
      y: topCard.top - 36 - Math.abs(fanOffset),
      minY: -Infinity,
      maxY: topCard.top - 16,
      isGap: false,
    })

    // 2. Inter-card gaps between adjacent cards in this column
    for (let i = 0; i < sorted.length - 1; i++) {
      const curr = sorted[i]
      const next = sorted[i + 1]
      const gap = next.top - curr.bottom
      // A gap >= 36px is an open horizontal freeway for lines to glide straight through
      if (gap >= 36) {
        passages.push({
          y: (curr.bottom + next.top) / 2,
          minY: curr.bottom + 14,
          maxY: next.top - 14,
          isGap: true,
        })
      }
    }

    // 3. Passage below the bottommost card
    const bottomCard = sorted[sorted.length - 1]
    passages.push({
      y: bottomCard.bottom + 36 + Math.abs(fanOffset),
      minY: bottomCard.bottom + 16,
      maxY: Infinity,
      isGap: false,
    })

    // If expectedY already lands inside an open passage/gap, use it directly without detours
    const directHit = passages.find(p => expectedY >= p.minY && expectedY <= p.maxY)
    if (directHit) {
      if (directHit.isGap) {
        return Math.max(directHit.minY + 4, Math.min(directHit.maxY - 4, expectedY))
      }
      return directHit.y
    }

    // Pick the passage with the minimum detour cost, strongly prioritizing natural inter-card gaps
    let best = passages[0]
    let minCost = Infinity

    for (const p of passages) {
      const distFromExpected = Math.abs(p.y - expectedY)
      const distFromEnd = Math.abs(p.y - endY)
      const gapBonus = p.isGap ? -50 : 0
      const cost = distFromExpected * 1.6 + distFromEnd * 0.7 + gapBonus

      if (cost < minCost) {
        minCost = cost
        best = p
      }
    }

    return best.y
  }

  // Build a crisp, professional orthogonal corridor route with smooth rounded corners (SmoothStep)
  const buildRoundedCorridorPath = (
    sx: number,
    sy: number,
    tx: number,
    ty: number,
    c1x: number,
    passY: number,
    c2x: number,
    radius = 14,
    targetFanY = 0
  ): string => {
    const landingY = ty + targetFanY

    // Ensure corner radius does not exceed half the shortest segment length
    const span1 = Math.abs(c1x - sx)
    const span2 = Math.abs(passY - sy)
    const span3 = Math.abs(c2x - c1x)
    const span4 = Math.abs(landingY - passY)
    const span5 = Math.abs(tx - c2x)

    const r = Math.max(
      4,
      Math.min(
        radius,
        span1 * 0.45,
        span2 * 0.45,
        span3 * 0.45,
        span4 * 0.45,
        span5 * 0.45
      )
    )

    const dirX1 = c1x >= sx ? 1 : -1
    const dirY1 = passY >= sy ? 1 : -1
    const dirX2 = c2x >= c1x ? 1 : -1
    const dirY2 = landingY >= passY ? 1 : -1
    const dirX3 = tx >= c2x ? 1 : -1

    const p1 = `M ${sx.toFixed(1)} ${sy.toFixed(1)} L ${(c1x - dirX1 * r).toFixed(1)} ${sy.toFixed(1)} `
    const p2 = `Q ${c1x.toFixed(1)} ${sy.toFixed(1)}, ${c1x.toFixed(1)} ${(sy + dirY1 * r).toFixed(1)} `
    const p3 = `L ${c1x.toFixed(1)} ${(passY - dirY1 * r).toFixed(1)} `
    const p4 = `Q ${c1x.toFixed(1)} ${passY.toFixed(1)}, ${(c1x + dirX2 * r).toFixed(1)} ${passY.toFixed(1)} `
    const p5 = `L ${(c2x - dirX2 * r).toFixed(1)} ${passY.toFixed(1)} `
    const p6 = `Q ${c2x.toFixed(1)} ${passY.toFixed(1)}, ${c2x.toFixed(1)} ${(passY + dirY2 * r).toFixed(1)} `
    const p7 = `L ${c2x.toFixed(1)} ${(landingY - dirY2 * r).toFixed(1)} `
    const p8 = `Q ${c2x.toFixed(1)} ${landingY.toFixed(1)}, ${(c2x + dirX3 * r).toFixed(1)} ${landingY.toFixed(1)} `
    let p9 = `L ${tx.toFixed(1)} ${landingY.toFixed(1)}`

    if (targetFanY !== 0) {
      p9 = `L ${(tx - dirX3 * 8).toFixed(1)} ${landingY.toFixed(1)} L ${tx.toFixed(1)} ${ty.toFixed(1)}`
    }

    return p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9
  }

  // Case 1: Standard Left-to-Right Flow (source on Right, target on Left)
  if (props.sourcePosition === Position.Right && props.targetPosition === Position.Left) {
    if (dx > 0) {
      // Clamped corridor control point distance strictly bounded inside [sx, tx]
      const corridorHalf = Math.max(16, Math.min(calcTangent(dx, dy), dx * 0.46))

      // Adjacent columns (gap <= 220px): pure, clean direct S-curve strictly inside corridor
      if (dx <= 220) {
        const cp1x = sx + corridorHalf
        const cp1y = startY
        const cp2x = tx - corridorHalf
        const cp2y = endY
        if (targetFanY === 0) {
          return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
        } else {
          return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${(tx - 8).toFixed(1)} ${endY.toFixed(1)} L ${tx.toFixed(1)} ${ty.toFixed(1)}`
        }
      }

      // Cross-layer columns: check if any intermediary node actually blocks the line
      const blockingObstacles = obstacles.filter(obs => {
        if (obs.centerX <= sx + 60 || obs.centerX >= tx - 60) return false
        const ratio = (obs.centerX - sx) / dx
        if (ratio <= 0.08 || ratio >= 0.92) return false
        const expectedY = startY + ratio * (endY - startY)
        return expectedY >= obs.top - 10 && expectedY <= obs.bottom + 10
      })

      if (blockingObstacles.length === 0) {
        const cp1x = sx + corridorHalf
        const cp1y = startY
        const cp2x = tx - corridorHalf
        const cp2y = endY
        if (targetFanY === 0) {
          return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
        } else {
          return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${(tx - 8).toFixed(1)} ${endY.toFixed(1)} L ${tx.toFixed(1)} ${ty.toFixed(1)}`
        }
      } else {
        const clusterCards = obstacles.filter(obs => {
          return blockingObstacles.some(bo => Math.abs(obs.centerX - bo.centerX) < 90)
        })
        const obsLeft = Math.min(...clusterCards.map(o => o.left))
        const obsRight = Math.max(...clusterCards.map(o => o.right))
        const midX = (obsLeft + obsRight) / 2
        const ratio = (midX - sx) / dx
        const expectedY = startY + ratio * (endY - startY)

        const safeY = findBestPassageY(clusterCards, expectedY, endY, fanOffset)

        const c1x = (sx + obsLeft) / 2
        const c2x = (obsRight + tx) / 2

        return buildRoundedCorridorPath(sx, startY, tx, ty, c1x, safeY, c2x, 14, targetFanY)
      }
    } else {
      // Target is slightly to the left: smooth graceful S-curve
      const tangent = Math.max(30, Math.min(120, Math.abs(dx) * 0.4))
      return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${(sx + tangent).toFixed(1)} ${startY.toFixed(1)}, ${(tx - tangent).toFixed(1)} ${endY.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
    }
  }

  // Case 2: Right-to-Left Flow (source on Left, target on Right)
  if (props.sourcePosition === Position.Left && props.targetPosition === Position.Right) {
    if (dx < 0) {
      // Clamped corridor control point distance strictly bounded inside [tx, sx]
      const corridorHalf = Math.max(16, Math.min(calcTangent(dx, dy), Math.abs(dx) * 0.46))

      // Adjacent columns (gap <= 220px): pure, clean direct S-curve strictly inside corridor
      if (Math.abs(dx) <= 220) {
        const cp1x = sx - corridorHalf
        const cp1y = startY
        const cp2x = tx + corridorHalf
        const cp2y = endY
        if (targetFanY === 0) {
          return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
        } else {
          return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${(tx + 8).toFixed(1)} ${endY.toFixed(1)} L ${tx.toFixed(1)} ${ty.toFixed(1)}`
        }
      }

      const blockingObstacles = obstacles.filter(obs => {
        if (obs.centerX <= tx + 60 || obs.centerX >= sx - 60) return false
        const ratio = (obs.centerX - tx) / Math.abs(dx)
        if (ratio <= 0.08 || ratio >= 0.92) return false
        const expectedY = endY + ratio * (startY - endY)
        return expectedY >= obs.top - 10 && expectedY <= obs.bottom + 10
      })

      if (blockingObstacles.length === 0) {
        const cp1x = sx - corridorHalf
        const cp1y = startY
        const cp2x = tx + corridorHalf
        const cp2y = endY
        if (targetFanY === 0) {
          return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
        } else {
          return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${(tx + 8).toFixed(1)} ${endY.toFixed(1)} L ${tx.toFixed(1)} ${ty.toFixed(1)}`
        }
      } else {
        const clusterCards = obstacles.filter(obs => {
          return blockingObstacles.some(bo => Math.abs(obs.centerX - bo.centerX) < 90)
        })
        const obsLeft = Math.min(...clusterCards.map(o => o.left))
        const obsRight = Math.max(...clusterCards.map(o => o.right))
        const midX = (obsLeft + obsRight) / 2
        const ratio = (sx - midX) / Math.abs(dx)
        const expectedY = startY + ratio * (endY - startY)

        const safeY = findBestPassageY(clusterCards, expectedY, endY, fanOffset)

        const c1x = (obsRight + sx) / 2
        const c2x = (tx + obsLeft) / 2

        return buildRoundedCorridorPath(sx, startY, tx, ty, c1x, safeY, c2x, 14, targetFanY)
      }
    } else {
      const tangent = Math.max(30, Math.min(120, Math.abs(dx) * 0.4))
      return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${(sx - tangent).toFixed(1)} ${startY.toFixed(1)}, ${(tx + tangent).toFixed(1)} ${endY.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
    }
  }

  // Case 3: Both handles on Right side (Outer right C-curve for same-column tables)
  if (props.sourcePosition === Position.Right && props.targetPosition === Position.Right) {
    const tangent = Math.max(45, Math.min(120, Math.abs(dy) * 0.35)) + Math.abs(fanOffset)
    const cpX = Math.max(sx, tx) + tangent
    return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${cpX.toFixed(1)} ${startY.toFixed(1)}, ${cpX.toFixed(1)} ${endY.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
  }

  // Case 4: Both handles on Left side (Outer left C-curve for same-column tables)
  if (props.sourcePosition === Position.Left && props.targetPosition === Position.Left) {
    const tangent = Math.max(45, Math.min(120, Math.abs(dy) * 0.35)) + Math.abs(fanOffset)
    const cpX = Math.min(sx, tx) - tangent
    return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${cpX.toFixed(1)} ${startY.toFixed(1)}, ${cpX.toFixed(1)} ${endY.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
  }

  // Fallback: graceful direct Bézier curve
  const tangent = Math.max(45, Math.abs(dx) * 0.4)
  const dir = dx >= 0 ? 1 : -1
  return `M ${sx.toFixed(1)} ${startY.toFixed(1)} C ${(sx + tangent * dir).toFixed(1)} ${startY.toFixed(1)}, ${(tx - tangent * dir).toFixed(1)} ${endY.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
})
</script>

<template>
  <g
    class="smart-obstacle-edge-group"
    :class="{
      'is-active': isLineActive,
      'is-dimmed': hasGlobalFocus && !isLineActive,
    }"
  >
    <defs>
      <!-- Standard / Default Arrow Marker -->
      <marker
        :id="`collink-arrow-${id}`"
        viewBox="0 0 10 10"
        refX="7.5"
        refY="5"
        markerWidth="6"
        markerHeight="6"
        orient="auto-start-reverse"
      >
        <path
          d="M 1 2 L 8 5 L 1 8 z"
          :fill="isLineActive ? (themeStore.isDark ? '#38bdf8' : '#2563eb') : (themeStore.isDark ? '#60a5fa' : '#3b82f6')"
        />
      </marker>
    </defs>

    <!-- Invisible Thick Hit Area for Smooth Hover & Selection -->
    <path
      v-if="edgePath"
      :d="edgePath"
      class="vue-flow__edge-interaction"
      fill="none"
      stroke="transparent"
      stroke-width="18"
      stroke-linecap="round"
      stroke-linejoin="round"
    />

    <!-- Active Line Glow Aura Layer (Rendered underneath when active) -->
    <path
      v-if="edgePath && isLineActive"
      :d="edgePath"
      class="edge-glow-aura"
      fill="none"
      stroke-linecap="round"
      stroke-linejoin="round"
    />

    <!-- Visible Lineage Bézier Path -->
    <path
      v-if="edgePath"
      :id="id"
      :d="edgePath"
      class="vue-flow__edge-path smart-bezier-edge smart-obstacle-edge"
      :class="{
        'is-active': isLineActive,
        'is-dimmed': hasGlobalFocus && !isLineActive,
      }"
      :style="style"
      :marker-end="`url(#collink-arrow-${id})`"
    />
  </g>
</template>

<style scoped>
.smart-obstacle-edge-group {
  pointer-events: visibleStroke;
  transition: opacity 0.2s ease;
}

.vue-flow__edge-interaction {
  cursor: pointer;
  pointer-events: stroke;
}

/* Beautiful Smooth Bézier Curve */
.smart-bezier-edge {
  fill: none;
  stroke: #3b82f6;
  stroke-width: 1.6px;
  stroke-linecap: round;
  stroke-linejoin: round;
  opacity: 0.65;
  transition: stroke 0.2s ease, stroke-width 0.2s ease, opacity 0.2s ease, filter 0.2s ease;
  pointer-events: stroke;
}

/* Active / Focused Lineage State */
.smart-bezier-edge.is-active,
.smart-obstacle-edge-group:hover .smart-bezier-edge {
  stroke: #2563eb;
  stroke-width: 2.6px;
  opacity: 1 !important;
  stroke-dasharray: 6 3;
  animation: edgeFlowDash 0.75s linear infinite;
}

:global(.dark) .smart-bezier-edge {
  stroke: #60a5fa;
  opacity: 0.65;
}

:global(.dark) .smart-bezier-edge.is-active,
:global(.dark) .smart-obstacle-edge-group:hover .smart-bezier-edge {
  stroke: #38bdf8;
  opacity: 1 !important;
}

/* Glow Aura underneath active line */
.edge-glow-aura {
  stroke: rgba(37, 99, 235, 0.35);
  stroke-width: 6px;
  opacity: 0.85;
  filter: blur(2px);
  pointer-events: none;
}

:global(.dark) .edge-glow-aura {
  stroke: rgba(56, 189, 248, 0.4);
}

/* Dimmed State when focusing on another field/edge */
.smart-obstacle-edge-group.is-dimmed .smart-bezier-edge {
  stroke: #94a3b8;
  opacity: 0.12 !important;
  stroke-width: 1.2px;
}

:global(.dark) .smart-obstacle-edge-group.is-dimmed .smart-bezier-edge {
  stroke: #52525b;
  opacity: 0.12 !important;
}

@keyframes edgeFlowDash {
  from {
    stroke-dashoffset: 18;
  }
  to {
    stroke-dashoffset: 0;
  }
}
</style>
