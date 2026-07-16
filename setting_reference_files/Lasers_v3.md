# Laser Systems Reference

## Architecture

532nm (green). Phased segmented array. Standardized 10 MW laser modules, each with its own mirror segment and independent cooling. Modules phase-lock via piston/tip/tilt actuators (±27nm path control at 532nm) to produce the diffraction pattern of the full aperture.

Each module: diode-pumped Nd:YAG at 1064nm, single-stage SHG to 532nm, feeding one hexagonal mirror segment. Modules organized into task groups for independent targeting.

No shared transmit optical element. Each outgoing beam touches only its own segment. The combining is far-field interference, not upstream beam merging.

In receive mode, each dark segment acts as an independent telescope with a coherent heterodyne channel. Phase, amplitude, Doppler, polarization, and timing are combined digitally using the segment-metrology solution. This provides a synthetic receive aperture without inventing a common focal plane that the transmit architecture does not have.

## Key Parameters

| Parameter | Value |
|-----------|-------|
| Wavelength | 532 nm |
| Module optical output | 10 MW |
| Wall-plug efficiency | 15% |
| Duty cycle | 0.5 |
| Airy central lobe | 84% of emitted power |
| Listed power | Total optical output (all modules at peak) |
| Beam-on-target | 84% of listed power |
| Average beam | 42% of listed power (duty × Airy) |

The 16% Airy ring loss is not compensated in the design spec. A ship listed at 1 GW delivers 840 MW to the central lobe at peak, 420 MW average.

## Installed Systems

| Platform | Aperture | Modules | Listed | Beam (peak) | Beam (avg) | Groups |
|----------|----------|---------|--------|-------------|------------|--------|
| BB main battery | 30m | 100 | 1 GW | 840 MW | 420 MW | 10 × 10 |
| BB secondary (×6+) | 10m | 20 | 200 MW | 168 MW | 84 MW | 4 × 5 |
| Corvette spinal | 15m | 50 | 500 MW | 420 MW | 210 MW | 5 × 10 |
| Standard PDL | 3m | 15 | 150 MW | 126 MW | 63 MW | 3 × 5 |
| Drone PDL | ~1m | 4 | 40 MW | 34 MW | 17 MW | 1 × 4 |

All platforms use the standard 10 MW module. Aperture sizes unchanged from prior spec.

## Segment Geometry

For a regular hexagon, **A = (3√3/2)s²**, flat-to-flat = **√3s**, and vertex-to-vertex = **2s**. Earlier dimensions accidentally listed values close to the side length as flat-to-flat.

| Platform | Total Area | Per Segment | Hex F-to-F | Hex V-to-V | Group Equivalent Aperture |
|----------|-----------|-------------|-----------|-----------|---------------------------|
| BB main (30m) | 707 m² | 7.07 m² | 2.86 m | 3.30 m | 9.5 m (10 segs) |
| BB secondary (10m) | 78.5 m² | 3.93 m² | 2.13 m | 2.46 m | 5.0 m (5 segs) |
| Corvette (15m) | 177 m² | 3.54 m² | 2.02 m | 2.33 m | 6.7 m (10 segs) |
| Standard PDL (3m) | 7.07 m² | 0.471 m² | 0.738 m | 0.852 m | 1.7 m (5 segs) |
| Drone PDL (1m) | 0.79 m² | 0.198 m² | 0.478 m | 0.551 m | — |

The group aperture is the equivalent filled circular diameter from total group area, used for first-order diffraction calculations. Actual grouped segments are sparse subsets of the parent aperture; beam shape, sidelobes, and receive geometry depend on which physical segments are assigned.

Inter-segment gap: ~5 mm actuator clearance throughout. Gap-induced far-field loss remains ~1–2% for close-packed geometry and the adopted element pattern [ENG].

## Diffraction and Flux

Effective central-lobe diameter: **d = 1.22 × 532 nm × R / D**

The tabulated flux is central-lobe power divided by the area inside this effective diameter. It is a conservative **mean central-lobe flux**, not the on-axis spatial maximum of the Airy pattern. The time-domain fluence model uses the actual segmented-array intensity distribution.

