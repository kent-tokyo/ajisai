import './Icons.css'

interface IconProps {
  size?: number
  color?: string
  className?: string
}

export function FileNewIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M8 1V7M11 4H5" stroke={color} strokeWidth="1.5" strokeLinecap="round" />
      <path d="M2 2H10L14 6V14H2V2Z" stroke={color} strokeWidth="1.5" strokeLinejoin="round" />
    </svg>
  )
}

export function FileOpenIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M1 3.5L7 2L14 3.5L14 12C14 13.1046 13.1046 14 12 14H2C0.89543 14 0 13.1046 0 12V4C0 3.44772 0.447715 3 1 3Z" stroke={color} strokeWidth="1.5" />
      <path d="M7 2L14 3.5" stroke={color} strokeWidth="1.5" strokeLinejoin="round" />
    </svg>
  )
}

export function FileSaveIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M2 1H10L14 5V14H2V1Z" stroke={color} strokeWidth="1.5" strokeLinejoin="round" />
      <path d="M4 5.5H12M6 9H10M6 11H8" stroke={color} strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  )
}

export function PlayIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M2 2L14 8L2 14V2Z" fill={color} stroke={color} strokeWidth="1" strokeLinejoin="round" />
    </svg>
  )
}

export function UndoIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M2 5C2 2.79086 3.79086 1 6 1H12C13.1046 1 14 1.89543 14 3V7" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M1 4L2 5.5L3 4" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  )
}

export function RedoIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M14 5C14 2.79086 12.2091 1 10 1H4C2.89543 1 2 1.89543 2 3V7" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M15 4L14 5.5L13 4" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  )
}

export function DeleteIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M1 4H15M2 4V13C2 14.1046 2.89543 15 4 15H12C13.1046 15 14 14.1046 14 13V4" stroke={color} strokeWidth="1.5" strokeLinejoin="round" />
      <path d="M5.5 8V12M8 8V12M10.5 8V12" stroke={color} strokeWidth="1.5" strokeLinecap="round" />
      <path d="M6 1H10V4H6V1Z" stroke={color} strokeWidth="1.5" strokeLinejoin="round" />
    </svg>
  )
}

export function StatusIdleIcon({ size = 12, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 12 12" fill="none">
      <circle cx="6" cy="6" r="4.5" stroke={color} strokeWidth="1" />
    </svg>
  )
}

export function StatusRunningIcon({ size = 12, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 12 12" fill="none">
      <circle cx="6" cy="6" r="4.5" fill={color} opacity="0.6" />
      <circle cx="6" cy="6" r="3" fill="none" stroke={color} strokeWidth="1" />
    </svg>
  )
}

export function StatusDoneIcon({ size = 12, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 12 12" fill="none">
      <circle cx="6" cy="6" r="5" fill={color} />
      <path d="M3 6L5 8L9 4" stroke="white" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  )
}

export function StatusErrorIcon({ size = 12, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 12 12" fill="none">
      <circle cx="6" cy="6" r="5" fill={color} />
      <path d="M4 4L8 8M8 4L4 8" stroke="white" strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  )
}

export function PipelineIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <circle cx="2" cy="8" r="1.5" fill={color} />
      <line x1="3.5" y1="8" x2="6.5" y2="8" stroke={color} strokeWidth="1.5" strokeLinecap="round" />
      <circle cx="8" cy="8" r="1.5" fill={color} />
      <line x1="9.5" y1="8" x2="12.5" y2="8" stroke={color} strokeWidth="1.5" strokeLinecap="round" />
      <circle cx="14" cy="8" r="1.5" fill={color} />
    </svg>
  )
}

export function SettingsIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="2" stroke={color} strokeWidth="1.5" />
      <path d="M8 1V3M8 13V15M3 8H1M15 8H13" stroke={color} strokeWidth="1.5" strokeLinecap="round" />
      <path d="M3.5 3.5L2.1 2.1M13.9 13.9L12.5 12.5M12.5 3.5L13.9 2.1M2.1 13.9L3.5 12.5" stroke={color} strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  )
}

export function WarningIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M8 1L15 14H1L8 1Z" stroke={color} strokeWidth="1.5" strokeLinejoin="round" />
      <path d="M8 6V10M8 12H8.01" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  )
}

export function ClipboardIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M5 1H11L12 2V14C12 14.5523 11.5523 15 11 15H3C2.44772 15 2 14.5523 2 14V2L3 1H5Z" stroke={color} strokeWidth="1.5" />
      <path d="M5 1C5 0.447715 5.44772 0 6 0H10C10.5523 0 11 0.447715 11 1" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M5 5H11M5 8H11M5 11H9" stroke={color} strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  )
}

export function RefreshIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M1 8C1 4.13401 4.13401 1 8 1C10.23 1 12.2 2.08333 13.35 3.75" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M15 8C15 11.866 11.866 15 8 15C5.77 15 3.8 13.9167 2.65 12.25" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M14 3V1H12M2 15V13H4" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  )
}

export function ChevronIcon({ size = 16, color = 'currentColor' }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path d="M6 2L12 8L6 14" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  )
}
