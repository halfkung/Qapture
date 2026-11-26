import type { Mark } from './markup'

export type Properties = Pick<Mark, 'color' | 'width' | 'opacity' | 'fontSize' | 'family' | 'bold' | 'italic' | 'align' | 'background'>
export const defaults: Properties = { color: '#f04462', width: 5, opacity: 1, fontSize: 24, family: 'Segoe UI', align: 'left' }

export function EditorProperties({ values, selected, onChange, onText }: {
  values: Properties
  selected?: Mark
  onChange: (change: Partial<Properties>) => void
  onText: (value: string) => void
}) {
  const effect = selected?.tool === 'blur' || selected?.tool === 'mosaic'
  const solid = selected?.tool === 'redact'
  return <>
    {!effect && <label>Color <input type="color" value={values.color} onChange={(e) => onChange({ color: e.target.value })} /></label>}
    {selected?.tool !== 'text' && !solid && <label>{effect ? 'Strength' : 'Stroke'} <input type="number" min="1" max="128" value={Math.round(values.width)}
      onChange={(e) => { if (e.target.value) onChange({ width: Math.max(1, Math.min(128, +e.target.value)) }) }} /></label>}
    {!solid && <label>Opacity <input type="range" min="0.05" max="1" step="0.05" value={values.opacity} onChange={(e) => onChange({ opacity: +e.target.value })} /></label>}
    {selected?.tool === 'text' && <>
      <label>Font <select value={values.family} onChange={(e) => onChange({ family: e.target.value })}>
        {['Segoe UI', 'Leelawadee UI', 'Tahoma', 'Arial', 'Georgia'].map((font) => <option key={font}>{font}</option>)}
      </select></label>
      <label>Size <input type="number" min="8" max="240" value={values.fontSize}
        onChange={(e) => { if (e.target.value) onChange({ fontSize: Math.max(8, Math.min(240, +e.target.value)) }) }} /></label>
      <label><input type="checkbox" checked={!!values.bold} onChange={(e) => onChange({ bold: e.target.checked })} /> Bold</label>
      <label><input type="checkbox" checked={!!values.italic} onChange={(e) => onChange({ italic: e.target.checked })} /> Italic</label>
      <label>Align <select value={values.align} onChange={(e) => onChange({ align: e.target.value as CanvasTextAlign })}>
        <option value="left">Left</option><option value="center">Center</option><option value="right">Right</option>
      </select></label>
      <label><input type="checkbox" checked={!!values.background}
        onChange={(e) => onChange({ background: e.target.checked ? '#ffffff' : undefined })} /> Background</label>
      {values.background && <label>Background color <input type="color" value={values.background} onChange={(e) => onChange({ background: e.target.value })} /></label>}
      <textarea key={selected.id} aria-label="Annotation text" rows={3} value={selected.text} placeholder="Enter annotation text"
        onChange={(e) => onText(e.target.value)} />
    </>}
  </>
}
