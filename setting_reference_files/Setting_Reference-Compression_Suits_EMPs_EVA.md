# Setting Reference — Compression, Suits, EMPs & EVA Combat

This document captures worldbuilding details established in conversation. It covers hyper-compression technology, suit life support and atmospheres, RCS propulsion, improvised EMP devices, pressure vessel stored energy, and EVA combat maneuvering.

---

## Hyper-Compression Technology

### Baseline (Current Era)

- Steel cylinders: 200–300 bar
- Carbon fiber composite overwrap (COPV) tanks: ~700 bar (hydrogen fuel cells)
- Experimental hydrogen storage: ~1,000 bar

### Setting Standard (~25th Century)

Carbon nanotube composite pressure vessels rated to **10,000 bar** as a standard high-quality tank. Theoretical CNT tensile strength is ~100x steel; at 50% of theoretical (manufacturing imperfections, fiber alignment), pressure vessel improvements are enormous. 30,000 bar is plausible for military/specialty applications.

### Gas Behavior at Extreme Pressure

Gases stop behaving ideally at high pressures. Key behaviors:

- **Supercritical fluids:** Above a substance's critical temperature and pressure, the liquid/gas distinction disappears. The fluid is dense like a liquid but fills its container like a gas. At room temperature, hydrogen and nitrogen are always above their critical temperatures — they cannot be liquefied by pressure alone, only compressed into denser and denser supercritical fluid.
- **Diminishing returns:** Heavy gases (xenon, krypton) approach liquid density at relatively modest pressures. Xenon at 300 bar is already ~75% of liquid density. Going from 300 to 10,000 bar only gains ~1.5x more density. Light gases (hydrogen, nitrogen) benefit proportionally more from extreme compression.
- **Practical consequence:** Extreme compression partially closes the density gap between light and heavy gases.

### Density at Pressure (Room Temperature, Approximate)

| Gas | 300 bar (kg/L) | 10,000 bar (kg/L) | Compression gain |
|-----|----------|-----------|------|
| H2 | ~0.025 | ~0.15 | ~6x |
| N2 | 0.28 | ~1.1 | ~4x |
| Ar | 0.9 | ~1.7 | ~1.9x |
| Xe | 2.2 | ~3.3 | ~1.5x |

---

## Suit Life Support

### Oxygen Supply

Human O2 consumption: ~35 g/hr. Eight hours requires ~280g of O2.

At 10,000 bar, O2 approaches liquid density (~1.14 kg/L). Storage requirements:

- **0.25 liters for 8 hours.** A coffee cup.
- **1 liter for ~32 hours.**
- **2 liters for ~3 days.**

A standard 88g airgun-sized cartridge (100 mL) at these pressures holds ~114g O2, or **~3.25 hours.** Two cartridges in a vest pocket provide a 6.5-hour combat mission. O2 storage is essentially a solved problem at this pressure regime.

### Suit Atmosphere

Pure O2 at **0.21 bar** total pressure provides identical partial pressure of oxygen (ppO2) to sea-level air. This eliminates the ~79% nitrogen dead weight in standard atmosphere, yielding ~4.76x the useful breathing gas from the same tank.

**Fire risk:** Not Apollo 1 (which was 1.1 bar pure O2, five times normal ppO2). At 0.21 bar ppO2, there is no additional oxygen present compared to air. However, 100% O2 environments lack nitrogen to slow flame propagation. Materials that self-extinguish in air can sustain combustion in pure O2 even at reduced total pressure. This is a materials-selection problem — suit interiors are designed with non-flammable materials (standard after centuries of spaceflight).

**Oxygen toxicity:** Zero concern at 0.21 bar ppO2. Toxicity onset is above ~0.5 bar ppO2.

**Decompression sickness (DCS):** The real problem. Transitioning from 1-bar station atmosphere to 0.21-bar suit causes dissolved nitrogen to outgas, forming bubbles in blood.

**Solutions:**
- Station atmospheres at lower pressure (0.5–0.7 bar with enriched O2)
- Suit pressure at 0.3–0.4 bar to reduce differential
- **Helium-oxygen breathing mix** — helium is much less soluble in blood than nitrogen. A 50/50 O2/He mix at 0.42 bar provides normal ppO2 with excellent DCS margins. This is the likely standard for military suits, with the bonus that helium is dual-use as emergency RCS propellant.

### CO2 Scrubbing

**Primary system: Disposable scrubber cartridges.**

LiOH chemistry: 2 LiOH + CO2 → Li2CO3 + H2O. Requires ~1.09 kg LiOH per kg CO2. Humans produce ~40g CO2/hr.

