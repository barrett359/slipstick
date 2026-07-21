# Systems Quick Reference

All figures from compiled setting documents and working conversations. Tags: **[HARD]** = textbook physics, **[ENG]** = defensible engineering extrapolation, **[SPEC]** = speculative.

---

## Reactors (D-³He Fusion)

| Parameter | Value | Tag |
|-----------|-------|-----|
| Ship reactor | 1–2 MW/kg | [ENG] |
| Missile reactor | 3–5 MW/kg (baseline 5) | [ENG] |
| Missile reactor (aggressive) | 5–10 MW/kg | [SPEC] |
| Hard ceiling | ~10 MW/kg | [HARD] |
| Reactor waste heat fraction | ~1–5% of fusion power (rest exits as exhaust KE) | [ENG] |
| Reactor+nozzle assembly | ~20% of ship dry mass | [ENG] |
| Power scaling rule | ~1.8 TW fusion per 10,000 t at 10 mg cruise | [ENG] |
| HTS power bus | ~0.5–2 t per GW of bus capacity | [ENG] |

---

## Lasers

| Parameter | Value | Tag |
|-----------|-------|-----|
| Wavelength | 532 nm (green, SHG Nd:YAG) | |
| Module unit | 10 MW optical output | |
| Wall-plug efficiency | 15% | |
| Duty cycle | 0.5 | |
| Airy central lobe | 84% of emitted power | [HARD] |
| Beam on target (peak) | 84% of listed power | |
| Beam on target (avg) | 42% of listed power | |

### Installed Systems

| Platform | Aperture | Modules | Listed | Beam (avg) |
|----------|----------|---------|--------|------------|
| BB main | 30 m | 100 | 1 GW | 420 MW |
| BB secondary (×6+) | 10 m | 20 | 200 MW | 84 MW |
| Corvette spinal | 15 m | 50 | 500 MW | 210 MW |
| Standard PDL | 3 m | 15 | 150 MW | 63 MW |
| Drone PDL | ~1 m | 4 | 40 MW | 17 MW |

---

## Flywheels (CNT)

| Parameter | Value | Tag |
|-----------|-------|-----|
| System-level specific energy | 2,500 Wh/kg | [ENG] |
| Rotor-dominated (setting range) | 2,000–4,000 Wh/kg | [ENG] |
| CNT theoretical ceiling | ~8,571 Wh/kg | [HARD] |

### Example Mass Budgets (BB Main, 1 GW, 15% eff → 6.67 GW input)

| Cycle Period | Buffer Energy | Flywheel Mass (@ 4,000 Wh/kg) |
|-------------|---------------|-------------------------------|
| 0.1s on / 0.1s off | 333 MJ | 23 t |
| 1s / 1s | 3.33 GJ | 231 t |
| 10s / 10s | 33.3 GJ | 2,310 t |

Shorter pulsing = less flywheel mass. Sub-second cycling is favored.

---

## Heat Sinks (Lithium)

| Parameter | Value | Tag |
|-----------|-------|-----|
| Specific heat | 3,560 J/kg·K | [HARD] |
| Heat of fusion | 435 J/g (melts at 454 K) | [HARD] |
| Sensible + latent to 800 K | ~2.2 MJ/kg | [HARD] |
| Sensible + latent to 1,000 K | ~4.6 MJ/kg | [HARD] |
| Heat of vaporization (emergency vent) | 19,600 kJ/kg | [HARD] |

### Combat Endurance (BB Main, 2.83 GW avg waste heat)

| Li Mass | Duration |
|---------|----------|
| 500 t | 76 s |
| 2,000 t | 5.1 min |
| 5,000 t | 12.7 min |
| 10,000 t | 25.4 min |

Rule of thumb: ~10 t Li per full 3s main-gun shot (absorbing 45 GJ waste heat at 60% waste fraction).

---

## Radiators

### High-Temperature (Reactor Loop, Always Deployed)

| Parameter | Value | Tag |
|-----------|-------|-----|
| Rejection at 2,000 K, ε=0.9 | 816 kW/m² | [HARD] |
| Rejection at 2,500 K | ~2.0 MW/m² | [HARD] |
| Rejection at 1,800 K | ~535 kW/m² | [HARD] |
| C-C panel mass | ~2 kg/m² | [ENG] |
| Refractory metal panel | ~15 kg/m² | [ENG] |
| Liquid droplet radiator | ~0.42–1 kg/m² | [ENG] |
| Ship radiator specific rejection | 1.6 MW/kg | [ENG] |
| Missile radiator (ablative) | 2.24 MW/kg (1.4× ship) | [ENG] |

