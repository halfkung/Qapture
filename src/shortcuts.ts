export const captureModes = ['region', 'full'] as const
export type CaptureMode = (typeof captureModes)[number]

export type ShortcutStatus = {
  settings: Record<CaptureMode, string>
  regionActive: boolean
  fullActive: boolean
  warning: string | null
  regionError: string | null
  fullError: string | null
}

type Keystroke = Pick<KeyboardEvent, 'code' | 'ctrlKey' | 'altKey' | 'shiftKey' | 'metaKey'>

export function bindingFor({ code, ctrlKey, altKey, shiftKey, metaKey }: Keystroke) {
  if (metaKey || (!ctrlKey && !altKey)) return null
  if (!/^(Key[A-Z]|Digit[0-9]|F(?:[1-9]|1[0-2]))$/.test(code)) return null
  return [ctrlKey && 'Ctrl', altKey && 'Alt', shiftKey && 'Shift', code].filter(Boolean).join('+')
}

export function keycaps(binding: string) {
  return binding.split('+').map((key) => key.replace(/^(Key|Digit)(?=[A-Z0-9]$)/, ''))
}
