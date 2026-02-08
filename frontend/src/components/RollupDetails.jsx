import { rollupMetadata, formatDuration, formatThreshold } from '../constants/rollupMetadata'
import { Tooltip } from './Tooltip'

function LivenessIndicator({ label, tooltip, ageSecs, cadenceSecs, delayedSecs }) {
  let colorClass = 'text-text-secondary'
  if (ageSecs != null) {
    if (ageSecs <= cadenceSecs) {
      colorClass = 'text-success'
    } else if (ageSecs <= delayedSecs) {
      colorClass = 'text-warning'
    } else {
      colorClass = 'text-error'
    }
  }

  return (
    <Tooltip text={tooltip}>
      <div className="flex items-center justify-between cursor-help flex-wrap gap-x-2 gap-y-0.5">
        <span className="text-xs text-text-secondary underline decoration-dotted decoration-text-secondary/40 underline-offset-2">{label}</span>
        <span className={`text-xs font-mono ${colorClass}`}>
          {ageSecs != null ? `${formatDuration(ageSecs)} ago` : '—'}
        </span>
      </div>
    </Tooltip>
  )
}

const livenessTooltips = {
  lastEvent: 'Time since any on-chain event was seen for this rollup. Green = within cadence, yellow = delayed, red = exceeds delayed threshold.',
  lastBatch: 'Time since the last batch/state update was posted to L1. Batches contain compressed L2 transaction data.',
  lastProof: 'Time since the last proof or assertion was submitted to L1. Proofs verify the correctness of L2 state transitions.',
}

const ruleTooltips = {
  batchCadence: 'Expected interval between batch submissions. If exceeded, batch liveness turns yellow.',
  proofCadence: 'Expected interval between proof submissions. If exceeded, proof liveness turns yellow.',
  delayed: 'If no events arrive within this window, the rollup status changes to "Delayed" (warning).',
  halted: 'If no events arrive within this window, the rollup status changes to "Halted" (error).',
}

const issueTooltips = {
  'No events': 'The backend has not received any on-chain events for this rollup since it started.',
  'No batch': 'Batch submissions have exceeded the expected cadence. The sequencer may be delayed.',
  'No proof': 'Proof submissions have exceeded the expected cadence. The prover may be delayed.',
  'halted threshold': 'No events for an extended period. The rollup may be experiencing a halt.',
  'delayed threshold': 'Events are arriving slower than expected. The rollup may be experiencing congestion.',
}

const eventTooltips = {
  BatchDelivered: 'Arbitrum batch of compressed L2 transactions posted to the SequencerInbox on L1.',
  ProofSubmitted: 'An assertion claiming a new L2 state root, submitted to the RollupCore contract.',
  ProofVerified: 'An assertion that passed the challenge period and was confirmed as valid on L1.',
  StateUpdate: 'Starknet state diff with a STARK proof, verified and recorded on the L1 core contract.',
  DisputeGameCreated: 'A new dispute game proposing an L2 output root, created via the DisputeGameFactory.',
  WithdrawalProven: 'A withdrawal proof submitted to the OptimismPortal, enabling funds to exit L2.',
  BlockCommit: 'A batch of L2 blocks committed to the zkSync Diamond contract on L1.',
  BlocksVerification: 'A ZK proof verifying a range of committed batches on L1.',
  BlockExecution: 'A verified batch executed and finalized on L1, making its state permanent.',
  MessageLog: 'An L1-to-L2 message logged on the Starknet core contract.',
}

const rollupTypeTooltips = {
  'Optimistic Rollup': 'Assumes transactions are valid by default. Fraud proofs can challenge invalid state within a dispute window.',
  'ZK Rollup': 'Submits cryptographic validity proofs (ZK proofs) to L1, guaranteeing correctness without a challenge period.',
  'OP Stack Rollup': 'Built on the OP Stack framework with fault proof games for dispute resolution.',
}

const contractTooltips = {
  'Sequencer Inbox': 'The L1 contract where the Arbitrum sequencer posts compressed transaction batches.',
  'Rollup Core': 'Manages Arbitrum assertions, challenges, and state confirmations on L1.',
  'Core Contract': 'The Starknet L1 contract that verifies STARK proofs and records state updates.',
  'Dispute Game Factory': 'Creates dispute games for each proposed L2 output root on OP Stack chains.',
  'Optimism Portal': 'The gateway contract for deposits and withdrawals between L1 and the OP Stack L2.',
  'Diamond Proxy': 'The upgradeable proxy contract for zkSync Era that handles commits, proofs, and execution.',
}

