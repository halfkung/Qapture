export type Tool = 'pen' | 'rectangle' | 'arrow' | 'line' | 'ellipse' | 'highlighter' | 'text' | 'step' | 'blur' | 'mosaic' | 'redact'
export type Point = { x: number; y: number }
export type ImageSize = { width: number; height: number }
export type Box = ImageSize & Point
export type Mark = {
  id: string
  tool: Tool
  points: Point[]
  color: string
  width: number
  opacity: number
  text?: string
  fontSize?: number
  family?: string
  bold?: boolean
  italic?: boolean
  align?: CanvasTextAlign
  background?: string
}
type Geometry = Pick<Mark, 'tool' | 'points'>

export function imagePoint(client: Point, bounds: DOMRect, size: ImageSize): Point {
  return {
    x: Math.max(0, Math.min(size.width, (client.x - bounds.left) * size.width / bounds.width)),
    y: Math.max(0, Math.min(size.height, (client.y - bounds.top) * size.height / bounds.height)),
  }
}

export function boundsOf(points: Point[]): Box {
  let left = Infinity, top = Infinity, right = -Infinity, bottom = -Infinity
  for (const { x, y } of points) {
    left = Math.min(left, x)
    top = Math.min(top, y)
    right = Math.max(right, x)
    bottom = Math.max(bottom, y)
  }
  return { x: left, y: top, width: right - left, height: bottom - top }
}

export function selectionBox(mark: Mark): Box {
  const box = boundsOf(mark.points)
  const padding = mark.tool === 'step' ? mark.width : Math.max(mark.width / 2, 3)
  return { x: box.x - padding, y: box.y - padding, width: box.width + padding * 2, height: box.height + padding * 2 }
}

export function visibleMark(mark: Geometry) {
  if (['pen', 'highlighter', 'step'].includes(mark.tool)) return true
  const { width, height } = boundsOf(mark.points)
  return ['arrow', 'line'].includes(mark.tool) ? width > 0 || height > 0 : width > 0 && height > 0
}

export function translate(mark: Mark, dx: number, dy: number): Mark {
  return { ...mark, points: mark.points.map(({ x, y }) => ({ x: x + dx, y: y + dy })) }
}

export function resize(mark: Mark, target: Point): Mark {
  const box = boundsOf(mark.points)
  if (mark.tool === 'step') return { ...mark, width: Math.max(4, Math.hypot(target.x - box.x, target.y - box.y) / Math.SQRT2) }
  const width = Math.max(2, target.x - box.x)
  const height = Math.max(2, target.y - box.y)
  return { ...mark, points: mark.points.map(({ x, y }, index) => ({
    x: box.x + (box.width ? (x - box.x) * width / box.width : index ? width : 0),
    y: box.y + (box.height ? (y - box.y) * height / box.height : index ? height : 0),
  })) }
}

export function inside(point: Point, box: Box, margin = 0) {
  return point.x >= box.x - margin && point.x <= box.x + box.width + margin
    && point.y >= box.y - margin && point.y <= box.y + box.height + margin
}

function paintText(ctx: CanvasRenderingContext2D, mark: Mark, box: Box) {
  const size = mark.fontSize ?? 24
  const align = mark.align ?? 'left'
  if (mark.background) {
    ctx.fillStyle = mark.background
    ctx.fillRect(box.x, box.y, box.width, box.height)
  }
  ctx.beginPath()
  ctx.rect(box.x, box.y, box.width, box.height)
  ctx.clip()
  ctx.fillStyle = mark.color
  ctx.font = [mark.italic ? 'italic' : '', mark.bold ? 'bold' : '', size + 'px', '"' + (mark.family ?? 'Segoe UI') + '", Leelawadee UI, sans-serif'].join(' ')
  ctx.textAlign = align
  const x = align === 'center' ? box.x + box.width / 2 : align === 'right' ? box.x + box.width - 4 : box.x + 4
  let y = box.y + size * 1.15
  for (const line of (mark.text ?? '').split('\n')) {
    ctx.fillText(line, x, y, Math.max(1, box.width - 8))
    y += size * 1.4
  }
}

