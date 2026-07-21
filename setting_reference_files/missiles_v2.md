# Missile Systems Reference

## Architecture

Two-stage design. A fusion-heated metallic hydrogen bus delivers torplets to the engagement zone. Torplets separate during terminal approach and close on their own MH drives.

The bus carries a D-He3 fusion reactor, MH propellant tanks, guidance and ECM systems, and a payload of MH-propelled submunitions (torplets). The reactor heats MH propellant through a magnetic nozzle, achieving exhaust velocities far beyond MH recombination alone. The torplets are self-contained weapons: Casaba Howitzer or bomb-pumped laser warhead, guidance package, MH propellant, and cold-gas maneuvering thrusters.

No shared propulsion between stages. The bus burns until its propellant is exhausted, then coasts. Torplets separate near the target and execute independent terminal attacks.

## Key Parameters

| Parameter | Value | Tag |
|-----------|-------|-----|
| Bus propellant | Metallic hydrogen | |
| Bus heating | D-He3 fusion, magnetic nozzle | |
| MH base exhaust velocity | 16,700 m/s | [HARD] |
| Bus exhaust velocity (fusion-heated) | 67–200 km/s (config dependent) | [ENG] |
| Reactor specific power (missile-grade) | 5 MW/kg | [ENG] |
| Radiator specific power (ablative) | 2.24 MW/kg | [ENG] |
| Waste heat fraction | 5% | |
| Torplet propellant | Metallic hydrogen (unheated) | |
| Torplet exhaust velocity | 16,700 m/s | [HARD] |
| Torplet ΔV | 30–36 km/s | |
| Torplet mass ratio | 6.0–8.6 (at 30–36 km/s) | |
| Tank mass fraction | 4% of propellant (CNT composite) | [ENG] |

## Missile-Grade Reactor

Ship reactors carry mass for crew shielding, redundancy, maintenance access, multi-mode output, conservative thermal margins, and decade-scale design life. A missile reactor fires once, for hours, then ceases to exist. The engineering philosophy is closer to a solid rocket motor than a naval powerplant.

**Neutron shielding [HARD]:** D-He3 side reactions produce ~1% of energy as neutrons. Over a 20-year ship life, cumulative neutron damage to superconductors and structural materials drives significant shielding mass. Over a 2-hour missile burn, cumulative fluence is five orders of magnitude lower. Shielding is stripped entirely. On a ship reactor this is 20–30% of total mass.

**Redundancy [ENG]:** Ship reactors need backup cooling loops, fault-tolerant magnet circuits, parallel systems. Missile: single-string everything. Failure equals missile loss — acceptable for expendable ordnance. Saves 15–25% of mass.

**Overdriven magnets [ENG]:** HTS superconductors are rated for degradation over years of operation. A missile runs its magnets at 2–3× rated current density, accepting degradation after tens of hours instead of tens of thousands. Higher current density means smaller magnet cross-section means lighter coils. This is the single biggest mass lever — magnets are typically the heaviest reactor component.

**No power conversion [ENG]:** A ship reactor must produce electricity for weapons, life support, and ship systems. A missile reactor's only job is to heat propellant. Fusion plasma transfers energy directly to propellant in the magnetic nozzle. No turbines, no generators, no power conditioning.

**Minimal structure [ENG]:** The reactor experiences only the missile's own acceleration (0.5–1.5g). Ship reactors must survive combat maneuvers, weapons shock, and vibration over decades.

| Category | Specific Power | Tag | Basis |
|----------|---------------|-----|-------|
| Ship reactor | 1–2 MW/kg | [ENG] | Long-life, crew-rated, redundant |
| Missile reactor | 3–5 MW/kg | [ENG] | No shielding, no redundancy, overdriven, direct-drive |
| Missile reactor (aggressive) | 5–10 MW/kg | [SPEC] | Near-theoretical material limits |
| Hard ceiling | ~10 MW/kg | [HARD] | Magnetic pressure containment limit |

Setting baseline for missile reactors: **5 MW/kg [ENG].**

## Radiator

Missile radiators are designed to ablate. A ship radiator must survive years of thermal cycling and potential combat damage. A missile radiator must reject waste heat for hours. It is sized to slowly vaporize over the burn duration, shedding mass as it erodes. This allows roughly 40% higher thermal rejection per kilogram compared to ship radiators rated for sustained service.

Setting baseline: **2.24 MW/kg [ENG]** (ship baseline 1.6 MW/kg × 1.4 ablative factor).

