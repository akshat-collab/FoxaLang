export const SAMPLES = {
  hello: `fn main() {
    print("Hello, Foxa!");
}

fn add(a: Int, b: Int) -> Int {
    a + b
}
`,
  features: `fn main() {
    let p = Point { x: 1, y: 2 };
    print(p.x);

    let mut i = 0;
    while i < 3 {
        print(i);
        i = i + 1;
    }

    for n in [10, 20] {
        print(n);
    }

    match Some(42) {
        Some(x) => print(x),
        None => print(0),
    }
}

struct Point {
    x: Int,
    y: Int,
}
`,
  loops: `fn main() {
    let mut sum = 0;
    let mut i = 1;
    while i <= 10 {
        sum = sum + i;
        i = i + 1;
    }
    print("sum 1..10:");
    print(sum);

    for x in [2, 4, 6, 8] {
        print(x * x);
    }
}
`,
  functions: `fn greet(name: String) -> String {
    "Hello, " + name
}

fn fib(n: Int) -> Int {
    if n < 2 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fn main() {
    print(greet("Foxa"));
    print("fib(8):");
    print(fib(8));
}
`,
};

export const LEARN_LESSONS = [
  {
    id: 'intro',
    title: 'What is Foxa?',
    minutes: 3,
    body: `Foxa is a modern systems language focused on safety, performance, and a great developer experience.

It blends unique ownership with explicit ARC sharing (\`shared T\`), algebraic data types, and a batteries-included toolchain — compiler, package manager, and (here) an in-browser playground.`,
    code: SAMPLES.hello,
  },
  {
    id: 'variables',
    title: 'Variables & mutability',
    minutes: 4,
    body: `Bind values with \`let\`. Add \`mut\` when you need reassignment. Types can be inferred locally; public APIs usually take explicit annotations.

\`\`\`foxa
let count = 0;
let mut total = 0;
total = total + 1;
\`\`\``,
    code: `fn main() {
    let name = "Foxa";
    let mut n = 0;
    n = n + 1;
    print(name);
    print(n);
}
`,
  },
  {
    id: 'control',
    title: 'Control flow',
    minutes: 5,
    body: `Foxa supports \`if\` / \`else\`, \`while\`, \`for … in\`, and \`match\` on enums like \`Option\`.

Loops and matches are expressions in spirit — keep bodies small and prefer returning values over deep nesting.`,
    code: SAMPLES.loops,
  },
  {
    id: 'functions',
    title: 'Functions',
    minutes: 5,
    body: `Define functions with \`fn\`. Parameters need types; return types use \`->\`. The last expression in a block can be the return value (no semicolon).

Every runnable program needs \`fn main()\`.`,
    code: SAMPLES.functions,
  },
  {
    id: 'structs',
    title: 'Structs & match',
    minutes: 6,
    body: `Group data with \`struct\`. Pattern-match with \`match\` — especially useful with \`Some\` / \`None\` instead of null.`,
    code: SAMPLES.features,
  },
  {
    id: 'ml',
    title: 'ML in Foxa (Lab)',
    minutes: 4,
    body: `The Foxa Lab mirrors a Colab notebook: markdown notes, code cells, and train cells.

A train cell is a Foxa script that declares \`model\`, \`epochs\`, \`learning_rate\`, \`batch_size\`, and \`dataset\`. The Lab engine runs a simulated training loop and streams epoch metrics — great for teaching ML workflows in Foxa syntax.`,
    code: `use foxa::ml::{Model, Dataset, Trainer};

fn main() {
    let data = Dataset::load("iris");
    let model = Model::dense([4, 16, 8, 3]);
    let trainer = Trainer {
        model: "dense",
        epochs: 10,
        learning_rate: 0.01,
        batch_size: 16,
        dataset: "iris",
    };
    print("ready to train");
}
`,
  },
];
