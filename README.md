# Slipstick

Warship design & ops calculator. One Rust binary, single user, local only.

```
cargo run
```

then open <http://localhost:8017>.

Everything persists to `./data/fleet.json` — human-readable, hand-editable,
versionable. Delete it (or edit it) freely; a default fleet is written on
first run: four ship templates, three missile templates, eight hull materials,
and one commissioned battleship.

## Division of labor

- **All physics lives in Rust** in [src/physics.rs](src/physics.rs) and its
  focused submodules, served
  behind `POST /api/calc/*`. One auditable place for every number.
- The server is otherwise a dumb JSON store: `GET`/`PUT /api/data`, whole
  document, written atomically.
- The frontend ([static/app.js](static/app.js)) owns schema and presentation
  and never computes physics — only straight sums of masses and unit
  formatting. Plots are a hand-rolled canvas module
  ([static/plot.js](static/plot.js)). No frameworks, no CDN dependencies;
  works offline. Static files are embedded in the binary.

## Calc endpoints

| Endpoint | Physics |
|---|---|
| `gear` | fixed jet power model: P_jet = P_fusion·f_exh·η_noz; F = 2P_jet/Ve; fuel vs afterburner split; nozzle thrust cap |
| `drive_curve` | thrust and Δv sampled across the gear range |
| `deltav` | Δv = Ve·ln(m_wet/m_floor), m_floor = m_dry·exp(Δv_reserve/Ve) |
| `travel` | analytic flip-and-burn (closed-form burn kinematics, bisection on burn-1 duration, m_flip ≥ √(m₀·m_floor)); reports max reachable distance when infeasible |
| `burn` | timed burn, prograde or retrograde: duration → Δv, end velocity, distance covered; clamps at the reserve floor |
| `sprint` | max-velocity intercept: burn everything above the floor accelerating, then coast — arrive fast, don't stop |
| `autosize` | linear fixed point for dry mass: drive/radiator/tank masses scale with the thrust they must support; reports the accel ceiling honestly when the target is infeasible |
| `laser` | d = 1.22λR/D; Φ = 4P/(πd²); vaporization drilling; per-shot electrical/waste, shots per bank, sink endurance |
| `laser_profiles` | penetration, diffraction spot, irradiance, pulse energy, and fluence vs range; kill and 1.5× open-fire ranges per profile |
| `radiator` | q = ε·σ·T⁴·A × integrity |
| `vent` | lithium venting: 19.6 MJ/kg dumped, capacity permanently scarred at 4.6 MJ/kg |
| `missile` | ordered mixed-propulsion stages with explicit dry/propellant mass, constant per-stage thrust, jettison events, cumulative Δv, and tagged acceleration samples |
| `missile_optimize` | sizes a fusion-heated metallic-hydrogen bus around its carried MH submunitions and searches exhaust velocity for maximum bus Δv, including reactor, radiator, tank, and guidance mass |
| `intercept` | stage-aware burn/coast schedules; enforces ignition order and flags misses and coast stalls |
| `design_report` | all Designer-tab consistency numbers in one call |
| `nav_tick` | system map: bodies on circular rails, ships integrated with symplectic leapfrog under gravity from every body plus active nav burns (with ignition delays); landing/impact detection; timestamped ship *and* body paths for course projection and local-frame views |
| `orbit_v` | circular/escape velocity and period at radius r, for placing ships |
| `burn_for_dv` | inverse rocket equation: burn time and propellant for a requested Δv, clamped at the floor |
| `nav_intercept` | Terra-Invicta-style planner: impulsive-burn search over departure time × heading × Δv (≤ budget) against the target's future position, integrated under full gravity from the current epoch, refined, and converted to a finite burn |
| `lidar_pd` | deterministic single-epoch lidar photon budget through jammer/chaff contamination, detector state, centroid bias, causal fire-control propagation, capture probability, and point-defense snapshot feasibility |

## Lidar & point defense

The **Lidar & PD** tab is a self-contained scenario lab. Its versioned JSON
request uses SI units and is sent to `POST /api/calc/lidar_pd` with a 1 MiB
request limit. Scenarios do not enter `fleet.json`; use the page's import and
export actions to retain them. Results can also be exported with the complete
geometry, signal, jammer, chaff, detector, fire-control, and point-defense
audit trail.