At 5% waste heat, the radiator is a small fraction of total power system mass. For a 10 GW reactor, waste heat is 500 MW, requiring ~223 kg of radiator. The reactor itself is 2,000 kg. Radiator mass is consistently 3–5% of power system mass across all designs.

## Exhaust Velocity Optimization

### The Derivative Problem

For a constant-thrust power-limited rocket where power system mass scales linearly with exhaust velocity, the bus ΔV optimality condition is:

**ln(R) = 1 − β·R**

where R = M/m_dry (mass ratio) and β = m_fixed/M (fixed mass fraction — torplets, guidance, tanks). When β → 0, optimal R = e ≈ 2.718. As payload fraction grows, optimal R drops below e.

The linearity holds because at constant thrust F₀, required fusion power P = F₀·v_e/2 scales linearly in v_e. Reactor mass = P/α_R follows the same scaling. The slope constant:

k = F₀ / (2 × η × α_R) ≈ 0.063 kg·s/m (for a 27t missile at 1g)

Meaning each additional km/s of exhaust velocity costs roughly 63 kg of reactor.

### Burn Time as the Binding Constraint

At 5 MW/kg, the reactor is light enough that the mass-ratio optimum pushes v_e toward physically implausible values (500+ km/s, plasma temperatures approaching fusion conditions). The optimizer has no internal mass-ratio penalty sufficient to cap v_e.

The practical constraint is burn time. Higher v_e means lower mass flow at constant thrust, meaning longer burns to exhaust the same propellant mass. Tactical missiles must complete their burns within operationally useful windows:

| Role | Max Burn | Basis |
|------|----------|-------|
| Tactical (ship-launched) | 2 hours | Jovian moon system engagement timelines |
| Strategic (platform-launched) | 4 hours | Extended-range strike, full system coverage |

The burn time cap forces v_e downward. At 1g with a 2-hour cap, the optimizer settles at 90–140 km/s depending on torplet payload. At 0.75g with a 4-hour cap, v_e reaches 140–200 km/s.

### Flow-Rate Cooling

MH propellant can absorb ~0.4 MJ/kg of waste heat before recombination risk becomes unacceptable [ENG]. H2 can absorb ~7 MJ/kg.

At optimal operating points (v_e > 67 km/s), propellant mass flow is too low for flow-rate cooling to absorb significant waste heat regardless of propellant choice. The crossover where H2 cooling saturates is only 16.7 km/s — far below any fusion-heated operating point. Flow-rate cooling is irrelevant to the design.

MH propellant wins over H2 by 1–2% because MH's recombination energy (providing 16,700 m/s for free) reduces the required reactor power. The saved reactor mass slightly exceeds any radiator savings from H2's superior cooling.

**Propellant selection: MH for both stages.**

### Exhaust Temperature

| v_e | Exhaust Temperature | Tag |
|-----|--------------------|----|
| 67 km/s | ~0.1 MK (10 eV) | [ENG] |
| 100 km/s | ~0.3 MK (26 eV) | [ENG] |
| 140 km/s | ~0.6 MK (50 eV) | [ENG] |
| 200 km/s | ~1.2 MK (100 eV) | [ENG] |

D-He3 fusion operates at 60–100 keV (700–1,200 MK). Bus exhaust temperatures are two orders of magnitude cooler than the fusion plasma itself. Magnetic nozzle containment is the same technology applied to an easier problem.

## Maneuvering

### Bus Stage

The fusion drive's magnetic nozzle provides thrust vectoring via field shaping — no mechanical gimbal required. The same hardware that confines and directs the fusion-heated exhaust can reshape the field to steer thrust. Mass penalty is negligible (integrated into existing nozzle hardware). Provides pitch and yaw control during powered flight.

Roll control and coast-phase attitude: cold-gas RCS, ~20–30 kg. No dedicated translation thrusters needed.

### Torplets

MH nozzle with mechanical gimbal or differential propellant injection for pitch/yaw. Cold-gas micro-thrusters for roll and fine pointing. Total maneuvering system mass: ~5–8 kg per torplet, included in dry mass estimates.

### Gearing

Thrust vectoring allows the missile to trade exhaust velocity for thrust (gearing down) at launch. This gets the missile off the rail faster and opens distance from the launching ship sooner — relevant when the launcher is under fire. The tradeoff is lower propellant efficiency: more mass expelled per unit impulse.