With 3–5x improvement from metal-organic frameworks (MOFs) or amine-functionalized sorbents over baseline LiOH: **300–500g of sorbent provides 8 hours.** A 1 kg scrubber cartridge yields 16–24 hours.

CO2 scrubber capacity is the primary suit duration limiter, not O2 supply. Suit endurance is measured in scrubber cartridges.

**Supplementary system (military/extended duration): CO2 regeneration.**

Solid oxide electrolysis: CO2 → CO + O2 at ~800°C. Demonstrated by MOXIE on Mars. Power requirement: ~200–250W continuous (real-world efficiency) for human CO2 output.

This does not replace the scrubber — the scrubber handles real-time CO2 removal (fast kinetics to keep ppCO2 safe moment-to-moment). The regenerator operates as a batch process on captured CO2, slowly reclaiming O2 to extend supply.

**Waste heat integration:** The 800°C electrolyzer produces significant waste heat that is routed into the RCS propellant via heat exchanger, improving Isp for free. This solves the suit's thermal management problem simultaneously. See RCS section below.

### Duration Summary

| Component | Mass | Duration |
|-----------|------|----------|
| O2 (2L tank, 10,000 bar) | ~2.3 kg | ~3 days |
| Scrubber cartridge (MOF) | ~1 kg | 16–24 hrs |
| Regenerator (military) | Several kg (system) | Extends O2 supply by recycling captured CO2 |

Routine EVA maintenance task: swapping scrubber cartridges. O2 tank refills are infrequent by comparison.

---

## RCS Propulsion

### Design Principles

For **volume-limited** applications (flush-mounted suit thrusters), the relevant metric is Isp × density. Heavy gases win. For **mass-limited** applications (backpack-frame RCS), only Isp matters. Light gases win.

### Cold Gas Performance at 10,000 Bar (Theoretical Isp, ~70% for Real Systems)

| Gas | Density (kg/L) | Isp (s) | Isp × density |
|-----|-------|------|----------|
| H2 | ~0.15 | ~270 | 40 |
| He | ~0.2 | ~170 | 34 |
| N2 | ~1.1 | ~80 | 88 |
| Ar | ~1.7 | ~57 | 97 |
| Xe | ~3.3 | ~31 | 102 |

### Heated Gas (Resistojet / Arcjet)

Isp scales with √(T/M). Light gas + high temperature = best performance.

| Method | Isp (H2) | Notes |
|--------|----------|-------|
| Cold gas | ~270s | Simple, no power draw |
| Electrolyzer waste heat (~800°C) | ~400–450s | Free heat, dual-use engineering |
| Resistojet (1000K) | ~500s | Moderate power draw |
| Arcjet (3000K) | ~950s | Heavy battery draw |

### H2/O2 Combustion (For Comparison)

Isp ~450s, but consumes oxygen (8 kg O2 per 1 kg H2) and produces 18 g/mol water vapor exhaust instead of 2 g/mol hydrogen. Delta-V per total propellant mass is better than cold gas but far worse than heated hydrogen per kg of H2 alone.

### Metallic Hydrogen

Isp **1500–1700s** from recombination energy. Exhaust temperature 3000–4000K. This is the premium military propellant.

**2 kg metallic hydrogen, 150 kg armored suit: ~206 m/s delta-V.** Genuine tactical maneuvering capability.

**Constraints:** Exhaust temperatures cook unarmored personnel and damage hull surfaces. Powered armor required for the operator. Cold gas backup mandatory for close-quarters work. Safety margins required to avoid crisping friendlies.

### Setting Hierarchy

| Tier | Propulsion | Use Case |
|------|-----------|----------|
| Civilian | Cold gas nitrogen or argon | Cheap, simple, adequate for worksite EVA |
| Standard military | Hydrogen heated by electrolyzer waste heat | Excellent performance, dual-use with regenerative CO2 system |
| Special operations | Dedicated arcjet (H2, ~950s) | Serious maneuvering, significant battery cost |
| Powered armor | Metallic hydrogen (~1600s) | Maximum performance, requires armor for thermal protection |
| Close-quarters / backup | Cold gas (any) | No thermal signature, safe near structures and personnel |

### Delta-V Reference (Mass-Limited, 120 kg System)

| Propellant | Mass | Method | Delta-V |
|------------|------|--------|---------|
| 2 kg N2 | Cold gas | ~13 m/s |
| 2 kg H2 | Cold gas | ~44 m/s |
| 2 kg H2 | Electrolyzer-heated | ~70 m/s |
| 2 kg H2 | Arcjet | ~155 m/s |
| 2 kg mH2 | Recombination | ~206 m/s |

