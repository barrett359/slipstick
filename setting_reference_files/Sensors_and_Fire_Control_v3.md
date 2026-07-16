# Sensors and Fire Control Reference

## The Fire-Control Problem

The BB main battery's effective central-lobe spot at 5,000 km is ~108 mm diameter under the convention used in the laser kill model. Merely placing the beam center near a maneuvering missile is not enough. Fire control must:

1. estimate the target state,
2. predict through the light-time delay,
3. know the outgoing weapon boresight,
4. place useful power on the target silhouette, and
5. hold enough fluence on one vulnerable patch to produce a kill.

The nominal angular scale remains **~10–20 nrad**, but that number is an end-to-end weapon-pointing requirement, not a detector-centroid requirement.

Vacuum removes atmospheric seeing, scintillation, and absorption [HARD]. It allows a well-built sensor to approach its diffraction limit. Practical accuracy remains limited by photon statistics, wavefront error, detector response, structural flexure, vibration, boresight calibration, target glint, speckle, and prediction error [HARD/ENG].

### Optical Convention Used in This Document

To remain consistent with the laser kill model, spot sizes use an **effective central-lobe diameter**:

**d_eff = 1.22 λR / D**

This is a modelling convention for the useful central lobe, not the full Airy-disc diameter. Radius, diameter, and encircled-energy conventions must not be mixed inside one calculation. Where a Gaussian beam model is required for deposited fluence, the conversion is stated explicitly.

## Sensor Chain Overview

Five sensor modalities, each with a distinct role. None is sufficient alone. Together they feed a common state estimator and weapon-pointing solution.

| Sensor | Provides | Role |
|--------|----------|------|
| Thermal / IR (passive) | Detection, coarse bearing, plume state | Search and maneuver warning |
| Radar (active) | Range, range-rate, useful bearing | Track-filter backbone and cold-object search |
| Lidar (active) | Precision range, range-rate, angle, surface return | Fire-control measurement |
| Passive RF / ESM | Drive/radar signature, bearing | Intelligence and early warning |
| Optical / visible (passive) | Angle, reflected-light geometry | Independent check and supplementary centroid |

**The kill chain:** IR detects and classifies thrusting contacts. Radar constrains range and range-rate and maintains tracks through poor optical geometry. Lidar supplies the highest-precision angular and surface-return measurements. Passive optical/IR, inertial references, and outgoing-beam metrology constrain boresight and target geometry. The weapon solution comes from the fused covariance, not from any one sensor.

A jinking missile cannot remain thermally dark [HARD]. Terminal evasion requires thrust, and thrust produces a bright plume. Coasting dark removes the plume but makes the trajectory more predictable. This is a real trade, not an automatic defender victory: the missile can still exploit light-time, process noise, target glint, duty-cycle timing, and countermeasures.

## Thermal / IR (Passive)

### Physics

Stefan-Boltzmann against a 3 K background. Point-source detection range: R = √(L × A_ap / 4π × P_min). Cryocooled detectors at ~10⁻¹⁶ W effective in-band sensitivity for a 1 m telescope [ENG].

### Detection Ranges (1 m Aperture, Staring)

| Target | Signature | Range |
|--------|-----------|-------|
| Capital ship radiators (5 GW, 2,600 K) | 5 GW | ~12 AU |
| Missile bus under burn | ~1 GW | ~5 AU |
| Coasting missile bus (300 K skin) | ~2 kW | ~1 Gm |
| Coasting torplet (~100 W) | 100 W | ~250,000 km |

Detection range is effectively infinite for tactical purposes [HARD]. No warm object crosses the sky unnoticed.

### Search Rate

Full-sky search at ~10 µrad pixels: ~10¹¹ pixels. With 100 Mpix focal planes and 1 s integrations, a full-sky sweep takes ~15 minutes per telescope. A fleet runs continuous coverage with wide-FOV staring arrays plus narrow-field trackers [ENG].

### Angular Precision

Raw diffraction at 10 µm / 1 m: ~12 µrad — useless for fire control. But centroiding on a bright point source beats diffraction by √N_photons. Against a burning missile, sub-100 nrad centroids are routine [ENG]. Caveat: the centroid is the plume's, not the hull's. Fire control must model the plume-hull offset.

### Limits

No range information. Targets near the solar disc are hidden in glare — attacking down the sun line is a real tactic [HARD].

---

## Radar (Active)

### Physics

P_r ∝ P_t G² λ² σ / R⁴. The R⁴ scaling limits range but fusion power and large apertures push back hard.

### Platform Specification [ENG]

| Parameter | Value | Basis |
|-----------|-------|-------|
| Architecture | Distributed conformal AESA | Hull-embedded T/R modules spanning 30 m baseline |
| Frequency | X-band (3 cm) search, W-band imaging | Standard naval bands |
| T/R modules | ~500,000 | ~5% hull coverage |
| Per-element power | 200 W average | CNT thermal spreaders, microchannel cooling [ENG] |
| Total radiated | 100 MW | ~3% of ship electrical budget |
| Efficiency | 85% | Mature wide-bandgap semiconductors [ENG] |
| System noise temp | 50 K | Cryocooled receiver front-end [ENG] |
| Gain | 70 dB (10⁷) | 30 m aperture at 3 cm |
| Instantaneous bandwidth | 5–10 GHz | Range resolution 1.5 cm |
| Processing | SAI-driven full-array STAP | ~3–6 dB gain over current systems [ENG] |

**Why 100 MW and not more:** Radar detection range scales as P^0.25. Spending 1,000× the power gains 5.6× the range. Every marginal GW diverted from the laser to the radar is a GW that degrades the actual survival mechanism. 100 MW is ~3% of the ship's electrical budget and already sees missiles well beyond weapon engagement range. Radar is a support sensor; it gets support-sensor resources.