Gearing does not change total ΔV (constant v_e is optimal in free space). It is a tactical tool for launch conditions, not a design multiplier.

## Torplet Design

### Mass Budget (Representative)

| Component | 25 kg dry | 35 kg dry | 50 kg dry | 60 kg dry | 75 kg dry |
|-----------|----------|----------|----------|----------|----------|
| Casaba warhead | 12–15 kg | 18–22 kg | 25–30 kg | 30–38 kg | 40–50 kg |
| Guidance/sensors | 5–6 kg | 6–8 kg | 8–10 kg | 10–12 kg | 12–14 kg |
| Gimbal + cold gas RCS | 3–4 kg | 4–5 kg | 5–7 kg | 6–8 kg | 7–9 kg |
| Structure/tanks | 3–4 kg | 4–5 kg | 5–7 kg | 6–8 kg | 7–10 kg |
| ECM | 2–3 kg | 3–4 kg | 4–5 kg | 4–5 kg | 5–7 kg |

### ΔV and Mass Ratio

| Torplet ΔV | Mass Ratio | 35 kg dry → wet | 50 kg dry → wet | 75 kg dry → wet |
|-----------|-----------|-----------------|-----------------|-----------------|
| 30 km/s | 6.0 | 211 kg | 301 kg | 452 kg |
| 33 km/s | 7.2 | 252 kg | 361 kg | 541 kg |
| 36 km/s | 8.6 | 302 kg | 432 kg | 648 kg |
| 40 km/s | 11.0 | 384 kg | 549 kg | 824 kg |
| 50 km/s | 20.0 | 699 kg | 998 kg | 1,498 kg |

The 30–36 km/s range is the design sweet spot. Below 30 km/s, torplets lack maneuvering budget for terminal evasion. Above 36 km/s, mass ratio climbs steeply — going from 36 to 50 km/s doubles the torplet wet mass for diminishing tactical return. Each additional km/s of torplet ΔV costs roughly 3–5 km/s of bus ΔV depending on configuration.

### Torplet Terminal Phase

Torplets separate from the bus during the final approach. Each torplet executes independent terminal maneuvers — corkscrewing, jinking, ECM deployment — to defeat point defense tracking. The bus continues on its ballistic trajectory (or may carry its own ECM to contribute to the jamming environment).

Torplet closure velocity equals the bus coast velocity (100–200 km/s) plus or minus the torplet's own ΔV expenditure on course correction. Terminal acceleration depends on remaining propellant — a torplet that has spent most of its ΔV on course changes arrives with less jink budget, making it easier to track. Torplet ΔV allocation between approach correction and terminal evasion is a fire-control decision made by the launching ship or the torplet's own SAI.

## Design Table

### Tactical Missiles — Ship-Launched, 2 Hour Max Burn

