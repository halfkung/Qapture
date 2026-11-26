import { describe, expect, it } from 'vitest'
import { decodeImage, imagePoint, visibleMark } from './markup'

describe('editor pixel coordinates', () => {
  it('maps a scaled preview to physical pixels and clamps drags beyond its edges', () => {
    const bounds = { left: 40, top: 20, width: 200, height: 100 } as DOMRect
    const size = { width: 800, height: 400 }
    expect(imagePoint({ x: 140, y: 45 }, bounds, size)).toEqual({ x: 400, y: 100 })
    expect(imagePoint({ x: 900, y: -3 }, bounds, size)).toEqual({ x: 800, y: 0 })
  })

  it('keeps brush dots and ignores zero-area rectangles', () => {
    const points = [{ x: 6, y: 4 }]
    expect(visibleMark({ tool: 'pen', points })).toBe(true)
    expect(visibleMark({ tool: 'rectangle', points })).toBe(false)
  })

  it('decodes the native RGBA packet and rejects a mismatched length', () => {
    const image = new Uint8Array([1, 0, 0, 0, 1, 0, 0, 0, 70, 80, 90, 255])
    expect(decodeImage(image.buffer)).toMatchObject({ width: 1, height: 1, pixels: new Uint8ClampedArray([70, 80, 90, 255]) })
    expect(() => decodeImage(image.slice(0, 11).buffer)).toThrow('invalid dimensions')
  })
})