Built-in detector, target, and weapon presets follow the v1 specification.
Installed design lasers can seed the weapon aperture, wavelength, and optical
power without replacing the scenario's beam-quality, duty-cycle, or service
assumptions. When the System Map has a mapped source ship and selected target,
**Use map geometry** copies the current range and positive radial closing speed
into the scenario as an editable snapshot.

Every editable field and displayed result has a keyboard-accessible `?`
explanation written in plain English. Tooltip text identifies the value's
meaning, units, expected direction of effect, and whether it is calculated,
assumed, or approximated. The tooltip registry is centralized in
`static/app.js`; a rendered field without metadata produces a visible coverage
warning and is exposed through `window.__LP_TOOLTIP_COVERAGE__` for browser
tests.

The v1 verdict is intentionally a constant-range snapshot. It does not
time-step closure, detector recovery, target maneuver history, chaff evolution,
or multi-target weapon service rate. Speckle diversity, diffuse return,
rectangular chaff scattering, and equivalent-isotropic capture are explicitly
labeled engineering assumptions or approximations in the audit output.

Missiles store a payload plus ordered stages. Each stage chooses metallic
hydrogen, antimatter thermal at an ISP tier, fusion bus, or a custom exhaust
velocity and carries explicit dry/propellant mass, ignition acceleration, and
jettison behavior. Laser wavelength,
radiator temperature, and pulse lengths are likewise option selects with a
custom escape hatch. Defaults for all of them live in `settings`, as do the
auto-size scaling parameters (`as_*`: reactor t/TW, radiator MW/kg, nozzle
t/MN, structure fraction, sink endurance minutes, flywheel fire seconds).
Unmapped ship states track a scalar velocity ledger. Once placed, the System
Map's 2D vector is authoritative and Drive & Travel programs the same nav-burn
state; the scalar value remains a derived compatibility readout.

The System Map tab holds a three-pane 2D planning workspace (default: Sol with the eight planets, Luna,
and the four Galilean moons on circular rails; bodies are add/edit/delete-able)
with a selectable reference frame — heliocentric or centered on any body, with
positions, velocities, and projected courses re-expressed relative to it.
Ships are placed from the fleet — landed, in circular orbit, or in deep
space — and advance by steps or continuous play. A map-selected source and
target share gear and reserve state with Drive & Travel, expose analytic
transfer comparisons, and feed the full-gravity planner. Nav burns (gear Ve +
direction + duration + optional ignition delay) thrust through ticks, draw
live propellant, and log one event when they end. Click a ship's projected
course to drop a KSP-style maneuver node and drag its ring to aim: heading and
Δv convert to a finite burn centred on the node, previewed live and committed
to the same burn machinery. The intercept planner searches departure windows
under full gravity for a burn (within a Δv budget) that reaches a body or
another ship, and loads the answer into the node planner. The sim state
(`system`: epoch, bodies, per-ship position/velocity) persists in `fleet.json`
like everything else.

Canon constants (f_exh = 0.753, η_noz = 0.85, Ve_max = 2,300 km/s, …) live in
`settings` in `fleet.json` and are editable from the ⚙ button — nothing is
hardcoded into the formulas.

## Tests

```
cargo test
```

After a live Lidar & PD response has been written to JSON, tooltip coverage can
also be checked with:

```
node scripts/check_lidar_tooltips.cjs /tmp/lidar_pd_response.json
```

covers the canon consistency check (1.82 MW fusion at Ve = 2,300 km/s →
1.013 N, 0.44 mg/s, within 2%), the MH-164 Asmodeus Δv (50 km/s), agreement
of the analytic travel solver with brute-force numeric integration, the
Lasers.md installed-systems kill table (BB main battery: 30 m / 10 GW /
200 nm / 0.01 s / 1 cm Ti-C → ~25 Mm), the three-phase intercept doctrine,
auto-size feasibility honesty (5 mg at MR 8 is over the ceiling; MR 4 works),
timed burns, sprint intercepts, vent scarring, and hit/miss/stall cases.

## Known issue (by design)

The research report is internally inconsistent on battleship cruise: 16 TW
gives ~1.2 milligee at 720 kt wet, rising to ~10 mg near dry. The calculator
computes honestly and the battleship template carries a note saying so.

## Deferred to v2

Evasion/miss-distance modeling, multi-ship task force views, drone endurance
tracking, particle beams, launch-rail energy bookkeeping beyond flywheel draw.
