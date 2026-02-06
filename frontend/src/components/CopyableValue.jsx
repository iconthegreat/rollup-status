import { useState } from 'react'

export function CopyableValue({ value, isHash = false, etherscanUrl = null }) {
  const [copied, setCopied] = useState(false)

  if (!value || value === '—') {
    return <span className="text-lg font-mono text-text-primary">—</span>
  }

  const handleCopy = async (e) => {
    e.preventDefault()
    e.stopPropagation()
    await navigator.clipboard.writeText(value)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const stringValue = String(value)
  const displayValue = stringValue.length > 16
    ? `${stringValue.slice(0, 8)}...${stringValue.slice(-6)}`
    : typeof value === 'number' ? value.toLocaleString() : value

  const content = (
    <span
      onClick={handleCopy}
      className="text-lg font-mono text-text-primary cursor-pointer hover:text-arbitrum transition-colors inline-flex items-center gap-2 group"
      title={copied ? 'Copied!' : `Click to copy: ${value}`}
    >
      {displayValue}
      <svg
        className={`w-4 h-4 opacity-0 group-hover:opacity-100 transition-opacity ${copied ? 'text-success' : 'text-text-secondary'}`}
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        {copied ? (
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
        ) : (
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
        )}
      </svg>
    </span>
  )

  if (etherscanUrl) {
    return (
      <div className="flex items-center gap-2">
        {content}
        <a
          href={etherscanUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="text-text-secondary hover:text-arbitrum transition-colors"
          title="View on Etherscan"
          onClick={(e) => e.stopPropagation()}
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
          </svg>
        </a>
      </div>
    )
  }

  return content
}
