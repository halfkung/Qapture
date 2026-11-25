import { describe, expect, it } from 'vitest'
import { bindingFor, keycaps } from './shortcuts'

const keys = { ctrlKey: true, altKey: false, shiftKey: true, metaKey: false }

describe('shortcut recorder', () => {
  it('uses the physical keyboard code and a consistent modifier order', () => {
    expect(bindingFor({ ...keys, code: 'KeyQ' })).toBe('Ctrl+Shift+KeyQ')
    expect(bindingFor({ ...keys, altKey: true, code: 'Digit2' })).toBe('Ctrl+Alt+Shift+Digit2')
  })

  it('rejects bare keys, modifier keys, and Windows-key chords', () => {
    expect(bindingFor({ ...keys, ctrlKey: false, code: 'F9' })).toBeNull()
    expect(bindingFor({ ...keys, code: 'ControlLeft' })).toBeNull()
    expect(bindingFor({ ...keys, metaKey: true, code: 'F9' })).toBeNull()
  })

  it('displays physical code names as familiar keycaps', () => {
    expect(keycaps('Ctrl+Alt+Digit5')).toEqual(['Ctrl', 'Alt', '5'])
    expect(keycaps('Ctrl+Shift+F9')).toEqual(['Ctrl', 'Shift', 'F9'])
  })
})
