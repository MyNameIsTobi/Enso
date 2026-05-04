use tauri::State;
use serde::{Serialize, Deserialize};
use std::path::Path;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevArtifact {
    pub id:            u32,
    pub project_root:  String,
    pub artifact_name: String,
    pub size:          u64,
    pub mtime:         i64,
    pub stale:         bool,
    pub kind:          u8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ArtifactKind {
    // ── JavaScript / TypeScript ──────────────────────────────────────────
    NodeModules    =  0,
    NextJs         =  1,
    NuxtJs         =  2,
    SvelteKit      =  3,
    AngularCache   =  4,
    ParcelCache    =  5,
    TurboCache     =  6,
    ExpoCache      =  7,
    JsDist         =  8,

    // ── Rust ────────────────────────────────────────────────────────────
    RustTarget     = 10,

    // ── Python ──────────────────────────────────────────────────────────
    PythonVenv     = 20,
    PythonCache    = 21,
    MypyCache      = 22,
    PytestCache    = 23,
    RuffCache      = 24,
    ToxCache       = 25,

    // ── Java / Kotlin / Android ─────────────────────────────────────────
    GradleCache    = 30,
    GradleBuild    = 31,
    MavenTarget    = 32,
    IdeaBuildOut   = 33,

    // ── .NET / C# ───────────────────────────────────────────────────────
    DotNetBin      = 40,

    // ── Go ──────────────────────────────────────────────────────────────
    GoVendor       = 50,

    // ── PHP ─────────────────────────────────────────────────────────────
    ComposerVendor = 60,

    // ── Ruby ────────────────────────────────────────────────────────────
    RubyVendor     = 70,
    RubyBundle     = 71,

    // ── Elixir ──────────────────────────────────────────────────────────
    ElixirBuild    = 80,
    ElixirDeps     = 81,

    // ── Swift / Apple ───────────────────────────────────────────────────
    SwiftBuild     = 90,
    DerivedData    = 91,
    CocoaPods      = 92,

    // ── Dart / Flutter ──────────────────────────────────────────────────
    DartTool       = 100,
    FlutterBuild   = 101,

    // ── Terraform / IaC ─────────────────────────────────────────────────
    TerraformDir   = 110,

    // ── CMake / C++ ─────────────────────────────────────────────────────
    CMakeBuild     = 120,

    // ── Scala / SBT ─────────────────────────────────────────────────────
    ScalaTarget    = 130,

    // ── Haskell ─────────────────────────────────────────────────────────
    HaskellStack   = 140,
    HaskellCabal   = 141,

    // ── Zig ─────────────────────────────────────────────────────────────
    ZigOut         = 150,
    ZigCache       = 151,

    // ── R ───────────────────────────────────────────────────────────────
    RPackrat       = 160,
}

#[inline]
fn sibling_exists(path: &Path, sibling: &str) -> bool {
    path.parent()
        .map(|p| p.join(sibling).exists())
        .unwrap_or(false)
}

#[inline]
fn any_sibling(path: &Path, siblings: &[&str]) -> bool {
    path.parent()
        .map(|p| siblings.iter().any(|s| p.join(s).exists()))
        .unwrap_or(false)
}

/// Post-pass over FileIndex — no re-walk needed.
#[tauri::command]
pub async fn get_dev_artifacts(
    state: State<'_, AppState>,
) -> Result<Vec<DevArtifact>, String> {
    let index = &state.index;
    let now_ms = chrono::Utc::now().timestamp_millis();
    let stale_threshold_ms = 7 * 86_400_000i64;

    let nodes_guard = index.nodes.read();
    let mut artifacts: Vec<DevArtifact> = Vec::new();

    for node in nodes_guard.iter() {
        if node.id == u32::MAX || !node.is_dir { continue; }

        let name = node.name.as_ref();
        let path = &node.path;

        let kind_opt: Option<ArtifactKind> = match name {
            // ── JavaScript / TypeScript ────────────────────────────────
            "node_modules"  => Some(ArtifactKind::NodeModules),
            ".next"         => Some(ArtifactKind::NextJs),
            ".nuxt"         => Some(ArtifactKind::NuxtJs),
            ".svelte-kit"   => Some(ArtifactKind::SvelteKit),
            ".parcel-cache" => Some(ArtifactKind::ParcelCache),
            ".expo"         => Some(ArtifactKind::ExpoCache),
            ".angular" => {
                if any_sibling(path, &["angular.json", "package.json"]) {
                    Some(ArtifactKind::AngularCache)
                } else { None }
            }
            ".turbo" => {
                if any_sibling(path, &["turbo.json", "package.json"]) {
                    Some(ArtifactKind::TurboCache)
                } else { None }
            }

            // ── Rust ──────────────────────────────────────────────────
            // (also handles Maven and Scala — all use "target/")
            "target" => {
                if sibling_exists(path, "Cargo.toml") {
                    Some(ArtifactKind::RustTarget)
                } else if sibling_exists(path, "pom.xml") {
                    Some(ArtifactKind::MavenTarget)
                } else if any_sibling(path, &["build.sbt", "build.sc"]) {
                    Some(ArtifactKind::ScalaTarget)
                } else { None }
            }

            // ── Python ────────────────────────────────────────────────
            "venv" | ".venv" => Some(ArtifactKind::PythonVenv),
            "__pycache__"    => Some(ArtifactKind::PythonCache),
            ".mypy_cache"    => Some(ArtifactKind::MypyCache),
            ".pytest_cache"  => Some(ArtifactKind::PytestCache),
            ".ruff_cache"    => Some(ArtifactKind::RuffCache),
            ".tox"           => Some(ArtifactKind::ToxCache),

            // ── Java / Kotlin / Android ────────────────────────────────
            ".gradle" => Some(ArtifactKind::GradleCache),

            // ── .NET ──────────────────────────────────────────────────
            "bin" => {
                if sibling_exists(path, "obj") {
                    Some(ArtifactKind::DotNetBin)
                } else { None }
            }

            // ── Elixir ────────────────────────────────────────────────
            "_build" => {
                if sibling_exists(path, "mix.exs") {
                    Some(ArtifactKind::ElixirBuild)
                } else { None }
            }
            "deps" => {
                if sibling_exists(path, "mix.exs") {
                    Some(ArtifactKind::ElixirDeps)
                } else { None }
            }

            // ── Swift / Apple ──────────────────────────────────────────
            ".build" => {
                if sibling_exists(path, "Package.swift") {
                    Some(ArtifactKind::SwiftBuild)
                } else { None }
            }
            "DerivedData" => Some(ArtifactKind::DerivedData),
            "Pods" => {
                if sibling_exists(path, "Podfile") {
                    Some(ArtifactKind::CocoaPods)
                } else { None }
            }

            // ── Dart / Flutter ─────────────────────────────────────────
            ".dart_tool" => Some(ArtifactKind::DartTool),

            // ── Terraform / IaC ────────────────────────────────────────
            ".terraform" => Some(ArtifactKind::TerraformDir),

            // ── Haskell ────────────────────────────────────────────────
            ".stack-work" => Some(ArtifactKind::HaskellStack),
            "dist-newstyle" => {
                if any_sibling(path, &["cabal.project", "cabal.project.local"]) {
                    Some(ArtifactKind::HaskellCabal)
                } else { None }
            }

            // ── Zig ────────────────────────────────────────────────────
            "zig-out" => {
                if sibling_exists(path, "build.zig") {
                    Some(ArtifactKind::ZigOut)
                } else { None }
            }
            "zig-cache" => Some(ArtifactKind::ZigCache),

            // ── R ──────────────────────────────────────────────────────
            "packrat" => {
                if any_sibling(path, &["packrat.lock", ".Rproj.user"]) {
                    Some(ArtifactKind::RPackrat)
                } else { None }
            }

            // ── Ambiguous names — require project file context ─────────
            "dist" => {
                if sibling_exists(path, "package.json") {
                    Some(ArtifactKind::JsDist)
                } else { None }
            }
            "build" => {
                if sibling_exists(path, "pubspec.yaml") {
                    Some(ArtifactKind::FlutterBuild)
                } else if sibling_exists(path, "CMakeLists.txt") {
                    Some(ArtifactKind::CMakeBuild)
                } else if any_sibling(path, &[
                    "build.gradle", "build.gradle.kts",
                    "settings.gradle", "settings.gradle.kts",
                ]) {
                    Some(ArtifactKind::GradleBuild)
                } else { None }
            }
            "out" => {
                // IntelliJ IDEA build output
                if sibling_exists(path, ".idea") {
                    Some(ArtifactKind::IdeaBuildOut)
                } else { None }
            }
            "vendor" => {
                if sibling_exists(path, "composer.json") {
                    Some(ArtifactKind::ComposerVendor)
                } else if sibling_exists(path, "Gemfile") {
                    Some(ArtifactKind::RubyVendor)
                } else if sibling_exists(path, "go.mod") {
                    Some(ArtifactKind::GoVendor)
                } else { None }
            }
            ".bundle" => {
                if any_sibling(path, &["Gemfile", "Gemfile.lock"]) {
                    Some(ArtifactKind::RubyBundle)
                } else { None }
            }

            other => {
                // cmake-build-debug, cmake-build-release, etc.
                if other.starts_with("cmake-build-") && sibling_exists(path, "CMakeLists.txt") {
                    Some(ArtifactKind::CMakeBuild)
                } else {
                    None
                }
            }
        };

        if let Some(kind) = kind_opt {
            let project_root = node.path.parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();

            let stale = (now_ms - node.mtime) > stale_threshold_ms;

            artifacts.push(DevArtifact {
                id:            node.id,
                project_root,
                artifact_name: name.to_string(),
                size:          node.size,
                mtime:         node.mtime,
                stale,
                kind:          kind as u8,
            });
        }
    }

    // Sort by size descending
    artifacts.sort_unstable_by(|a, b| b.size.cmp(&a.size));

    Ok(artifacts)
}
