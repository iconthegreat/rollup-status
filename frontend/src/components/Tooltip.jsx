import { useState, useEffect, useRef } from 'react'

export function Tooltip({ text, children }) {
  const [open, setOpen] = useState(false)
  const ref = useRef(null)

  // Close on outside tap (mobile)
  useEffect(() => {
    if (!open) return
    function handleDown(e) {
      if (ref.current && !ref.current.contains(e.target)) {
        setOpen(false)
      }
    }
    document.addEventListener('pointerdown', handleDown)
    return () => document.removeEventListener('pointerdown', handleDown)
  }, [open])

  return (
    <div
      ref={ref}
      className="relative group/tip"
      onClick={(e) => {
        // Only toggle on touch devices (coarse pointer)
        if (window.matchMedia('(pointer: coarse)').matches) {
          e.stopPropagation()
          setOpen((v) => !v)
        }
      }}
    >
      {children}
      <div className={`
        pointer-events-none absolute bottom-full left-1/2 -translate-x-1/2 mb-1.5
        px-2.5 py-1.5 rounded bg-bg-primary border border-border text-xs text-text-primary
        whitespace-normal max-w-[220px] text-center transition-opacity z-10 shadow-lg
        ${open ? 'opacity-100' : 'opacity-0 group-hover/tip:opacity-100'}
      `}>
        {text}
        <div className="absolute top-full left-1/2 -translate-x-1/2 -mt-px border-4 border-transparent border-t-border" />
      </div>
    </div>
  )
}
