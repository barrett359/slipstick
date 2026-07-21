# Slipstick agent CLI

Slipstick's Rust binary is the canonical interface for Codex, Claude Code,
humans, and scripts. It contains no model SDK, API key, or provider-specific
prompt. Run `slipstick` or `slipstick serve` for the existing web app; use
`slipstick agent ...` for JSON automation. During development, substitute
`cargo run --quiet --` for `slipstick` in every example below.

## Transport contract

- Supply substantial JSON with `--input path.json` or `--input -` for stdin.
- Stdout contains exactly one compact JSON envelope. Diagnostics go to stderr.
- Every envelope has `schema_version`, `command`, `revision`, `summary`,
  `data`, `warnings`, and `artifacts`.
- Exit codes are stable: `0` success, `2` invalid input or validation, `3`
  revision conflict, `4` missing object, `5` failed simulation, and `6` I/O.
- Calculator arrays larger than the context-safe threshold are summarized in
  the envelope and written in full under `data/agent-workspaces/artifacts/`.
  Combat timelines and ensembles are always artifact-backed.

Start with:

```sh
slipstick agent capabilities
slipstick agent schema fleet
slipstick agent schema missile_design
slipstick agent schema missile
slipstick agent fields search radiator
slipstick agent snapshot --select /designs/0
```

`schema missile_design` describes the persisted entity. `schema missile`
describes the calculator input and output. Generated schemas include plain
descriptions, `x-unit` metadata, inferred probability/percentage bounds, and
an explicit assumptions list.

## Isolated design workflow

Create a draft and retain the returned ID:

```sh
slipstick agent draft create --name "credible escort"
slipstick agent draft get DRAFT_ID --select /designs
```

Patch requests use the `add`, `replace`, and `remove` subset of RFC 6902. A
draft may temporarily be invalid, which makes multi-step redesign practical;
validation and commit are the gates.

```json
[
  {"op":"replace","path":"/designs/0/name","value":"Ascendant refit"},
  {"op":"replace","path":"/designs/0/mr","value":6.5},
  {"op":"add","path":"/designs/0/components/0/combat","value":{
    "role":"drive","exposure":0.35,"vulnerability":1.0,
    "redundancy_group":"main-drive","degraded_at":0.7,
    "disabled_at":0.25,"destroyed_at":0.05
  }}
]
```

```sh
slipstick agent draft patch DRAFT_ID --input refit.patch.json
slipstick agent draft validate DRAFT_ID
slipstick agent evaluate --draft DRAFT_ID --design battleship
slipstick agent draft diff DRAFT_ID
slipstick agent draft rollback DRAFT_ID --steps 1
```

`draft validate` reports schema/reference problems and design evaluations;
`evaluate` reports addressed mass, tankage, propulsion, acceleration, delta-v,
power, radiator, sink, flywheel, laser, and ordnance findings without changing
the design. Drafts live under ignored
`data/agent-workspaces/DRAFT_ID/` with their base revision, working fleet,
append-only `operations.jsonl`, revision history, scenarios, and runs.

Commit is revision-checked and entity-selective. Preview first, then apply the
same selection. Omitting `--select` is an error.

```sh
slipstick agent draft commit DRAFT_ID --select designs:battleship
slipstick agent draft commit DRAFT_ID --select designs:battleship --apply
```

Valid selectors include `settings`, `system`, an entire top-level collection,
or `designs:ID`, `missiles:ID`, and `states:ID`.
After a combat run has saved a scenario, `scenarios:SLUG-SEED` commits that
versioned scenario to `data/scenarios/` without changing the fleet document.

## Calculators

All HTTP calculators use the same Rust functions as the CLI:

```sh
slipstick agent schema gear
slipstick agent calculate gear --input gear.json
slipstick agent schema lidar_pd
slipstick agent calculate lidar_pd --input lidar-scenario.json
slipstick agent schema missile_engagement
slipstick agent calculate missile_engagement --input engagement.json
```

The catalog returned by `capabilities` is authoritative. It covers settings,
drive and travel, auto-sizing, laser profiles, radiators and heat, staged
missiles and intercepts, the System Map navigation layer, and Lidar/PD.

## Map-backed combat

Combat runs only against a draft and never mutate its fleet. Participants must
refer to commissioned ship states. Their starting navigation may come from the
draft System Map or from scenario-local `initial_nav` overrides. Retrieve the
contract with `slipstick agent schema combat`.

```json
{
  "schema_version":"1.0",
  "name":"Escort breaks contact",
  "duration_s":3600,
  "step_s":10,
  "seed":2049,
  "samples":100,
  "objective":"Determine whether Blue can disengage before losing its drive.",
  "initial_nav":{
    "blue-escort":{"x":0,"y":0,"vx":0,"vy":0,"landed_on":null},
    "red-raider":{"x":50000000,"y":0,"vx":-1500,"vy":0,"landed_on":null}
  },
  "participants":[
    {"ship_id":"blue-escort","team":"blue","doctrine":{
      "rules_of_engagement":"return_fire","sensor_range_m":100000000,
      "sensor_cadence_s":10,"missile_range_m":75000000,"missile_salvo":4,
      "defensive_reserve":12,"laser_fire":true,"retreat_integrity":0.35,
      "target_priority":["red-raider"]}},
    {"ship_id":"red-raider","team":"red","doctrine":{
      "rules_of_engagement":"weapons_free","sensor_range_m":100000000,
      "sensor_cadence_s":10,"missile_range_m":75000000,"missile_salvo":4,
      "defensive_reserve":12,"laser_fire":true,"retreat_integrity":0.2,
      "target_priority":["blue-escort"]}}
  ]
}
```

```sh
slipstick agent simulate run --draft DRAFT_ID --input engagement.json
slipstick agent simulate summary --draft DRAFT_ID --run RUN_ID
slipstick agent simulate events --draft DRAFT_ID --run RUN_ID --offset 0 --limit 50
slipstick agent simulate compare --draft DRAFT_ID --run RUN_A --run RUN_B
```

Each run writes `scenario.json`, `summary.json`, `timeline.jsonl`,
`ensemble.json`, and deterministic `report.md`. The ensemble includes outcome
probabilities, detection/fire/hit/kill timing distributions, ammunition use,
and component-loss rates. The representative result records propellant, heat,
flywheel energy, ammunition, tracks, retreat state, and each component's
`intact`, `degraded`, `disabled`, or `destroyed` condition.

The current damage model is deliberately functional: it does not infer armor
facings, internal geometry, fragmentation, or blast propagation. Those remain
explicitly outside this schema version.