### BB Main — Full Array (30 m, 840 MW in Central Lobe While On)

| Range | Spot Ø | Mean Central-Lobe Flux, Beam On | Time-Averaged Mean Central-Lobe Flux |
|-------|--------|-----------|----------|
| 500 km | 10.8 mm | 9,150 GW/m² | 4,570 GW/m² |
| 1,000 km | 21.6 mm | 2,290 GW/m² | 1,140 GW/m² |
| 2,800 km | 60.6 mm | 292 GW/m² | 146 GW/m² |
| 5,000 km | 108 mm | 91.5 GW/m² | 45.8 GW/m² |
| 10,000 km | 216 mm | 22.9 GW/m² | 11.5 GW/m² |
| 50,000 km | 1.08 m | 917 MW/m² | 459 MW/m² |
| 100,000 km | 2.16 m | 229 MW/m² | 115 MW/m² |
| 500,000 km | 10.8 m | 9.17 MW/m² | 4.59 MW/m² |
| 1 Gm | 21.6 m | 2.29 MW/m² | 1.15 MW/m² |

## Missile Kill Model

### Scope and Reference Threat

This section is the clean fire-control baseline. It assumes the defender has identified the real target, maintains an unobscured line of sight, and receives usable tracking data. Jamming, false tracks, decoys, sensor dazzling, plume obscuration, and kill-assessment errors are not included here. Those effects can be added later by degrading the tracking terms defined below.

Reference threat: fusion-bus submunitions arriving at 120–150 km/s (nominal 130 km/s) with 36 km/s own delta-v. Maximum acceleration ~100g (981 m/s²). Nominal lateral maneuver authority is ~50%, giving **a_lat ≈ 490 m/s²**. This is the missile's physical lateral capability, not automatically the defender's prediction error. Casaba standoff: 50 km (torplet) to 300 km (large warhead).

The missile's maneuver-command persistence, **T_j**, is independent of range and light delay. It is the average time a commanded acceleration vector remains valid before the missile changes it. The correct value depends on actuator bandwidth, control law, propellant economy, and terminal doctrine. It is a scenario variable, not yet a fixed missile specification.

### Kill Threshold

Structural kill: 1cm penetration of Ti-C hybrid armor. Requires ~1 GJ/m² deposited (includes ~50% coupling loss from reflectivity, re-radiation, and conduction). Guidance/sensor soft kill at roughly 10× lower fluence.

### Observation-to-Effect Delay

The relevant fire-control delay is not one-way light time. The defender must receive the observation, calculate an aimpoint, and send the beam back to the target:

**τ = 2R/c + t_proc**

Where:

- **R** = current range
- **c** = speed of light
- **t_proc** = sensor integration, fire-control computation, and beam-control latency

For the clean baseline, t_proc is treated as negligible unless otherwise stated. It remains in the model so later sensor and hardware work has somewhere to live.

### Fire-Control Input Contract

The sensor network supplies a target package at the proposed beam-arrival epoch rather than one scalar “tracking probability”:

- predicted target state **x̂, v̂, â**;
- two-dimensional target-plane covariance **Σ_target**;
- track-association probability **p_assoc**;
- projected silhouette **S(x)** and aspect uncertainty;
- vulnerable-component / damage-cell map **V(x)**;
- firing-platform boresight and jitter angular covariance **Σ_boresight**, projected into the target plane at range **R**;
- weighted EWAR hypotheses and post-shot kill confidence.

The weapon combines the target and platform terms:

**Σ_effect = Σ_target + Σ_boresight**

with correlations and coordinate transforms retained in the actual filter. The laser owns aperture, beam shape, raster pattern, power allocation, and service scheduling. The sensor model owns the probability distribution. This prevents both documents from maintaining slightly different private religions about **P_track**.

### Body Intercept and Patch Retention

Diffraction-limited effective central-lobe diameter:

**d = 1.22λR/D**

For an instantaneous aim error **e**, the fraction of central-lobe energy intersecting the projected missile is:

**f_body(e) = ∫_S I(x − e) dx / ∫ I(x) dx**

The expected body-intercept fraction is the convolution over the target-plane error distribution:

**f̄_body = E_e[f_body(e)]**

Structural penetration also requires energy to accumulate in one damage cell. Over a beam-on interval **T**, define:

**E_lobe = P_lobe T**

**E_body = ∫₀ᵀ ∫_S I(x − e(t),t) dx dt**

**E_cell,max = max_c ∫₀ᵀ ∫_{A_c} I(x − e(t),t) dx dt**

where **A_c** is a candidate damage cell chosen large enough to contain the clean reference spot. Then:

**f̄_body = E_body/E_lobe**

**f_patch = E_cell,max/E_body**

Both factors are dimensionless and approach one for a clean centered dwell. A missile can therefore have **f̄_body ≈ 1** while **f_patch ≪ 1** if the spot walks across its skin.

For quick doctrine estimates only, two overlap radii may be used:

- **r_body,eff ≈ r_target + d/2** for any significant body overlap;
- **r_patch,eff ≈ r_damage + d/2** for overlap with the intended damage cell.

These are not hard physical boundaries. The Airy/segmented beam convolution is the real calculation.

Fire control predicts acceleration **â** while the missile executes **a**. The residual acceleration error is:

**Δa = a − â**

For scalar doctrine calculations:

**a_u = |Δa|**

If residual acceleration is approximately constant over a short interval, aimpoint error grows as:

**x_error(t) = ½a_u t²**

The corresponding approximate loss time for body or patch overlap is:

**t_loss,j = √(2r_j,eff/a_u)**, where **j ∈ {body, patch}**

If prediction is exact over the interval, **a_u** approaches zero and maneuver-induced loss disappears. Ordinary covariance, boresight, association, and beam errors remain.

### Resolved-Maneuver Approximation

When the missile holds each maneuver command long enough for the defender to observe and correct it:

**T_j ≥ τ**

For either body overlap or patch overlap, the binary-overlap approximation gives:

**P_j = 1 − max(0, τ − t_loss,j) / T_j**

clipped to 0–1. In this approximation **P_body** estimates time with useful power on the missile, while **P_patch** estimates time with useful power on the intended damage cell. They are not interchangeable.

The previous dwell model is the special point-patch case obtained by setting **T_j = τ**, **r_damage → 0**, and **a_u = a_lat**:

**P_patch = √(d/a_lat)/(2R/c)**

It never described whole-body capture for a missile wider than the beam.

### Point-Patch Continuous-Hold Range

For a negligible damage-cell radius, continuous patch hold occurs when the residual motion cannot cross half the beam footprint before correction arrives:

**R_crit,patch = 1.22λc² / (4Da_u)**

| Platform | R_crit,patch, f_u = 1.0 | f_u = 0.5 | f_u = 0.25 |
|----------|-------------------------|-----------|------------|
| BB main (30 m) | 1,000 km | 2,000 km | 4,000 km |
| BB secondary (10 m) | 3,000 km | 6,000 km | 12,000 km |
| Corvette (15 m) | 2,000 km | 4,000 km | 8,000 km |
| Standard PDL (3 m) | 10,000 km | 20,000 km | 40,000 km |
| Drone PDL (1 m) | 30,000 km | 60,000 km | 120,000 km |

Here **f_u = a_u/a_lat**. This table is a patch-hold diagnostic, not a body-hit envelope and not an energy-range table. Whole-body overlap generally persists much farther because the missile silhouette is larger than the diffraction spot at point-defense range.

Smaller effective apertures produce wider spots and tolerate larger prediction error; larger apertures concentrate energy more strongly. The weapon may deliberately choose an intermediate aperture rather than accepting either extreme.

### Maneuver Prediction and Residual Error

Prediction does not directly return a percentage of lost dwell. It reduces the residual acceleration error:

**a_u = f_u × a_lat**

Where **f_u** is the residual maneuver fraction:

- **f_u = 1.0:** no useful prediction; the full lateral maneuver is uncertain
- **f_u = 0.5:** prediction removes half the acceleration error
- **f_u = 0.25:** strong prediction; only one quarter remains uncertain
- **f_u = 0:** exact future command knowledge over the interval