| Name | Mass | a₀ | Torplets | v_e | Bus ΔV | MR | P | m_pow | m_torp | m_prop | Burn | a_f | @3hr | @6hr |
|------|-----:|---:|---------|----:|-------:|---:|--:|------:|-------:|-------:|-----:|----:|-----:|-----:|
| Dart | 8t | 1.50g | 4×25@30 | 199 km/s | 151 km/s | 2.1 | 12.3 GW | 2,728 kg | 603 kg | 4,249 kg | 120 min | 3.2g | 1.1 Gm | 2.7 Gm |
| Stiletto | 10t | 1.25g | 4×35@30 | 130 km/s | 148 km/s | 3.1 | 8.2 GW | 1,831 kg | 844 kg | 6,803 kg | 120 min | 3.9g | 1.1 Gm | 2.7 Gm |
| Javelin | 12t | 1.00g | 6×25@30 | 92 km/s | 135 km/s | 4.4 | 5.5 GW | 1,219 kg | 904 kg | 9,257 kg | 120 min | 4.4g | 1.0 Gm | 2.4 Gm |
| Harpoon | 15t | 1.00g | 6×35@30 | 92 km/s | 134 km/s | 4.3 | 6.9 GW | 1,535 kg | 1,266 kg | 11,490 kg | 120 min | 4.3g | 1.0 Gm | 2.4 Gm |
| Trident | 15t | 1.50g | 4×50@33 | 200 km/s | 149 km/s | 2.1 | 23.1 GW | 5,128 kg | 1,443 kg | 7,864 kg | 119 min | 3.2g | 1.1 Gm | 2.7 Gm |
| Pike | 18t | 1.00g | 8×30@30 | 91 km/s | 136 km/s | 4.4 | 8.2 GW | 1,823 kg | 1,447 kg | 13,924 kg | 120 min | 4.4g | 1.0 Gm | 2.4 Gm |
| Longbow | 20t | 1.00g | 8×35@33 | 94 km/s | 131 km/s | 4.0 | 9.4 GW | 2,087 kg | 2,020 kg | 15,041 kg | 120 min | 4.0g | 0.9 Gm | 2.4 Gm |
| Claymore | 25t | 1.00g | 10×30@30 | 90 km/s | 139 km/s | 4.7 | 11.2 GW | 2,486 kg | 1,808 kg | 19,668 kg | 120 min | 4.7g | 1.0 Gm | 2.5 Gm |
| Broadside | 27t | 0.75g | 10×35@36 | 67 km/s | 105 km/s | 4.8 | 6.5 GW | 1,454 kg | 3,022 kg | 21,417 kg | 120 min | 3.6g | 0.8 Gm | 1.9 Gm |
| Tempest | 27t | 1.00g | 10×35@36 | 95 km/s | 129 km/s | 3.9 | 12.8 GW | 2,853 kg | 3,022 kg | 20,073 kg | 120 min | 3.9g | 0.9 Gm | 2.3 Gm |
| Salvo | 27t | 1.00g | 16×25@30 | 92 km/s | 135 km/s | 4.3 | 12.4 GW | 2,753 kg | 2,411 kg | 20,756 kg | 120 min | 4.3g | 1.0 Gm | 2.4 Gm |
| Lancer | 27t | 1.25g | 8×50@36 | 137 km/s | 141 km/s | 2.8 | 23.6 GW | 5,240 kg | 3,454 kg | 17,362 kg | 120 min | 3.5g | 1.0 Gm | 2.5 Gm |
| Sledgehammer | 27t | 1.50g | 6×60@36 | 200 km/s | 144 km/s | 2.1 | 41.5 GW | 9,231 kg | 3,108 kg | 13,856 kg | 116 min | 3.1g | 1.1 Gm | 2.6 Gm |
| Avalanche | 35t | 1.00g | 12×35@33 | 91 km/s | 136 km/s | 4.4 | 15.9 GW | 3,541 kg | 3,030 kg | 27,095 kg | 120 min | 4.4g | 1.0 Gm | 2.4 Gm |
| Thunderbolt | 35t | 1.00g | 8×60@36 | 96 km/s | 128 km/s | 3.8 | 16.8 GW | 3,724 kg | 4,144 kg | 25,848 kg | 120 min | 3.8g | 0.9 Gm | 2.3 Gm |

### Strategic Missiles — Platform/Station-Launched, 4 Hour Max Burn

| Name | Mass | a₀ | Torplets | v_e | Bus ΔV | MR | P | m_pow | m_torp | m_prop | Burn | a_f | @6hr | @10hr |
|------|-----:|---:|---------|----:|-------:|---:|--:|------:|-------:|-------:|-----:|----:|-----:|------:|
| Leviathan | 50t | 0.75g | 10×50@36 | 140 km/s | 198 km/s | 4.1 | 26.7 GW | 5,941 kg | 4,317 kg | 37,829 kg | 240 min | 3.1g | 2.8 Gm | 5.7 Gm |
| Behemoth | 50t | 0.75g | 8×75@36 | 144 km/s | 192 km/s | 3.8 | 27.5 GW | 6,105 kg | 5,180 kg | 36,841 kg | 240 min | 2.8g | 2.8 Gm | 5.5 Gm |
| Colossus | 75t | 0.50g | 12×75@36 | 87 km/s | 145 km/s | 5.3 | 16.2 GW | 3,611 kg | 7,770 kg | 60,787 kg | 240 min | 2.6g | 2.1 Gm | 4.2 Gm |
| Titan | 75t | 0.75g | 10×75@36 | 139 km/s | 199 km/s | 4.2 | 39.9 GW | 8,875 kg | 6,475 kg | 56,970 kg | 240 min | 3.1g | 2.9 Gm | 5.7 Gm |
| Armageddon | 100t | 0.50g | 16×75@36 | 87 km/s | 145 km/s | 5.3 | 21.7 GW | 4,820 kg | 10,361 kg | 80,981 kg | 240 min | 2.6g | 2.1 Gm | 4.2 Gm |

### Reading the Table

