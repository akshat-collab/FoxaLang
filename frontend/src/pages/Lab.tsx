import { useMemo, useRef, useState } from 'react';
import { CodeEditor } from '../components/CodeEditor';
import { FileUploader } from '../components/FileUploader';
import { StatusChip } from '../components/StatusChip';
import { runFoxa } from '../lib/foxaInterpreter';
import {
  DATASET_PRESETS,
  MODEL_PRESETS,
  SAMPLE_SCRIPT_CELLS,
  SAMPLE_TRAIN_SCRIPT,
  parseTrainScript,
  trainModel,
  type EpochMetric,
  type TrainConfig,
} from '../lib/mlEngine';
import './Lab.css';

type CellType = 'markdown' | 'code' | 'train';
type Cell = { id: string; type: CellType; content: string; collapsed?: boolean };
type RunState = 'idle' | 'running' | 'ok' | 'err';

export function Lab() {
  const [cells, setCells] = useState<Cell[]>(SAMPLE_SCRIPT_CELLS.map((c) => ({ ...c })));
  const [outputs, setOutputs] = useState<Record<string, string[]>>({});
  const [cellState, setCellState] = useState<Record<string, RunState>>({});
  const [metrics, setMetrics] = useState<EpochMetric[]>([]);
  const [progress, setProgress] = useState(0);
  const [training, setTraining] = useState(false);
  const [config, setConfig] = useState<TrainConfig>({
    model: 'dense',
    epochs: 12,
    learningRate: 0.01,
    batchSize: 16,
    dataset: 'iris',
  });
  const abortRef = useRef<AbortController | null>(null);

  const chartPoints = useMemo(() => {
    if (!metrics.length) return '';
    const w = 280;
    const h = 80;
    const maxLoss = Math.max(...metrics.map((m) => Math.max(m.loss, m.valLoss)), 0.1);
    return metrics
      .map((m, i) => {
        const x = (i / Math.max(metrics.length - 1, 1)) * w;
        const y = h - (m.loss / maxLoss) * (h - 8) - 4;
        return `${x},${y}`;
      })
      .join(' ');
  }, [metrics]);

  const updateCell = (id: string, content: string) => {
    setCells((prev) => prev.map((c) => (c.id === id ? { ...c, content } : c)));
  };

  const toggleCollapse = (id: string) => {
    setCells((prev) => prev.map((c) => (c.id === id ? { ...c, collapsed: !c.collapsed } : c)));
  };

  const addCell = (type: CellType) => {
    const id = String(Date.now());
    const content =
      type === 'markdown'
        ? '## New note'
        : type === 'train'
          ? SAMPLE_TRAIN_SCRIPT
          : `fn main() {\n    show("new cell");\n}\n`;
    setCells((prev) => [...prev, { id, type, content }]);
  };

  const removeCell = (id: string) => {
    setCells((prev) => prev.filter((c) => c.id !== id));
    setOutputs((prev) => {
      const next = { ...prev };
      delete next[id];
      return next;
    });
  };

  const runCodeCell = (cell: Cell) => {
    setCellState((s) => ({ ...s, [cell.id]: 'running' }));
    requestAnimationFrame(() => {
      const r = runFoxa(cell.content);
      setOutputs((prev) => ({
        ...prev,
        [cell.id]: r.ok ? r.output : [...r.output, `error: ${r.error}`],
      }));
      setCellState((s) => ({ ...s, [cell.id]: r.ok ? 'ok' : 'err' }));
    });
  };

  const runTrainCell = async (cell: Cell) => {
    const parsed = parseTrainScript(cell.content);
    const cfg: TrainConfig = {
      model: parsed.model ?? config.model,
      epochs: parsed.epochs ?? config.epochs,
      learningRate: parsed.learningRate ?? config.learningRate,
      batchSize: parsed.batchSize ?? config.batchSize,
      dataset: parsed.dataset ?? config.dataset,
    };
    setConfig(cfg);
    setTraining(true);
    setMetrics([]);
    setProgress(0);
    setCellState((s) => ({ ...s, [cell.id]: 'running' }));
    abortRef.current = new AbortController();
    try {
      const result = await trainModel(
        cfg,
        (m, p) => {
          setMetrics((prev) => [...prev, m]);
          setProgress(p);
        },
        abortRef.current.signal,
      );
      setOutputs((prev) => ({
        ...prev,
        [cell.id]: [
          `trained ${result.modelName}`,
          `accuracy ${(result.finalAccuracy * 100).toFixed(2)}%`,
          `duration ${(result.durationMs / 1000).toFixed(1)}s`,
        ],
      }));
      setCellState((s) => ({ ...s, [cell.id]: 'ok' }));
    } catch (err) {
      setOutputs((prev) => ({
        ...prev,
        [cell.id]: [err instanceof Error ? err.message : String(err)],
      }));
      setCellState((s) => ({ ...s, [cell.id]: 'err' }));
    } finally {
      setTraining(false);
      abortRef.current = null;
    }
  };

  return (
    <div className="lab">
      <div className="toolbar">
        <span className="lab-title mono">notebook</span>
        <span className="toolbar-sep" />
        <button type="button" className="btn btn-ghost btn-sm" onClick={() => addCell('code')}>
          + Code
        </button>
        <button type="button" className="btn btn-ghost btn-sm" onClick={() => addCell('train')}>
          + Train
        </button>
        <button type="button" className="btn btn-ghost btn-sm" onClick={() => addCell('markdown')}>
          + Markdown
        </button>
        <FileUploader
          accept=".foxa,.txt,.md"
          onLoad={(name, content) => {
            const type: CellType = name.endsWith('.md')
              ? 'markdown'
              : content.includes('Trainer') || content.includes('epochs')
                ? 'train'
                : 'code';
            setCells((prev) => [...prev, { id: String(Date.now()), type, content }]);
          }}
        />
        <span className="toolbar-sep" />
        <StatusChip state={training ? 'running' : 'idle'} detail={training ? `${Math.round(progress * 100)}%` : undefined} />
      </div>

      <div className="lab-body">
        <div className="lab-cells">
          {cells.map((cell, idx) => {
            const state = cellState[cell.id] ?? 'idle';
            return (
              <article key={cell.id} className="nb-cell run-rail" data-state={state === 'idle' ? 'active' : state}>
                <div className="nb-head">
                  <button type="button" className="nb-collapse mono" onClick={() => toggleCollapse(cell.id)}>
                    {cell.collapsed ? '▸' : '▾'} In [{idx + 1}]
                  </button>
                  <span className="nb-kind mono">{cell.type}</span>
                  <div className="nb-actions">
                    {cell.type === 'code' && (
                      <button
                        type="button"
                        className="btn btn-run btn-sm"
                        data-state={state === 'running' ? 'running' : undefined}
                        onClick={() => runCodeCell(cell)}
                      >
                        ▶
                      </button>
                    )}
                    {cell.type === 'train' && (
                      <>
                        <button
                          type="button"
                          className="btn btn-run btn-sm"
                          data-state={state === 'running' ? 'running' : undefined}
                          disabled={training}
                          onClick={() => void runTrainCell(cell)}
                        >
                          ▶
                        </button>
                        {training && cellState[cell.id] === 'running' && (
                          <button type="button" className="btn btn-danger btn-sm" onClick={() => abortRef.current?.abort()}>
                            Stop
                          </button>
                        )}
                      </>
                    )}
                    <button type="button" className="btn btn-ghost btn-sm" onClick={() => removeCell(cell.id)}>
                      ×
                    </button>
                  </div>
                </div>

                {!cell.collapsed && (
                  <>
                    {cell.type === 'markdown' ? (
                      <textarea
                        className="nb-md"
                        value={cell.content}
                        onChange={(e) => updateCell(cell.id, e.target.value)}
                        rows={3}
                      />
                    ) : (
                      <CodeEditor value={cell.content} onChange={(v) => updateCell(cell.id, v)} minHeight={160} />
                    )}
                    {outputs[cell.id] && (
                      <div className="nb-out">
                        <div className="nb-out-label mono">Out [{idx + 1}]</div>
                        <pre className="console-body">
                          {outputs[cell.id].map((line, i) => (
                            <div key={i} className={line.startsWith('error') ? 'err-line' : 'ok-line'}>
                              {line}
                            </div>
                          ))}
                        </pre>
                      </div>
                    )}
                  </>
                )}
              </article>
            );
          })}
        </div>

        <aside className="lab-rail">
          <div className="panel-head">
            <span>Training</span>
          </div>
          <div className="lab-rail-body">
            <div className="field">
              <label>Model</label>
              <select value={config.model} onChange={(e) => setConfig((c) => ({ ...c, model: e.target.value }))}>
                {MODEL_PRESETS.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.label}
                  </option>
                ))}
              </select>
            </div>
            <div className="field">
              <label>Dataset</label>
              <select value={config.dataset} onChange={(e) => setConfig((c) => ({ ...c, dataset: e.target.value }))}>
                {DATASET_PRESETS.map((d) => (
                  <option key={d.id} value={d.id}>
                    {d.label}
                  </option>
                ))}
              </select>
            </div>
            <div className="lab-rail-row">
              <div className="field">
                <label>Epochs</label>
                <input
                  type="number"
                  min={1}
                  max={50}
                  value={config.epochs}
                  onChange={(e) => setConfig((c) => ({ ...c, epochs: Number(e.target.value) }))}
                />
              </div>
              <div className="field">
                <label>Batch</label>
                <input
                  type="number"
                  min={1}
                  max={256}
                  value={config.batchSize}
                  onChange={(e) => setConfig((c) => ({ ...c, batchSize: Number(e.target.value) }))}
                />
              </div>
            </div>
            <div className="field">
              <label>LR</label>
              <input
                type="number"
                step="0.001"
                value={config.learningRate}
                onChange={(e) => setConfig((c) => ({ ...c, learningRate: Number(e.target.value) }))}
              />
            </div>

            <button
              type="button"
              className="btn btn-ghost"
              style={{ width: '100%', marginTop: 8 }}
              onClick={() => {
                const script = `fn main() {
    let trainer = Trainer {
        model: "${config.model}",
        epochs: ${config.epochs},
        learning_rate: ${config.learningRate},
        batch_size: ${config.batchSize},
        dataset: "${config.dataset}",
    };
    show("config ready");
}
`;
                setCells((prev) => [...prev, { id: String(Date.now()), type: 'train', content: script }]);
              }}
            >
              Insert train cell
            </button>

            <div className="lab-metrics">
              <div className="lab-metrics-head mono">
                <span>loss</span>
                <span>{Math.round(progress * 100)}%</span>
              </div>
              <div className="lab-progress">
                <div style={{ width: `${progress * 100}%` }} />
              </div>
              {metrics.length > 0 ? (
                <>
                  <svg viewBox="0 0 280 80" className="lab-chart" aria-label="Loss curve">
                    <polyline fill="none" stroke="var(--accent)" strokeWidth="1.5" points={chartPoints} />
                  </svg>
                  <div className="lab-metric-grid mono">
                    <span>ep {metrics[metrics.length - 1].epoch}</span>
                    <span>loss {metrics[metrics.length - 1].loss}</span>
                    <span>acc {(metrics[metrics.length - 1].accuracy * 100).toFixed(1)}%</span>
                    <span>val {(metrics[metrics.length - 1].valAccuracy * 100).toFixed(1)}%</span>
                  </div>
                </>
              ) : (
                <p className="lab-hint">Run a train cell to stream metrics.</p>
              )}
            </div>
          </div>
        </aside>
      </div>
    </div>
  );
}
