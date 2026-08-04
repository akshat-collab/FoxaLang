type RunState = 'idle' | 'running' | 'ok' | 'err' | 'active';

export function StatusChip({ state, detail }: { state: RunState; detail?: string }) {
  const label =
    state === 'running' ? 'running' : state === 'ok' ? 'ok' : state === 'err' ? 'error' : state === 'active' ? 'ready' : 'idle';
  return (
    <span className="status-chip" data-state={state === 'active' ? 'idle' : state} title={detail}>
      {label}
      {detail ? <span style={{ opacity: 0.7, textTransform: 'none', letterSpacing: 0 }}>· {detail}</span> : null}
    </span>
  );
}
