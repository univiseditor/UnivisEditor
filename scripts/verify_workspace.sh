#!/usr/bin/env bash

set -euo pipefail

export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
mode="${1:-all}"

core_checks=(
  "cargo check -p univis_node_graph --lib --quiet"
  "cargo check -p univis_scene --lib --quiet"
  "cargo check -p univis_editor_runtime --lib --quiet"
)

core_tests=(
  "cargo test -p univis_node_graph --test core_api --quiet"
  "cargo test -p univis_node_graph --test document_ops --quiet"
  "cargo test -p univis_node_graph --test document_workflows --quiet"
  "cargo test -p univis_node_graph --test graph_validation --quiet"
  "cargo test -p univis_editor_runtime --lib --quiet"
)

builtin_checks=(
  "cargo check -p univis_editor_nodes_builtin --lib --quiet"
)

builtin_tests=(
  "cargo test -p univis_editor_nodes_builtin --test input_nodes --quiet"
  "cargo test -p univis_editor_nodes_builtin --test math_nodes --quiet"
  "cargo test -p univis_editor_nodes_builtin --test logic_nodes --quiet"
  "cargo test -p univis_editor_nodes_builtin --test scene_nodes --quiet"
)

persistence_checks=(
  "cargo check -p univis_editor_persistence --lib --quiet"
)

persistence_tests=(
  "cargo test -p univis_editor_persistence --test persistence_defaults --quiet"
  "cargo test -p univis_editor_persistence --test persistence_format --quiet"
  "cargo test -p univis_editor_persistence --test workflow_smoke --quiet"
)

workflow_checks=(
  "cargo check -p univis_editor_commands --lib --quiet"
  "cargo check -p univis_editor_ui --lib --quiet"
  "cargo check -p univis_editor_workflows --lib --quiet"
)

workflow_tests=(
  "cargo test -p univis_editor_ui --test editor_smoke --quiet"
  "cargo test -p univis_editor_workflows --test workflow_assets_smoke --quiet"
)

app_checks=(
  "cargo check -p univis_editor_app --lib --quiet"
)

fmt_checks=(
  "cargo fmt --all --check"
)

clippy_core_checks=(
  "cargo clippy -p univis_node_graph -p univis_scene -p univis_editor_runtime --lib --tests -- -D warnings"
)

clippy_editor_checks=(
  "cargo clippy -p univis_editor_nodes_builtin -p univis_editor_ui -p univis_editor_workflows -p univis_editor_persistence -p univis_editor_app --lib --tests --examples -- -D warnings"
)

run_commands() {
  local label="$1"
  shift
  local commands=("$@")

  echo
  echo "## $label"
  for command in "${commands[@]}"; do
    echo "==> $command"
    eval "$command"
  done
}

case "$mode" in
  core)
    run_commands "Core checks" "${core_checks[@]}"
    run_commands "Core tests" "${core_tests[@]}"
    ;;
  builtin)
    run_commands "Builtin checks" "${builtin_checks[@]}"
    run_commands "Builtin tests" "${builtin_tests[@]}"
    ;;
  persistence)
    run_commands "Persistence checks" "${persistence_checks[@]}"
    run_commands "Persistence tests" "${persistence_tests[@]}"
    ;;
  workflows)
    run_commands "Workflow checks" "${workflow_checks[@]}"
    run_commands "Workflow tests" "${workflow_tests[@]}"
    ;;
  app)
    run_commands "App checks" "${app_checks[@]}"
    ;;
  fmt)
    run_commands "Format checks" "${fmt_checks[@]}"
    ;;
  clippy-core)
    run_commands "Clippy core checks" "${clippy_core_checks[@]}"
    ;;
  clippy-editor)
    run_commands "Clippy editor checks" "${clippy_editor_checks[@]}"
    ;;
  all)
    run_commands "Core checks" "${core_checks[@]}"
    run_commands "Core tests" "${core_tests[@]}"
    run_commands "Builtin checks" "${builtin_checks[@]}"
    run_commands "Builtin tests" "${builtin_tests[@]}"
    run_commands "Persistence checks" "${persistence_checks[@]}"
    run_commands "Persistence tests" "${persistence_tests[@]}"
    run_commands "Workflow checks" "${workflow_checks[@]}"
    run_commands "Workflow tests" "${workflow_tests[@]}"
    run_commands "App checks" "${app_checks[@]}"
    ;;
  *)
    echo "Usage: $0 [core|builtin|persistence|workflows|app|fmt|clippy-core|clippy-editor|all]" >&2
    exit 1
    ;;
esac
