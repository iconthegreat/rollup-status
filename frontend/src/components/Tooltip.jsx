export function Tooltip({ text, children }) {
  return (
    <div className="relative group/tip">
      {children}
      <div className="pointer-events-none absolute bottom-full left-1/2 -translate-x-1/2 mb-1.5 px-2.5 py-1.5 rounded bg-bg-primary border border-border text-xs text-text-primary whitespace-normal max-w-[220px] text-center opacity-0 group-hover/tip:opacity-100 transition-opacity z-10 shadow-lg">
        {text}
        <div className="absolute top-full left-1/2 -translate-x-1/2 -mt-px border-4 border-transparent border-t-border" />
      </div>
    </div>
  )
}