Because **t_loss ∝ 1/√a_u**, cutting maneuver uncertainty in half improves dwell by √2, not by a flat additive percentage.

Provisional scenario ranges for clean-fire-control modeling:

| Scenario | Residual fraction f_u | Notes |
|----------|-----------------------|-------|
| Unknown missile family, opening contact | 0.8–1.0 | Thrust envelope known; control law and priorities poorly known |
| Known missile family | 0.6–0.8 | Actuator limits, doctrine, and common maneuver choices available |
| Observed control law, mid-engagement | 0.4–0.6 | Behavior and objective weighting increasingly constrained |
| Salvo coordination inferred | 0.3–0.5 | Collision avoidance and shared geometry reduce independent choices |
| Damaged, fuel-limited, or firing-geometry constrained | 0.2–0.4 | Available maneuvers occupy a much smaller solution space |

These are model inputs, not validated performance claims.

Prediction improves when fire control knows or estimates:

- missile thrust and actuator limits;
- remaining propellant and thermal margin;
- warhead firing geometry and pointing requirements;
- sensor orientation requirements;
- collision-avoidance constraints inside a dense salvo;
- manufacturer control-law architecture;
- salvo-level coordination objectives;
- damage to specific thrusters or control surfaces;
- the missile's balance between lateral displacement and preserving terminal delta-v.

Prediction worsens when missiles have excess maneuver reserve, independently varied control laws, hidden sensor inputs, many equally useful approach paths, or deliberately accept inefficient maneuvers to break the defender's model.

A secure pseudorandom generator does not require fresh quantum entropy for every maneuver. A small secret seed can generate a long sequence that is computationally unpredictable, including against quantum search when the seed is sufficiently large. Fire control therefore does not rely on casually cracking missile random-number generators. It predicts the constrained physical and tactical consequences of the missile's choices. Randomness prevents exact command prediction; it does not remove thrust limits, fuel limits, collision geometry, or mission requirements.

Human weapons officers remain useful because they can recognize intent, deception, doctrinal habits, and deliberately irrational sacrifices that a narrow fire-control SAI may weight poorly.

### Candidate-Maneuver Model

A detailed filter carries several possible future maneuvers rather than one scalar prediction. For candidate maneuver **i**:

**Δa_i = a_i − â**

**t_loss,i,j = √(2r_j,eff/|Δa_i|)**

**P_i,j = 1 − max(0, τ − t_loss,i,j)/T_j**

for **j ∈ {body, patch}**. If candidate **i** has probability **p_i**:

**P_body = Σ p_i P_i,body**

**P_patch = Σ p_i P_i,patch**

These binary factors remain doctrine approximations. The coupled simulation uses the full target-plane covariance and beam convolution.

### Unresolved-Maneuver Regime

When:

**T_j < τ**

several maneuver decisions are hidden inside the observation-to-effect delay:

**N_hidden ≈ τ/T_j**

The resolved-jink equations no longer close the problem. The sensor/fire-control system propagates a future target-plane covariance **Σ_target**. A useful scalar bookkeeping model is:

**σ_total² = σ_track² + (τσ_v)² + σ_maneuver² + (Rσ_θ)²**

For independent piecewise-constant residual acceleration changes with one-axis RMS **σ_a** and **τ ≫ T_j**:

**σ_maneuver² ≈ σ_a² T_j τ³ / 3**

This scalar form is useful for sanity checks. The weapon does not turn it into a single hard capture circle. It convolves **Σ_effect** with the target silhouette and selected beam pattern to obtain **f̄_body** and the time-domain patch fluence.

### Uncertainty-Matched Fire

The defender does not always use the narrowest available beam. When prediction covariance is larger than the diffraction footprint and excess flux exists, the weapon can trade concentration for robustness.

Available methods:

- use a smaller effective aperture;
- phase-taper or deliberately defocus the array;
- raster the beam across the covariance ellipse;
- split groups across adjacent aim hypotheses;
- hold one group on the nominal target while others probe likely offsets.

For a desired effective beam radius **r_beam** at range **R**, a first-order aperture choice is:

**D_eff ≈ 0.61λR/r_beam**

