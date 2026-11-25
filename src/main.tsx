import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { App } from './App'
import { Editor } from './Editor'
import './styles.css'

const View = new URLSearchParams(window.location.search).get('view') === 'editor' ? Editor : App
createRoot(document.getElementById('root')!).render(<StrictMode><View /></StrictMode>)
