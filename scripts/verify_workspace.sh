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

app_checks=(
  "cargo check -p univis_editor_app --lib --quiet"
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
  app)
    run_commands "App checks" "${app_checks[@]}"
    ;;
  all)
    run_commands "Core checks" "${core_checks[@]}"
    run_commands "Core tests" "${core_tests[@]}"
    run_commands "Builtin checks" "${builtin_checks[@]}"
    run_commands "Builtin tests" "${builtin_tests[@]}"
    run_commands "Persistence checks" "${persistence_checks[@]}"
    run_commands "Persistence tests" "${persistence_tests[@]}"
    run_commands "App checks" "${app_checks[@]}"
    ;;
  *)
    echo "Usage: $0 [core|builtin|persistence|app|all]" >&2
    exit 1
    ;;
esac