The control law chooses **r_beam** from **Σ_effect**, target size, and available flux. A common starting point is to cover a selected probability contour of the target distribution while retaining enough mean flux to reach kill fluence before standoff.

Broadening usually raises **f̄_body** and lowers peak patch heating. EWAR therefore increases the time and energy spent servicing a target rather than producing an automatic clean miss.

### Illustrative Resolved Patch-Hold Example — BB Main

This is an example, not a missile specification. It assumes **T_j = 20 ms**, **t_proc = 0**, a point-like intended damage cell, and no EWAR. At ranges where **τ > 20 ms**, the maneuver becomes unresolved and the table stops pretending one neat percentage solves it.

| Range | τ | P_patch (f_u = 1.0) | P_patch (f_u = 0.5) | P_patch (f_u = 0.25) |
|-------|---|-----------------------|-----------------------|------------------------|
| 1,000 km | 6.7 ms | 99.9% | 100% | 100% |
| 2,000 km | 13.3 ms | 80.3% | 99.7% | 100% |
| 3,000 km | 20.0 ms | 57.5% | 81.3% | 100% |
| Above ~3,000 km | τ > T_j | Unresolved covariance model required | Unresolved covariance model required | Unresolved covariance model required |

Whole-body overlap is substantially better than this table when the missile body is wider than the beam.

### Kill Time, Service Time, and Kill Range

Clean structural dwell on one patch is:

**t_dwell,clean = E_kill / Φ_mean**

where **Φ_mean** is the mean central-lobe flux while the beam is on. Under uncertainty:

**t_dwell,eff ≈ t_dwell,clean / (f̄_body f_patch)**

Target service time is:

**t_service = t_steer + t_settle + t_dwell,eff + t_confirm + t_reassign + t_reacq**

The missile remains available from current range to Casaba standoff:

**t_flight = (R − r_Casaba) / v_closure**

Flux rises as **1/R²** during closure. The real capacity calculation therefore integrates target service across the trajectory rather than evaluating one range:

**N_kill ≈ ∫ dt / t_service(R)**

A conventional missile with a maintained body track inside the relevant energy envelope is unlikely to survive a dedicated beam indefinitely. The tactical question is how much service time each target consumes and how many targets demand service at once.

### Service-Time Budget

The previous model called a 50 ms aggregate delay “slew.” That hid several different processes:

| Term | Meaning |
|---|---|
| **t_steer** | Electronic phase command and coarse repoint |
| **t_settle** | Segment actuator, structure, and boresight stabilization |
| **t_dwell,eff** | Actual damaging illumination |
| **t_confirm** | Determine whether the target remains threatening |
| **t_reassign** | Select and validate the next target/beam mode |
| **t_reacq** | Additional delay after dropout or false-track handoff |

Close-range capacity may be dominated by confirmation rather than mirror motion. The tables below retain **50 ms of aggregate non-dwell service latency** only as an illustrative scenario, not a hardware constant.

### Illustrative Service-Limited Rate — BB Main Full Array

Assumptions: **T_j = 20 ms**, **f_u = 0.5**, no EWAR, point-patch approximation, 1 GJ/m² structural threshold, and **50 ms aggregate non-dwell service latency**.

| Range | Mean Lobe Flux, Beam On | P_patch | Effective Dwell | Non-Dwell Service | Total Service | Illustrative Rate |
|-------|--------------------------|---------|-----------------|-------------------|---------------|-------------------|
| 1,000 km | 2,290 GW/m² | 100% | 0.4 ms | 50 ms | 50.4 ms | ~20/s |
| 2,000 km | 572 GW/m² | 99.7% | 1.8 ms | 50 ms | 51.8 ms | ~19/s |
| 3,000 km | 254 GW/m² | 81.3% | 4.8 ms | 50 ms | 54.8 ms | ~18/s |

These are scheduling examples. Faster kill confirmation raises them; EWAR, patch wandering, false associations, and multi-bearing load lower them.

### Illustrative Inner-Defense Rates

Inside each platform's point-patch continuous-hold range, with a clean association and the same 50 ms non-dwell assumption:

| Platform | Engagement Range | Mean Lobe Flux, Beam On | Clean Dwell | Total Service | Illustrative Rate |
|----------|------------------|-------------------------|-------------|---------------|-------------------|
| BB secondary (10 m) | 500 km | 203 GW/m² | 4.9 ms | 54.9 ms | 18/s |
| Corvette (15 m) | 1,000 km | 286 GW/m² | 3.5 ms | 53.5 ms | 19/s |
| Standard PDL (3 m) | 100 km | 343 GW/m² | 2.9 ms | 52.9 ms | 19/s |
| Drone PDL (1 m) | 20 km | 257 GW/m² | 3.9 ms | 53.9 ms | 19/s |

The convergence near 18–20/s is created by the assumed service latency, not by a deep law of laser combat.

The following capacities are retained only as one scheduling scenario. They are not canonical platform limits and do not include outer unresolved-regime fire.

| Platform | Illustrative Start | Inner Limit | Window at 130 km/s | Capacity with 50 ms Non-Dwell Service |
|----------|--------------------|-------------|--------------------|---------------------------------------|
| BB main (full) | ~1,000 km | 300 km | 5.4 s | ~105 |
| Corvette | ~2,000 km | 300 km | 13.1 s | ~240 |
| BB secondary | ~1,500 km | 300 km | 9.2 s | ~170 |
| Standard PDL | ~300 km | 50 km | 1.9 s | ~36 |
| Drone PDL | ~50 km | 50 km | ~0 s | ~0 |

Drone PDLs mounted on the defended hull remain ineffective against 130 km/s torplets because their useful energy window is nearly identical to a 50 km Casaba standoff. Forward deployment is required.

## Task Group Operations

**Concentrated fire.** All modules phase-locked, full aperture, one target. Maximum concentration and best anti-ship performance. Against missiles, use it when the covariance footprint is small enough that the extra concentration shortens service time.

**Distributed fire.** Groups tasked independently. Up to 10 (BB main), 5 (corvette), 4 (BB secondary), or 3 (PDL) simultaneous targets. Each group uses its own sub-aperture. Per-target flux falls while total target throughput rises.

**Staggered fire.** Some groups fire while others receive, cool, or charge. This provides continuous weapon output and a sparse coherent receive aperture, but both beam and sensor geometry vary with the schedule.

**Uncertainty-matched fire.** Groups or the full array are broadened, tapered, rastered, or split across hypotheses to match the predicted target-plane covariance. This is often preferable to firing the narrowest possible beam into empty probability space.

**Transition doctrine.** Use the beam pattern and group count that minimize expected target service time, not the configuration with the most impressive instantaneous flux. As the salvo closes and covariance shrinks, transition from broad or rastered concentration to narrow sequential fire, then to distributed terminal defense.

### Sensor/Weapon Scheduling

| Weapon schedule | Fire-control consequence |
|---|---|
| All groups fire together | Maximum transmit concentration; precision tracking relies on dedicated telescopes or stolen segments |
| All groups dark together | Full 30 m coherent receive update; no weapon output during the update |
| Half fire / half receive | Continuous output with roughly half collecting area on a sparse 30 m baseline |
| Groups fire asynchronously | Smooth power draw and target service; receive covariance becomes time-varying and anisotropic |
| Randomized schedule | Harder for missiles to time jinks; more complex flywheel, thermal, and filter management |

The scheduler explicitly weighs another interval of weapon output against the value of a large-aperture measurement that may shorten several later target services.

### BB Main Group Performance (9.5 m Equivalent Aperture, 84 MW Central-Lobe Power)

| Range | Mean Lobe Flux, Beam On | Point-Patch Factor | Effective Dwell | Illustrative Rate with 50 ms Non-Dwell Service |
|-------|--------------------------|--------------------|-----------------|------------------------------------------------|
| 270 km | 314 GW/m² | 100% | 3.2 ms | 19/s |
| 500 km | 91.5 GW/m² | 100% | 10.9 ms | 16/s |
| 1,000 km | 22.9 GW/m² | 100% | 43.7 ms | 11/s |

