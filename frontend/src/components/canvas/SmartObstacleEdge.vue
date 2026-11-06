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
  const edgeIdx = visibleEdges.findIndex(e => e.id === props.id)
  const idx = edgeIdx >= 0 ? edgeIdx : Math.abs(props.id.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0))
  // subtle tangent modulation factor (-18px ~ +18px)
  const fanOffset = ((idx % 7) - 3) * 6

  // Case 1: Standard Left-to-Right Flow (source on Right, target on Left)
  if (props.sourcePosition === Position.Right && props.targetPosition === Position.Left) {
    if (dx > 0) {
      // Adjacent columns (gap <= 220px): pure, clean direct S-curve
      if (dx <= 220) {
        const tangent = Math.max(45, Math.min(180, dx * 0.45))
        const cp1x = sx + tangent + fanOffset
        const cp1y = sy
        const cp2x = tx - tangent - fanOffset
        const cp2y = ty
        return `M ${sx.toFixed(1)} ${sy.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
      }

      // Cross-layer columns: check if any intermediary node actually blocks the line
      const blockingObstacles = obstacles.filter(obs => {
        if (obs.centerX <= sx + 60 || obs.centerX >= tx - 60) return false
        const ratio = (obs.centerX - sx) / dx
        if (ratio <= 0.1 || ratio >= 0.9) return false
        const expectedY = sy + ratio * dy
        return expectedY >= obs.top && expectedY <= obs.bottom
      })

      if (blockingObstacles.length === 0) {
        const tangent = Math.max(50, Math.min(220, dx * 0.45))
        const cp1x = sx + tangent + fanOffset
        const cp1y = sy
        const cp2x = tx - tangent - fanOffset
        const cp2y = ty
        return `M ${sx.toFixed(1)} ${sy.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
      } else {
        const minTop = Math.min(...blockingObstacles.map(o => o.top))
        const maxBottom = Math.max(...blockingObstacles.map(o => o.bottom))
        const midObsX = blockingObstacles.reduce((acc, o) => acc + o.centerX, 0) / blockingObstacles.length
        const avgY = (sy + ty) / 2

        const distToTop = Math.abs(minTop - 16 - avgY)
        const distToBottom = Math.abs(maxBottom + 16 - avgY)
        const safeY = distToTop <= distToBottom
          ? minTop - 20 - Math.abs(fanOffset)
          : maxBottom + 20 + Math.abs(fanOffset)

        const t1 = Math.max(30, Math.min(140, (midObsX - sx) * 0.5))
        const t2 = Math.max(30, Math.min(140, (tx - midObsX) * 0.5))

        return `M ${sx.toFixed(1)} ${sy.toFixed(1)} ` +
          `C ${(sx + t1).toFixed(1)} ${sy.toFixed(1)}, ${(midObsX - t1).toFixed(1)} ${safeY.toFixed(1)}, ${midObsX.toFixed(1)} ${safeY.toFixed(1)} ` +
          `C ${(midObsX + t2).toFixed(1)} ${safeY.toFixed(1)}, ${(tx - t2).toFixed(1)} ${ty.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
      }
    } else {
      // Target is slightly to the left: smooth graceful S-curve
      const tangent = Math.max(45, Math.abs(dx) * 0.4)
      return `M ${sx.toFixed(1)} ${sy.toFixed(1)} C ${(sx + tangent).toFixed(1)} ${sy.toFixed(1)}, ${(tx - tangent).toFixed(1)} ${ty.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
    }
  }

  // Case 2: Right-to-Left Flow (source on Left, target on Right)
  if (props.sourcePosition === Position.Left && props.targetPosition === Position.Right) {
    if (dx < 0) {
      if (Math.abs(dx) <= 220) {
        const tangent = Math.max(45, Math.min(180, Math.abs(dx) * 0.45))
        const cp1x = sx - tangent - fanOffset
        const cp1y = sy
        const cp2x = tx + tangent + fanOffset
        const cp2y = ty
        return `M ${sx.toFixed(1)} ${sy.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
      }

      const blockingObstacles = obstacles.filter(obs => {
        if (obs.centerX <= tx + 60 || obs.centerX >= sx - 60) return false
        const ratio = (obs.centerX - tx) / Math.abs(dx)
        if (ratio <= 0.1 || ratio >= 0.9) return false
        const expectedY = ty + ratio * (sy - ty)
        return expectedY >= obs.top && expectedY <= obs.bottom
      })

      if (blockingObstacles.length === 0) {
        const tangent = Math.max(50, Math.min(220, Math.abs(dx) * 0.45))
        const cp1x = sx - tangent - fanOffset
        const cp1y = sy
        const cp2x = tx + tangent + fanOffset
        const cp2y = ty
        return `M ${sx.toFixed(1)} ${sy.toFixed(1)} C ${cp1x.toFixed(1)} ${cp1y.toFixed(1)}, ${cp2x.toFixed(1)} ${cp2y.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
      } else {
        const minTop = Math.min(...blockingObstacles.map(o => o.top))
        const maxBottom = Math.max(...blockingObstacles.map(o => o.bottom))
        const midObsX = blockingObstacles.reduce((acc, o) => acc + o.centerX, 0) / blockingObstacles.length
        const avgY = (sy + ty) / 2

        const distToTop = Math.abs(minTop - 16 - avgY)
        const distToBottom = Math.abs(maxBottom + 16 - avgY)
        const safeY = distToTop <= distToBottom
          ? minTop - 20 - Math.abs(fanOffset)
          : maxBottom + 20 + Math.abs(fanOffset)

        const t1 = Math.max(30, Math.min(140, (sx - midObsX) * 0.5))
        const t2 = Math.max(30, Math.min(140, (midObsX - tx) * 0.5))

        return `M ${sx.toFixed(1)} ${sy.toFixed(1)} ` +
          `C ${(sx - t1).toFixed(1)} ${sy.toFixed(1)}, ${(midObsX + t1).toFixed(1)} ${safeY.toFixed(1)}, ${midObsX.toFixed(1)} ${safeY.toFixed(1)} ` +
          `C ${(midObsX - t2).toFixed(1)} ${safeY.toFixed(1)}, ${(tx + t2).toFixed(1)} ${ty.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
      }
    } else {
      const tangent = Math.max(45, Math.abs(dx) * 0.4)
      return `M ${sx.toFixed(1)} ${sy.toFixed(1)} C ${(sx - tangent).toFixed(1)} ${sy.toFixed(1)}, ${(tx + tangent).toFixed(1)} ${ty.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
    }
  }

  // Case 3: Both handles on Right side (Outer right C-curve for same-column tables)
  if (props.sourcePosition === Position.Right && props.targetPosition === Position.Right) {
    const tangent = Math.max(45, Math.min(120, Math.abs(dy) * 0.35)) + Math.abs(fanOffset)
    const cpX = Math.max(sx, tx) + tangent
    return `M ${sx.toFixed(1)} ${sy.toFixed(1)} C ${cpX.toFixed(1)} ${sy.toFixed(1)}, ${cpX.toFixed(1)} ${ty.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
  }

  // Case 4: Both handles on Left side (Outer left C-curve for same-column tables)
  if (props.sourcePosition === Position.Left && props.targetPosition === Position.Left) {
    const tangent = Math.max(45, Math.min(120, Math.abs(dy) * 0.35)) + Math.abs(fanOffset)
    const cpX = Math.min(sx, tx) - tangent
    return `M ${sx.toFixed(1)} ${sy.toFixed(1)} C ${cpX.toFixed(1)} ${sy.toFixed(1)}, ${cpX.toFixed(1)} ${ty.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
  }

  // Fallback: graceful direct Bézier curve
  const tangent = Math.max(45, Math.abs(dx) * 0.4)
  const dir = dx >= 0 ? 1 : -1
  return `M ${sx.toFixed(1)} ${sy.toFixed(1)} C ${(sx + tangent * dir).toFixed(1)} ${sy.toFixed(1)}, ${(tx - tangent * dir).toFixed(1)} ${ty.toFixed(1)}, ${tx.toFixed(1)} ${ty.toFixed(1)}`
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
  stroke-width: 1.8px;
  stroke-linecap: round;
  stroke-linejoin: round;
  opacity: 0.85;
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
  stroke: #38bdf8;
}

:global(.dark) .smart-bezier-edge.is-active,
:global(.dark) .smart-obstacle-edge-group:hover .smart-bezier-edge {
  stroke: #38bdf8;
}

/* Glow Aura underneath active line */
.edge-glow-aura {
  stroke: rgba(37, 99, 235, 0.35);
  stroke-width: 7px;
  opacity: 0.8;
  filter: blur(2px);
  pointer-events: none;
}

:global(.dark) .edge-glow-aura {
  stroke: rgba(56, 189, 248, 0.4);
}

/* Dimmed State when focusing on another field/edge */
.smart-obstacle-edge-group.is-dimmed .smart-bezier-edge {
  stroke: #94a3b8;
  opacity: 0.18 !important;
  stroke-width: 1.2px;
}

:global(.dark) .smart-obstacle-edge-group.is-dimmed .smart-bezier-edge {
  stroke: #52525b;
  opacity: 0.15 !important;
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