function paintEffect(ctx: CanvasRenderingContext2D, mark: Mark, box: Box) {
  const pixel = document.createElement('canvas')
  const block = Math.max(4, mark.width)
  pixel.width = Math.max(1, Math.ceil(mark.tool === 'mosaic' ? box.width / block : box.width))
  pixel.height = Math.max(1, Math.ceil(mark.tool === 'mosaic' ? box.height / block : box.height))
  const layer = pixel.getContext('2d')
  if (!layer) return
  layer.drawImage(ctx.canvas, box.x, box.y, box.width, box.height, 0, 0, pixel.width, pixel.height)
  ctx.beginPath()
  ctx.rect(box.x, box.y, box.width, box.height)
  ctx.clip()
  ctx.imageSmoothingEnabled = mark.tool !== 'mosaic'
  if (mark.tool === 'blur') ctx.filter = 'blur(' + block + 'px)'
  ctx.drawImage(pixel, box.x, box.y, box.width, box.height)
}

export function paintMark(ctx: CanvasRenderingContext2D, mark: Mark) {
  if (!visibleMark(mark)) return
  const first = mark.points[0]
  const last = mark.points.at(-1)!
  const box = boundsOf(mark.points)
  ctx.save()
  ctx.globalAlpha = mark.tool === 'redact' ? 1 : mark.opacity
  ctx.strokeStyle = ctx.fillStyle = mark.color
  ctx.lineWidth = mark.width
  ctx.lineJoin = ctx.lineCap = 'round'
  ctx.beginPath()
  switch (mark.tool) {
    case 'blur':
    case 'mosaic':
      paintEffect(ctx, mark, box)
      break
    case 'redact':
      ctx.fillRect(Math.floor(box.x), Math.floor(box.y), Math.ceil(box.x + box.width) - Math.floor(box.x), Math.ceil(box.y + box.height) - Math.floor(box.y))
      break
    case 'text':
      paintText(ctx, mark, box)
      break
    case 'step':
      ctx.arc(first.x, first.y, mark.width, 0, Math.PI * 2)
      ctx.fill()
      ctx.fillStyle = '#fff'
      ctx.textAlign = 'center'
      ctx.textBaseline = 'middle'
      ctx.font = 'bold ' + mark.width + 'px "Segoe UI", sans-serif'
      ctx.fillText(mark.text ?? '1', first.x, first.y)
      break
    case 'rectangle':
      ctx.strokeRect(box.x, box.y, box.width, box.height)
      break
    case 'ellipse':
      ctx.ellipse(box.x + box.width / 2, box.y + box.height / 2, box.width / 2, box.height / 2, 0, 0, Math.PI * 2)
      ctx.stroke()
      break
    default:
      if (mark.points.length === 1) {
        ctx.arc(first.x, first.y, mark.width / 2, 0, Math.PI * 2)
        ctx.fill()
        break
      }
      ctx.moveTo(first.x, first.y)
      for (const point of mark.points.slice(1)) ctx.lineTo(point.x, point.y)
      if (mark.tool === 'arrow') {
        const angle = Math.atan2(last.y - first.y, last.x - first.x)
        const head = Math.min(Math.hypot(last.x - first.x, last.y - first.y) * .6, Math.max(12, mark.width * 3.5))
        for (const offset of [-Math.PI / 6, Math.PI / 6]) {
          ctx.moveTo(last.x, last.y)
          ctx.lineTo(last.x - head * Math.cos(angle + offset), last.y - head * Math.sin(angle + offset))
        }
      }
      ctx.stroke()
  }
  ctx.restore()
}

export function decodeImage(packet: ArrayBuffer) {
  if (packet.byteLength < 8) throw Error('The capture could not be decoded.')
  const header = new DataView(packet)
  const width = header.getUint32(0, true)
  const height = header.getUint32(4, true)
  if (!width || !height || width * height * 4 !== packet.byteLength - 8) {
    throw Error('The capture has invalid dimensions.')
  }
  return { width, height, pixels: new Uint8ClampedArray(packet, 8) }
}