**Angular resolution and estimation:** The 30 m X-band aperture has a raw beamwidth of ~1 mrad. Monopulse, phase interferometry, and high-SNR estimation can locate an isolated target to a fraction of that beamwidth [HARD/ENG], so the beamwidth is not itself the bearing-error floor. Even with aggressive calibration, however, X-band angular accuracy remains orders of magnitude short of the ~10 nrad end-to-end weapon requirement. Radar contributes bearing and track validation; it is not the primary laser fire-control angle sensor.

### Detection Ranges

From the standard radar range equation at SNR = 20:

| Target (σ) | 1 ms Integration | 1 s Coherent Integration |
|------------|-------------------|--------------------------|
| Missile, ~1 m² | ~135,000 km | ~760,000 km |
| Capital ship, ~1,000 m² | ~760,000 km | ~4.3 Gm |

1 ms integration is the realistic number against jinking missiles. Longer coherent integration requires motion compensation and degrades against targets with unknown acceleration profiles [ENG].

### Search vs Track

**Track mode (directed):** Full array coherent, pencil beam, 70 dB gain. Maximum performance against a known target.

**Search mode (wide area):** Split into 100 sub-arrays of 3 m each. 100 simultaneous beams at 10 mrad width, ~50 dB gain. Hemisphere sweep in ~1 minute. Missile detection at ~42,000 km. Ship detection at ~240,000 km.

Practical approach [ENG]: dedicated wide-FOV search arrays (small apertures, always scanning) for initial detection, handoff to main array in track mode for precision measurement. Same architecture as modern naval radar (volume search + fire control), unchanged by better technology.

### ESM Asymmetry

100 MW radiated through 70 dB gain gives an on-axis EIRP of ~10¹⁵ W. Under an ideal free-space, narrowband upper-bound calculation, a 1 m² receiver with a 10⁻¹⁶ W threshold detects the main beam at roughly **6,000 AU** and a −40 dB sidelobe at roughly **60 AU**.

Those distances are not practical intercept ranges. A real ESM receiver must search bandwidth, direction, polarization, pulse structure, and dwell time; its effective threshold can be many orders of magnitude worse than the ideal single-channel number [ENG]. The tactical conclusion survives the correction: one-way interception falls as R² while radar detection of a target falls as R⁴. A radar generally reveals its existence far beyond the range at which it detects a missile.

Radar use is therefore a tactical choice. Narrow pencil-beam track emissions reveal less to uninvolved observers than wide-area search, but the illuminated target and any receiver in a sidelobe still gain warning.

### Mass Budget [ENG]

| Component | Mass |
|-----------|------|
| T/R modules (500k × ~20g) | ~10 tonnes |
| Structural backing / hull integration | ~5 tonnes |
| Cooling system | ~3 tonnes |
| Power conditioning | ~2 tonnes |
| Processing | ~1 tonne |
| Cabling / fiber distribution | ~2 tonnes |
| **Total** | **~23 tonnes** |

Rounding error on a capital ship. Radar is cheap. Everyone has it.

### Radar's Role

Radar is the best wide-area active sensor for cold objects and the most robust source of range and range-rate before optical fire-control lock. It also supplies useful bearing information, classification features, and independent track validation. Active lidar can detect cold objects too, but its narrow beam makes it a poor search instrument. Radar cannot, by itself, provide the final angular solution for a 10 nrad-class laser shot.

## Passive RF / ESM

Detection of emissions follows R² (one-way), while the emitter's detection follows R⁴. ESM detects radar emissions at vastly greater range than the radar sees targets [HARD].

Fusion plumes radiate broadband RF noise — a burning drive is detectable and coarsely direction-findable by RF alone [ENG]. Angular precision is poor (long wavelengths), but interferometry across a 60 km fleet baseline gives useful bearings [ENG].

Role: intelligence and early warning. Tells you someone is out there, roughly where, and whether they're transmitting.

---

## Optical / Visible (Passive)

Reflected sunlight. At 2.9 AU, solar flux is ~160 W/m². A dark-coated missile bus reflects maybe 50–100 W total — weaker than its own thermal emission in most geometries. Passive visible is a supplement, not a pillar [ENG].

Its virtue: short wavelengths on modest apertures give the tightest raw diffraction of any passive sensor, and the hardest band to spoof cheaply.

---

## Lidar (Active) — The Fire-Control Sensor

Lidar is the sensor that closes the kill chain. Everything else is supporting cast.

### The Fundamental Difference: Wavelength

X-band radar: 3 cm. Lidar: 266 nm. Ratio: **~113,000×.** Everything below follows from this single number.

Same 1 m aperture under the document's effective-diameter convention:
- X-band radar central-lobe scale: ~37 mrad
- 266 nm lidar central-lobe scale: ~0.32 µrad

The wavelength advantage is enormous, but a diffraction-sized return is not automatically a weapon-quality aim point. Target extent, speckle, boresight, platform jitter, and light-time prediction remain in the budget.

Same power, same aperture, same range — lidar puts ~10¹⁰× more irradiance on target than X-band radar [HARD]. The tradeoff: the tight beam means lidar can only look where it's already pointed. It's a track sensor, not a search sensor.

### Wavelength Selection

The weapon operates at 532 nm for coating damage threshold and power handling reasons. The lidar does not share these constraints — it pushes 1–10 MW, not GW. Coating damage is irrelevant.

**266 nm (deep UV)** is optimal [ENG]:
- Fourth harmonic of Nd:YAG (same laser platform as the weapon)
- 2× tighter beam than 532 nm for same aperture
- 4× higher irradiance on target
- 2× better centroid precision
- Smaller speckle correlation scale for the same target geometry
- Solar background ~3× lower than at 532 nm
- Different wavelength from weapon → forces the jammer to cover a separate optical channel

