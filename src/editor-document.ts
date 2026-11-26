import { paintMark, type Box, type ImageSize, type Mark } from './markup'

export type Document = { marks: Mark[]; crop?: Box }
export type History = { past: Document[]; current: Document; future: Document[] }

export const emptyHistory: History = { past: [], current: { marks: [] }, future: [] }

const tools = new Set(['pen', 'rectangle', 'arrow', 'line', 'ellipse', 'highlighter', 'text', 'step', 'blur', 'mosaic', 'redact'])
const color = /^#[0-9a-f]{6}$/i

export function restoreDocument(value: unknown, size: ImageSize): Document {
  if (value === null) return emptyHistory.current
  const doc = value as Document
  const invalid = () => Error('The editable project contains invalid annotations.')
  if (!doc || !Array.isArray(doc.marks)) throw invalid()
  const seen = new Set<string>()
  for (const mark of doc.marks) {
    if (!mark || typeof mark.id !== 'string' || seen.has(mark.id) || !tools.has(mark.tool)
      || !Array.isArray(mark.points) || !mark.points.length || !color.test(mark.color)
      || !Number.isFinite(mark.width) || mark.width < 1 || mark.width > 2048
      || !Number.isFinite(mark.opacity) || mark.opacity < 0 || mark.opacity > 1
      || (mark.text !== undefined && typeof mark.text !== 'string')
      || (mark.fontSize !== undefined && (!Number.isFinite(mark.fontSize) || mark.fontSize < 8 || mark.fontSize > 240))
      || (mark.background !== undefined && !color.test(mark.background))) throw invalid()
    if (mark.points.some((point) => !point || !Number.isFinite(point.x) || !Number.isFinite(point.y)
      || Math.abs(point.x) > size.width * 8 || Math.abs(point.y) > size.height * 8)) throw invalid()
    seen.add(mark.id)
  }
  const crop = doc.crop
  if (crop && (![crop.x, crop.y, crop.width, crop.height].every(Number.isInteger)
    || crop.x < 0 || crop.y < 0 || crop.width < 1 || crop.height < 1
    || crop.x + crop.width > size.width || crop.y + crop.height > size.height)) throw invalid()
  return doc
}

export function commit(history: History, current: Document): History {
  if (current === history.current) return history
  return { past: [...history.past.slice(-49), history.current], current, future: [] }
}

export function undo(history: History): History {
  const previous = history.past.at(-1)
  if (!previous) return history
  return { past: history.past.slice(0, -1), current: previous, future: [history.current, ...history.future] }
}

export function redo(history: History): History {
  const next = history.future[0]
  if (!next) return history
  return { past: [...history.past, history.current], current: next, future: history.future.slice(1) }
}

export function renderDocument(canvas: HTMLCanvasElement, source: HTMLCanvasElement, marks: Mark[]) {
  const context = canvas.getContext('2d', { willReadFrequently: true })
  if (!context) throw Error('Could not prepare the image.')
  context.clearRect(0, 0, canvas.width, canvas.height)
  context.drawImage(source, 0, 0)
  for (const mark of marks) paintMark(context, mark)
  return context
}

export function compose(source: HTMLCanvasElement, doc: Document) {
  const canvas = document.createElement('canvas')
  canvas.width = source.width
  canvas.height = source.height
  renderDocument(canvas, source, doc.marks)
  if (!doc.crop) return canvas
  const { x, y, width, height } = doc.crop
  const crop = document.createElement('canvas')
  crop.width = width
  crop.height = height
  const context = crop.getContext('2d', { willReadFrequently: true })
  if (!context) throw Error('Could not prepare the crop.')
  context.drawImage(canvas, x, y, width, height, 0, 0, width, height)
  return crop
}

export function encode(canvas: HTMLCanvasElement, mime: string): Promise<Blob> {
  return new Promise((resolve, reject) => canvas.toBlob((blob) => {
    if (blob?.type === mime) resolve(blob)
    else reject(Error('The selected image format is unavailable.'))
  }, mime, .93))
}

export function imagePixels(canvas: HTMLCanvasElement) {
  const context = canvas.getContext('2d')
  if (!context) throw Error('Could not read the exported pixels.')
  return context.getImageData(0, 0, canvas.width, canvas.height).data
}

export function imagePacket(canvas: HTMLCanvasElement) {
  const pixels = imagePixels(canvas)
  const packet = new Uint8Array(8 + pixels.length)
  const header = new DataView(packet.buffer)
  header.setUint32(0, canvas.width, true)
  header.setUint32(4, canvas.height, true)
  packet.set(pixels, 8)
  return packet
}