For comparison, NASA's MMU provided ~25 m/s total. The SAFER rescue unit: ~3 m/s.

---

## EVA Combat Maneuvering

### Primary Maneuvering: Poles and Tethers

Propulsive maneuvering is reserved for open-space transit, emergency repositioning, and situations where stealth is irrelevant. The primary combat maneuvering system is mechanical — poles, tethers, and surface grip — providing fast, silent, zero-signature movement along and between structures.

### Poles

Telescoping nanotube poles for pushing off structures. Limiting factor is buckling (compressive failure by bowing), not tensile strength. Critical buckling load drops with the square of length.

| Specification | Mass/Meter | 10m Weight | Buckling Limit | Use Case |
|--------------|-----------|-----------|---------------|----------|
| 2cm hollow, 1mm wall | ~78g | ~0.8 kg | ~250N | Standard pushoffs, repositioning |
| 3cm hollow, 2mm wall | ~230g | ~2.3 kg | ~1700N | Aggressive combat pushoffs at 20m reach |

### Tethers

Nanotube tethers on belt-mounted reels with grapple heads. Tension has no buckling limit.

- 1mm diameter nanotube cord: ~1 g/m, handles ~39,000N (enough to tow a small vehicle)
- 100m tether: 100 grams on a belt reel

### Surface Grip (Pole Tips and Grapples)

Hull materials are not guaranteed to be ferromagnetic — aluminum, titanium, and carbon composites are common. Electromagnetic grapples are unreliable as a universal solution.

**Grip systems:**

- **Mechanical claws / hardened spikes:** Nanotube-tipped articulated claws that bite into hull surface micro-irregularities. Reliable on most materials. Downside: leaves marks (stealth concern). Reduced effectiveness on hardened military hulls designed to resist grip.
- **Gecko adhesion (van der Waals pads):** Dry adhesive pads using microscopic hair structures. ~10 N/cm² on smooth surfaces regardless of material. A 100 cm² pad holds a person in microgravity trivially. Leaves no trace, releases by peeling. Limitation: performance degrades on dusty, frosted, or heavily pitted surfaces.
- **Eddy current braking:** Works on any conductive surface (not just ferromagnetic). Spinning magnet array induces resistive eddy currents. Only produces force during relative motion — good for controlled deceleration onto a hull, useless for static anchoring.
- **Harpoon / spike anchors:** For tether attachment on structural surfaces where damage is acceptable.

**Standard loadout:** Hybrid grapples with gecko pads for stealth operations, mechanical claws for reliability, mission-dependent selection. Pole tips use the same dual-mode system.

### Tactical Picture

Boarding troops maneuver primarily by pole-pushing off structure and tether-swinging between attachment points. Fast, silent, zero thermal signature, zero propellant consumption. Cold gas thrusters for fine adjustment and crossing open gaps. Metallic hydrogen for rapid repositioning or emergency maneuvers when stealth is irrelevant.

The resulting combat style resembles aggressive rock climbing crossed with spider-silk swinging rather than jetpack flight.

---

## EMP / Flux Compression Generators

### Physics of EMP in Space

**Nuclear EMP (HEMP):** Requires atmospheric interaction. Gamma rays hit atmospheric molecules via Compton scattering; freed electrons spiral in a planetary magnetic field, generating the pulse. No atmosphere = no amplification. Not relevant to station/ship combat.

**Non-nuclear EMP (flux compression generators, Marx generators, vircators):** Generate EM pulses directly from electrical discharge. Work perfectly in vacuum — electromagnetic waves propagate better in vacuum than atmosphere.

### How a Flux Compression Generator Works

1. Copper coil wrapped around a metal tube packed with explosives (or pressurized gas — see below)
2. Seed capacitor energizes the coil, creating a magnetic field
3. Explosive/gas detonation expands the tube outward into the coil
4. Magnetic flux is conserved: as inductance drops (coil compressed), current rises proportionally
5. Peak current reaches tens of megaamps for microseconds
6. Device destroys itself (one-shot)

**Coil geometry is the primary driver of effectiveness,** not explosive volume. More turns, longer coil, larger initial inductance = more flux = higher peak current = stronger pulse. The explosive charge just needs to be sufficient to expand the tube uniformly and fast enough that resistive losses don't eat the current before peak compression.

### Reusable Devices

Capacitor bank discharged through pulse-forming network into antenna. Explosives store 50–500x more energy per kg than capacitors, so a reusable device with comparable output to a backpack FCG is furniture-sized minimum, plus requires a power supply and recharge time between pulses.

