import { invoke } from '@tauri-apps/api/core'
import { useEffect, useRef, useState } from 'react'
import { decodeImage } from './markup'

export function Pin() {
  const canvas = useRef<HTMLCanvasElement>(null)
  const [error, setError] = useState('')
  useEffect(() => {
    let active = true
    void invoke<ArrayBuffer>('load_pin_image').then((packet) => {
      if (!active || !canvas.current) return
      const image = decodeImage(packet)
      canvas.current.width = image.width
      canvas.current.height = image.height
      canvas.current.getContext('2d')?.putImageData(new ImageData(image.pixels, image.width, image.height), 0, 0)
    }).catch((problem) => { if (active) setError(String(problem)) })
    return () => { active = false }
  }, [])
  return <main className="pinned-image">
    {error ? <p role="alert">{error}</p> : <canvas ref={canvas} aria-label="Pinned capture" />}
  </main>
}