Group point-patch **R_crit** is ~3,200 km with no useful maneuver model and scales inversely with residual acceleration error. In distributed mode, ten groups at 500 km have an illustrative scheduling upper bound near 160 kills/s under the same 50 ms assumption. Target association, body/patch efficiency, receiver scheduling, and EWAR decide how much of that upper bound exists outside the table.

## Anti-Ship Profile

Against ships, dwell is unlimited — the target is large and predictable. Average flux is the relevant metric for sustained radiator damage.

**Radiator break-even.** Refractory radiator panels operating at 2,500K emit ~2.1 MW/m² via Stefan-Boltzmann. The BB main's average beam must exceed this to cause net heating. Break-even range: ~740,000 km.

| Range | Time-Averaged Mean Lobe Flux | Net Heating | Effect |
|-------|----------|-------------|--------|
| 50,000 km | 459 MW/m² | 457 MW/m² | Rapid vaporization |
| 100,000 km | 115 MW/m² | 113 MW/m² | Ablation in seconds |
| 300,000 km | 12.7 MW/m² | 10.6 MW/m² | Sustained degradation |
| 500,000 km | 4.59 MW/m² | 2.49 MW/m² | Slow degradation |
| 740,000 km | 2.1 MW/m² | ~0 | Break-even |

Colder targets (sensors, optics, antenna structures) are vulnerable at significantly longer range.

Practical anti-ship envelope: meaningful radiator ablation inside ~300,000 km; rapid structural damage inside ~100,000 km.

## Engagement Sequence

Against an incoming salvo, the laser changes beam shape, group count, and receive schedule as geometry and covariance change. This is an optical baseline, not a complete PD doctrine.

**Phase 1 — Outer intercept (~10,000–3,000 km).** The BB main uses uncertainty-matched concentrated fire: full power, but not necessarily the full diffraction-limited aperture. The scheduler broadens or rasters the beam over the predicted covariance when that produces more expected body fluence than a narrow shot. Full-dark or sparse-aperture receive intervals update the solution. Counter-missiles supplement this layer.

**Phase 2 — Midcourse transition (~3,000–500 km).** Individual maneuvers become increasingly resolved and the target covariance contracts. The main battery transitions toward narrow sequential fire. Corvettes and BB secondaries add independent bearings, receive apertures, and weapon channels. Destroying those pickets degrades both gun count and the battleship's fire-control package.

**Phase 3 — Terminal (500–50 km).** The BB main shifts to distributed groups against separated torplets. At 500 km, ten cleanly associated groups have an illustrative scheduling upper bound near 160 kills/s under the 50 ms non-dwell assumption. BB secondaries overlap this layer. Standard PDLs engage leakers inside ~300 km, with only ~1.9 seconds before a 50 km torplet Casaba standoff at 130 km/s.

**Saturation threshold.** No single canonical salvo threshold is assigned. The oft-quoted ~105 BB-main kills between 1,000 km and a 300 km standoff is only the 50 ms service-latency scenario. Actual capacity depends on the sensor target package, uncertainty-matched beam choice, body and patch fluence, kill confirmation, receive scheduling, EWAR, multi-bearing geometry, and battle damage.

The first salvo is usually hardest to constrain because fire control has little data on the missile family's control laws and coordination. Later salvos improve model fitting, while secure random variation prevents exact command prediction. The defender learns physics and doctrine, not the secret seed by wishing harder.

## Thermal Management

### BB Main Battery Heat Budget

| Parameter | Peak | Average (50% duty) |
|-----------|------|---------------------|
| Optical output | 1 GW | 500 MW |
| Electrical input | 6.67 GW | 3.33 GW |
| Laser waste heat | 5.67 GW | 2.83 GW |

Reactor (40% thermal-to-electrical) at average load:

| Parameter | Value |
|-----------|-------|
| Reactor thermal | 8.33 GW |
| Reactor waste heat | 5.0 GW at 2,600K |
| HT radiator area | 2,050 m² |

High-temp refractory radiators stay deployed in combat — compact, tough, small target.

### Low-Temperature Laser Cooling

Laser components reject heat at 600–800K. At 700K: 12.8 kW/m².