### Effective Range

**Inside a metal hull:** The hull acts as a reflective cavity. EM energy cannot escape and bounces internally until absorbed by electronics and cabling. Cable runs thread through every compartment, acting as antennas that couple the pulse into physically distant systems. A backpack-sized FCG effective to 15–30m in open space could affect most systems in an ISS-sized metal structure detonated inside the hull.

**Inside rock/asteroid stations:** Rock is a poor EM reflector. Behaves like free-space propagation. Internal metal infrastructure (conduits, supports, pipes) still couples the pulse into connected systems, but no cavity amplification. Effective radius for unhardened electronics:

| Device | Approximate Radius (Free Space / Rock) |
|--------|-------|
| Backpack FCG (~15 kg) | 15–30m |
| Large suitcase FCG (~40 kg) | 30–60m |
| Shipping container (one-shot) | Hundreds of meters |
| Shipping container (reusable) | 50–100m per pulse, minutes between shots |

### Improvised Construction (The Security Problem)

A flux compression generator can be built entirely from station life-support hardware. No component triggers explosives detection.

**Materials:**
- Water (electrolyzed into hydrogen and oxygen)
- Two hyper-compression gas tanks (ubiquitous — every vac suit carries them)
- Copper wire
- A small battery and seed capacitor
- An airtight metal tube (the FCG body)
- A timer, a filament or spark igniter

**Procedure:**
1. Electrolyze water into H2 and O2, capture into separate hyper-compression tanks
2. Place tanks inside a metal tube wrapped with copper coil
3. Rig timed valve release to mix gases inside the tube
4. Rig slightly longer timer on ignition source (heated filament or spark)
5. Weld tube shut

**H2/O2 as ersatz explosive:** Stoichiometric H2/O2 detonates at ~2,840 m/s (faster when compressed). Slower than military high explosives (6,000–8,000 m/s) but adequate for flux compression. Energy density ~3.4 MJ/kg vs ~4.6 MJ/kg for TNT. The result is a less efficient but fully functional FCG.

**Residual tank pressure** adds to detonation energy — at 10,000 bar, the tanks themselves contain ~2 MJ of stored mechanical energy (hand-grenade equivalent), which supplements the chemical detonation.

**Coil geometry determines effectiveness more than charge size.** A device disguised as a structural pipe (long, skinny, many coil turns) would be more effective than a compact suitcase design — and harder to detect in a station full of pipes.

### Countermeasures

The corporations cannot ban water, compressed gas tanks, or copper wire without shutting down station operations. Viable countermeasures are limited to:

- **Surveillance:** Monitoring for electrolysis activity and unusual fabrication (expensive, intrusive)
- **Internal Faraday caging** of critical systems (expensive, unlikely on small/cheap stations)
- **Deterrence:** Reliance on enforcer reprisal as psychological prevention (consistent with established Solarian Union enforcement doctrine)

This represents a nearly unsolvable security vulnerability for the Solarian Union, particularly on small stations where hardening is uneconomical and the population has the strongest motivation for sabotage.

---

## Pressure Vessel Stored Energy (The Pipe Bomb Problem)

### Stored Energy at Setting Pressures

The energy stored in a compressed gas tank scales roughly with P×V.

| Tank | Pressure | Stored Energy | Equivalent |
|------|----------|--------------|------------|
| 2L | 10,000 bar | ~2 MJ | Half a kg of TNT / hand grenade |
| 2L | 30,000 bar | ~4–6 MJ | Couple pounds of TNT |

**Every person in a vac suit is carrying grenade-equivalent stored energy.**

### Failure Modes

Nanotube composite tanks do not fail like steel pipes. Composite pressure vessels fail by **crack propagation** — they split and vent directionally rather than fragmenting into shrapnel. Designed to leak-before-burst. Centuries of refinement make accidental catastrophic failure vanishingly rare.

The energy release from a composite failure is still violent (supersonic gas jet that can cut through flesh) but it's not a fragmentation event.

**Deliberate sabotage** — scoring the shell, weakening one spot — directs all stored energy through the failure point. Improvised shaped charge. Small arms fire hitting a tank in a firefight produces similar directed failure.

### Implications

- Station architecture must account for routine presence of grenade-energy pressure vessels: blast-resistant bulkheads, automatic compartment sealing
- Cultural norms around where fully charged high-pressure tanks can be brought
- Combat in and around suited personnel has inherent explosive risk from tank rupture
- Damaged suits are not just a decompression hazard but a blast hazard