### Low-Temperature (Laser Loop, Retracted in Combat)

| Parameter | Value | Tag |
|-----------|-------|-----|
| Rejection at 500 K, ε=0.9 | 3.19 kW/m² | [HARD] |
| Rejection at 700 K | 12.8 kW/m² | [HARD] |
| LT area for sustained BB main (2.83 GW @ 700 K) | ~221,000 m² | |

The LT array is the lasestar's defining structural feature and primary vulnerability.

### Example Ship Budgets

| Class | HT Radiator Area | HT Radiator Mass (@ 2 kg/m²) |
|-------|-------------------|-------------------------------|
| BB (90,000 t dry) | ~131,000 m² | ~262 t (~0.3% dry) |
| Corvette (5,000 t dry) | ~11,000 m² | ~22 t |

---

## Crew & Life Support

| Parameter | Value | Tag |
|-----------|-------|-----|
| ECLSS equivalent system mass | ~730 kg/person | [ENG] |
| Full budget (habitat + consumables + shelter) | 1.5–3 t/person | [ENG] |
| Habitable volume | 20–40 m³/person | [ENG] |

### By Class

| Class | Crew | Life Support Mass | Habitable Volume |
|-------|------|-------------------|------------------|
| BB | ~250 | 375–750 t | ~6,000 m³ |
| Corvette | ~25 | 38–75 t | ~600 m³ |

---

## Jammers & EWAR

### Lidar Dazzler (Self-Screening, On-Missile)

| Parameter | Value | Tag |
|-----------|-------|-----|
| Principle | R² vs R⁴ asymmetry | [HARD] |
| 1 W through 1 cm lens @ 266 nm | J/S ~20,000,000:1 at 10,000 km | [HARD] |
| Limitation | Collocated with target — centroid doesn't move | [HARD] |
| Useful effect | Detector saturation, background noise, range spoofing | [ENG] |
| Tuning | OPO, retunes in µs to match detected lidar λ | [ENG] |
| Defender counter | PRF jitter, frequency hopping (1064/532/355/266 nm) | [ENG] |

Dazzlers degrade rather than blind. Effective in combination with chaff and drones.

### Jammer Drone (Stand-Off)

| Parameter | Value | Tag |
|-----------|-------|-----|
| Reference: 100 W, 10 cm aperture, 266 nm | ~50 kg total | [ENG] |
| Geometry | Trails salvo on approach axis, colinear from defender's view | |
| Range gating | Reduces effective power ~15,000× — R² advantage still wins | [HARD] |

**Effect on PD Envelope (vs 1 m dedicated telescope):**

| Drone Power | Effective PD Compressed To |
|-------------|---------------------------|
| 100 W | ~5,000 km |
| 1 kW | ~2,000 km |

**30 m mirror in receive (off-phase):** Collects 900× more signal than 1 m telescope. Drops J/S by 900×. This is why duty-cycle receive scheduling matters — the big mirror recovers most of what the jammer takes away.

### Chaff (UV Nanoparticle Clouds)

| Parameter | Value | Tag |
|-----------|-------|-----|
| Particle type | ~500 nm aluminum, Mie scattering | [ENG] |
| Mass per puff (2×2×0.1 m, τ≈2) | ~7 g | [ENG] |
| 30–50 puffs per missile | 0.3–1 kg total system | [ENG] |
| Lidar burn-through time | ms to sub-second | [ENG] |
| Weapon beam burn-through | ~7 µs at 5,000 km | [ENG] |

Chaff is a repeatable acquisition breaker and dwell reset, not a persistent shield. Competes directly with terminal propellant mass.

### Radar

Radar is the range/velocity sensor, not fire-control. Angular resolution (mrad-class) is too poor for weapon pointing. Radar jamming is not formalized yet — noted as an open item for the EWAR document.

---

## Sensors (Summary)

| Sensor | Role | Precision | Range |
|--------|------|-----------|-------|
| IR/Thermal (passive) | Search | ~µrad | Effectively infinite for burning drives |
| Radar (active) | Range & velocity | ~mrad angular, cm range | Tactical |
| Lidar (active, 266 nm) | Fire control | ~nrad (3–10 nrad floor w/ speckle) | FC-quality to ~50,000 km (1 MW), ~160,000 km (10 MW) |
| Passive RF/ESM | Intel & early warning | Poor angular | R² detection advantage over emitter |
| Optical/Visible | Supplement | Tightest raw diffraction | Weak signal (reflected sunlight) |

Lidar is the sensor that closes the kill chain. Everything else is supporting cast.