| Mode | Waste Heat | LT Radiator Area |
|------|-----------|-------------------|
| Sustained average | 2.83 GW | 221,000 m² (~470m × 470m) |
| Sustained at 800K | 2.83 GW | 130,000 m² (~360m × 360m) |

These low-temperature radiator arrays are the lasestar's defining structural feature and primary vulnerability.

### Combat Endurance (Radiators Retracted)

With LT radiators retracted behind armor, laser waste heat goes to lithium phase-change heat sinks (432 kJ/kg, melts at 453K).

| Heat Sink Mass | Duration at 2.83 GW avg |
|---------------|------------------------|
| 500 tonnes | 76 seconds |
| 2,000 tonnes | 5.1 minutes |
| 5,000 tonnes | 12.7 minutes |
| 10,000 tonnes | 25.4 minutes |

After heat sink saturation: reduce duty cycle, deploy LT radiators (accepting vulnerability), or cease fire.

### Energy Storage

Reactor runs at average load. Flywheel banks buffer the peak-to-average difference (3.33 GW) over the duty cycle period.

| Cycle Period | Buffer Energy | Flywheel Mass (4,000 Wh/kg) |
|-------------|---------------|------------------------------|
| 0.1s on / 0.1s off | 333 MJ | 23 tonnes |
| 1s / 1s | 3.33 GJ | 231 tonnes |
| 10s / 10s | 33.3 GJ | 2,310 tonnes |

Shorter cycle periods require less flywheel mass. Rapid pulsing (sub-second) is favored. Internal power transfer via HTS superconducting bus at ~150K.

## Graceful Degradation

| Modules Lost | Remaining Power | Beam Quality | Status |
|-------------|----------------|--------------|--------|
| 10 | 90% | Negligible degradation | Full combat effective |
| 25 | 75% | Minor sidelobe increase | Reduced range, fully functional |
| 50 | 50% | Noticeable degradation | 420 MW beam, reduced kill rate |
| 75 | 25% | Poor beam quality | Short-range PD only |
| 90 | 10% | Severely degraded | Marginal capability |

Battle damage typically kills modules by destroying their local cooling first. The weapon erodes with the ship's radiators rather than failing catastrophically.

## Open Questions

- **Time-domain fluence model:** actual segmented-array intensity, projected missile body, vulnerable-component map, correlated aim wander, cooling, and ablation.
- **Fire-control input contract:** implement the common **Σ_target / p_assoc / silhouette / vulnerability / Σ_boresight / C_kill** package in the simulation.
- **Service-time budget:** measure or select **t_steer, t_settle, t_confirm, t_reassign, and t_reacq** instead of hiding them inside 50 ms.
- **Missile maneuver-command persistence T_j:** actuator bandwidth and control doctrine determine the resolved/unresolved boundary.
- **Residual maneuver model:** establish **f_u** and correlated three-axis maneuver spectra for specific missile families.
- **Uncertainty-matched fire:** optimize aperture, phase taper, raster pattern, and multi-hypothesis group allocation against covariance and remaining flight time.
- **Sparse coherent receive:** quantify angular precision, speckle averaging, and sidelobes for all-dark, half-dark, asynchronous, and battle-damaged segment layouts.
- **Processing latency t_proc:** sensor integration, coherent combination, computation, actuator response, and waveform timing.
- **EWAR degradation:** how jamming, false tracks, dazzling, chaff, and target-association errors alter the canonical target package.
- **Kill assessment:** ablation feedback, radar/IR consistency, damaged-warhead behavior, and the confidence threshold for leaving a target.
- **Module mass breakdown and total weapon-system mass.**
- **Radiator material ablation rates:** tungsten, hafnium carbide, coatings, and coolant-channel failure.
- **Multi-bearing saturation:** group allocation and receive-aperture loss against simultaneous threat axes.
- **Salvo cross-correlation:** how much shared control-law architecture constrains independently randomized missiles.
- **Delta-v tax:** how continuous evasion reduces terminal acceleration, standoff geometry, and warhead pointing.
- **Wavelength-selective missile coatings:** UV absorption/roughness, green reflectivity, radar treatment, contamination, and ablation durability.
