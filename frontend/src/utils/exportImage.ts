/**
 * Clean SVG-Serialization & 2x Offscreen Canvas Image Exporter
 *
 * Solves:
 * 1. Black block artifacts: explicitly fills solid #F8FAFC canvas background and strips problematic drop-shadow / backdrop filters.
 * 2. Giant whitespace: tightly computes bounding box of only active table nodes + edges with a clean padding.
 * 3. 2x Super Retina resolution without jagged lines or blurry text.
 */

export interface ExportImageOptions {
  canvasWrapper: HTMLElement
  nodes: Array<{ id: string; position: { x: number; y: number }; data?: any }>
  filename?: string
  padding?: number
  scale?: number
  backgroundColor?: string
}

export async function exportCanvasAsImage(options: ExportImageOptions): Promise<void> {
  const {
    canvasWrapper,
    nodes,
    filename = 'ColLink_Architecture.png',
    padding = 70,
    scale = 2,
    backgroundColor = '#F8FAFC',
  } = options

  if (nodes.length === 0) {
    throw new Error('画布中暂无数据表，无法导出')
  }

  // 1. Calculate tight bounding box of all table cards from actual DOM
  let minX = Infinity
  let minY = Infinity
  let maxX = -Infinity
  let maxY = -Infinity

  const nodeDimensions = new Map<string, { w: number; h: number }>()

  nodes.forEach(n => {
    const nodeEl = canvasWrapper.querySelector(`[data-id="${n.id}"] .table-card`) as HTMLElement
    const w = nodeEl ? nodeEl.offsetWidth : 280
    const h = nodeEl ? nodeEl.offsetHeight : 300
    nodeDimensions.set(n.id, { w, h })

    if (n.position.x < minX) minX = n.position.x
    if (n.position.y < minY) minY = n.position.y
    if (n.position.x + w > maxX) maxX = n.position.x + w
    if (n.position.y + h > maxY) maxY = n.position.y + h
  })

  const exportWidth = Math.round(maxX - minX + padding * 2)
  const exportHeight = Math.round(maxY - minY + padding * 2)

  // Offset vector to shift (minX, minY) to (padding, padding)
  const offsetX = -minX + padding
  const offsetY = -minY + padding

  // 2. Clone visible edge paths from SVG layer
  const edgesSvgEl = canvasWrapper.querySelector('.vue-flow__edges') as SVGSVGElement
  let edgesContent = ''

  if (edgesSvgEl) {
    // Collect all visible edge path elements
    const edgePaths = edgesSvgEl.querySelectorAll('.smart-obstacle-edge, .vue-flow__edge-path')
    edgePaths.forEach(p => {
      const d = p.getAttribute('d')
      if (d) {
        // Strip animations and drop-shadows for crisp static print
        edgesContent += `<path d="${d}" fill="none" stroke="#3B82F6" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" marker-end="url(#collink-arrow-marker)" />\n`
      }
    })
  }

  // 3. Clone and sanitize node HTML cards
  const nodesContent: string[] = []

  nodes.forEach(n => {
    const nodeCardEl = canvasWrapper.querySelector(`[data-id="${n.id}"] .table-card`) as HTMLElement
    if (!nodeCardEl) return

    const { w, h } = nodeDimensions.get(n.id) || { w: 280, h: 300 }
    const clone = nodeCardEl.cloneNode(true) as HTMLElement

    // Remove interactive edit widgets from export view
    clone.querySelectorAll('.field-handle, .table-card-search, .btn-more, .btn-filter-mapped, .table-node-menu').forEach(el => {
      el.remove()
    })

    // Neutralize any filters/shadows that might cause rasterizer bugs
    clone.style.filter = 'none'
    clone.style.boxShadow = '0 2px 8px rgba(15, 23, 42, 0.08)'
    clone.style.borderRadius = '9px'
    clone.style.border = '1px solid #CBD5E1'
    clone.style.backgroundColor = '#FFFFFF'
    clone.style.width = `${w}px`
    clone.style.height = `${h}px`
    clone.style.overflow = 'hidden'

    // Recursively strip filters, backdrop-filters, and animations
    clone.querySelectorAll('*').forEach((el: any) => {
      if (el.style) {
        el.style.filter = 'none'
        el.style.webkitFilter = 'none'
        el.style.backdropFilter = 'none'
        el.style.webkitBackdropFilter = 'none'
        el.style.animation = 'none'
      }
    })

    const nodeX = n.position.x + offsetX
    const nodeY = n.position.y + offsetY

    const serializer = new XMLSerializer()
    const cardHtml = serializer.serializeToString(clone)

    nodesContent.push(`
      <foreignObject x="${nodeX}" y="${nodeY}" width="${w}" height="${h}">
        <div xmlns="http://www.w3.org/1999/xhtml" style="width: 100%; height: 100%; margin: 0; padding: 0;">
          ${cardHtml}
        </div>
      </foreignObject>
    `)
  })

  // 4. Standalone self-contained CSS styles for foreignObject
  const inlineStyles = `
    * { box-sizing: border-box; }
    body, div, span, p, h1, h2, h3 { margin: 0; padding: 0; }
    .table-card {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
      background: #FFFFFF !important;
      border: 1px solid #CBD5E1 !important;
      border-radius: 9px !important;
      box-shadow: 0 2px 8px rgba(15, 23, 42, 0.08) !important;
      width: 280px;
      overflow: hidden;
      position: relative;
    }
    .table-top-line {
      height: 3px;
      background: #3B82F6;
      width: 100%;
    }
    .table-card-header {
      padding: 10px 12px;
      background: #F8FAFC;
      border-bottom: 1px solid #E2E8F0;
      display: flex;
      align-items: center;
      justify-content: space-between;
    }
    .header-left {
      display: flex;
      align-items: center;
      gap: 8px;
      overflow: hidden;
    }
    .table-icon-wrap {
      width: 24px;
      height: 24px;
      border-radius: 6px;
      background: #EFF6FF;
      color: #3B82F6;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
    }
    .table-icon-wrap svg {
      width: 14px;
      height: 14px;
      fill: currentColor;
    }
    .table-titles {
      display: flex;
      flex-direction: column;
      overflow: hidden;
    }
    .table-name {
      font-size: 13px;
      font-weight: 600;
      color: #0F172A;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      line-height: 1.3;
    }
    .table-comment {
      font-size: 11px;
      color: #64748B;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      line-height: 1.3;
    }
    .header-right {
      display: flex;
      align-items: center;
    }
    .col-count-badge {
      font-size: 11px;
      font-weight: 600;
      color: #3B82F6;
      background: #EFF6FF;
      border: 1px solid #DBEAFE;
      border-radius: 12px;
      padding: 1px 7px;
    }
    .column-list {
      padding: 4px 0;
      background: #FFFFFF;
    }
    .column-item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 6px 12px;
      font-size: 12px;
      border-bottom: 1px solid #F8FAFC;
    }
    .col-content {
      display: flex;
      align-items: center;
      justify-content: space-between;
      width: 100%;
    }
    .col-info {
      display: flex;
      flex-direction: column;
      overflow: hidden;
      max-width: 175px;
    }
    .col-name {
      font-size: 12px;
      font-weight: 500;
      color: #1E293B;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      line-height: 1.4;
    }
    .col-comment {
      font-size: 10px;
      color: #94A3B8;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      line-height: 1.3;
    }
    .col-right-group {
      display: flex;
      align-items: center;
      gap: 6px;
      flex-shrink: 0;
    }
    .col-type {
      font-size: 10px;
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      padding: 1px 5px;
      border-radius: 4px;
      white-space: nowrap;
      font-weight: 500;
      background: #F1F5F9;
      color: #475569;
    }
    .type-num { background: #FEF3C7; color: #92400E; }
    .type-str { background: #EEF2FF; color: #3730A3; }
    .type-date { background: #DCFCE7; color: #166534; }
    .type-other { background: #F1F5F9; color: #475569; }
    .mapping-icon-wrap {
      display: flex;
      align-items: center;
      color: #3B82F6;
    }
    .mapping-icon-wrap svg {
      width: 12px;
      height: 12px;
    }
  `

  // 5. Construct Master SVG String
  const svgString = `
    <svg xmlns="http://www.w3.org/2000/svg" width="${exportWidth}" height="${exportHeight}" viewBox="0 0 ${exportWidth} ${exportHeight}">
      <defs>
        <style type="text/css">
          ${inlineStyles}
        </style>
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

      <!-- 1. Explicit Solid Canvas Background (Prevents any black transparency compositing) -->
      <rect width="100%" height="100%" fill="${backgroundColor}" />

      <!-- 2. Edges Layer -->
      <g transform="translate(${offsetX}, ${offsetY})">
        ${edgesContent}
      </g>

      <!-- 3. Nodes Layer -->
      <g>
        ${nodesContent.join('\n')}
      </g>
    </svg>
  `

  // 6. Rasterize via 2x Offscreen Canvas
  const svgBlob = new Blob([svgString], { type: 'image/svg+xml;charset=utf-8' })
  const svgUrl = URL.createObjectURL(svgBlob)

  try {
    const img = new Image()
    img.crossOrigin = 'anonymous'

    await new Promise<void>((resolve, reject) => {
      img.onload = () => resolve()
      img.onerror = () => reject(new Error('SVG 序列化生成失败，请重试'))
      img.src = svgUrl
    })

    const canvas = document.createElement('canvas')
    canvas.width = Math.round(exportWidth * scale)
    canvas.height = Math.round(exportHeight * scale)

    const ctx = canvas.getContext('2d')
    if (!ctx) {
      throw new Error('无法初始化 Canvas 绘图上下文')
    }

    // Explicit solid background on Canvas
    ctx.fillStyle = backgroundColor
    ctx.fillRect(0, 0, canvas.width, canvas.height)

    // 2x Retina draw
    ctx.scale(scale, scale)
    ctx.drawImage(img, 0, 0, exportWidth, exportHeight)

    // Convert to PNG Blob and trigger download
    const pngBlob: Blob = await new Promise((resolve, reject) => {
      canvas.toBlob(blob => {
        if (blob) resolve(blob)
        else reject(new Error('Canvas 转 PNG 数据流失败'))
      }, 'image/png')
    })

    const downloadUrl = URL.createObjectURL(pngBlob)
    const downloadLink = document.createElement('a')
    downloadLink.download = filename
    downloadLink.href = downloadUrl
    document.body.appendChild(downloadLink)
    downloadLink.click()
    document.body.removeChild(downloadLink)

    setTimeout(() => {
      URL.revokeObjectURL(downloadUrl)
    }, 1500)
  } finally {
    URL.revokeObjectURL(svgUrl)
  }
}
