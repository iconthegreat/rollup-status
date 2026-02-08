export function Header({ connectionStatus }) {
  const getConnectionConfig = () => {
    switch (connectionStatus) {
      case 'connected':
        return {
          dot: 'bg-success',
          text: 'Connected',
          animate: false,
        }
      case 'connecting':
        return {
          dot: 'bg-warning',
          text: 'Connecting...',
          animate: true,
        }
      case 'reconnecting':
        return {
          dot: 'bg-warning',
          text: 'Reconnecting...',
          animate: true,
        }
      default:
        return {
          dot: 'bg-error',
          text: 'Disconnected',
          animate: false,
        }
    }
  }

  const config = getConnectionConfig()

  return (
    <header className="border-b border-border bg-bg-secondary">
      <div className="max-w-7xl mx-auto px-4 py-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 sm:w-10 sm:h-10 rounded-lg bg-arbitrum/20 flex items-center justify-center">
              <svg
                className="w-5 h-5 sm:w-6 sm:h-6 text-arbitrum"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                />
              </svg>
            </div>
            <div>
              <h1 className="text-lg sm:text-xl font-semibold text-text-primary">
                Rollup Status
              </h1>
              <p className="text-sm text-text-secondary hidden sm:block">
                Real-time Rollup Analytics Dashboard
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2 text-sm">
            <span
              className={`w-2 h-2 rounded-full ${config.dot} ${config.animate ? 'animate-pulse' : ''}`}
            />
            <span className="text-text-secondary hidden sm:inline">{config.text}</span>
          </div>
        </div>
      </div>
    </header>
  )
}
