<script setup lang="ts">
import { computed } from 'vue'
import { Position, useVueFlow } from '@vue-flow/core'

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

interface Point {
  x: number
  y: number
}

interface Rect {
  id: string
  left: number
  right: number
  top: number
  bottom: number
}

// Check if an axis-aligned segment intersects a bounding box
function isSegmentIntersectingBox(x1: number, y1: number, x2: number, y2: number, box: Rect): boolean {
  const minX = Math.min(x1, x2)
  const maxX = Math.max(x1, x2)
  const minY = Math.min(y1, y2)
  const maxY = Math.max(y1, y2)

  // Horizontal segment
  if (Math.abs(y1 - y2) < 0.5) {
    const y = y1
    if (y >= box.top && y <= box.bottom) {
      if (Math.max(minX, box.left) < Math.min(maxX, box.right)) {
        return true
      }
    }
  }
  // Vertical segment
  else if (Math.abs(x1 - x2) < 0.5) {
    const x = x1
    if (x >= box.left && x <= box.right) {
      if (Math.max(minY, box.top) < Math.min(maxY, box.bottom)) {
        return true
      }
    }
  }
  return false
}

// Find all obstacles intersecting any segment in the polyline
function getIntersectingObstacles(pts: Point[], obstacles: Rect[], excludeIds: string[]): Rect[] {
  const hit: Rect[] = []
  for (let i = 0; i < pts.length - 1; i++) {
    const p1 = pts[i]
    const p2 = pts[i + 1]
    for (const obs of obstacles) {
      if (excludeIds.includes(obs.id)) continue
      if (isSegmentIntersectingBox(p1.x, p1.y, p2.x, p2.y, obs)) {
        if (!hit.some(h => h.id === obs.id)) {
          hit.push(obs)
        }
      }
    }
  }
  return hit
}

// Simplify redundant collinear points
function simplifyPoints(pts: Point[]): Point[] {
  if (pts.length <= 2) return pts
  const res: Point[] = [pts[0]]
  for (let i = 1; i < pts.length - 1; i++) {
    const prev = res[res.length - 1]
    const curr = pts[i]
    const next = pts[i + 1]

    const isCollinearX = Math.abs(prev.x - curr.x) < 0.5 && Math.abs(curr.x - next.x) < 0.5
    const isCollinearY = Math.abs(prev.y - curr.y) < 0.5 && Math.abs(curr.y - next.y) < 0.5
    const isSameAsPrev = Math.abs(prev.x - curr.x) < 0.5 && Math.abs(prev.y - curr.y) < 0.5

    if (!isSameAsPrev && !isCollinearX && !isCollinearY) {
      res.push(curr)
    }
  }
  res.push(pts[pts.length - 1])
  return res
}

// Convert points to smooth rounded SVG path
function pointsToRoundedSvgPath(pts: Point[], radius = 8): string {
  const cleanPts = simplifyPoints(pts)
  if (cleanPts.length === 0) return ''
  if (cleanPts.length === 1) return `M ${cleanPts[0].x.toFixed(1)} ${cleanPts[0].y.toFixed(1)}`
  if (cleanPts.length === 2) {
    return `M ${cleanPts[0].x.toFixed(1)} ${cleanPts[0].y.toFixed(1)} L ${cleanPts[1].x.toFixed(1)} ${cleanPts[1].y.toFixed(1)}`
  }

  let d = `M ${cleanPts[0].x.toFixed(1)} ${cleanPts[0].y.toFixed(1)}`

  for (let i = 1; i < cleanPts.length - 1; i++) {
    const prev = cleanPts[i - 1]
    const curr = cleanPts[i]
    const next = cleanPts[i + 1]

    const dPrev = Math.hypot(curr.x - prev.x, curr.y - prev.y)
    const dNext = Math.hypot(next.x - curr.x, next.y - curr.y)
    const r = Math.min(radius, dPrev / 2, dNext / 2)

    if (r < 1) {
      d += ` L ${curr.x.toFixed(1)} ${curr.y.toFixed(1)}`
      continue
    }

    const vPrevX = (prev.x - curr.x) / dPrev
    const vPrevY = (prev.y - curr.y) / dPrev
    const vNextX = (next.x - curr.x) / dNext
    const vNextY = (next.y - curr.y) / dNext

    const q1X = curr.x + vPrevX * r
    const q1Y = curr.y + vPrevY * r
    const q2X = curr.x + vNextX * r
    const q2Y = curr.y + vNextY * r

    d += ` L ${q1X.toFixed(1)} ${q1Y.toFixed(1)} Q ${curr.x.toFixed(1)} ${curr.y.toFixed(1)} ${q2X.toFixed(1)} ${q2Y.toFixed(1)}`
  }

  const last = cleanPts[cleanPts.length - 1]
  d += ` L ${last.x.toFixed(1)} ${last.y.toFixed(1)}`
  return d
}

