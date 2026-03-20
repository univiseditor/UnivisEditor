# Graph Core Performance Model

## Purpose

This note defines the intended performance model for `univis_graph_core` after
the executable layer, validation integration, and runtime adapter work.

The goal is not premature micro-optimization. The goal is to make rebuild,
propagation, and scheduling costs explicit so higher layers can treat
`ExecutableGraph` as a predictable execution kernel.

## Permanent State In `ExecutableGraph`

The following data should be stored permanently in `ExecutableGraph` and reused
across execution passes until the graph structure changes:

- executable nodes
- direct incoming and outgoing links
- `validation_report`
- per-node diagnostics
- execution-order seed
- build-partiality state
- per-node execution state
- authored input buffers
- resolved input buffers
- output buffers
- runtime `custom_data`

These are part of the execution truth, not temporary scratch state.

## Rebuilt Only On Structural Change

The following work should happen only when authored graph structure changes:

- rebuilding the executable graph from `GraphDocument`
- rebuilding direct adjacency
- rebuilding validation-derived node diagnostics
- reseeding execution order from topology
- rebuilding build-time blocked state

Structural change means changes such as:

- node added or removed
- edge added, removed, or rewired
- node definition id changed
- input/output arity changed
- registry/schema behavior changed in a way that affects validation or build

Ordinary authored-value edits should not force a full rebuild.

## Dirty Model

A node becomes dirty when any of the following happens:

- one of its authored inputs changes
- one of its externally synchronized outputs changes
- it is explicitly marked dirty by a higher layer
- one of its upstream nodes produces changed outputs

Dirty state should propagate only downstream.

Dirty state should not imply immediate execution by itself. A node still needs
to satisfy readiness rules.

## Readiness Model

A node is eligible to run only when all of the following are true:

- it is enabled
- it is dirty
- it is not build-blocked
- it is not execution-blocked by missing upstream data
- it has no pending upstream inputs

This keeps execution cost proportional to the set of runnable nodes rather than
the full graph size.

## Output Change Model

Outputs are considered changed when the post-run output buffer is not equal to
the previous output buffer.

In the current model this is a full `Vec<Value>` equality check per executed
node.

That means:

- unchanged outputs short-circuit downstream propagation
- changed outputs mark direct downstream nodes dirty

This is the key mechanism that keeps steady-state execution cheap for graphs
whose values stabilize quickly.

## Cost Model

The intended operational cost model is:

- full rebuild: proportional to node count + edge count + validation cost
- input resolution for one node: proportional to its input count and incoming
  links inspected
- one execution pass over ready nodes: proportional to runnable nodes plus
  downstream propagation triggered by changed outputs
- downstream dirtiness propagation: proportional to direct outgoing links of
  nodes whose outputs changed

The important distinction is:

- rebuild cost is graph-structural
- rerun cost is graph-dynamic

The system should pay rebuild cost rarely and rerun cost often.

## Rebuild Policy

Rebuild `ExecutableGraph` only when structure-level authored truth changes.

Typical rebuild triggers:

- live document structure changed
- node registry changed
- schema validation behavior changed

Do not rebuild for:

- ordinary authored input edits
- external widget-driven output changes
- runtime `custom_data` updates
- dirty propagation itself

## Rerun Policy

Rerun execution when:

- there is at least one dirty node
- and at least one node becomes ready after input resolution

Do not rerun blocked or disabled nodes.

Do not continue propagation past a node whose outputs did not change.

This means the normal steady-state runtime path should be:

1. synchronize authored inputs / external outputs / custom data
2. mark affected nodes dirty
3. resolve inputs for dirty nodes
4. run ready nodes
5. propagate only when outputs changed

## Adapter Guidance

Higher runtime layers should treat `ExecutableGraph` as the long-lived
execution cache.

Adapters may project compatibility views such as:

- ECS-facing resolved input caches
- ECS-facing connectivity summaries
- editor diagnostics resources

But they should not own a second execution engine.

## Future Optimization Boundary

If the current model becomes too expensive later, optimize in this order:

1. reduce unnecessary rebuild triggers
2. reduce unnecessary dirty marks
3. improve output-change comparison strategy
4. introduce more incremental structural rebuilds only if profiling demands it

Until profiling says otherwise, correctness and a single execution truth matter
more than speculative caching complexity.
