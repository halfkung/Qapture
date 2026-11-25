export type Tool = 'pen' | 'rectangle' | 'arrow'
export type Point = { x: number; y: number }
export type ImageSize = { width: number; height: number }
export type Mark = { tool: Tool; points: Point[]; color: string; width: number }

export function imagePoint(client: Point, bounds: DOMRect, size: ImageSize): Point {
  return {
    x: Math.max(0, Math.min(size.width, (client.x - bounds.left) * size.width / bounds.width)),
    y: Math.max(0, Math.min(size.height, (client.y - bounds.top) * size.height / bounds.height)),
  }
}

export function visibleMark(mark: Mark) {
  if (mark.tool === 'pen') return true
  const first = mark.points[0]
  const last = mark.points.at(-1)!
  return mark.tool === 'rectangle'
    ? last.x !== first.x && last.y !== first.y
    : last.x !== first.x || last.y !== first.y
}

export function paintMark(ctx: CanvasRenderingContext2D, mark: Mark) {
  if (!visibleMark(mark)) return
  const first = mark.points[0]
  const last = mark.points.at(-1)!
  ctx.strokeStyle = mark.color
  ctx.fillStyle = mark.color
  ctx.lineWidth = mark.width
  ctx.lineJoin = 'round'
  ctx.lineCap = 'round'
  ctx.beginPath()
  if (mark.tool === 'rectangle') {
    ctx.strokeRect(first.x, first.y, last.x - first.x, last.y - first.y)
  } else if (mark.tool === 'pen') {
    if (mark.points.length === 1) {
      ctx.arc(first.x, first.y, mark.width / 2, 0, 2 * Math.PI)
      ctx.fill()
      return
    }
    ctx.moveTo(first.x, first.y)
    for (const point of mark.points.slice(1)) ctx.lineTo(point.x, point.y)
    ctx.stroke()
  } else {
    const angle = Math.atan2(last.y - first.y, last.x - first.x)
    const distance = Math.hypot(last.x - first.x, last.y - first.y)
    const head = Math.min(distance * .6, Math.max(12, mark.width * 3.5))
    ctx.moveTo(first.x, first.y)
    ctx.lineTo(last.x, last.y)
    for (const offset of [-Math.PI / 6, Math.PI / 6]) {
      ctx.moveTo(last.x, last.y)
      ctx.lineTo(last.x - head * Math.cos(angle + offset), last.y - head * Math.sin(angle + offset))
    }
    ctx.stroke()
  }
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
