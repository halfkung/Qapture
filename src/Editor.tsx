import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { ArrowDownToLine, ArrowUpRight, Blend, Circle, Clipboard, Crop, Grid2X2, Hash, Highlighter, Minus, MousePointer2, Pencil, Pin, Plus, Redo2, Square, Type, Undo2, X } from 'lucide-react'
import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from 'react'
import icon from '../src-tauri/icons/icon.ico?url'
import { boundsOf, decodeImage, imagePoint, inside, resize, selectionBox, translate, visibleMark, type ImageSize, type Mark, type Point, type Tool } from './markup'
import { commit, compose, emptyHistory, encode, imagePacket, imagePixels, redo, renderDocument, restoreDocument, undo, type Document } from './editor-document'
import { defaults, EditorProperties, type Properties } from './EditorProperties'
import type { OutputPreferences } from './History'
import './editor.css'

type EditorTool = Tool | 'select' | 'crop'
const tools = [
  { id: 'select', label: 'Select', icon: MousePointer2, key: 'V' },
  { id: 'pen', label: 'Pen', icon: Pencil, key: 'P' },
  { id: 'rectangle', label: 'Rectangle', icon: Square, key: 'R' },
  { id: 'ellipse', label: 'Ellipse', icon: Circle, key: 'O' },
  { id: 'arrow', label: 'Arrow', icon: ArrowUpRight, key: 'A' },
  { id: 'line', label: 'Line', icon: Minus, key: 'L' },
  { id: 'highlighter', label: 'Highlighter', icon: Highlighter, key: 'H' },
  { id: 'text', label: 'Text', icon: Type, key: 'T' },
  { id: 'step', label: 'Step number', icon: Hash, key: 'N' },
  { id: 'blur', label: 'Blur', icon: Blend, key: 'B' },
  { id: 'mosaic', label: 'Mosaic', icon: Grid2X2 },
  { id: 'redact', label: 'Solid redaction', icon: Square },
  { id: 'crop', label: 'Crop', icon: Crop },
] as const

type Gesture = { pointer: number; start: Point; action: 'draw' | 'move' | 'resize' | 'crop'; original: Mark; mark: Mark }
const preferencesKey = 'qapture.editor.tools'
const presetsKey = 'qapture.editor.presets'
type Preset = { name: string; values: Properties }

function loadPresets(): Preset[] {
  try {
    const saved = JSON.parse(localStorage.getItem(presetsKey) ?? '[]')
    return Array.isArray(saved) ? saved : []
  } catch { return [] }
}

function loadPreferences(): Partial<Record<EditorTool, Properties>> {
  try { return JSON.parse(localStorage.getItem(preferencesKey) ?? '{}') ?? {} } catch { return {} }
}