Conversion efficiency (~20–25% from fundamental) is half that of 532 nm, but for a 1 MW lidar requiring ~5 MW electrical, this is invisible against a 3.3 GW ship power budget.

**Below 190 nm:** Fused silica optics absorb. CaF₂/MgF₂ optics available but exotic. Diminishing returns.

**EUV/X-ray:** Dead end. Grazing-incidence optics destroy the effective aperture advantage. A Wolter telescope with 1.2 m physical mirrors has effective aperture equivalent to a ~5 cm visible telescope. No practical coherent sources. The wavelength advantage is annihilated by optics penalties [HARD].

### Lidar Architecture

| Parameter | Value | Basis |
|-----------|-------|-------|
| Wavelength | 266 nm primary; agile harmonics/OPO bands secondary | 4th harmonic Nd:YAG [ENG] |
| Optical output | 1 MW average during precision track; scalable toward 10 MW | Dedicated track transmitter [ENG] |
| Waveforms | Coherent FMCW / phase-coded chirps plus short direct-detection bursts | Range, velocity, anti-jam diversity [ENG] |
| Instantaneous bandwidth | ~1–10 GHz | ~15 cm to 1.5 cm matched-filter range scale [ENG] |
| Track-solution rate | kHz-class filter output | Not identical to pulse rate |
| Dedicated receive telescopes | 1 m × 4–8 units | Hull-distributed for coverage |
| Receive via stolen segment | One 7.07 m² BB main segment (~3.0 m equivalent circular aperture; 2.86 m flat-to-flat) | 1% weapon-aperture loss |
| Receive via weapon mirror | 30 m (707 m²), off-phase | Large-aperture precision updates |
| Effective divergence (1 m transmitter) | 0.32 µrad diameter | d_eff = 1.22 λ/D |
| Direct detector | SNSPD-class, >95% QE, <10 ps detector jitter | Detector timing is not total system resolution [ENG] |
| Coherent detector | Heterodyne / homodyne | Fine Doppler and phase history [ENG] |
| Spectral filtering | ~10 pm class, agile | Solar and weapon-scatter rejection |
| Outgoing-beam reference | Pickoff metrology on every firing channel | Measures where the weapon actually went |
| Structural metrology | Distributed laser interferometers and inertial references | Tracks mirror/telescope flexure in real time |

**Waveform note:** “1 MW lidar” means optical power delivered during track operation, not a fixed train of nanosecond pulses with absurd terawatt peaks. Coherent chirps and coded bursts provide range through bandwidth and correlation. Short-pulse modes are available for acquisition, ambiguity resolution, and anti-jamming. Receive-gate width and duty factor therefore depend on the selected waveform rather than one universal 7 ns number.

**Range precision note:** A 10 ps detector can timestamp an individual photon to a millimetre-class light-travel interval. Real range accuracy also includes transmitted waveform width, clock stability, surface depth, estimator bias, and SNR. Sub-millimetre precision is plausible only after coherent integration on a stable return, not as a universal per-pulse capability [ENG].

### Receiver Modes

**Dedicated telescopes (primary):** 1 m apertures, hull-mounted and always available. They provide continuous measurements without taking weapon segments offline. Photon-limited angular precision is excellent through the point-defense envelope, but speckle and end-to-end pointing errors become dominant well before the photons run out.

**Stolen segment:** One 7.07 m² BB-main segment is configured as a 266 nm receiver while adjacent segments transmit at 532 nm. The segment is ~2.86 m flat-to-flat and has the collecting area of a 3.0 m circular aperture. This costs ~1% of weapon area. It requires dual-band coatings, a local receive focal plane or coherent receiver, baffling, fast shutters, and aggressive isolation from weapon scatter. A bandpass filter alone would have a brief but memorable career.

**Segment-coherent full-aperture receive:** The weapon has no shared downstream optical element. In receive mode, each dark segment therefore acts as its own telescope. A phase-locked local oscillator heterodyne-detects the 266 nm return at each segment; the ship digitizes phase, amplitude, Doppler, polarization, and timing, then combines the channels as a synthetic optical aperture. Internal metrology supplies the instantaneous position and orientation of every segment. Adding segment intensities would gain photon count but would not automatically provide 30 m angular performance [ENG].

**Weapon mirror (all-dark phase):** When all weapon modules are dark, the hundred segment receivers provide 707 m² total collection and a 30 m coherent baseline. The ~900× collection advantage over a 1 m telescope greatly improves photon statistics and spatial speckle averaging. It also ties the best fire-control updates to weapon scheduling.

**Partial and changing apertures:** Staggered or distributed firing leaves only some groups available to receive. The fire-control processor treats the active receive geometry as a time-varying sparse aperture rather than pretending the mirror is either fully available or absent.

| Weapon state | Available precision receive geometry |
|---|---|
| All groups firing | Dedicated telescopes plus deliberately stolen segments |
| All groups dark | Full 30 m coherent aperture |
| Half the groups firing | Sparse 30 m-baseline aperture with roughly half the collection area |
| Distributed asynchronous fire | Continuously changing sparse aperture; lower and anisotropic angular precision |
| Battle-damaged array | Irregular aperture solved from surviving segment metrology |

**Boresight and flexure control:** The receive telescopes, outgoing weapon channels, inertial reference, and mirror segments are tied together by internal laser metrology. A 10 nrad differential angle across a 30 m structure corresponds to ~300 nm of relative geometry. This is not a passive construction tolerance. It is a continuously measured and corrected state [ENG].

