# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "matplotlib",
#   "numpy",
# ]
# ///
import json
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np

BASE = Path("target/criterion/worker count")
WORKER_COUNTS = [10, 100, 1000, 2000, 4000]
BENCHMARKS = ["gpsc", "tokio mpsc"]


def load_stats(path):
    with open(path) as f:
        data = json.load(f)
    mean = data["mean"]["point_estimate"] / 1_000_000
    lower = data["mean"]["confidence_interval"]["lower_bound"] / 1_000_000
    upper = data["mean"]["confidence_interval"]["upper_bound"] / 1_000_000
    return mean, lower, upper


# Collect data: results[benchmark][worker_count] = (mean, lower, upper)
results = {}
for bench in BENCHMARKS:
    results[bench] = {}
    for wc in WORKER_COUNTS:
        path = BASE / bench / str(wc) / "new" / "estimates.json"
        if path.exists():
            results[bench][wc] = load_stats(path)
        else:
            print(f"Missing: {path}")

# Plot
x = np.arange(len(WORKER_COUNTS))
n = len(BENCHMARKS)
width = 0.22
offsets = np.linspace(-(n - 1) / 2, (n - 1) / 2, n) * width
colors = ["#4C72B0", "#DD8452", "#55A868"]

fig, ax = plt.subplots(figsize=(11, 5))

for i, bench in enumerate(BENCHMARKS):
    means = [results[bench][wc][0] for wc in WORKER_COUNTS]
    means = [results[bench][wc][0] for wc in WORKER_COUNTS]
    lowers = [results[bench][wc][0] - results[bench][wc][1] for wc in WORKER_COUNTS]
    uppers = [results[bench][wc][2] - results[bench][wc][0] for wc in WORKER_COUNTS]

    bars = ax.bar(
        x + offsets[i],
        means,
        width,
        label=bench,
        color=colors[i],
        yerr=[lowers, uppers],
        capsize=4,
        error_kw={"elinewidth": 1.5, "ecolor": "#555"},
    )
    ax.bar_label(bars, fmt="%.2f ms", padding=4, fontsize=7.5)

ax.set_ylabel("Mean latency (ms)")
ax.set_title("Processing 1k messages by producer count (lower is better)")
ax.set_xticks(x)
ax.set_xticklabels([f"{wc} worker{'s' if wc > 1 else ''}" for wc in WORKER_COUNTS])
ax.legend(title="Channel type")
ax.set_yscale("log")
ax.yaxis.set_major_formatter(plt.FuncFormatter(lambda y, _: f"{y:g} ms"))
ax.spines[["top", "right"]].set_visible(False)
plt.tight_layout()
plt.savefig("benchmark_comparison.png", dpi=150)
plt.show()
