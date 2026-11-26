import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState, type ChangeEvent } from 'react'
import './history.css'
import { imagePacket } from './editor-document'

export type OutputPreferences = {
  retainProjects: boolean
  autoSave: boolean
  directory: string
  prefix: string
}
type Project = { id: string; title: string }

export function History() {
  const [items, setItems] = useState<Project[]>([])
  const [search, setSearch] = useState('')
  const [preferences, setPreferences] = useState<OutputPreferences>()
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState('')
  const [clear, setClear] = useState(false)
  const [deleting, setDeleting] = useState<string>()
  const [startWithWindows, setStartWithWindows] = useState(false)

  useEffect(() => {
    let mounted = true
    Promise.all([
      invoke<Project[]>('list_projects'),
      invoke<OutputPreferences>('output_preferences'),
      invoke<boolean>('startup_enabled'),
    ]).then(([projects, settings, startup]) => {
      if (!mounted) return
      setItems(projects)
      setPreferences(settings)
      setStartWithWindows(startup)
    }).catch((problem) => { if (mounted) setMessage(String(problem)) })
    return () => { mounted = false }
  }, [])

  async function perform(action: () => Promise<void>) {
    if (busy) return
    setBusy(true)
    setMessage('')
    try { await action() } catch (problem) { setMessage(String(problem)) } finally { setBusy(false) }
  }

  function loadFile(event: ChangeEvent<HTMLInputElement>) {
    const file = event.currentTarget.files?.[0]
    event.currentTarget.value = ''
    if (!file) return
    void perform(async () => {
      if (!['image/png', 'image/jpeg', 'image/webp', 'image/bmp'].includes(file.type)) {
        throw Error('Choose a PNG, JPG, WebP or BMP image.')
      }
      const bitmap = await createImageBitmap(file)
      try {
        if (bitmap.width * bitmap.height > 24 * 1024 * 1024) throw Error('Choose an image smaller than 24 megapixels.')
        const canvas = document.createElement('canvas')
        canvas.width = bitmap.width
        canvas.height = bitmap.height
        const ctx = canvas.getContext('2d', { willReadFrequently: true })
        if (!ctx) throw Error('Could not read the selected image.')
        ctx.drawImage(bitmap, 0, 0)
        await invoke('import_image', imagePacket(canvas))
      } finally {
        bitmap.close()
      }
    })
  }

  return <section className="history-page" aria-labelledby="history-title">
    <h1 id="history-title">Capture history</h1>
    <div className="history-actions">
      <label className="history-search">Search <input type="search" value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Project name" /></label>
      <label className="shortcut-button import-button">
        Import image <input type="file" accept="image/png,image/jpeg,image/webp,image/bmp" onChange={loadFile} disabled={busy} />
      </label>
      <button className="shortcut-button" disabled={busy || !items.length} onClick={() => setClear(true)}>Clear history</button>
    </div>
    {clear && <div className="history-confirm" role="alert">
      <p>Delete all editable project files on this device?</p>
      <button className="shortcut-button" onClick={() => setClear(false)}>Cancel</button>
      <button className="shortcut-button" disabled={busy} onClick={() => void perform(async () => {
        await invoke('clear_projects')
        setItems([])
        setClear(false)
      })}>Delete all</button>
    </div>}
    <ul className="history-list">
      {items.filter((item) => item.title.toLowerCase().includes(search.toLowerCase())).map((item) => <li key={item.id}>
        <span>{item.title || 'Capture'}</span>
        <div className="history-actions">
          <button className="shortcut-button" disabled={busy} onClick={() => void perform(async () => { await invoke('reopen_project', { id: item.id }) })}>Open</button>
          {deleting === item.id && <button className="shortcut-button" onClick={() => setDeleting(undefined)}>Cancel</button>}
          <button className="shortcut-button" disabled={busy} onClick={() => {
            if (deleting !== item.id) { setDeleting(item.id); return }
            void perform(async () => {
              await invoke('delete_project', { id: item.id })
              setItems((current) => current.filter((entry) => entry.id !== item.id))
              setDeleting(undefined)
            })
          }}>{deleting === item.id ? 'Confirm delete' : 'Delete'}</button>
        </div>
      </li>)}
    </ul>
    {!items.length && <p className="history-muted">No editable projects saved yet.</p>}
    {preferences && <form className="history-settings" onSubmit={(event) => {
      event.preventDefault()
      void perform(async () => {
        setPreferences(await invoke<OutputPreferences>('set_output_preferences', { preferences }))
        setMessage('Settings saved')
      })
    }}>
      <h2>Storage settings</h2>
      <label><input type="checkbox" checked={preferences.retainProjects} onChange={(event) => setPreferences({ ...preferences, retainProjects: event.target.checked })} /> Allow saving editable projects</label>
      <p className="history-muted">Editable projects include the original pixels. Share a flattened export to share only the visible result.</p>
      <label><input type="checkbox" checked={preferences.autoSave} onChange={(event) => setPreferences({ ...preferences, autoSave: event.target.checked })} /> Auto Save when closing the editor</label>
      <label>Export folder <input value={preferences.directory} onChange={(event) => setPreferences({ ...preferences, directory: event.target.value })} placeholder="Full path to an existing folder" /></label>
      <label>File name prefix <input value={preferences.prefix} onChange={(event) => setPreferences({ ...preferences, prefix: event.target.value })} maxLength={48} required /></label>
      <div className="history-actions">
        <button className="shortcut-button" disabled={busy}>Save storage settings</button>
        <label><input type="checkbox" checked={startWithWindows} disabled={busy} onChange={(event) => {
          const enabled = event.target.checked
          void perform(async () => {
            await invoke('set_startup', { enabled })
            setStartWithWindows(enabled)
          })
        }} /> Start with Windows</label>
      </div>
    </form>}
    <p role="status">{message}</p>
  </section>
}