export function Editor() {
  const [image, setImage] = useState<ReturnType<typeof decodeImage>>()
  const [tool, setTool] = useState<EditorTool>('pen')
  const [preferences, setPreferences] = useState(loadPreferences)
  const [history, setHistory] = useState(emptyHistory)
  const [selectedId, setSelectedId] = useState<string>()
  const [draft, setDraft] = useState<Gesture>()
  const [available, setAvailable] = useState<ImageSize>({ width: 0, height: 0 })
  const [zoom, setZoom] = useState<number>()
  const [mime, setMime] = useState('image/png')
  const [output, setOutput] = useState<OutputPreferences>()
  const [projectTitle, setProjectTitle] = useState('')
  const [keeping, setKeeping] = useState(false)
  const [autoSaving, setAutoSaving] = useState(false)
  const [autoSaveFailed, setAutoSaveFailed] = useState(false)
  const [error, setError] = useState('')
  const [notice, setNotice] = useState('')
  const [exporting, setExporting] = useState<'copy' | 'save' | 'pin'>()
  const [presets, setPresets] = useState(loadPresets)
  const [presetName, setPresetName] = useState('')
  const opening = useRef<Promise<{ capture: ReturnType<typeof decodeImage>; restored: Document }> | null>(null)
  const editor = useRef<HTMLElement>(null)
  const workspace = useRef<HTMLDivElement>(null)
  const source = useRef<HTMLCanvasElement | null>(null)
  const preview = useRef<HTMLCanvasElement>(null)
  const gesture = useRef<Gesture | null>(null)
  const typing = useRef(false)
  const closeRequest = useRef<() => void>(() => undefined)
  const closeActive = useRef(false)

  const doc = history.current
  const selected = doc.marks.find((item) => item.id === selectedId)
  const activeMark = draft && (draft.action === 'move' || draft.action === 'resize') ? draft.mark : selected
  const propertyTool = selected?.tool ?? tool
  const standard = propertyTool === 'highlighter' ? { ...defaults, color: '#ffdc29', width: 22, opacity: .4 } : defaults
  const props = { ...standard, ...preferences[propertyTool] }
  const values = selected ?? props
  const fit = image ? Math.min(1, available.width / image.width, available.height / image.height) : 0
  const scale = zoom ?? fit

  useEffect(() => {
    let active = true
    opening.current ??= Promise.all([
      invoke<ArrayBuffer>('load_edit_image').then(decodeImage),
      invoke<unknown>('load_edit_document'),
    ]).then(([capture, data]) => ({ capture, restored: restoreDocument(data, capture) }))
    void opening.current.then(({ capture, restored }) => {
      if (!active) return
      setHistory({ ...emptyHistory, current: restored })
      setImage(capture)
    }).catch((problem) => {
      if (active) setError(String(problem))
    })
    void invoke<OutputPreferences>('output_preferences').then((prefs) => {
      if (active) setOutput(prefs)
    }).catch((problem) => { if (active) setError(String(problem)) })
    return () => { active = false }
  }, [])

  useEffect(() => {
    const unlisten = getCurrentWindow().onCloseRequested((event) => {
      event.preventDefault()
      closeRequest.current()
    })
    void unlisten.catch((problem) => setError(String(problem)))
    return () => { void unlisten.then((dispose) => dispose(), () => undefined) }
  }, [])

  useEffect(() => {
    let pending = Promise.resolve()
    function focusChanged() {
      const target = document.activeElement
      const active = document.hasFocus() && target instanceof HTMLElement && target.matches('textarea, input:not([type]), input[type="number"]')
      pending = pending.then(() => invoke<void>('set_editor_text_input', { active })).catch((problem) => setError(String(problem)))
    }
    document.addEventListener('focusin', focusChanged)
    document.addEventListener('focusout', focusChanged)
    window.addEventListener('focus', focusChanged)
    window.addEventListener('blur', focusChanged)
    return () => {
      document.removeEventListener('focusin', focusChanged)
      document.removeEventListener('focusout', focusChanged)
      window.removeEventListener('focus', focusChanged)
      window.removeEventListener('blur', focusChanged)
    }
  }, [])

  useEffect(() => {
    try { localStorage.setItem(preferencesKey, JSON.stringify(preferences)) } catch { /* Preferences remain available in this window. */ }
  }, [preferences])

  useEffect(() => {
    try { localStorage.setItem(presetsKey, JSON.stringify(presets)) } catch { /* Presets remain available in this window. */ }
  }, [presets])

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
    const canvas = document.createElement('canvas')
    canvas.width = image.width
    canvas.height = image.height
    const context = canvas.getContext('2d')
    if (!context) return
    context.putImageData(new ImageData(image.pixels, image.width, image.height), 0, 0)
    source.current = canvas
  }, [image])

  useLayoutEffect(() => {
    if (!image || !source.current || !preview.current) return
    const marks = draft?.action === 'draw' ? [...doc.marks, draft.mark] : doc.marks.map(
      (mark) => draft?.mark.id === mark.id ? draft.mark : mark,
    )
    renderDocument(preview.current, source.current, marks)
  }, [image, doc.marks, draft])

  function edit(update: (previous: Document) => Document) {
    setHistory((previous) => commit(previous, update(previous.current)))
  }

  function choose(next: EditorTool) {
    if (gesture.current) return
    if (next !== 'select') setSelectedId(undefined)
    setTool(next)
  }

  function pointFor(event: PointerEvent<HTMLCanvasElement>) {
    return imagePoint({ x: event.clientX, y: event.clientY }, event.currentTarget.getBoundingClientRect(), image!)
  }

  function start(event: PointerEvent<HTMLCanvasElement>) {
    if (!image || event.button !== 0 || gesture.current || exporting) return
    const point = pointFor(event)
    let action: Gesture['action'] = 'draw'
    let mark: Mark
    if (tool === 'select') {
      const bounds = selected && selectionBox(selected)
      const onHandle = bounds && Math.hypot(point.x - bounds.x - bounds.width, point.y - bounds.y - bounds.height) < 14 / scale
      const item = onHandle ? selected : [...doc.marks].reverse().find((entry) => inside(point, selectionBox(entry), 4 / scale))
      setSelectedId(item?.id)
      if (!item) return
      mark = item
      action = onHandle ? 'resize' : 'move'
    } else {
      action = tool === 'crop' ? 'crop' : 'draw'
      mark = { ...props, id: crypto.randomUUID(), tool: tool === 'crop' ? 'rectangle' : tool, points: [point] }
      if (tool === 'step') {
        const number = Math.max(0, ...doc.marks.filter((item) => item.tool === 'step').map((item) => +(item.text ?? 0))) + 1
        mark = { ...mark, width: Math.max(12, mark.width), text: String(number) }
      }
      if (tool === 'text') mark.text = ''
    }
    gesture.current = { pointer: event.pointerId, start: point, action, original: mark, mark }
    setDraft(gesture.current)
    event.currentTarget.focus()
    event.currentTarget.setPointerCapture(event.pointerId)
    setNotice('')
  }

  function extend(event: PointerEvent<HTMLCanvasElement>) {
    const active = gesture.current
    if (!active || active.pointer !== event.pointerId) return
    const point = pointFor(event)
    const { original, start, action } = active
    let { mark } = active
    if (action === 'move') mark = translate(original, point.x - start.x, point.y - start.y)
    else if (action === 'resize') {
      const padding = (selectionBox(original).width - boundsOf(original.points).width) / 2
      mark = resize(original, original.tool === 'step' ? point : { x: point.x - padding, y: point.y - padding })
    } else if (mark.tool === 'pen' || mark.tool === 'highlighter') {
      const last = mark.points.at(-1)!
      if (point.x !== last.x || point.y !== last.y) mark = { ...mark, points: [...mark.points, point] }
    } else if (mark.tool !== 'step') mark = { ...mark, points: [start, point] }
    if (mark === active.mark) return
    gesture.current = { ...active, mark }
    setDraft(gesture.current)
  }

  function cancel() {
    gesture.current = null
    setDraft(undefined)
  }

  function finish(event: PointerEvent<HTMLCanvasElement>) {
    if (gesture.current?.pointer !== event.pointerId) return
    const wasClick = Math.hypot(pointFor(event).x - gesture.current.start.x, pointFor(event).y - gesture.current.start.y) < 2 / scale
    extend(event)
    const { action, mark } = gesture.current
    cancel()
    if (action === 'crop') {
      const { x, y, width, height } = boundsOf(mark.points)
      const crop = { x: Math.floor(x), y: Math.floor(y), width: Math.ceil(x + width) - Math.floor(x), height: Math.ceil(y + height) - Math.floor(y) }
      if (!wasClick && crop.width > 1 && crop.height > 1) edit((previous) => ({ ...previous, crop }))
      return
    }
    if (action !== 'draw') {
      if (!wasClick) edit((previous) => ({ ...previous, marks: previous.marks.map((entry) => entry.id === mark.id ? mark : entry) }))
      return
    }
    if (mark.tool === 'text' && image) {
      const { x, y } = mark.points[0]
      if (wasClick || !visibleMark(mark)) {
        const left = Math.max(0, Math.min(x, image.width - 100))
        const top = Math.max(0, Math.min(y, image.height - 85))
        mark.points = [{ x: left, y: top }, { x: Math.min(left + 350, image.width), y: Math.min(top + 115, image.height) }]
      }
    }
    if (!visibleMark(mark)) return
    edit((previous) => ({ ...previous, marks: [...previous.marks, mark] }))
    if (mark.tool === 'text') {
      setTool('select')
      setSelectedId(mark.id)
      requestAnimationFrame(() => editor.current?.querySelector('textarea')?.focus())
    }
  }

  function updateProperties(change: Partial<Properties>) {
    setPreferences((previous) => ({ ...previous, [propertyTool]: { ...props, ...change } }))
    if (selected) edit((previous) => ({
      ...previous, marks: previous.marks.map((mark) => mark.id === selected.id ? { ...mark, ...change } : mark),
    }))
  }

  function updateText(text: string) {
    if (!selected) return
    const coalesce = typing.current
    typing.current = true
    setHistory((previous) => {
      const current = { ...previous.current, marks: previous.current.marks.map((mark) => mark.id === selected.id ? { ...mark, text } : mark) }
      return coalesce ? { ...previous, current } : commit(previous, current)
    })
  }

  function objectAction(action: 'delete' | 'duplicate' | 'front' | 'back') {
    if (!selected || draft) return
    const remaining = doc.marks.filter((mark) => mark.id !== selected.id)
    if (action === 'duplicate') {
      const clone = { ...translate(selected, 12, 12), id: crypto.randomUUID() }
      edit(() => ({ ...doc, marks: [...doc.marks, clone] }))
      setSelectedId(clone.id)
      return
    }
    const marks = action === 'delete' ? remaining : action === 'front' ? [...remaining, selected] : [selected, ...remaining]
    edit(() => ({ ...doc, marks }))
    if (action === 'delete') setSelectedId(undefined)
  }

  async function exportImage(action: 'copy' | 'save' | 'pin', chooseLocation = false) {
    if (!source.current || exporting || gesture.current) return
    setExporting(action)
    setError('')
    setNotice('')
    try {
      const composed = compose(source.current, doc)
      if (action === 'save') {
        const blob = await encode(composed, mime)
        const bytes = new Uint8Array(await blob.arrayBuffer())
        if (output?.directory && !chooseLocation) {
          await invoke('save_to_export_folder', bytes)
          setNotice('Image saved to the export folder')
        } else {
          const saved = await invoke<boolean>('save_edited_image', bytes)
          setNotice(saved ? 'Image saved' : 'Save cancelled')
        }
      } else if (action === 'pin') {
        await invoke('pin_image', imagePacket(composed))
        setNotice('Pinned above other windows')
      } else {
        if (doc.crop) {
          await invoke('copy_edited_image', imagePacket(composed), { headers: { 'X-Qapture-Cropped': '1' } })
        } else await invoke('copy_edited_image', new Uint8Array(imagePixels(composed).buffer))
        setNotice('Copied to clipboard')
      }
    } catch (problem) {
      setError(String(problem))
    } finally {
      setExporting(undefined)
    }
  }

  async function keepProject() {
    if (keeping || !image) return
    setKeeping(true)
    setError('')
    setNotice('')
    try {
      await invoke('keep_project', { title: projectTitle.trim() || 'Capture ' + new Date().toLocaleString(), document: doc })
      setNotice('Editable project saved to History')
    } catch (problem) {
      setError(String(problem))
    } finally {
      setKeeping(false)
    }
  }

  async function close(skipAutoSave = false) {
    if (closeActive.current || exporting || keeping) return
    closeActive.current = true
    try {
      if (!skipAutoSave && output?.autoSave && source.current) {
        setAutoSaving(true)
        setAutoSaveFailed(false)
        const blob = await encode(compose(source.current, doc), mime)
        await invoke('auto_save_image', new Uint8Array(await blob.arrayBuffer()))
      }
      await getCurrentWindow().destroy()
    } catch (problem) {
      setError(String(problem))
      setAutoSaving(false)
      setAutoSaveFailed(!skipAutoSave && !!output?.autoSave)
    } finally {
      closeActive.current = false
    }
  }

  useLayoutEffect(() => { closeRequest.current = () => { void close() } })

  function keys(event: KeyboardEvent<HTMLElement>) {
    if (event.nativeEvent.isComposing || event.key === 'Process' || exporting || autoSaving) return
    const control = (event.target as HTMLElement).closest('input, textarea, select, [contenteditable]')
    if (control) {
      if (event.key === 'Escape') (control as HTMLElement).blur()
      return
    }
    if (event.key === 'Escape') {
      event.preventDefault()
      if (gesture.current) cancel()
      else if (selected) setSelectedId(undefined)
      else void close()
      return
    }
    if (event.repeat || draft || event.altKey || event.metaKey) return
    if (event.ctrlKey) {
      const actions: Record<string, () => void> = {
        KeyC: () => { void exportImage('copy') },
        KeyS: () => { void exportImage('save', event.shiftKey) },
        KeyZ: () => setHistory(event.shiftKey ? redo : undo),
        KeyY: () => setHistory(redo),
        KeyD: () => objectAction('duplicate'),
      }
      const action = actions[event.code]
      if (action) { event.preventDefault(); action() }
    } else if (event.key === 'Delete') {
      event.preventDefault()
      objectAction('delete')
    } else {
      const match = tools.find((item) => 'key' in item && 'Key' + item.key === event.code)
      if (match) { event.preventDefault(); choose(match.id) }
    }
  }

  const shownCrop = draft?.action === 'crop' ? boundsOf(draft.mark.points) : doc.crop
  const dimensions = doc.crop ?? image
  const boxStyle = (box: { x: number; y: number; width: number; height: number }) => ({
    left: box.x * scale, top: box.y * scale, width: box.width * scale, height: box.height * scale,
  })

  return (
    <main className="studio" ref={editor} tabIndex={-1} onKeyDown={keys}>
      <header className="studio-header">
        <img src={icon} alt="" width="30" height="30" />
        <div className="studio-heading">
          <h1>Quick editor</h1>
          {dimensions && <p>{dimensions.width} × {dimensions.height} px</p>}
        </div>
        <button type="button" className="studio-copy" title="Copy final image (Ctrl+C)" onClick={() => void exportImage('copy')} disabled={!image || !!exporting || !!draft}>
          <Clipboard size={17} aria-hidden="true" /> {exporting === 'copy' ? 'Copying…' : 'Copy image'}
        </button>
        <div className="studio-save">
          <button type="button" className="studio-tool" title="Save image (Ctrl+S)" onClick={() => void exportImage('save')} disabled={!image || !!exporting || !!draft}>
            <ArrowDownToLine size={17} aria-hidden="true" /> {exporting === 'save' ? 'Saving…' : 'Save'}
          </button>
          <select aria-label="Export format" value={mime} onChange={(event) => setMime(event.target.value)} disabled={!!exporting}>
            <option value="image/png">PNG</option><option value="image/jpeg">JPG</option><option value="image/webp">WebP</option>
          </select>
        </div>
        {output?.directory && <button type="button" className="studio-tool" title="Choose where to save (Ctrl+Shift+S)"
          onClick={() => void exportImage('save', true)} disabled={!image || !!exporting || !!draft}>Save as…</button>}
        <button type="button" className="studio-icon" aria-label="Pin image" title="Pin image above other windows" onClick={() => void exportImage('pin')} disabled={!image || !!exporting || !!draft}>
          <Pin size={17} aria-hidden="true" />
        </button>
        <button type="button" className="studio-icon" aria-label="Close editor" title="Close editor (Esc)" onClick={() => void close()} disabled={!!exporting || autoSaving || keeping}>
          <X size={20} aria-hidden="true" />
        </button>
      </header>
      <div className="studio-controls">
        <div className="studio-tools" aria-label="Annotation tools" role="group">
          {tools.map((item) => (
            <button key={item.id} type="button" className="studio-tool" aria-label={item.label} aria-pressed={tool === item.id}
              onClick={() => choose(item.id)} title={'key' in item ? item.label + ' (' + item.key + ')' : item.label} disabled={!image || !!draft}>
              <item.icon size={17} aria-hidden="true" />
            </button>
          ))}
          <div className="studio-history">
            <button type="button" className="studio-icon" aria-label="Undo" title="Undo (Ctrl+Z)" onClick={() => setHistory(undo)} disabled={!history.past.length || !!draft}>
              <Undo2 size={18} aria-hidden="true" />
            </button>
            <button type="button" className="studio-icon" aria-label="Redo" title="Redo (Ctrl+Shift+Z)" onClick={() => setHistory(redo)} disabled={!history.future.length || !!draft}>
              <Redo2 size={18} aria-hidden="true" />
            </button>
          </div>
        </div>
        <div className="studio-properties" onBlur={(event) => { if (event.target instanceof HTMLTextAreaElement) typing.current = false }}>
          {image && (selected || !['select', 'crop'].includes(tool)) && <EditorProperties selected={selected} values={values} onChange={updateProperties} onText={updateText} />}
          {image && (selected || !['select', 'crop'].includes(tool)) && <details className="studio-presets">
            <summary>Presets</summary>
            {presets.length > 0 && <label>Apply <select value="" onChange={(event) => {
              const preset = presets.find((entry) => entry.name === event.target.value)
              if (preset) updateProperties(preset.values)
            }}><option value="" disabled>Choose preset</option>{presets.map(({ name }) => <option key={name}>{name}</option>)}</select></label>}
            <label>Preset name <input value={presetName} maxLength={40} onChange={(event) => setPresetName(event.target.value)} /></label>
            <button disabled={!presetName.trim()} onClick={() => {
              const name = presetName.trim()
              const { color, width, opacity, fontSize, family, bold, italic, align, background } = values
              setPresets((previous) => [...previous.filter((item) => item.name !== name), { name, values: { color, width, opacity, fontSize, family, bold, italic, align, background } }])
              setPresetName('')
            }}>Save preset</button>
          </details>}
          {selected && <div className="studio-object-actions">
            <button onClick={() => objectAction('duplicate')} title="Duplicate (Ctrl+D)">Duplicate</button>
            <button onClick={() => objectAction('back')}>Send to back</button>
            <button onClick={() => objectAction('front')}>Bring to front</button>
            <button onClick={() => objectAction('delete')} title="Delete selected object (Del)">Delete</button>
          </div>}
          {doc.crop && <button onClick={() => edit(({ marks }) => ({ marks }))}>Reset crop</button>}
          {output?.retainProjects && <div className="studio-project">
            <input aria-label="Project name" placeholder="Project name" value={projectTitle} maxLength={100} onChange={(event) => setProjectTitle(event.target.value)} />
            <button onClick={() => void keepProject()} disabled={!image || keeping || !!draft}>Keep editable project</button>
          </div>}
        </div>
      </div>

      <div className="studio-viewport" ref={workspace}>
        <div className="studio-workspace">
          {image ? (
            <div className="studio-stage" style={{ width: image.width * scale, height: image.height * scale }}>
              <canvas ref={preview} className="studio-markup" data-tool={tool} width={image.width} height={image.height} tabIndex={0} aria-label="Annotate the capture"
                onPointerDown={start} onPointerMove={extend} onPointerUp={finish}
                onPointerCancel={cancel} onLostPointerCapture={cancel} />
              {shownCrop && <div className="studio-crop" style={boxStyle(shownCrop)} aria-hidden="true" />}
              {activeMark && <div className="studio-selection" style={boxStyle(selectionBox(activeMark))} aria-hidden="true" />}
            </div>
          ) : !error && <p role="status">Opening capture…</p>}
        </div>
      </div>
      <footer className="studio-footer">
        <p role="status" className={error ? 'error' : undefined}>{autoSaving ? 'Saving image…' : error || notice || (tool === 'redact' ? 'Solid redaction covers pixels in the exported image.' : tool === 'crop' ? 'Drag to set the export crop.' : '')}</p>
        {autoSaveFailed && <button className="studio-tool" disabled={autoSaving} onClick={() => void close(true)}>Close without saving</button>}
        <div className="studio-zoom">
          <button aria-label="Zoom out" title="Zoom out" onClick={() => setZoom(Math.max(.1, scale / 1.25))} disabled={!image || scale <= .1}><Minus size={15} /></button>
          <button onClick={() => setZoom(undefined)} disabled={!image} title="Fit to screen">Fit</button>
          <span>{Math.round(scale * 100)}%</span>
          <button aria-label="Zoom in" title="Zoom in" onClick={() => setZoom(Math.min(4, scale * 1.25))} disabled={!image || scale >= 4}><Plus size={15} /></button>
        </div>
      </footer>
    </main>
  )
}
