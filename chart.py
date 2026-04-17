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

BASE = Path("target/criterion")
GROUPS = ["high throughput", "low throughput"]
BENCHMARKS = ["gpsc", "tokio mpsc recv", "tokio mpsc recv_many"]


def load_stats(path):
    with open(path) as f:
        data = json.load(f)
    mean = data["mean"]["point_estimate"] / 1_000_000
    lower = data["mean"]["confidence_interval"]["lower_bound"] / 1_000_000
    upper = data["mean"]["confidence_interval"]["upper_bound"] / 1_000_000
    return mean, lower, upper


# Collect data: results[benchmark][group] = (mean, lower, upper)
results = {}
for bench in BENCHMARKS:
    results[bench] = {}
    for group in GROUPS:
        path = BASE / group / bench / "new" / "estimates.json"
        if path.exists():
            results[bench][group] = load_stats(path)
        else:
            print(f"Missing: {path}")

# Plot
x = np.arange(len(BENCHMARKS))
width = 0.35
colors = ["#4C72B0", "#DD8452"]

fig, ax = plt.subplots(figsize=(9, 5))

for i, group in enumerate(GROUPS):
    means = [results[b][group][0] for b in BENCHMARKS]
    lowers = [results[b][group][0] - results[b][group][1] for b in BENCHMARKS]
    uppers = [results[b][group][2] - results[b][group][0] for b in BENCHMARKS]

    bars = ax.bar(
        x + i * width,
        means,
        width,
        label=group,
        color=colors[i],
        yerr=[lowers, uppers],
        capsize=5,
        error_kw={"elinewidth": 1.5, "ecolor": "#555"},
    )
    ax.bar_label(bars, fmt="%.2f ms", padding=4, fontsize=8)

ax.set_ylabel("Mean latency (ms)")
ax.set_title("MPSC Channel Benchmarks — High vs Low Throughput")
ax.set_xticks(x + width / 2)
ax.set_xticklabels(BENCHMARKS)
ax.legend()
ax.set_ylim(0, max(results[b][g][0] for b in BENCHMARKS for g in GROUPS) * 1.3)
ax.spines[["top", "right"]].set_visible(False)

plt.tight_layout()
plt.savefig("benchmark_comparison.png", dpi=150)
plt.show()
