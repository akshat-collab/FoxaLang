import { useMemo, useRef, useState } from 'react';
import { Play, Plus, Trash2, Square, Upload } from 'lucide-react';
import { CodeEditor } from '../components/CodeEditor';
import { FileUploader } from '../components/FileUploader';
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
type Cell = { id: string; type: CellType; content: string };

export function Lab() {
  const [cells, setCells] = useState<Cell[]>(SAMPLE_SCRIPT_CELLS);
  const [outputs, setOutputs] = useState<Record<string, string[]>>({});
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
    const w = 320;
    const h = 100;
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

  const addCell = (type: CellType) => {
    const id = String(Date.now());
    const content =
      type === 'markdown'
        ? '## New note'
        : type === 'train'
          ? SAMPLE_TRAIN_SCRIPT
          : `fn main() {\n    print("new cell");\n}\n`;
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
    const r = runFoxa(cell.content);
    setOutputs((prev) => ({
      ...prev,
      [cell.id]: r.ok ? r.output : [...r.output, `error: ${r.error}`],
    }));
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
          `final accuracy: ${(result.finalAccuracy * 100).toFixed(2)}%`,
          `duration: ${(result.durationMs / 1000).toFixed(1)}s`,
          `epochs: ${result.metrics.length}`,
        ],
      }));
    } catch (err) {
      setOutputs((prev) => ({
        ...prev,
        [cell.id]: [err instanceof Error ? err.message : String(err)],
      }));
    } finally {
      setTraining(false);
      abortRef.current = null;
    }
  };

  const stopTrain = () => abortRef.current?.abort();

  return (
    <div className="lab container-wide">
      <header className="page-head anim-fade-up">
        <h1>ML Lab</h1>
        <p>
          Colab-style notebook for Foxa: markdown, scripts, and model training cells with live epoch
          metrics.
        </p>
      </header>

      <div className="lab-layout">
        <div className="lab-notebook anim-fade-up anim-delay-1">
          <div className="lab-bar">
            <FileUploader
              accept=".foxa,.txt,.md"
              onLoad={(name, content) => {
                const id = String(Date.now());
                const type: CellType = name.endsWith('.md') ? 'markdown' : content.includes('Trainer') || content.includes('epochs') ? 'train' : 'code';
                setCells((prev) => [...prev, { id, type, content }]);
              }}
            />
            <div className="lab-add">
              <button type="button" className="btn btn-ghost btn-sm" onClick={() => addCell('code')}>
                <Plus size={14} /> Code
              </button>
              <button type="button" className="btn btn-ghost btn-sm" onClick={() => addCell('train')}>
                <Plus size={14} /> Train
              </button>
              <button type="button" className="btn btn-ghost btn-sm" onClick={() => addCell('markdown')}>
                <Plus size={14} /> Markdown
              </button>
            </div>
          </div>

          {cells.map((cell, idx) => (
            <article key={cell.id} className={`nb-cell nb-${cell.type}`}>
              <div className="nb-cell-head">
                <span className="nb-tag mono">
                  [{idx + 1}] {cell.type}
                </span>
                <div className="nb-cell-actions">
                  {cell.type === 'code' && (
                    <button type="button" className="btn btn-mint btn-sm" onClick={() => runCodeCell(cell)}>
                      <Play size={14} /> Run
                    </button>
                  )}
                  {cell.type === 'train' && (
                    <>
                      <button
                        type="button"
                        className="btn btn-primary btn-sm"
                        disabled={training}
                        onClick={() => void runTrainCell(cell)}
                        style={training ? { animation: 'run-blink 1.2s infinite' } : undefined}
                      >
                        <Play size={14} /> {training ? 'Training…' : 'Train'}
                      </button>
                      {training && (
                        <button type="button" className="btn btn-danger btn-sm" onClick={stopTrain}>
                          <Square size={14} /> Stop
                        </button>
                      )}
                    </>
                  )}
                  <button type="button" className="btn btn-ghost btn-sm" onClick={() => removeCell(cell.id)}>
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>

              {cell.type === 'markdown' ? (
                <textarea
                  className="md-area"
                  value={cell.content}
                  onChange={(e) => updateCell(cell.id, e.target.value)}
                  rows={4}
                />
              ) : (
                <CodeEditor value={cell.content} onChange={(v) => updateCell(cell.id, v)} minHeight={200} />
              )}

              {outputs[cell.id] && (
                <pre className="nb-out mono">
                  {outputs[cell.id].map((line, i) => (
                    <div key={i}>{line}</div>
                  ))}
                </pre>
              )}
            </article>
          ))}
        </div>

        <aside className="lab-side anim-fade-up anim-delay-2">
          <h2>Training panel</h2>
          <p className="side-desc">Presets sync into train cells when you run them, or edit the script directly.</p>

          <div className="field">
            <label>Model</label>
            <select
              value={config.model}
              onChange={(e) => setConfig((c) => ({ ...c, model: e.target.value }))}
            >
              {MODEL_PRESETS.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.label} — {m.desc}
                </option>
              ))}
            </select>
          </div>
          <div className="field">
            <label>Dataset</label>
            <select
              value={config.dataset}
              onChange={(e) => setConfig((c) => ({ ...c, dataset: e.target.value }))}
            >
              {DATASET_PRESETS.map((d) => (
                <option key={d.id} value={d.id}>
                  {d.label} — {d.desc}
                </option>
              ))}
            </select>
          </div>
          <div className="field-row">
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
            <label>Learning rate</label>
            <input
              type="number"
              step="0.001"
              min={0.0001}
              max={1}
              value={config.learningRate}
              onChange={(e) => setConfig((c) => ({ ...c, learningRate: Number(e.target.value) }))}
            />
          </div>

          <button
            type="button"
            className="btn btn-ghost"
            style={{ width: '100%', marginTop: '0.5rem' }}
            onClick={() => {
              const script = `// Foxa ML training script
use foxa::ml::{Model, Dataset, Trainer};

fn main() {
    let data = Dataset::load("${config.dataset}");
    let trainer = Trainer {
        model: "${config.model}",
        epochs: ${config.epochs},
        learning_rate: ${config.learningRate},
        batch_size: ${config.batchSize},
        dataset: "${config.dataset}",
    };
    print("config ready");
}
`;
              const id = String(Date.now());
              setCells((prev) => [...prev, { id, type: 'train', content: script }]);
            }}
          >
            <Upload size={14} /> Insert train script from panel
          </button>

          <div className="metrics-box">
            <div className="metrics-head">
              <span>Loss curve</span>
              <span className="mono">{Math.round(progress * 100)}%</span>
            </div>
            <div className="progress-bar">
              <div style={{ width: `${progress * 100}%` }} />
            </div>
            {metrics.length > 0 ? (
              <>
                <svg viewBox="0 0 320 100" className="loss-chart" role="img" aria-label="Loss over epochs">
                  <polyline fill="none" stroke="var(--accent)" strokeWidth="2" points={chartPoints} />
                </svg>
                <div className="metric-latest mono">
                  <div>epoch {metrics[metrics.length - 1].epoch}</div>
                  <div>loss {metrics[metrics.length - 1].loss}</div>
                  <div>acc {(metrics[metrics.length - 1].accuracy * 100).toFixed(1)}%</div>
                  <div>val {(metrics[metrics.length - 1].valAccuracy * 100).toFixed(1)}%</div>
                </div>
              </>
            ) : (
              <p className="side-desc">Run a train cell to stream metrics.</p>
            )}
          </div>
        </aside>
      </div>
    </div>
  );
}
