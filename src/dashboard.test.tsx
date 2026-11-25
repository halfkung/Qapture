import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it } from 'vitest'
import { App } from './App'

describe('dashboard controls', () => {
  const markup = renderToStaticMarkup(<App />)

  it('shows both capture actions while native settings load', () => {
    expect(markup).toMatch(/<button[^>]+class="capture-card region"[^>]+disabled=""[^>]*>.*Capture a region/)
    expect(markup).toMatch(/<button[^>]+class="capture-card full"[^>]+disabled=""[^>]*>.*Copy full desktop/)
  })

  it('gives each shortcut a visible Edit button with a distinct accessible name', () => {
    expect(markup).toMatch(/aria-label="Edit region capture shortcut" aria-expanded="false".*?<\/svg> Edit<\/button>/)
    expect(markup).toMatch(/aria-label="Edit full desktop shortcut" aria-expanded="false".*?<\/svg> Edit<\/button>/)
    expect(markup).not.toContain('<form')
  })
})