const edgePath = computed(() => {
  const MARGIN = 16
  const STUB = 24

  // Collect obstacles (bounding boxes of all table cards)
  const allNodes = getNodes.value || []
  const obstacles: Rect[] = allNodes.map(n => {
    const x = n.position.x
    const y = n.position.y
    const w = (n as any).dimensions?.width || 280
    const h = (n as any).dimensions?.height || 360
    return {
      id: n.id,
      left: x - MARGIN,
      right: x + w + MARGIN,
      top: y - MARGIN,
      bottom: y + h + MARGIN,
    }
  })

  // Lane offset for parallel lines to prevent overlapping
  const visibleEdges = edges.value.filter(e => !e.hidden)
  const edgeIdx = visibleEdges.findIndex(e => e.id === props.id)
  const idx = edgeIdx >= 0 ? edgeIdx : Math.abs(props.id.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0))
  const laneOffset = ((idx % 5) - 2) * 6 // -12, -6, 0, +6, +12

  const pSrc: Point = { x: props.sourceX, y: props.sourceY }
  const pTgt: Point = { x: props.targetX, y: props.targetY }

  const pStart: Point = {
    x: props.sourcePosition === Position.Left ? props.sourceX - STUB : props.sourceX + STUB,
    y: props.sourceY,
  }
  const pEnd: Point = {
    x: props.targetPosition === Position.Left ? props.targetX - STUB : props.targetX + STUB,
    y: props.targetY,
  }

  let rawPoints: Point[] = []

  // Case 1: Source Right -> Target Left (Standard left-to-right)
  if (props.sourcePosition === Position.Right && props.targetPosition === Position.Left) {
    if (pStart.x < pEnd.x) {
      const xMid = (pStart.x + pEnd.x) / 2 + laneOffset
      const tryPoints: Point[] = [
        pSrc,
        pStart,
        { x: xMid, y: pStart.y },
        { x: xMid, y: pEnd.y },
        pEnd,
        pTgt,
      ]

      const hits = getIntersectingObstacles(tryPoints, obstacles, [props.source, props.target])
      if (hits.length === 0) {
        rawPoints = tryPoints
      } else {
        // Detour above or below obstacles
        const minTop = Math.min(...hits.map(h => h.top))
        const maxBottom = Math.max(...hits.map(h => h.bottom))
        const avgY = (pStart.y + pEnd.y) / 2
        const safeYAbove = minTop - 20 - Math.abs(laneOffset)
        const safeYBelow = maxBottom + 20 + Math.abs(laneOffset)

        const safeY = Math.abs(safeYAbove - avgY) <= Math.abs(safeYBelow - avgY) ? safeYAbove : safeYBelow
        rawPoints = [
          pSrc,
          pStart,
          { x: pStart.x, y: safeY },
          { x: pEnd.x, y: safeY },
          pEnd,
          pTgt,
        ]
      }
    } else {
      // Reversed/overlapping: loop around
      const minTop = Math.min(pStart.y, pEnd.y) - 30 - Math.abs(laneOffset)
      rawPoints = [
        pSrc,
        pStart,
        { x: pStart.x, y: minTop },
        { x: pEnd.x, y: minTop },
        pEnd,
        pTgt,
      ]
    }
  }
  // Case 2: Source Left -> Target Right (Right-to-left)
  else if (props.sourcePosition === Position.Left && props.targetPosition === Position.Right) {
    if (pStart.x > pEnd.x) {
      const xMid = (pStart.x + pEnd.x) / 2 + laneOffset
      const tryPoints: Point[] = [
        pSrc,
        pStart,
        { x: xMid, y: pStart.y },
        { x: xMid, y: pEnd.y },
        pEnd,
        pTgt,
      ]

      const hits = getIntersectingObstacles(tryPoints, obstacles, [props.source, props.target])
      if (hits.length === 0) {
        rawPoints = tryPoints
      } else {
        const minTop = Math.min(...hits.map(h => h.top))
        const maxBottom = Math.max(...hits.map(h => h.bottom))
        const avgY = (pStart.y + pEnd.y) / 2
        const safeYAbove = minTop - 20 - Math.abs(laneOffset)
        const safeYBelow = maxBottom + 20 + Math.abs(laneOffset)

        const safeY = Math.abs(safeYAbove - avgY) <= Math.abs(safeYBelow - avgY) ? safeYAbove : safeYBelow
        rawPoints = [
          pSrc,
          pStart,
          { x: pStart.x, y: safeY },
          { x: pEnd.x, y: safeY },
          pEnd,
          pTgt,
        ]
      }
    } else {
      const minTop = Math.min(pStart.y, pEnd.y) - 30 - Math.abs(laneOffset)
      rawPoints = [
        pSrc,
        pStart,
        { x: pStart.x, y: minTop },
        { x: pEnd.x, y: minTop },
        pEnd,
        pTgt,
      ]
    }
  }
  // Case 3: Both Right (C-shape outer right corridor)
  else if (props.sourcePosition === Position.Right && props.targetPosition === Position.Right) {
    let maxRight = Math.max(pStart.x, pEnd.x)
    const minY = Math.min(pStart.y, pEnd.y)
    const maxY = Math.max(pStart.y, pEnd.y)

    for (const obs of obstacles) {
      if (maxY >= obs.top && minY <= obs.bottom) {
        if (obs.right > maxRight) {
          maxRight = obs.right
        }
      }
    }
    const xCorridor = maxRight + 20 + laneOffset
    rawPoints = [
      pSrc,
      pStart,
      { x: xCorridor, y: pStart.y },
      { x: xCorridor, y: pEnd.y },
      pEnd,
      pTgt,
    ]
  }
  // Case 4: Both Left (C-shape outer left corridor)
  else if (props.sourcePosition === Position.Left && props.targetPosition === Position.Left) {
    let minLeft = Math.min(pStart.x, pEnd.x)
    const minY = Math.min(pStart.y, pEnd.y)
    const maxY = Math.max(pStart.y, pEnd.y)

    for (const obs of obstacles) {
      if (maxY >= obs.top && minY <= obs.bottom) {
        if (obs.left < minLeft) {
          minLeft = obs.left
        }
      }
    }
    const xCorridor = minLeft - 20 - laneOffset
    rawPoints = [
      pSrc,
      pStart,
      { x: xCorridor, y: pStart.y },
      { x: xCorridor, y: pEnd.y },
      pEnd,
      pTgt,
    ]
  }
  // Fallback: Default orthogonal
  else {
    const xMid = (pStart.x + pEnd.x) / 2
    rawPoints = [
      pSrc,
      pStart,
      { x: xMid, y: pStart.y },
      { x: xMid, y: pEnd.y },
      pEnd,
      pTgt,
    ]
  }

  return pointsToRoundedSvgPath(rawPoints, 8)
})
</script>