**Ablation feedback:** A successful hit produces a bright broadband plume. Passive optical/IR sensors centroid that flash and use it as a confirmation and surface-location channel. It improves an existing hit; it cannot solve the first-shot prediction problem.

### Photon Budget (266 nm, 1 MW, ρ = 0.1, 1 m² Target)

Assumptions [ENG]:

- 1 MW optical output
- 1 m² projected diffuse target
- Lambertian reflectivity ρ = 0.1
- 1 m circular receive aperture
- 50% net receive throughput and detector efficiency
- effective transmitted diameter d_eff = 1.22 λR/D
- intercepted power capped at transmitter output when the target is larger than the spot

The table reports the **photon-statistical centroid term only**:

**σ_ph ≈ θ_eff / √N**

It does not include speckle, target glint, boresight, platform motion, or prediction.

**1 m receive telescope:**

| Range | Effective Spot Diameter | Spot Area | Power Intercepted | Detected Photons/ms | σ_ph |
|-------|-------------------------|-----------|-------------------|---------------------|------|
| 1,000 km | 0.325 m | 0.083 m² | 1.0 MW | 1.7×10⁷ | 0.08 nrad |
| 3,000 km | 0.974 m | 0.744 m² | 1.0 MW | 1.9×10⁶ | 0.24 nrad |
| 5,000 km | 1.62 m | 2.07 m² | 0.48 MW | 3.2×10⁵ | 0.57 nrad |
| 10,000 km | 3.25 m | 8.27 m² | 0.12 MW | 2.0×10⁴ | 2.3 nrad |
| 50,000 km | 16.2 m | 207 m² | 4.8 kW | 32 | 57 nrad |

**30 m weapon mirror receiver (off-phase), ~900× detected signal:**

| Range | Detected Photons/ms | σ_ph |
|-------|---------------------|------|
| 10,000 km | 1.8×10⁷ | 0.08 nrad |
| 50,000 km | 2.9×10⁴ | 1.9 nrad |
| 100,000 km | 1.8×10³ | 7.6 nrad |

The photon budget supports precision tracking throughout the point-defense envelope under clean geometry. It does **not** prove that the total weapon-pointing error is below 10 nrad. Within tactical range, the dominant terms are expected to be speckle, target-center bias, prediction, platform jitter, and sensor-to-weapon boresight.

### Target Interaction

**Diffuse surfaces (roughness >> 266 nm):** A rough surface returns light broadly. Total return scales with reflectivity, illuminated projected area, incidence angle, and shadowing. It is less aspect-sensitive than a specular surface, but not aspect-independent [HARD]. Typical UV reflectivity spans roughly 0.03–0.9 depending on coating, oxidation, contamination, and finish.

**Specular surfaces (sub-µm polish):** A polished surface produces a strong aspect-dependent glint when its local surface normal satisfies the reflection geometry and very little return at other aspects. A polished convex nose is **not** a retroreflector. Purpose-built corner-cube or cat's-eye geometry would return light toward the source, which is precisely why nobody sensible puts one on an incoming missile.

**Target center of light:** Lidar measures the centroid or resolved distribution of returned light, not the missile's center of mass and not automatically its most vulnerable component. Changing aspect, plume illumination, specular glints, partial shadowing, chaff, and ablation can move the center of light across the body. Fire control therefore tracks a target shape model and covariance, not a single perfect point.

**Optimal surface for lidar evasion:** Dark, rough, and thermally tolerant at the lidar wavelength. Low 266 nm reflectivity reduces photon return; deliberate roughness increases speckle and suppresses stable glints. Because the weapon operates at 532 nm, a missile can pursue a wavelength-selective stack that is absorbing and rough in the UV while reflective in green light, with radar treatment in deeper layers. That is an engineering compromise rather than a hard contradiction: coatings must survive heating, contamination, particle impact, ablation, and rapidly changing incidence. Active, sacrificial, or multilayer coatings can optimize several bands imperfectly, not all bands for free.

### Speckle

Coherent light reflected from a rough target produces a random interference field [HARD]. A useful angular correlation scale is approximately:

**θ_speckle ~ λ / d_target**

For a 1 m target at 266 nm, θ_speckle is ~0.27 µrad. At a 10,000 km receiver range, a spatial speckle cell is of order 2.7 m across.

A 1 m receiver samples less than one spatial cell and can suffer large return-amplitude and center-of-light fluctuations. A 30 m receiver spans of order one hundred cell areas in the ideal geometry, giving much stronger spatial averaging. The exact improvement depends on aperture shape, illumination coherence, target extent, and correlation between cells.

Independent diversity modes reduce speckle approximately as:

**σ_speckle ∝ 1 / √(M_spatial M_temporal M_λ M_pol)**

where the M terms are the numbers of genuinely independent spatial, temporal, wavelength, and polarization realizations.

A kHz pulse rate does **not** imply a thousand independent temporal speckles per second. Decorrelation requires a changed target aspect, changed illuminated patch, wavelength diversity, polarization diversity, or sufficient relative motion. Repeated samples of the same coherent geometry repeat substantially the same speckle.

For current combat modelling, adopt a residual speckle/center-of-light floor of **~3–10 nrad with the 30 m receiver and good diversity**, and substantially worse performance on a 1 m telescope against a deliberately rough target [ENG]. This is an engineering placeholder to be refined with a target-surface simulation, not a result derived solely from pulse count.

### ESM Properties (Reversed from Radar)

A 1 m transmitter at 266 nm has a ~0.32 µrad effective central-lobe diameter. At 10,000 km that is a ~3.2 m footprint. Receivers outside the narrow beam generally see little or nothing unless they intercept sidelobes, target scatter, or plume/chaff fluorescence [HARD/ENG].

