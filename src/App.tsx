import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ArrowRight, Check, Keyboard, Monitor, Pencil, Scan, Settings2 } from 'lucide-react'
import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent } from 'react'
import icon from '../src-tauri/icons/icon.ico?url'
import { bindingFor, captureModes, keycaps, type CaptureMode, type ShortcutStatus } from './shortcuts'

type CaptureReport = {
  outcome: 'copied' | 'cancelled' | 'editing' | 'error'
  overlayMs: number | null
  clipboardMs: number | null
  message: string
}

type DashboardInfo = {
  shortcuts: ShortcutStatus
  phase: 'idle' | 'capturing' | 'selecting' | 'editing' | 'exporting'
  last: CaptureReport | null
}

const actions = {
  region: { title: 'Capture a region', detail: 'Select an area, then edit', setting: 'Region capture', icon: Scan },
  full: { title: 'Copy full desktop', detail: 'Copy all connected displays', setting: 'Full desktop', icon: Monitor },
}

function Keys({ binding }: { binding: string }) {
  return <span className="keys">{keycaps(binding).map((key, index) => <kbd key={index}>{key}</kbd>)}</span>
}

export function App() {
  const [info, setInfo] = useState<DashboardInfo>()
  const [recording, setRecording] = useState<CaptureMode>()
  const [draft, setDraft] = useState('')
  const [pendingShortcut, setPendingShortcut] = useState<CaptureMode>()
  const [error, setError] = useState('')
  const [shortcutError, setShortcutError] = useState('')
  const [shortcutNotice, setShortcutNotice] = useState('')
  const [capturePending, setCapturePending] = useState(false)
  const editButtons = useRef<Partial<Record<CaptureMode, HTMLButtonElement | null>>>({})
  const shortcutRecorder = useRef<HTMLButtonElement>(null)
  const shortcutEditor = useRef<HTMLFormElement>(null)
  const returnFocus = useRef<CaptureMode | undefined>(undefined)

  useEffect(() => {
    if (!isTauri()) return
    let active = true
    let requestId = 0
    const refresh = () => {
      const id = ++requestId
      return invoke<DashboardInfo>('dashboard_info').then((result) => {
        if (active && requestId === id) setInfo(result)
      }).catch(() => {
        if (active) setError('Could not load Qapture’s local settings.')
      })
    }
    const subscriptions = [
      listen('qapture-status', refresh),
      listen<string>('qapture-shortcut-recorded', ({ payload }) => {
        if (active) {
          setDraft(payload)
          setShortcutError('')
        }
      }),
    ]
    for (const subscription of subscriptions) {
      void subscription.catch(() => {
        if (active) setError('Could not subscribe to Qapture’s updates.')
      })
    }
    void refresh()
    void invoke('dashboard_ready').catch(() => undefined)
    return () => {
      active = false
      for (const subscription of subscriptions) {
        void subscription.then((unlisten) => unlisten(), () => undefined)
      }
    }
  }, [])

  useEffect(() => {
    if (!recording || pendingShortcut !== undefined) return
    void invoke('set_shortcut_recording', { recording: true }).catch(() => undefined)
    return () => { void invoke('set_shortcut_recording', { recording: false }).catch(() => undefined) }
  }, [recording, pendingShortcut])

  useLayoutEffect(() => {
    if (pendingShortcut !== undefined) return
    if (recording !== undefined) {
      shortcutRecorder.current?.focus()
      shortcutEditor.current?.scrollIntoView({ block: 'nearest' })
    } else if (returnFocus.current !== undefined) {
      editButtons.current[returnFocus.current]?.focus()
      returnFocus.current = undefined
    }
  }, [recording, pendingShortcut])

  async function capture(mode: CaptureMode) {
    setError('')
    setShortcutNotice('')
    setCapturePending(true)
    try {
      await invoke('start_capture', { mode })
    } catch (problem) {
      setError(String(problem))
    } finally {
      setCapturePending(false)
    }
  }

  function closeEditor(mode: CaptureMode) {
    returnFocus.current = mode
    setRecording(undefined)
    setShortcutError('')
  }

  function record(event: KeyboardEvent<HTMLButtonElement>) {
    if (event.key === 'Escape' || event.key === 'Tab') return
    event.preventDefault()
    if (event.repeat || ['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) return
    const binding = bindingFor(event)
    if (binding) {
      setDraft(binding)
      setShortcutError('')
    } else {
      setShortcutError('Hold Ctrl or Alt with a letter, number, or F1–F12.')
    }
  }

  async function saveShortcut(mode: CaptureMode) {
    if (!draft || pendingShortcut !== undefined) return
    setPendingShortcut(mode)
    setShortcutError('')
    try {
      const shortcuts = await invoke<ShortcutStatus>('set_shortcut', { mode, binding: draft })
      setInfo((current) => current && { ...current, shortcuts })
      closeEditor(mode)
    } catch (problem) {
      setShortcutError(String(problem))
    } finally {
      setPendingShortcut(undefined)
    }
  }

  async function recoverShortcut(mode: CaptureMode) {
    if (pendingShortcut !== undefined) return
    setPendingShortcut(mode)
    setError('')
    setShortcutNotice('')
    try {
      const shortcuts = await invoke<ShortcutStatus>('recover_shortcut', { mode })
      setInfo((current) => current && { ...current, shortcuts })
      setShortcutNotice(`${actions[mode].setting} shortcut ready: ${keycaps(shortcuts.settings[mode]).join(' + ')}.`)
      returnFocus.current = mode
    } catch (problem) {
      setError(String(problem))
    } finally {
      setPendingShortcut(undefined)
    }
  }

  const shortcuts = info?.shortcuts
  const busy = Boolean(info && info.phase !== 'idle')
  const last = info?.last
  const editingCapture = info?.phase === 'editing'
  const status = error || (editingCapture ? 'Capture open in Quick editor' : busy ? 'Capture in progress…' : last?.message)

  return (
    <div className="dashboard-shell">
      <header className="app-header">
        <div className="brand content-width">
          <img src={icon} alt="" width="28" height="28" />
          <span>Qapture</span>
          <a className="settings-link" href="#shortcuts-heading"><Settings2 size={16} aria-hidden="true" /> Shortcuts</a>
        </div>
      </header>

      <main className="dashboard">
        <section aria-labelledby="capture-heading">
          <div className="capture-hero">
            <div className="hero-copy">
              <h1 id="capture-heading">Capture your screen</h1>
              <p>Select a region to open the editor.</p>
            </div>
            <div className="hero-art" aria-hidden="true">
              <span className="art-window" />
              <span className="art-selection" />
            </div>
          </div>
          <div className="capture-actions">
            {captureModes.map((mode) => {
              const action = actions[mode]
              const ActionIcon = action.icon
              const active = mode === 'region' ? shortcuts?.regionActive : shortcuts?.fullActive
              return (
                <button key={mode} className={`capture-card ${mode}`} disabled={!info || busy || capturePending || recording !== undefined}
                  onClick={() => void capture(mode)}>
                  <span className="action-icon"><ActionIcon size={22} aria-hidden="true" /></span>
                  <span className="action-copy">
                    <span className="action-title">{action.title}</span>
                    <span className="action-detail">{action.detail}</span>
                  </span>
                  <span className="action-end">
                    {active && shortcuts && <Keys binding={shortcuts.settings[mode]} />}
                    <ArrowRight size={18} aria-hidden="true" />
                  </span>
                </button>
              )
            })}
          </div>
        </section>

        <section aria-labelledby="shortcuts-heading" className="settings">
          <div className="settings-heading">
            <Keyboard size={20} aria-hidden="true" />
            <div>
              <h2 id="shortcuts-heading">Keyboard shortcuts</h2>
              <p>Shortcuts work even when this window is closed.</p>
            </div>
          </div>
          <div className="shortcut-list">
            {captureModes.map((mode) => {
              const problem = mode === 'region' ? shortcuts?.regionError : shortcuts?.fullError
              const isEditing = recording === mode
              const editorId = `shortcut-editor-${mode}`
              const errorId = `${editorId}-error`
              return (
                <div className="shortcut-item" key={mode}>
                  <div className="shortcut-row">
                    <span className="shortcut-name">{actions[mode].setting}</span>
                    <div className="shortcut-controls">
                      {shortcuts && <Keys binding={shortcuts.settings[mode]} />}
                      <button className="shortcut-button"
                        ref={(button) => { editButtons.current[mode] = button }}
                        type="button" disabled={!shortcuts || pendingShortcut !== undefined}
                        aria-label={`Edit ${actions[mode].setting.toLowerCase()} shortcut`}
                        aria-expanded={isEditing} aria-controls={isEditing ? editorId : undefined}
                        onClick={() => {
                          if (isEditing) {
                            closeEditor(mode)
                          } else {
                            setShortcutError('')
                            setShortcutNotice('')
                            setError('')
                            setDraft('')
                            setRecording(mode)
                          }
                        }}>
                        <Pencil size={14} aria-hidden="true" /> Edit
                      </button>
                    </div>
                  </div>
                  {problem && <div className="shortcut-problem">
                    <span className="error">{problem}</span>
                    {!isEditing && <button type="button" className="recover-shortcut"
                      disabled={pendingShortcut !== undefined || recording !== undefined}
                      aria-label={`Use available shortcut for ${actions[mode].setting.toLowerCase()}`}
                      onClick={() => void recoverShortcut(mode)}>Use available shortcut</button>}
                  </div>}
                  {isEditing && <form id={editorId} ref={shortcutEditor} className="shortcut-form"
                    onKeyDown={(event) => {
                      if (event.key === 'Escape' && pendingShortcut === undefined) {
                        event.preventDefault()
                        closeEditor(mode)
                      }
                    }}
                    onSubmit={(event) => {
                      event.preventDefault()
                      void saveShortcut(mode)
                    }}>
                      <button ref={shortcutRecorder} type="button" className="shortcut-recorder" onKeyDown={record}
                        aria-label="Record keyboard shortcut" aria-describedby={shortcutError ? errorId : undefined}
                        disabled={pendingShortcut !== undefined}>
                        <Keyboard size={17} aria-hidden="true" />
                        <span aria-live="polite">{draft ? <Keys binding={draft} /> : 'Press a key combination'}</span>
                      </button>
                    <div className="edit-actions">
                      <button className="text-button" type="button" disabled={pendingShortcut !== undefined}
                        onClick={() => closeEditor(mode)}>Cancel</button>
                        <button className="save-button" type="submit" disabled={!draft || pendingShortcut !== undefined}>
                        {pendingShortcut === mode ? 'Saving…' : 'Save'}
                      </button>
                    </div>
                    {shortcutError && <p id={errorId} role="alert" className="edit-error error">{shortcutError}</p>}
                  </form>}
                </div>
              )
            })}
            {shortcuts?.warning && <p role="alert" className="notice error">{shortcuts.warning}</p>}
          </div>
          <p className="sr-only" role="status">{shortcutNotice}</p>
        </section>

        <div className="capture-status">
          <p className="status-message" role="status">
            {status && <>
              {last?.outcome === 'copied' && !error && !busy && <Check size={16} aria-hidden="true" />}
              <span className={error || (!busy && last?.outcome === 'error') ? 'error' : ''}>{status}</span>
            </>}
          </p>
          {busy && !editingCapture && <button className="text-button" onClick={() => void invoke('cancel_capture').catch((problem) => setError(String(problem)))}>Cancel</button>}
        </div>
      </main>
    </div>
  )
}
