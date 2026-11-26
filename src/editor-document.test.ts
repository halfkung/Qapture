import { describe, expect, it } from 'vitest'
import { commit, emptyHistory, redo, restoreDocument, undo } from './editor-document'
import { boundsOf, resize, translate, type Mark } from './markup'

const mark: Mark = {
  id: 'first', tool: 'text', points: [{ x: 5, y: 10 }, { x: 65, y: 50 }],
  color: '#000000', opacity: 1, width: 2, text: 'ทดสอบ\nQapture',
}

describe('non-destructive editor commands', () => {
  it('preserves Unicode, move and resize across undo and redo', () => {
    const initial = commit(emptyHistory, { marks: [mark] })
    const placed = resize(translate(mark, 15, 20), { x: 100, y: 110 })
    const updated = commit(initial, { marks: [placed] })
    expect(boundsOf(placed.points)).toEqual({ x: 20, y: 30, width: 80, height: 80 })
    expect(placed.text).toBe(mark.text)
    expect(undo(updated).current.marks).toEqual([mark])
    expect(redo(undo(updated)).current.marks).toEqual([placed])
  })

  it('restores crop and deleted objects and clears redo after a new edit', () => {
    const initial = commit(emptyHistory, { marks: [mark] })
    const cropped = commit(initial, { ...initial.current, crop: { x: 2, y: 3, width: 80, height: 70 } })
    const deleted = commit(cropped, { ...cropped.current, marks: [] })
    expect(undo(deleted).current).toEqual(cropped.current)
    expect(undo(undo(deleted)).current.crop).toBeUndefined()
    expect(commit(undo(deleted), { marks: [] }).future).toEqual([])
  })

  it('rejects inconsistent crop and annotation data from a local project', () => {
    const image = { width: 100, height: 100 }
    expect(restoreDocument({ marks: [mark] }, image).marks[0].text).toBe(mark.text)
    expect(() => restoreDocument({ marks: [{ ...mark, color: 'transparent' }] }, image)).toThrow('invalid annotations')
    expect(() => restoreDocument({ marks: [mark], crop: { x: 99, y: 0, width: 2, height: 50 } }, image)).toThrow('invalid annotations')
  })
})