The illuminated target sees the lidar with extraordinary clarity: megawatt-class optical power concentrated onto a few square metres. Its warning receiver rapidly characterizes wavelength, polarization, coding, and timing structure. **A target under precision lidar track normally knows it** [HARD].

The asymmetry is the inverse of radar. Radar search can announce the emitter broadly while concealing which contact matters most. Lidar reveals much less to the wider battlespace, but gives the selected target a very specific warning.

### Solar Background

Solar spectral irradiance at 266 nm is ~3× lower than at 532 nm [HARD]. Through a 10 pm bandpass filter on a 1 m aperture, solar background per µrad² pixel: ~10⁴ photons/s. Against lidar returns of 10⁶–10⁸ photons/s: comfortable SNR.

Exception: within a few degrees of the solar disc. Background jumps by orders of magnitude. Lidar FC degrades badly along the sun line. Attacking down the sun is viable doctrine [HARD].

---

## Fire-Control Synthesis

### End-to-End Pointing Budget

The weapon aim error is the combination of several correlated terms:

**σ_aim² ≈ σ_measure² + σ_speckle² + σ_target-bias² + σ_prediction² + σ_boresight² + σ_platform² + σ_beam²**

This root-sum-square form is a bookkeeping approximation. In the actual filter, jamming, target glint, maneuver estimates, and prediction errors are correlated and must remain in the state covariance.

| Error Term | Physical Source | Primary Control |
|------------|-----------------|-----------------|
| Measurement | Photon noise, detector noise | Aperture, power, integration |
| Speckle / target bias | Rough surface, glint, changing aspect | Diversity, resolved imaging, multistatic views |
| Prediction | Unknown acceleration and jerk during light-time | Process model, repeated observations, salvo geometry |
| Boresight | Sensor-to-weapon alignment | Outgoing-beam pickoff and internal metrology |
| Platform | Hull flexure, vibration, thruster impulse | Inertial references, isolation, active optics |
| Beam | Wavefront error, segment phasing, beam wander | Adaptive optics and weapon diagnostics |

### Light-Time and the Causal Blind Interval

At range R, the newest lidar return describes the target at approximately R/c in the past. A weapon fired after receiving that return arrives another R/c later. The shot must therefore predict through approximately:

**Δt_causal ≈ 2R/c**

KHz updates provide many delayed measurements. They do not remove the delay.

| Range | 2R/c | Unmodelled 10 g displacement | 20 g | 50 g |
|-------|------|------------------------------|------|------|
| 1,000 km | 6.7 ms | 2.2 mm | 4.4 mm | 10.9 mm |
| 3,000 km | 20.0 ms | 19.6 mm | 39.3 mm | 98 mm |
| 5,000 km | 33.4 ms | 54.6 mm | 109 mm | 273 mm |
| 7,000 km | 46.7 ms | 107 mm | 214 mm | 535 mm |
| 10,000 km | 66.7 ms | 218 mm | 436 mm | 1.09 m |

These values are the displacement from a step change in lateral acceleration not represented in the filter. Real missile motion is limited by thrust direction, structure, propellant, and jerk, but the table shows why prediction can dominate even when the angular measurement is excellent.

### From Aim Error to Damage

A laser kill is not a binary test of whether the beam center falls within one beam radius of the target center. The target has finite projected size, and a beam can strike the body while wandering across different patches.

The correct quantity is deposited fluence over the target surface:

**F(x) = ∫ I(x − e(t), t) dt**

where **e(t)** is the time-dependent weapon aim error in the target plane. A kill occurs when a vulnerable region exceeds its required fluence or penetration threshold.

Three regimes matter:

1. **Patch hold:** the beam remains on one small region and achieves the clean dwell time.
2. **Body hit / patch wander:** power remains on the missile but moves across its skin, delaying penetration.
3. **Body miss:** most useful power leaves the projected silhouette.

The fire-control filter does not reduce those regimes to one “tracking percentage.” It delivers a predicted target-plane probability distribution and target model. The laser model then convolves that distribution with its actual beam pattern and the projected silhouette.

For simplified campaign calculations, define over a beam-on interval **T**:

- **E_lobe = P_lobe T:** central-lobe energy emitted,
- **E_body:** central-lobe energy intersecting the projected target,
- **E_cell,max:** maximum energy accumulated in any relevant damage cell,
- **f_body = E_body/E_lobe,**
- **f_patch = E_cell,max/E_body,** with the damage cell chosen large enough to contain the clean reference spot,
- **t_reacq:** dead time after a track break or false-track handoff.

Both efficiency factors are dimensionless and approach one for a clean centered dwell. **f_body** measures whether the missile is hit; **f_patch** measures whether those hits stay concentrated.

Then an approximate effective dwell is:

**t_dwell,eff ≈ t_dwell,clean / (f_body f_patch)**

and target service time is:

**t_service = t_steer + t_settle + t_dwell,eff + t_confirm + t_reassign + t_reacq**

This replaces the former independent **P_point** multiplier. Jamming changes the state covariance, association probability, and time history of **e(t)**; it is not a separate coin flip applied after prediction.

### Clean-Condition Closure

The sensor chain is:

1. IR and ESM detect thrust and emissions.
2. Radar constrains range, range-rate, and bearing.
3. Lidar and passive optical sensors estimate angular state and target-return geometry.
4. Internal metrology ties those measurements to the outgoing weapon beam.
5. The track filter predicts through 2R/c with explicit acceleration and jerk process noise.
6. Weapon diagnostics and ablation feedback update the damage estimate.

Under clean conditions the photon budget is not the bottleneck inside the point-defense envelope. Whether the complete 10–20 nrad requirement closes depends on speckle, platform/boresight control, and the missile's unpredictable acceleration spectrum.

