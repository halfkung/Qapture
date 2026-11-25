import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { ArrowUpRight, Clipboard, Pencil, Redo2, Square, Undo2, X } from 'lucide-react'
import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from 'react'
import icon from '../src-tauri/icons/icon.ico?url'
import { decodeImage, imagePoint, paintMark, visibleMark, type ImageSize, type Mark, type Tool } from './markup'
import './editor.css'

const tools = [
  { id: 'pen', label: 'Pen', icon: Pencil },
  { id: 'rectangle', label: 'Rectangle', icon: Square },
  { id: 'arrow', label: 'Arrow', icon: ArrowUpRight },
] as const

const colors = [
  { label: 'Coral', value: '#f04462' },
  { label: 'Amber', value: '#ffbc43' },
  { label: 'Mint', value: '#45dc9c' },
  { label: 'Violet', value: '#7260ff' },
  { label: 'White', value: '#ffffff' },
  { label: 'Black', value: '#171726' },
]

export function Editor() {
  const [image, setImage] = useState<ReturnType<typeof decodeImage>>()
  const [tool, setTool] = useState<Tool>('pen')
  const [color, setColor] = useState(colors[0].value)
  const [width, setWidth] = useState(5)
  const [history, setHistory] = useState<{ marks: Mark[]; undone: Mark[] }>({ marks: [], undone: [] })
  const [draft, setDraft] = useState<Mark>()
  const [available, setAvailable] = useState<ImageSize>({ width: 0, height: 0 })
  const [error, setError] = useState('')
  const [notice, setNotice] = useState('')
  const [copying, setCopying] = useState(false)
  const opening = useRef<Promise<ReturnType<typeof decodeImage>> | null>(null)
  const editor = useRef<HTMLElement>(null)
  const workspace = useRef<HTMLDivElement>(null)
  const base = useRef<HTMLCanvasElement>(null)
  const markup = useRef<HTMLCanvasElement>(null)
  const stroke = useRef<{ pointer: number; mark: Mark } | null>(null)

  useEffect(() => {
    let active = true
    opening.current ??= invoke<ArrayBuffer>('load_edit_image').then(decodeImage)
    void opening.current.then((capture) => {
      if (active) setImage(capture)
    }).catch((problem) => {
      if (active) setError(String(problem))
    })
    return () => { active = false }
  }, [])

  useLayoutEffect(() => {
    editor.current?.focus()
    const target = workspace.current
    if (!target) return
    const observer = new ResizeObserver(([entry]) => {
      setAvailable({ width: entry.contentRect.width, height: entry.contentRect.height })
    })
    observer.observe(target)
    return () => observer.disconnect()
  }, [])

  useLayoutEffect(() => {
    if (!image) return
    const ctx = base.current?.getContext('2d')
    if (!ctx) return
    ctx.putImageData(new ImageData(image.pixels, image.width, image.height), 0, 0)
  }, [image])

  useLayoutEffect(() => {
    if (!image) return
    const ctx = markup.current?.getContext('2d')
    if (!ctx) return
    ctx.clearRect(0, 0, image.width, image.height)
    for (const mark of history.marks) paintMark(ctx, mark)
    if (draft) paintMark(ctx, draft)
  }, [image, history.marks, draft])

  function undo() {
    setHistory(({ marks, undone }) => {
      const last = marks.at(-1)
      return last ? { marks: marks.slice(0, -1), undone: [...undone, last] } : { marks, undone }
    })
  }

  function redo() {
    setHistory(({ marks, undone }) => {
      const next = undone.at(-1)
      return next ? { marks: [...marks, next], undone: undone.slice(0, -1) } : { marks, undone }
    })
  }

  function startMark(event: PointerEvent<HTMLCanvasElement>) {
    if (!image || event.button !== 0 || stroke.current) return
    const point = imagePoint({ x: event.clientX, y: event.clientY }, event.currentTarget.getBoundingClientRect(), image)
    const mark = { tool, color, width, points: [point] }
    stroke.current = { pointer: event.pointerId, mark }
    event.currentTarget.focus()
    event.currentTarget.setPointerCapture(event.pointerId)
    setNotice('')
    setDraft(mark)
  }

  function extendMark(event: PointerEvent<HTMLCanvasElement>) {
    const active = stroke.current
    if (!image || active?.pointer !== event.pointerId) return
    const next = imagePoint({ x: event.clientX, y: event.clientY }, event.currentTarget.getBoundingClientRect(), image)
    const points = active.mark.points
    const last = points.at(-1)!
    if (next.x === last.x && next.y === last.y) return
    active.mark = { ...active.mark, points: active.mark.tool === 'pen' ? [...points, next] : [points[0], next] }
    setDraft(active.mark)
  }

  function endMark(event: PointerEvent<HTMLCanvasElement>) {
    if (stroke.current?.pointer !== event.pointerId) return
    extendMark(event)
    const mark = stroke.current.mark
    stroke.current = null
    setDraft(undefined)
    if (visibleMark(mark)) {
      setHistory(({ marks }) => ({ marks: [...marks, mark], undone: [] }))
    }
  }

  function cancelMark(event: PointerEvent<HTMLCanvasElement>) {
    if (stroke.current?.pointer === event.pointerId) {
      stroke.current = null
      setDraft(undefined)
    }
  }

  async function copyImage() {
    if (!image || !base.current || copying || draft) return
    setCopying(true)
    setError('')
    setNotice('')
    try {
      const composed = document.createElement('canvas')
      composed.width = image.width
      composed.height = image.height
      const ctx = composed.getContext('2d', { willReadFrequently: true })
      if (!ctx) throw Error('Could not prepare the image to copy.')
      ctx.drawImage(base.current, 0, 0)
      for (const mark of history.marks) paintMark(ctx, mark)
      const { data } = ctx.getImageData(0, 0, image.width, image.height)
      await invoke('copy_edited_image', new Uint8Array(data.buffer))
      setNotice('Copied to clipboard')
    } catch (problem) {
      setError(String(problem))
    } finally {
      setCopying(false)
    }
  }

  function close() {
    void getCurrentWindow().close().catch((problem) => setError(String(problem)))
  }

  function keys(event: KeyboardEvent<HTMLElement>) {
    if (event.key === 'Escape') {
      event.preventDefault()
      close()
    } else if (event.ctrlKey && !event.altKey && ['KeyC', 'KeyZ', 'KeyY'].includes(event.code)) {
      event.preventDefault()
      if (event.repeat || draft || copying) return
      if (event.code === 'KeyC') void copyImage()
      else if (event.code === 'KeyY' || event.shiftKey) redo()
      else undo()
    }
  }

  const scale = image ? Math.min(1, available.width / image.width, available.height / image.height) : 0

  return (
    <main className="studio" ref={editor} tabIndex={-1} onKeyDown={keys}>
      <header className="studio-header">
        <img src={icon} alt="" width="30" height="30" />
        <div className="studio-heading">
          <h1>Quick editor</h1>
          {image && <p>{image.width} × {image.height} px</p>}
        </div>
        <button type="button" className="studio-copy" onClick={() => void copyImage()} disabled={!image || copying || !!draft}>
          <Clipboard size={17} aria-hidden="true" /> {copying ? 'Copying…' : 'Copy image'}
        </button>
        <button type="button" className="studio-icon" aria-label="Close editor" title="Close editor (Esc)" onClick={close}>
          <X size={20} aria-hidden="true" />
        </button>
      </header>

      <div className="studio-tools" aria-label="Markup controls" role="group">
        <div className="studio-tool-group">
          {tools.map(({ id, label, icon: Icon }) => (
            <button key={id} type="button" className="studio-tool" aria-pressed={tool === id}
              onClick={() => setTool(id)} title={label} disabled={!image}>
              <Icon size={17} aria-hidden="true" /> <span>{label}</span>
            </button>
          ))}
        </div>
        <div className="studio-colors" role="group" aria-label="Color">
          {colors.map((item) => (
            <button key={item.value} type="button" aria-label={item.label} title={item.label}
              aria-pressed={item.value === color} onClick={() => setColor(item.value)}
              style={{ backgroundColor: item.value }} disabled={!image} />
          ))}
        </div>
        <label className="stroke-control">Stroke
          <input type="range" min="2" max="22" value={width} onChange={(event) => setWidth(Number(event.target.value))} disabled={!image} />
          <span>{width} px</span>
        </label>
        <div className="studio-history">
          <button type="button" className="studio-icon" aria-label="Undo" title="Undo (Ctrl+Z)" onClick={undo} disabled={!history.marks.length || !!draft}>
            <Undo2 size={18} aria-hidden="true" />
          </button>
          <button type="button" className="studio-icon" aria-label="Redo" title="Redo (Ctrl+Y)" onClick={redo} disabled={!history.undone.length || !!draft}>
            <Redo2 size={18} aria-hidden="true" />
          </button>
        </div>
      </div>

      <div className="studio-viewport">
        <div className="studio-workspace" ref={workspace}>
          {image ? (
            <div className="studio-stage" style={{ width: image.width * scale, height: image.height * scale }}>
              <canvas ref={base} className="studio-image" width={image.width} height={image.height} aria-label="Captured region" />
              <canvas ref={markup} className="studio-markup" width={image.width} height={image.height} tabIndex={0} aria-label="Draw on the capture"
                onPointerDown={startMark} onPointerMove={extendMark} onPointerUp={endMark}
                onPointerCancel={cancelMark} onLostPointerCapture={cancelMark} />
            </div>
          ) : !error && <p role="status">Opening capture…</p>}
        </div>
      </div>
      <footer className="studio-footer">
        <p role="status" className={error ? 'error' : undefined}>{error || notice}</p>
        {scale > 0 && <span>{Math.round(scale * 100)}%</span>}
      </footer>
    </main>
  )
}