<template>
  <g class="smart-obstacle-edge-group">
    <defs>
      <marker
        id="collink-arrow-marker"
        viewBox="0 0 10 10"
        refX="7"
        refY="5"
        markerWidth="6"
        markerHeight="6"
        orient="auto-start-reverse"
      >
        <path d="M 0 1.5 L 8 5 L 0 8.5 z" fill="#3B82F6" />
      </marker>
    </defs>
    <!-- Background hit area for easy clicking and hover -->
    <path
      :d="edgePath"
      class="vue-flow__edge-interaction"
      fill="none"
      stroke="transparent"
      stroke-width="16"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <!-- Visible animated path -->
    <path
      :id="id"
      :d="edgePath"
      class="vue-flow__edge-path smart-obstacle-edge is-animated"
      :style="style"
      marker-end="url(#collink-arrow-marker)"
    />
  </g>
</template>

<style scoped>
.smart-obstacle-edge-group {
  pointer-events: visibleStroke;
}

.vue-flow__edge-interaction {
  cursor: pointer;
  pointer-events: stroke;
}

.smart-obstacle-edge {
  fill: none;
  stroke: #2563eb;
  stroke-width: 2px;
  stroke-linecap: round;
  stroke-linejoin: round;
  transition: stroke 0.2s, stroke-width 0.2s, filter 0.2s;
  pointer-events: stroke;
}

.smart-obstacle-edge.is-animated {
  stroke-dasharray: 6 4;
  animation: edgeDashFlow 0.8s linear infinite;
}

.smart-obstacle-edge-group:hover .smart-obstacle-edge {
  stroke: #1d4ed8;
  stroke-width: 3px;
  filter: drop-shadow(0 0 5px rgba(29, 78, 216, 0.45));
}

@keyframes edgeDashFlow {
  from {
    stroke-dashoffset: 20;
  }
  to {
    stroke-dashoffset: 0;
  }
}
</style>