## Lidar Countermeasures

### Geometry Constraint

Lidar fire control examines a ~µrad-class angular cell. At 10,000 km that cell maps to metres at the target range. A jammer physically beside the target naturally shares the cell, but a source at another range can also share it if the source, target, and receiver are closely aligned.

**A jammer must appear inside the target's angular resolution element or cause shared-receiver saturation/scatter** [HARD]. For ordinary imaging receivers this usually means a source on the missile, immediately adjacent to it, or nearly collinear at another range. A source a few pixels away does not automatically add uniform background across the focal plane. This makes most lidar countermeasures self-screening, with the important exception of a deliberately aligned stand-off jammer.

### Jammer Drones (R² vs R⁴ Exploitation)

A diffuse lidar return falls approximately as R⁴: inverse-square illumination outbound and inverse-square collection inbound. A jammer transmitting directly toward the defender falls as R². The jammer-to-return ratio therefore improves strongly with range [HARD].

**Geometry:** A stand-off drone rides with the salvo and establishes a line of sight nearly collinear with one or more missiles from a chosen defender. It may sit behind the salvo, ahead of it, or offset enough to attack shared optics. Post-separation lateral spread means one drone cannot remain inside every missile's resolution element for every defending ship.

**The jammer specification must include:**

- optical output power,
- transmitter aperture and divergence,
- spectral width and wavelength accuracy,
- pointing accuracy and jitter,
- polarization,
- range from the defender,
- angular separation from the target in each defender's view,
- modulation and timing strategy,
- receiver spatial/spectral/temporal rejection.

“100 W jammer” is not a complete model. A 100 W lamp and a 100 W diffraction-limited laser are not tactical relatives.

**Reference stress case [SPEC]:** A 100 W optical jammer with a 10 cm aperture at 266 nm has an effective central-lobe diameter of ~3.2 µrad, or ~32 m at 10,000 km. If precisely pointed, spectrally matched, and inside the same receiver resolution element, its continuous one-way signal can dominate a 1 m diffuse-target return even after strong temporal rejection. This establishes plausibility, not a universal J/S value.

**Range/correlation rejection:** Direct-detection bursts reject continuous light through narrow receive windows. Coherent coded waveforms reject uncorrelated light through matched filtering. The effective suppression is waveform-specific and must include gate duty factor, code-processing gain, detector dynamic range, and internal scatter.

**Causality:** A trailing drone farther from the defender than the missile cannot wait to observe an unpredictable outgoing pulse and then place a response inside the earlier return gate of the nearer missile. That return window has already passed by the time the pulse reaches the drone. A trailing drone must jam continuously, predict a regular waveform, or receive advance timing data. An **ahead-of-target** jammer can observe a pulse earlier and add delay, but is harder to position and protect.

PRF and code jitter therefore defeat predictive stand-off timing. They do not defeat a jammer physically collocated with the target, which receives the outgoing pulse at the same epoch that creates the natural reflection.

### Tracking Degradation from Jamming

For a simple photon-noise-limited centroid, added uncorrelated jammer photons produce:

**σ_measure,jammed ≈ σ_measure,clean √(1 + J/S)**

This relation applies only to the measurement-noise term. It does not directly produce a weapon miss probability. A jammer can also create:

- detector saturation and recovery dead time,
- biased center-of-light estimates,
- false range/Doppler peaks,
- track swaps between nearby objects,
- increased covariance and prediction error,
- intermittent dropouts that force reacquisition.

The fire-control model must propagate those effects through the track covariance and then through the surface-fluence model. Previous tables comparing one-sigma centroid error directly to beam radius have been removed because they treated a finite missile as a point and ignored patch wander.

### Illustrative Track-Efficiency Sensitivity [SPEC]

Until a time-domain target/beam simulation is complete, contested PD capacity should be expressed as sensitivity to track efficiency rather than a single canonical number.

| Effective f_body × f_patch | Interpretation | Approximate Capacity Relative to Clean |
|----------------------------|----------------|----------------------------------------|
| 0.7 | Mild jitter; beam usually holds useful structure | ~70%, before reacquisition losses |
| 0.3 | Beam often remains on body but walks across patches | ~30% |
| 0.1 | Severe dropouts, false returns, or repeated patch resets | ~10% |

Reacquisition time reduces capacity further, especially at close range where each lost second consumes a large fraction of the remaining engagement window. These are sensitivity bands, not setting-final kill counts.

### Duty Cycle Vulnerability

The fire-control system alternates between two aperture states:

**Weapon firing:** Continuous tracking relies on dedicated 1 m telescopes and any stolen segments. These have poorer photon statistics and much weaker spatial speckle averaging than the full primary.

**Weapon off-phase:** The 30 m mirror provides high-SNR, spatially averaged updates and can resolve return structure more effectively.

Large-aperture updates reduce state covariance, but they do not eliminate future process noise. Prediction carries the track through the firing interval and degrades according to the missile's acceleration/jerk spectrum.

**Attacker exploit:** Measure the firing rhythm and concentrate unpredictable lateral thrust during intervals when the defender lacks the large receiver. Coast or use lower-information thrust during large-aperture observation.

**Defender counters:**

- randomize firing and receive phases,
- stagger segments rather than switching the whole mirror together,
- use stolen segments for simultaneous receive,
- alternate illumination/receive duties between ships,
- preserve unpredictable short large-aperture looks.

Randomization costs weapon scheduling flexibility and may reduce average output, but it prevents a missile from treating the duty cycle as a public timetable.

### Chaff (UV Nanoparticle Clouds)

UV-optimized nanoparticle clouds are plausible disposable obscurants. They are better modelled as **temporary extinction, multiple-scattering, and center-of-light disturbances** than as clean off-axis point decoys.

