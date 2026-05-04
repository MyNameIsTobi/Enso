export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  if (bytes < 1024 ** 4) return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
  return `${(bytes / 1024 ** 4).toFixed(1)} TB`;
}

export function formatCount(n: number): string {
  return n.toLocaleString("en-US");
}

export function formatAge(mtime: number): string {
  const diffMs  = Date.now() - mtime;
  const diffMin = Math.floor(diffMs / 60_000);
  if (diffMin < 1)   return "just now";
  if (diffMin < 60)  return `${diffMin}m ago`;
  const diffH = Math.floor(diffMin / 60);
  if (diffH < 24)    return `${diffH}h ago`;
  const diffD = Math.floor(diffH / 24);
  if (diffD < 365)   return `${diffD}d ago`;
  return `${Math.floor(diffD / 365)}y ago`;
}

export function formatAgeDays(mtime: number): number {
  return Math.floor((Date.now() - mtime) / 86_400_000);
}

// ████████░░░░░░░ progress bar
export function asciiBar(ratio: number, width = 16): string {
  const filled = Math.round(Math.max(0, Math.min(1, ratio)) * width);
  return "█".repeat(filled) + "░".repeat(width - filled);
}

export const CATEGORY_LABELS: Record<number, string> = {
  0: "images",
  1: "videos",
  2: "docs",
  3: "dev",
  4: "archives",
  5: "other",
};

export const ARTIFACT_LABELS: Record<number, string> = {
  // JavaScript / TypeScript
   0: "node_modules",
   1: "next.js",
   2: "nuxt.js",
   3: "svelte-kit",
   4: "angular",
   5: "parcel cache",
   6: "turbo cache",
   7: "expo",
   8: "js dist",
  // Rust
  10: "rust target",
  // Python
  20: "python venv",
  21: "__pycache__",
  22: "mypy cache",
  23: "pytest cache",
  24: "ruff cache",
  25: "tox",
  // Java / Kotlin / Android
  30: "gradle cache",
  31: "gradle build",
  32: "maven target",
  33: "idea out",
  // .NET
  40: ".net bin",
  // Go
  50: "go vendor",
  // PHP
  60: "composer",
  // Ruby
  70: "ruby vendor",
  71: "bundler",
  // Elixir
  80: "elixir build",
  81: "elixir deps",
  // Swift / Apple
  90: "swift build",
  91: "xcode data",
  92: "cocoapods",
  // Dart / Flutter
  100: "dart tool",
  101: "flutter build",
  // Terraform
  110: "terraform",
  // CMake / C++
  120: "cmake build",
  // Scala
  130: "scala target",
  // Haskell
  140: "haskell stack",
  141: "haskell cabal",
  // Zig
  150: "zig out",
  151: "zig cache",
  // R
  160: "r packrat",
};
