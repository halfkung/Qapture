import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { App } from './App'
import { Editor } from './Editor'
import { Pin } from './Pin'
import './styles.css'

const view = new URLSearchParams(window.location.search).get('view')
const View = view === 'editor' ? Editor : view === 'pin' ? Pin : App
createRoot(document.getElementById('root')!).render(<StrictMode><View /></StrictMode>)