Assume ~500 nm aluminum particles in the Mie-scattering regime. The following mass estimates are order-of-magnitude [ENG]:

| Optical Depth (per metre) | Particles/m³ | Mass/m³ |
|---------------------------|--------------|---------|
| τ = 0.5 | 2.5×10¹² | 0.44 g |
| τ = 1 | 5×10¹² | 0.88 g |
| τ = 2 | 1×10¹³ | 1.76 g |
| τ = 3 | 1.5×10¹³ | 2.64 g |

A 2 × 2 × 0.1 m sheet at τ ≈ 2 contains roughly **7 g** of aluminum particles before packaging and dispersal hardware.

**Coverage against a 1 m, 266 nm transmitter:**

| Range | Effective Lidar Diameter | 2 m Cloud Coverage |
|-------|--------------------------|--------------------|
| 1,000 km | 0.32 m | Larger than footprint |
| 3,000 km | 0.97 m | Larger than footprint |
| 5,000 km | 1.62 m | Comparable |
| 10,000 km | 3.25 m | Smaller than footprint |

### What Chaff Can Do

- attenuate the direct missile return,
- add a bright, structured scattering volume around the target,
- increase speckle and center-of-light instability,
- force the defender to resolve range/shape rather than centroid one point,
- create short track dropouts while the missile changes velocity,
- generate plasma and debris when burned through.

### What Chaff Cannot Do Reliably

A cloud displaced outside the transmitted footprint is not illuminated and cannot drag the centroid. At close range the beam is narrow, so a 0.5–1 m off-axis cloud may simply be invisible to that lidar shot. The maximum decoy bias is constrained by the illuminated footprint and by the receiver's angular/range resolution.

The strongest close-range use is therefore not “laser aims a metre away forever.” It is:

**cloud appears → return blooms or attenuates → defender resolves/burns through → missile thrusts during the interruption → fresh cloud appears at the new state.**

### Range Discrimination

A 7 ns direct receive window corresponds to about **1.05 m one-way range depth**, not centimetres. GHz-class coherent bandwidth can produce centimetre-to-decimetre matched-filter range scales, but the effective resolution also includes surface depth, multiple scattering, waveform ambiguity, and SNR. A cloud only 0.1 m from the missile is not automatically rejected by “cm-class range” unless the selected waveform and geometry actually support it.

### Lidar Burn-Through

The lidar itself is a megawatt-class UV source. For a 7 g, 4 m² τ ≈ 2 sheet, only the portion inside the beam must be heated, melted, vaporized, charged, or pushed aside. Order-of-magnitude energy deposition suggests **millisecond to sub-second burn-through** across the inner engagement envelope, depending on absorption, particle size, replenishment, and plasma shielding [ENG].

That is far shorter than the geometric cloud-dispersal lifetime. Chaff is therefore a repeatable acquisition breaker and dwell reset, not a persistent invisible wall.

**Against the weapon beam:** The 532 nm weapon destroys the illuminated cloud even faster. Chaff is not armor. The useful effect is the transient interruption, plasma flash, and forced reacquisition.

### Mass Budget

| Component | Approximate Mass |
|-----------|------------------|
| Particle charge | ~7 g per puff |
| Packaging / valve / nozzle | several grams to tens of grams |
| 30–50 puffs | ~0.3–1 kg total system |

This competes directly with terminal propellant and structural margin. It is worthwhile only if the saved dwell and reduced jink demand exceed the lost maneuver capability.

### Missile Dazzlers (Self-Screening)

A missile-mounted optical transmitter can tune toward the detected lidar wavelength through an OPO or a bank of harmonic channels [ENG]. Being collocated with the target solves the angular-alignment problem.

Possible effects:

- **Saturation:** overload the detector or analogue front end and force recovery time.
- **Noise injection:** raise shot noise inside the target pixel and matched-filter band.
- **False return:** transmit a coded or pulsed response that resembles a reflection.
- **Surface-mask support:** illuminate chaff or ablation products to strengthen a false center of light.

A collocated transponder can react to each outgoing pulse: it receives the pulse at the same epoch that creates the natural reflection, then transmits back toward the defender. PRF jitter alone does not defeat it. Hardware latency appears as extra path length:

**ΔR_false = c τ_latency / 2**

A reactive dazzler can readily create returns **behind** the true target in range. It cannot create a causally earlier, closer return unless it predicts the waveform.

Defender counters include:

- secret or rapidly changing phase codes,
- coherent Doppler consistency checks,
- polarization diversity,
- wavelength agility,
- multiple simultaneous viewing angles,
- detector protection and fast recovery,
- comparison with radar and passive plume motion.

Dazzlers degrade and confuse; they do not erase the target from every modality.

### Decoys

Lidar decoys face tighter precision constraints than radar decoys. Fine coherent range resolution and narrow angular cells separate objects unless they remain close in both range and angle or deliberately attack the receiver's ambiguity structure. The exact separation threshold depends on waveform, SNR, and target depth.

**Towed body:** Nanotube tether, reflective object offset laterally. At 10,000 km, a 100 m offset is 10 µrad — 30× the beam divergence. The two objects are in completely different beam positions. Not viable as a lidar decoy at long range [ENG].

**IR/radar decoys (bus phase):** Cheap heated bodies on similar trajectories. Force the defender to track 10× more contacts than real missiles. Defeated by radar cross-section and acceleration profile discrimination at closer range, but they've served their purpose — consuming search time and attention [ENG].

### Surface Treatment

Optimal missile surface for lidar evasion: dark matte, with low UV reflectivity and deliberate µm-scale roughness that suppresses stable glints and increases speckle [ENG]. A mirror-finish nose produces strong aspect-dependent glints, not automatic retroreflection. Those glints can give the defender exceptionally clean measurements at unfortunate attitudes.