Columns: v_e = bus exhaust velocity (fusion-heated MH). Bus ΔV = bus stage only. MR = bus mass ratio. P = reactor fusion power. m_pow = reactor + radiator mass. m_torp = total torplet payload mass (all torplets wet). m_prop = bus MH propellant. Burn = bus burn duration. a_f = acceleration at burnout. @3hr/@6hr/@10hr = total distance reached at that flight time (burn + coast). Torplet notation: count × dry_mass @ torplet_ΔV.

Bus guidance, ECM, and RCS mass: 250 kg (tactical), 400 kg (50t strategic), 600 kg (75t+ strategic). Tank mass fraction: 4% of propellant. These are included in all mass budgets but not broken out separately.

## Design Patterns

### Acceleration Drives Everything

At 5 MW/kg reactor specific power, acceleration is the primary design lever. Higher acceleration means higher thrust, which means higher required reactor power, which means heavier power system. The optimizer responds by lowering v_e — burning propellant less efficiently but faster. Lower acceleration allows higher v_e and longer burns, producing more bus ΔV from the same propellant mass.

The tradeoff is direct: acceleration determines the missile's character.

| Acceleration | Character | v_e Range | Bus ΔV Range | Burn Time |
|-------------|-----------|----------|-------------|-----------|
| 0.50g | Long-range, high ΔV, slow launch | 67–90 km/s | 100–145 km/s | Burns full window |
| 0.75g | Extended-range strike | 90–145 km/s | 105–200 km/s | Burns full window |
| 1.00g | Standard fleet torpedo | 90–96 km/s | 128–139 km/s | Burns full window |
| 1.25g | Fast fleet torpedo | 130–140 km/s | 141–148 km/s | Near full window |
| 1.50g | Sprint / precision strike | 199–200 km/s | 144–151 km/s | Near full window |
| 2.00g+ | Short-range sprint | Natural optimum, under window | Diminishing returns | Burn time slack |

Above ~2g, the missile exhausts its natural ΔV budget before the burn time cap matters. Acceleration itself becomes the binding constraint as in the uncapped optimization. Diminishing returns set in.

### Light vs Heavy

Light tactical missiles (8–15t) carry few torplets (4–6) but are cheap enough to fire in volume. A corvette carrying a rack of Darts can put 16–24 torplets into the battlespace from four to six missiles. Each missile is individually survivable — losing one costs 4 torplets, not 10.

Heavy tactical missiles (27–35t) carry more torplets per missile (8–16) and benefit from the mass scaling: a larger fraction of total mass is available for propellant and payload after fixed costs (guidance, ECM, structural minimum) are absorbed. The Claymore at 25t gets 10 torplets at 139 km/s coast. The Javelin at 12t gets 6 torplets at 135 km/s. The heavier missile is more efficient per torplet delivered.

Strategic missiles (50–100t) push this further. The Titan at 75t delivers 10 heavy torplets (75 kg dry — serious Casabas with real standoff range) at 199 km/s coast velocity. Nothing in the tactical table matches that combination of lethality and speed. These are carrier-killers and station-assault weapons, not fleet ordnance.

### Saturation vs Precision

The Salvo (16×25 kg @ 30 km/s) and the Tempest (10×35 kg @ 36 km/s) are the same missile — 27t, 1g, same bus. They represent a doctrinal choice.

The Salvo delivers 60% more torplets at a slightly higher coast velocity. Each torplet is lighter (25 kg dry — roughly 12 kg warhead) with less terminal ΔV. It is a saturation weapon designed to overwhelm point defense through sheer volume. If the defender's PD capacity is 750 kills, flooding it with 160 torplets from 10 Salvos is a different problem than 100 torplets from 10 Tempests.

The Tempest delivers fewer, individually more capable torplets. Each carries a heavier warhead (18–22 kg), more terminal ΔV (36 km/s vs 30 km/s for longer jink budget), and by extension better ECM. Against a defender with robust PD, the torplets that leak through are more likely to be individually lethal.

Solarian doctrine (mass production, expendable ordnance, quantity over quality) probably favors Salvo-type designs. Chanduran doctrine (AI-optimized precision, reusable assets, quality over quantity) probably favors something between the Tempest and the Lancer.

## Engagement Geometry

### Closure Velocity and PD Windows

The bus burns out and the missile coasts. Torplets separate during the final approach. The bus coast velocity is effectively the torplet closure velocity — the torplets use their ΔV for terminal maneuvering, not additional acceleration along the approach vector.