function getIssueTooltip(issue) {
  for (const [key, tip] of Object.entries(issueTooltips)) {
    if (issue.toLowerCase().includes(key.toLowerCase())) return tip
  }
  return issue
}

export function RollupDetails({ rollup, health, sequencer }) {
  const meta = rollupMetadata[rollup]
  if (!meta) return null

  const { thresholds, contracts, events, type: rollupType } = meta

  return (
    <div className="space-y-4 pt-3">
      {/* Liveness Indicators */}
      <div>
        <Tooltip text="How recently the backend observed on-chain activity for this rollup. Color-coded: green = on schedule, yellow = behind cadence, red = exceeds threshold.">
          <p className="text-xs text-text-secondary uppercase tracking-wide mb-2 cursor-help underline decoration-dotted decoration-text-secondary/40 underline-offset-2 w-fit">
            Liveness
          </p>
        </Tooltip>
        <div className="space-y-1.5 bg-bg-tertiary rounded-md p-2.5 sm:p-3">
          <LivenessIndicator
            label="Last Event"
            tooltip={livenessTooltips.lastEvent}
            ageSecs={health?.last_event_age_secs}
            cadenceSecs={thresholds.batchCadenceSecs}
            delayedSecs={thresholds.delayedSecs}
          />
          <LivenessIndicator
            label="Last Batch"
            tooltip={livenessTooltips.lastBatch}
            ageSecs={health?.last_batch_age_secs}
            cadenceSecs={thresholds.batchCadenceSecs}
            delayedSecs={thresholds.delayedSecs}
          />
          <LivenessIndicator
            label="Last Proof"
            tooltip={livenessTooltips.lastProof}
            ageSecs={health?.last_proof_age_secs}
            cadenceSecs={thresholds.proofCadenceSecs}
            delayedSecs={thresholds.delayedSecs}
          />
        </div>
      </div>

      {/* Health Rules */}
      <div>
        <Tooltip text="Configurable thresholds that determine when a rollup is flagged as delayed or halted based on event cadence.">
          <p className="text-xs text-text-secondary uppercase tracking-wide mb-2 cursor-help underline decoration-dotted decoration-text-secondary/40 underline-offset-2 w-fit">
            Health Rules
          </p>
        </Tooltip>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-4 gap-y-1.5 bg-bg-tertiary rounded-md p-2.5 sm:p-3">
          <Tooltip text={ruleTooltips.batchCadence}>
            <div className="flex justify-between cursor-help">
              <span className="text-xs text-text-secondary underline decoration-dotted decoration-text-secondary/40 underline-offset-2">Batch cadence</span>
              <span className="text-xs font-mono text-text-primary">{formatThreshold(thresholds.batchCadenceSecs)}</span>
            </div>
          </Tooltip>
          <Tooltip text={ruleTooltips.proofCadence}>
            <div className="flex justify-between cursor-help">
              <span className="text-xs text-text-secondary underline decoration-dotted decoration-text-secondary/40 underline-offset-2">Proof cadence</span>
              <span className="text-xs font-mono text-text-primary">{formatThreshold(thresholds.proofCadenceSecs)}</span>
            </div>
          </Tooltip>
          <Tooltip text={ruleTooltips.delayed}>
            <div className="flex justify-between cursor-help">
              <span className="text-xs text-text-secondary underline decoration-dotted decoration-text-secondary/40 underline-offset-2">Delayed after</span>
              <span className="text-xs font-mono text-warning">{formatThreshold(thresholds.delayedSecs)}</span>
            </div>
          </Tooltip>
          <Tooltip text={ruleTooltips.halted}>
            <div className="flex justify-between cursor-help">
              <span className="text-xs text-text-secondary underline decoration-dotted decoration-text-secondary/40 underline-offset-2">Halted after</span>
              <span className="text-xs font-mono text-error">{formatThreshold(thresholds.haltedSecs)}</span>
            </div>
          </Tooltip>
        </div>
      </div>

      {/* Active Issues */}
      {health?.issues?.length > 0 && (
        <div>
          <Tooltip text="Current problems detected by the health monitor. Issues clear automatically when normal activity resumes.">
            <p className="text-xs text-text-secondary uppercase tracking-wide mb-2 cursor-help underline decoration-dotted decoration-text-secondary/40 underline-offset-2 w-fit">
              Active Issues
            </p>
          </Tooltip>
          <div className="flex flex-wrap gap-1.5">
            {health.issues.map((issue, i) => (
              <Tooltip key={i} text={getIssueTooltip(issue)}>
                <span
                  className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs bg-warning/20 text-warning cursor-help"
                >
                  <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                  </svg>
                  {issue}
                </span>
              </Tooltip>
            ))}
          </div>
        </div>
      )}

      {/* Sequencer Metrics */}
      {sequencer && (
        <div>
          <Tooltip text="Detailed metrics from polling the L2 sequencer's RPC endpoint for block production activity.">
            <p className="text-xs text-text-secondary uppercase tracking-wide mb-2 cursor-help underline decoration-dotted decoration-text-secondary/40 underline-offset-2 w-fit">
              Sequencer Metrics
            </p>
          </Tooltip>
          <div className="space-y-1.5 bg-bg-tertiary rounded-md p-2.5 sm:p-3">
            <div className="flex items-center justify-between flex-wrap gap-x-2 gap-y-0.5">
              <Tooltip text="How long ago the sequencer produced its most recent block. A rising value may signal the sequencer has stalled.">
                <span className="text-xs text-text-secondary cursor-help underline decoration-dotted decoration-text-secondary/40 underline-offset-2">Seconds since last block</span>
              </Tooltip>
              <span className="text-xs font-mono text-text-primary">
                {sequencer.seconds_since_last_block != null
                  ? `${formatDuration(sequencer.seconds_since_last_block)}`
                  : '—'}
              </span>
            </div>
            <div className="flex items-center justify-between flex-wrap gap-x-2 gap-y-0.5">
              <Tooltip text="When the backend last queried the L2 RPC for new block data.">
                <span className="text-xs text-text-secondary cursor-help underline decoration-dotted decoration-text-secondary/40 underline-offset-2">Last polled</span>
              </Tooltip>
              <span className="text-xs font-mono text-text-primary">
                {sequencer.last_polled
                  ? new Date(sequencer.last_polled * 1000).toLocaleTimeString()
                  : '—'}
              </span>
            </div>
            <div className="flex items-center justify-between flex-wrap gap-x-2 gap-y-0.5">
              <Tooltip text="The timestamp embedded in the latest L2 block, set by the sequencer when the block was produced.">
                <span className="text-xs text-text-secondary cursor-help underline decoration-dotted decoration-text-secondary/40 underline-offset-2">Block timestamp</span>
              </Tooltip>
              <span className="text-xs font-mono text-text-primary">
                {sequencer.latest_block_timestamp
                  ? new Date(sequencer.latest_block_timestamp * 1000).toLocaleTimeString()
                  : '—'}
              </span>
            </div>
          </div>
        </div>
      )}

      {/* Rollup Metadata */}
      <div>
        <Tooltip text="Technical details about this rollup including its type, the on-chain events we track, and the L1 smart contracts being monitored.">
          <p className="text-xs text-text-secondary uppercase tracking-wide mb-2 cursor-help underline decoration-dotted decoration-text-secondary/40 underline-offset-2 w-fit">
            Rollup Info
          </p>
        </Tooltip>
        <div className="bg-bg-tertiary rounded-md p-2.5 sm:p-3 space-y-2">
          <div className="flex items-center gap-2 flex-wrap">
            <Tooltip text={rollupTypeTooltips[rollupType] || rollupType}>
              <span className="inline-flex px-2 py-0.5 rounded text-xs bg-bg-secondary text-text-primary border border-border cursor-help">
                {rollupType}
              </span>
            </Tooltip>
            {events.map((evt) => (
              <Tooltip key={evt} text={eventTooltips[evt] || `On-chain event: ${evt}`}>
                <span
                  className="inline-flex px-1.5 py-0.5 rounded text-xs bg-bg-secondary text-text-secondary border border-border cursor-help"
                >
                  {evt}
                </span>
              </Tooltip>
            ))}
          </div>
          <div className="space-y-1.5">
            {contracts.map((c) => (
              <div key={c.address} className="flex items-center justify-between flex-wrap gap-x-2 gap-y-0.5">
                <Tooltip text={contractTooltips[c.label] || `L1 contract: ${c.label}`}>
                  <span className="text-xs text-text-secondary cursor-help underline decoration-dotted decoration-text-secondary/40 underline-offset-2">{c.label}</span>
                </Tooltip>
                <a
                  href={`https://etherscan.io/address/${c.address}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-xs font-mono text-text-primary hover:text-arbitrum transition-colors inline-flex items-center gap-1"
                >
                  {c.address.slice(0, 6)}...{c.address.slice(-4)}
                  <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                  </svg>
                </a>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