A reflective finish is desirable for reducing absorbed weapon energy, creating a real design tension: surfaces that survive irradiation better may provide brighter and more stable optical returns. Sacrificial dark outer layers over reflective thermal layers are one possible compromise.

---

## Cooperative Fleet Fire Control

A fleet is not one receiver. Distributed sensing is one of the defender's strongest counters to optical deception.

### Multistatic Advantages

- A jammer collinear with a missile from one ship is generally not collinear from another.
- Chaff that masks one line of sight exposes different edges and depth structure from another.
- Independent apertures sample different speckle patterns.
- Separate bearings provide parallax and constrain target/jammer geometry.
- One platform can illuminate while another receives.
- Pickets can maintain track while a battleship's primary mirror is firing.
- Radar, passive IR, and lidar returns can be checked for mutually consistent acceleration.

A 60 km task-force baseline against a target at 10,000 km subtends ~6 mrad. That is enormous compared with a µrad-class lidar field. Countermeasures must therefore be designed against the **fleet geometry**, not one telescope.

### Tactical Consequence

Corvettes and sensor drones are not merely extra guns. They are pieces of the fire-control aperture. Destroying or driving off pickets collapses parallax, reduces independent speckle samples, creates blind aspects, and makes stand-off jammer alignment easier. A battleship can retain full laser power while losing much of its ability to apply that power efficiently.

The attacker therefore has a preliminary sensor-war objective: kill the distributed eyes before the main missile wall enters the decisive envelope.

### Canonical Fire-Control Output to the Weapon

For each candidate target and proposed weapon-arrival epoch, the fused sensor network provides one standard target package:

| Output | Meaning |
|---|---|
| **x̂, v̂, â** | Predicted target state at beam arrival |
| **Σ_target** | Two-dimensional target-plane covariance from measurement, velocity, maneuver, and association uncertainty |
| **p_assoc** | Probability that the maintained track is the real threatening object rather than chaff, decoy, or jammer |
| **S(x)** | Projected target silhouette and aspect estimate |
| **V(x)** | Vulnerable-component / damage-cell map with uncertainty |
| **Σ_boresight** | Firing-platform boresight, jitter, and outgoing-beam angular covariance, projected into the target plane at range R |
| **H_EW** | Weighted jammer, dazzler, chaff, and false-track hypotheses |
| **C_kill** | Post-shot confidence that the target is no longer threatening |

The weapon model combines the target and platform terms:

**Σ_effect = Σ_target + Σ_boresight**

with coordinate transforms and correlations retained in the actual filter. The laser then chooses aperture, beam shape, raster pattern, power allocation, and service time. This is the interface between the two reference documents. Sensors own the probability distribution; the laser owns what to do with it.

---

## PD Capacity Under Contested Conditions

The clean-sky capacity from the laser document remains an **upper bound**. The previous 130–380 kill estimate has been withdrawn because it was generated by comparing centroid sigma directly with beam radius and treating the missile as a point.

Contested capacity must be calculated by integrating target service time across the closing trajectory:

**N_kill ≈ ∫ dt / t_service(R, geometry, covariance, target state)**

with:

**t_service = t_steer + t_settle + t_dwell,clean/(f_body f_patch) + t_confirm + t_reassign + t_reacq**

Required simulation inputs:

- missile projected geometry and vulnerable-component map,
- Gaussian/segmented weapon intensity distribution,
- time-correlated aim error rather than one independent error per sample,
- target acceleration and jerk spectrum,
- lidar/radar filter covariance,
- jammer waveform and angular geometry,
- chaff extinction and burn-through,
- receiver duty cycle and cooperative fleet views,
- thermal damage accumulation and cooling between patch visits.

### Current Defensible Conclusion

Layered EWAR can compress the effective engagement window and reduce dwell efficiency enough that missile numbers remain tactically relevant. The direction of effect is robust. The exact reduction in kills per battleship is not yet closed and should remain scenario-dependent rather than canonized as one number.

## Open Questions

- **Time-domain fluence simulation:** finite target body, vulnerable components, Gaussian/segmented beam, correlated aim wander, cooling, and ablation.
- **Missile maneuver spectrum:** achievable lateral acceleration, jerk, minimum impulse, thrust-vector slew, and propellant cost during the 2R/c causal interval.
- **Full pointing budget:** boresight metrology, mirror phasing, platform vibration, thruster impulses, and outgoing-beam direction knowledge.
- **Target-return model:** UV BRDF, roughness, glint statistics, plume-hull offset, and aspect-dependent projected area.
- **Speckle simulation:** spatial modes across 1 m and 30 m apertures; temporal, wavelength, and polarization decorrelation.
- **Lidar waveform definition:** coherent bandwidth, code family, ambiguity resolution, update cadence, detector dynamic range, and anti-jam processing gain.
- **Jammer drone design:** aperture, pointing, optical efficiency, thermal load, alignment geometry, and procurement cost.
- **Reactive dazzler limits:** latency, false-range structure, phase-code capture, and detector-recovery attacks.
- **Chaff burn-through:** absorption/scattering by particle size, plasma shielding, charging, radiation pressure, replenishment rate, and multi-wavelength mixtures.
- **Cooperative fire control:** validate the canonical target package, communications latency, cross-platform clock/boresight calibration, sparse-aperture receive, and performance after picket losses.
- **Weapon/sensor scheduling:** full-dark coherent receive, sparse-aperture staggered fire, randomized firing, attacker timing, and flywheel/thermal costs.
- **Contested PD capacity:** rerun only after the fluence, filter, jammer, and target models are coupled.