From the laser reference document, the BB main battery's outer engagement capacity (750 torplet kills from 10,000 km) assumes 130 km/s closure. Actual closure velocities from these designs:

| Design Class | Coast Velocity | PD Window from 10,000 km | Relative Capacity |
|-------------|---------------|------------------------|-------------------|
| Broadside (0.75g, slow) | 105 km/s | 95 s | ~125% of baseline |
| Standard (1g) | 129–139 km/s | 72–78 s | ~95–100% of baseline |
| Fast (1.25–1.5g) | 141–151 km/s | 66–71 s | ~85–90% of baseline |
| Strategic (0.75g, 4hr) | 192–199 km/s | 50–52 s | ~65–70% of baseline |

Strategic missiles at 200 km/s give the defender roughly two-thirds the engagement window of the standard tactical case. This directly reduces the number of torplets killed before Casaba standoff range — from ~750 to ~490 for a single BB main battery. The torplets themselves are also heavier (75 kg Casabas vs 25–35 kg), meaning each leaker does more damage.

### Range Bands

Distance covered is burn distance plus coast distance. Burn distance is roughly half the bus ΔV times burn duration (missile accelerates from zero to coast velocity). Coast distance is coast velocity times remaining flight time.

| Role | Flight Time | Typical Range | Covers |
|------|------------|---------------|--------|
| Short engagement | 3 hours | 0.8–1.1 Gm | Adjacent moon orbits |
| Medium engagement | 6 hours | 1.9–2.9 Gm | Cross-system (moon system diameter) |
| Long engagement | 10 hours | 3.4–5.7 Gm | Full system, approaching fleet |

Strategic missiles at 4-hour burn reach farther at the same flight time because they coast faster. A Titan covers 2.9 Gm in 6 hours versus 2.3 Gm for a Tempest. The difference is one to two additional hours of flight time for the same tactical reach — or equivalently, the strategic missile can be launched later and still arrive on schedule.

## Thermal Management

### Bus

The reactor runs continuously during the burn. At 5% waste heat, a 10 GW reactor produces 500 MW of waste heat — rejected through ablative radiators sized for the burn duration.

At 2.24 MW/kg, 500 MW requires 223 kg of radiator. This mass is small compared to the reactor (2,000 kg at 5 MW/kg) and trivial compared to total missile mass. The radiator is not a significant design driver for missiles.

The reactor itself operates at high temperature (the fusion plasma is magnetically confined, not in contact with structure). The magnetic nozzle is the primary thermal challenge — it must contain 0.1–1.2 MK exhaust plasma without physical contact. This is the same technology as the fusion reactor's confinement system, applied to cooler plasma. [ENG]

### Torplets

MH torplets have no reactor and minimal waste heat. The MH recombination nozzle operates for tens of seconds to minutes of total burn time. Thermal management is handled by the nozzle's thermal mass — it absorbs heat during the brief burn and radiates passively during coast. No dedicated radiator.

## Open Questions

- Torplet separation timing and doctrine — how far from the target do torplets separate? What triggers separation? How does the bus contribute to the jamming environment post-separation?
- Bus survivability — the bus is a large, hot target during its burn. How vulnerable is it to long-range laser fire during approach? Does the bus need its own ECM, or is it expendable once the burn is complete?
- Salvo coordination — multiple missiles from the same ship arriving simultaneously vs staggered. Timing windows for convergent attacks from different bearings.
- Counter-missile interaction — how do defensive counter-missiles engage a fusion-bus torpedo? The bus is accelerating at 1g+ and is much larger and harder to kill than a torplet. Do you target the bus or wait for torplet separation?
- Reactor startup time — how long does a cold missile reactor take to reach operating power? This affects launch doctrine — are missiles kept in standby with warm reactors, or started from cold?
- ECM integration — does the bus carry dedicated jamming systems? Can its fusion drive plume serve as an IR/radar obscurant for the torplets behind it?
- Magazine depth — how many of each design does a battleship, carrier, or corvette carry? What is the reload cycle?
- Antimatter warheads — post-Demetry Ring, do any designs get antimatter-enhanced Casabas? What yield increase justifies the cost?
- Tracking degradation model — the laser reference document identifies jamming (fleet lasers and trailing jammer drones exploiting R² vs R⁴ asymmetry) as a modifier to effective dwell. This model needs to be formalized and cross-referenced with torplet ECM budgets.

---

*Document compiled from missile design optimization sessions. All designs computed from first principles using the stated parameters. Subject to revision as setting development continues.*
