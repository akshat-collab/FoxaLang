export type TrainConfig = {
  model: string;
  epochs: number;
  learningRate: number;
  batchSize: number;
  dataset: string;
};

export type EpochMetric = {
  epoch: number;
  loss: number;
  accuracy: number;
  valLoss: number;
  valAccuracy: number;
};

export type TrainResult = {
  metrics: EpochMetric[];
  finalAccuracy: number;
  modelName: string;
  durationMs: number;
};

const DATASETS: Record<string, { samples: number; features: number; classes: number }> = {
  iris: { samples: 150, features: 4, classes: 3 },
  mnist_tiny: { samples: 2000, features: 64, classes: 10 },
  housing: { samples: 506, features: 13, classes: 1 },
  sentiment: { samples: 5000, features: 128, classes: 2 },
};

/** Simulated Foxa ML training loop (browser-side demo engine). */
export async function trainModel(
  config: TrainConfig,
  onEpoch?: (m: EpochMetric, progress: number) => void,
  signal?: AbortSignal,
): Promise<TrainResult> {
  const started = performance.now();
  const ds = DATASETS[config.dataset] ?? DATASETS.iris;
  const metrics: EpochMetric[] = [];
  let loss = 2.2 + Math.random() * 0.4;
  let acc = 0.12 + Math.random() * 0.08;
  let valLoss = loss + 0.15;
  let valAcc = acc - 0.03;

  const lrFactor = Math.min(1.4, Math.max(0.5, config.learningRate / 0.01));
  const batchFactor = Math.min(1.2, Math.max(0.7, 32 / config.batchSize));

  for (let epoch = 1; epoch <= config.epochs; epoch++) {
    if (signal?.aborted) throw new Error('training aborted');

    await delay(180 + Math.random() * 120);

    const decay = Math.exp(-0.18 * epoch * lrFactor * batchFactor);
    loss = Math.max(0.02, loss * (0.82 + Math.random() * 0.08) * (0.95 + decay * 0.05));
    valLoss = Math.max(0.03, loss * (1.05 + Math.random() * 0.08));
    acc = Math.min(0.995, acc + (1 - acc) * (0.18 * lrFactor) * (0.7 + Math.random() * 0.3));
    valAcc = Math.min(0.99, acc - 0.01 - Math.random() * 0.04);

    // model complexity nudge
    if (config.model.includes('deep') || config.model.includes('cnn')) {
      acc = Math.min(0.998, acc + 0.01);
      valAcc = Math.min(0.992, valAcc + 0.008);
    }

    const m: EpochMetric = {
      epoch,
      loss: round(loss),
      accuracy: round(acc),
      valLoss: round(valLoss),
      valAccuracy: round(valAcc),
    };
    metrics.push(m);
    onEpoch?.(m, epoch / config.epochs);
  }

  void ds;
  return {
    metrics,
    finalAccuracy: metrics[metrics.length - 1]?.accuracy ?? 0,
    modelName: `${config.model}_${config.dataset}`,
    durationMs: performance.now() - started,
  };
}

export function parseTrainScript(source: string): Partial<TrainConfig> {
  const cfg: Partial<TrainConfig> = {};
  const model = source.match(/model\s*[:=]\s*["']?([\w.]+)["']?/i);
  const epochs = source.match(/epochs?\s*[:=]\s*(\d+)/i);
  const lr = source.match(/learning_?rate\s*[:=]\s*([\d.]+)/i);
  const batch = source.match(/batch_?size\s*[:=]\s*(\d+)/i);
  const dataset = source.match(/dataset\s*[:=]\s*["']?([\w.]+)["']?/i);
  if (model) cfg.model = model[1];
  if (epochs) cfg.epochs = Number(epochs[1]);
  if (lr) cfg.learningRate = Number(lr[1]);
  if (batch) cfg.batchSize = Number(batch[1]);
  if (dataset) cfg.dataset = dataset[1];
  return cfg;
}

export const SAMPLE_TRAIN_SCRIPT = `// Foxa ML training script
use foxa::ml::{Model, Dataset, Trainer};

fn main() {
    let data = Dataset::load("iris");
    let model = Model::dense([4, 16, 8, 3]);

    let trainer = Trainer {
        model: model,
        epochs: 12,
        learning_rate: 0.01,
        batch_size: 16,
        dataset: "iris",
    };

    let result = trainer.fit(data);
    print("accuracy:");
    print(result.accuracy);
}
`;

export const SAMPLE_SCRIPT_CELLS = [
  {
    id: '1',
    type: 'markdown' as const,
    content:
      '# Foxa ML Lab\nTrain models in Foxa — Colab-style cells. Run cells top-to-bottom.',
  },
  {
    id: '2',
    type: 'code' as const,
    content: `fn main() {
    print("Loading iris dataset...");
    let n = 150;
    print("samples:");
    print(n);
}`,
  },
  {
    id: '3',
    type: 'train' as const,
    content: SAMPLE_TRAIN_SCRIPT,
  },
];

function delay(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

function round(n: number) {
  return Math.round(n * 10000) / 10000;
}

export const MODEL_PRESETS = [
  { id: 'dense', label: 'Dense MLP', desc: 'Fully connected layers' },
  { id: 'dense_deep', label: 'Deep MLP', desc: 'Wider/deeper network' },
  { id: 'cnn_tiny', label: 'Tiny CNN', desc: 'Conv net for images' },
  { id: 'linear', label: 'Linear', desc: 'Fast baseline regressor' },
];

export const DATASET_PRESETS = [
  { id: 'iris', label: 'Iris', desc: '150 samples · 3 classes' },
  { id: 'mnist_tiny', label: 'MNIST Tiny', desc: '2k digits · 10 classes' },
  { id: 'housing', label: 'Housing', desc: 'Regression benchmark' },
  { id: 'sentiment', label: 'Sentiment', desc: 'Text binary classify' },
];
