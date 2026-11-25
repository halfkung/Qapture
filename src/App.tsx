import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ArrowRight, Check, Keyboard, Monitor, Pencil, Scan } from 'lucide-react'
import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent } from 'react'
import icon from '../src-tauri/icons/icon.ico?url'
import { bindingFor, captureModes, keycaps, type CaptureMode, type ShortcutStatus } from './shortcuts'

type CaptureReport = {
  outcome: 'copied' | 'cancelled' | 'error'
  overlayMs: number | null
  clipboardMs: number | null
  message: string
}

type DashboardInfo = {
  shortcuts: ShortcutStatus
  phase: 'idle' | 'capturing' | 'selecting' | 'exporting'
  last: CaptureReport | null
}

const actions = {
  region: { title: 'Capture a region', detail: 'Select an area of your screen', setting: 'Region capture', icon: Scan },
  full: { title: 'Copy full desktop', detail: 'Capture all connected displays', setting: 'Full desktop', icon: Monitor },
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
  const [capturePending, setCapturePending] = useState(false)
  const editButtons = useRef<Partial<Record<CaptureMode, HTMLButtonElement | null>>>({})
  const shortcutInput = useRef<HTMLInputElement>(null)
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
    const subscription = listen('qapture-status', refresh)
    void subscription.catch(() => {
      if (active) setError('Could not subscribe to Qapture’s capture status.')
    })
    void refresh()
    void invoke('dashboard_ready').catch(() => undefined)
    return () => {
      active = false
      void subscription.then((unlisten) => unlisten(), () => undefined)
    }
  }, [])

  useLayoutEffect(() => {
    if (pendingShortcut !== undefined) return
    if (recording !== undefined) {
      shortcutInput.current?.focus()
      shortcutEditor.current?.scrollIntoView({ block: 'nearest' })
    } else if (returnFocus.current !== undefined) {
      editButtons.current[returnFocus.current]?.focus()
      returnFocus.current = undefined
    }
  }, [recording, pendingShortcut])

  async function capture(mode: CaptureMode) {
    setError('')
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

  function record(event: KeyboardEvent<HTMLInputElement>) {
    if (!(event.ctrlKey && !event.altKey && !event.shiftKey && /^Key[ACVXYZ]$/.test(event.code))) {
      const binding = bindingFor(event)
      if (binding) {
        event.preventDefault()
        setDraft(binding)
        setShortcutError('')
      }
    }
  }

  async function saveShortcut(mode: CaptureMode) {
    setPendingShortcut(mode)
    setShortcutError('')
    try {
      const shortcuts = await invoke<ShortcutStatus>('set_shortcut', { mode, binding: draft.trim() })
      setInfo((current) => current && { ...current, shortcuts })
      closeEditor(mode)
    } catch (problem) {
      setShortcutError(String(problem))
    } finally {
      setPendingShortcut(undefined)
    }
  }

  const shortcuts = info?.shortcuts
  const busy = Boolean(info && info.phase !== 'idle')
  const last = info?.last
  const status = error || (busy ? 'Capture in progress…' : last?.message)

  return (
    <>
      <header className="app-header">
        <div className="brand content-width">
          <img src={icon} alt="" width="28" height="28" />
          <span>Qapture</span>
        </div>
      </header>

      <main className="dashboard content-width">
        <section aria-labelledby="capture-heading">
          <h1 id="capture-heading">New capture</h1>
          <p className="intro">Captures go straight to your clipboard.</p>
          <div className="capture-actions">
            {captureModes.map((mode) => {
              const action = actions[mode]
              const ActionIcon = action.icon
              const active = mode === 'region' ? shortcuts?.regionActive : shortcuts?.fullActive
              return (
                <button key={mode} className={`capture-card ${mode}`} disabled={!info || busy || capturePending}
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
                            setDraft(shortcuts?.settings[mode] ?? '')
                            setRecording(mode)
                          }
                        }}>
                        <Pencil size={14} aria-hidden="true" /> Edit
                      </button>
                    </div>
                  </div>
                  {problem && <p className="shortcut-problem error">{problem}</p>}
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
                    <label>
                      <span>Press or type a shortcut</span>
                      <input ref={shortcutInput} value={draft} maxLength={64} required onKeyDown={record}
                        onChange={(event) => {
                          setDraft(event.target.value)
                          setShortcutError('')
                        }}
                        aria-invalid={!!shortcutError}
                        aria-describedby={shortcutError ? errorId : undefined}
                        disabled={pendingShortcut !== undefined} />
                    </label>
                    <div className="edit-actions">
                      <button className="text-button" type="button" disabled={pendingShortcut !== undefined}
                        onClick={() => closeEditor(mode)}>Cancel</button>
                      <button className="save-button" type="submit" disabled={pendingShortcut !== undefined}>
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
        </section>

        <div className="capture-status">
          <p className="status-message" role="status">
            {status && <>
              {last?.outcome === 'copied' && !error && !busy && <Check size={16} aria-hidden="true" />}
              <span className={error || (!busy && last?.outcome === 'error') ? 'error' : ''}>{status}</span>
            </>}
          </p>
          {busy && <button className="text-button" onClick={() => void invoke('cancel_capture').catch((problem) => setError(String(problem)))}>Cancel</button>}
        </div>
      </main>
    </>
  )
}
