import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ArrowUpRight, Check, Keyboard, Monitor, Scan, ShieldCheck } from 'lucide-react'
import { useEffect, useState, type KeyboardEvent } from 'react'
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
  region: { title: 'Capture region', detail: 'Select an area to copy', icon: Scan },
  full: { title: 'Copy full desktop', detail: 'Capture all connected displays', icon: Monitor },
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

  function record(event: KeyboardEvent<HTMLInputElement>) {
    if (event.code === 'Escape') {
      event.preventDefault()
      setRecording(undefined)
      setShortcutError('')
    } else if (!(event.ctrlKey && !event.altKey && !event.shiftKey && /^Key[ACVXYZ]$/.test(event.code))) {
      const binding = bindingFor(event)
      if (binding) {
        event.preventDefault()
        setDraft(binding)
      }
    }
  }

  async function saveShortcut(mode: CaptureMode) {
    setPendingShortcut(mode)
    setShortcutError('')
    try {
      const shortcuts = await invoke<ShortcutStatus>('set_shortcut', { mode, binding: draft.trim() })
      setInfo((current) => current && { ...current, shortcuts })
      setRecording(undefined)
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
    <main className="dashboard">
      <header className="brand">
        <img src={icon} alt="" width="36" height="36" />
        <span>Qapture</span>
      </header>

      <section aria-labelledby="capture-heading" className="capture">
        <h1 id="capture-heading">Capture your screen</h1>
        <p className="intro">Use a shortcut from any app or choose what to capture.</p>
        <div className="capture-actions">
          {captureModes.map((mode) => {
            const action = actions[mode]
            const ActionIcon = action.icon
            const active = mode === 'region' ? shortcuts?.regionActive : shortcuts?.fullActive
            return (
              <button key={mode} className={`capture-card ${mode}`} disabled={!info || busy || capturePending}
                onClick={() => void capture(mode)}>
                <span className="action-top">
                  <span className="action-icon"><ActionIcon size={21} aria-hidden="true" /></span>
                  <ArrowUpRight size={17} aria-hidden="true" />
                </span>
                <span className="action-title">{action.title}</span>
                <span className="action-detail">{action.detail}</span>
                {active && shortcuts && <Keys binding={shortcuts.settings[mode]} />}
              </button>
            )
          })}
        </div>
      </section>

      <section aria-labelledby="shortcuts-heading" className="settings">
        <div className="settings-heading">
          <span className="keyboard-icon"><Keyboard size={19} aria-hidden="true" /></span>
          <div>
            <h2 id="shortcuts-heading">Keyboard shortcuts</h2>
            <p>Global shortcuts stay active when the window is closed.</p>
          </div>
        </div>
        {captureModes.map((mode) => {
          const problem = mode === 'region' ? shortcuts?.regionError : shortcuts?.fullError
          return (
            <div key={mode}>
              <div className="shortcut-row">
                <div><span>{actions[mode].title}</span>{problem && <small className="error">{problem}</small>}</div>
                <button className={`shortcut-button ${recording === mode ? 'recording' : ''}`}
                  type="button" disabled={!shortcuts || pendingShortcut !== undefined}
                  aria-label={`Change shortcut for ${actions[mode].title.toLowerCase()}`}
                  aria-expanded={recording === mode}
                  onClick={() => {
                    setShortcutError('')
                    setDraft(shortcuts?.settings[mode] ?? '')
                    setRecording(recording === mode ? undefined : mode)
                  }}>
                  {shortcuts ? <Keys binding={shortcuts.settings[mode]} /> : '—'}
                </button>
              </div>
              {recording === mode && <form className="shortcut-form" onSubmit={(event) => {
                event.preventDefault()
                void saveShortcut(mode)
              }}>
                <label>
                  <span>Press or type a shortcut</span>
                  <input value={draft} maxLength={64} required autoFocus onKeyDown={record}
                    onChange={(event) => setDraft(event.target.value)} placeholder="Ctrl+Shift+F9"
                    disabled={pendingShortcut !== undefined} />
                </label>
                <button className="text-button" type="button" disabled={pendingShortcut !== undefined}
                  onClick={() => { setRecording(undefined); setShortcutError('') }}>Cancel</button>
                <button className="save-button" type="submit" disabled={pendingShortcut !== undefined}>
                  {pendingShortcut === mode ? 'Saving…' : 'Save'}
                </button>
              </form>}
            </div>
          )
        })}
        {(shortcutError || shortcuts?.warning) && <p role="alert" className="notice error">{shortcutError || shortcuts?.warning}</p>}
      </section>

      <footer>
        <p><ShieldCheck size={16} aria-hidden="true" /> Captures stay on this device.</p>
        <div className="status" role="status" aria-live="polite">
          {status && <>
            {last?.outcome === 'copied' && !error && !busy && <Check size={16} aria-hidden="true" />}
            <span className={error || (!busy && last?.outcome === 'error') ? 'error' : ''}>{status}</span>
            {busy && <button className="text-button" onClick={() => void invoke('cancel_capture').catch((problem) => setError(String(problem)))}>Cancel</button>}
          </>}
        </div>
      </footer>
    </main>
  )
}
