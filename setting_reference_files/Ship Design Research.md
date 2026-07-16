# Ship Mass and Size Analysis for a Hard-SF Warship Setting: Drive, Thermal, and Combat Systems

## TL;DR
- **Heat rejection, not thrust or firepower, sets every hard limit in this setting.** A D-³He "geared" fusion warship at ~10 milligee cruise and 8:1 mass ratio is buildable on defensible physics, but its sustained reactor output is capped by high-temperature radiator area (~816 kW rejected per m² at 2000 K, ~2–6 kg/m²), and its combat endurance is capped by lithium heat-sink mass (~4.6 MJ/kg fully melted-and-heated). These two numbers, not weapon count, decide ship size.
- **The drive is sound; the "gearing" (afterburner) is the key lever.** Pure-plasma D-³He exhaust velocity is ~8.9% c physically but is throttled to ~2,300 km/s (85% nozzle efficiency, Atomic Rockets figure) at 1.82 MW/N; afterburner injection of metallic hydrogen trades that down to hundreds of km/s for combat thrust. At Ve ≈ 2,300 km/s and mass ratio 8, Δv ≈ 4,780 km/s in pure cruise — enormous — so the real design driver is thrust and heat, not Δv.
- **Ship classes are internally consistent at ~5,000–90,000 t dry.** A corvette lands near ~5,000 t dry, a battleship near ~90,000 t dry, dominated by radiators, magnetic nozzle, tankage, and (on the battleship) flywheel/laser mass. Spin-polarized D-³He gives a robust ~1.5× cross-section boost and modest (~1–20%) propulsive-efficiency gain, but the often-quoted "90% neutron suppression" is a patent-level optimistic claim, not established physics.

## Key Findings

1. **Fusion fuel trade (D-³He vs ³He-³He vs p-p).** D-³He is the correct setting choice. It is the highest-performing *practically ignitable* aneutronic-ish fuel. ³He-³He is fully aneutronic but requires ~10× more difficult ignition (roughly billion-Kelvin plasma) and doubles the scarce-fuel demand; p-p fusion is physically impossible in a reactor (Lawson criterion off the chart — it relies on the weak force and only works in stellar cores).
2. **Δv is abundant; thrust is scarce.** At the setting's 8:1 mass ratio the rocket equation gives cruise Δv far beyond any in-system need, so propellant is effectively spent on *thrust* (afterburner) rather than *velocity*. This inverts the usual SF constraint.
3. **The master constraint is a two-loop thermal architecture.** A high-temperature refractory loop (1800–2500 K) sheds reactor waste heat and stays deployed in combat; a low-temperature loop (400–600 K) for electronics/life-support/lasers retracts behind armor, with combat heat dumped into lithium phase-change reservoirs. Endurance is literally measured in kilograms of lithium.
4. **B⁴ scaling makes 50 T reactors compact.** Fusion power density ∝ B⁴, so a sustained 50 T field (vs ITER's toroidal-field coils rated at a maximum 11.8 T) is ~300× the power density — the reactor core is small; its *mass* is dominated by magnets, structure, and shielding, and its *output* is throttled by radiators.
5. **Weapons are energy-storage problems, not reactor problems.** Lasers fire from CNT flywheel banks (defensibly ~2,000–3,000 Wh/kg with CNT rotors), not the reactor directly; the reactor recharges them over minutes. Missiles dominate offensive mass/volume budgets.

## Details

### 0. Method and epistemic key
Throughout I tag claims: **[HARD]** = textbook physics (rocket equation, Stefan-Boltzmann, energy storage); **[ENG]** = defensible engineering extrapolation from real studies (NASA/PPPL fusion, LDR radiators, CNT COPVs); **[SPEC]** = speculative but internally consistent (50 T sustained HTS, metastable metallic hydrogen, metamaterial emissivity). Primary reference is Atomic Rockets (projectrho.com) supplemented by NASA/PPPL/arXiv primary sources.

### 1. Fusion Fuel Trade Study

**Reaction energetics [HARD]** (from Atomic Rockets Fusion Fuel table, masses in u × 931.494 MeV):
- **D + ³He → ⁴He (3.6 MeV) + p (14.7 MeV):** 18.3 MeV/fusion, ~353 TJ/kg, **100% charged products** for the primary reaction. Pure-fusion exhaust velocity ~8.9% c.
- **³He + ³He → ⁴He + 2p:** 12.9 MeV/fusion, ~205 TJ/kg, fully aneutronic, exhaust velocity ~6.8% c.
- **p + p (proton-proton chain) → ⁴He:** 26.7 MeV but weak-force-mediated; Lawson criterion "off the top of the chart," ~11.7% c theoretical. Not reactor-feasible.

**Why D-³He wins.** Atomic Rockets quotes a D-³He reactor at 85% nozzle efficiency giving Ve = 0.0077 c = 2,300 km/s, requiring 1.82 MW/N of thrust, of which 0.36 MW is bremsstrahlung loss, 0.09 MW neutron loss (from parasitic D-D), and 1.37 MW enters the exhaust. Fuel burn is 0.17 mg/s D + 0.26 mg/s ³He per newton. D-³He's Lawson criterion is 16 (vs 500 for p-B11), the lowest of the aneutronic-class fuels.

**³He-³He tradeoff.** Gains: truly aneutronic (no D-D side reactions → no neutron embrittlement/activation, thinner shielding, longer reactor life). Losses: (a) ignition temperature roughly a billion Kelvin vs ~hundreds of millions for D-³He, driving bremsstrahlung losses up (brems ∝ T·Z² and rises with temperature); (b) it consumes *two* ³He nuclei per event — doubling demand for the setting's scarcest resource (producing one ton of helium-3 would require moving more than 100 million tons of lunar material at 10–20 ppb concentration — per USGS astrogeologist L. Keszthelyi via SpaceNews, "You have to process somewhere between 100,000 to 1 million tons [of regolith] to get a kilogram [of Helium-3]"; gas-giant mining assumed); (c) ~40% lower energy per fusion. Net: ³He-³He is a niche "clean" fuel for reactors that can afford the temperature, not a warship drive.

**p-p / hydrogen burning.** Physically the ultimate fuel (18 billion years of Type-I civilization per Atomic Rockets), but bremsstrahlung exceeds fusion power at any reactor-achievable density because the rate-limiting step is weak-force β⁺ decay. Even the CNO-catalyzed Bussard variant cannot beat bremsstrahlung. **Verdict: impossible as written; correctly excluded from the setting.**

**Spin-polarization effects on D-³He [ENG, with flags].** This is the setting's key enabling assumption and the literature is specific:
- **Cross-section enhancement: ~1.5× (+50%)** when D and ³He nuclear spins are aligned parallel. This is robust, tracing to R. M. Kulsrud, H. P. Furth, E. J. Valeo (Princeton Plasma Physics Lab) and M. Goldhaber (Brookhaven), *Phys. Rev. Lett.* 49, 1248 (published 25 October 1982): "Nuclear fusion rates can be enhanced or suppressed by polarization of the reacting nuclei. In a magnetic fusion reactor, the depolarization time is estimated to be longer than the reaction time." Both D-T and D-³He proceed via a spin-3/2 (⁵Li/⁵He) resonance; as Baylor et al., *Nucl. Fusion* 63, 076009 (2023) state, "if the initial spins are parallel, the reaction rate is enhanced by a factor of 3/2." The governing relation (Parisi, Diallo & Meschini, arXiv:2504.09869, 2025) is σ/σ̄ = 1 + (P_D·P_³He)/2, so full alignment → 1.5×. The *Maxwellian-averaged reactivity* boost is smaller (~+32% by analogy to D-T).
- **Neutron suppression: uncertain, do NOT claim 90%.** The parasitic D-D branches produce the neutrons and tritium. The often-cited "D-D suppressed by 1/20 (−90%)" comes from US Statutory Invention Registration H446 — a patent, not peer-reviewed. The peer-reviewed Quintet Suppression Factor is highly uncertain: predictions span QSF ≈ 0.1 (10× suppression) to 2.5 (enhancement), and **no experimental measurement yet exists** (the PolFusion experiment at PNPI Gatchina/Jülich is designed to settle it). Setting should treat strong neutron suppression as a *plausible engineering bet*, not settled science.
- **Power density: up to ~1.5× from polarization alone**, or up to ~3.6× including favorable D-D and secondary burn effects at 50 keV under optimistic QSF (arXiv:2504.09869). "Order of magnitude" gains quoted in that paper require optimistic QSF + fuel-ratio optimization + direct energy conversion stacked together.
- **Propulsive efficiency: +1–20%** (not >30%), from directing fewer charged products into the magnetic-nozzle loss cone (Bruhaug & Kish, arXiv:2108.01211, U. Rochester LLE). That paper separately reports up to **+45% fusion burn-up (gain)** and, for D-³He, neutron load falling to ~76% of unpolarized (≈24% reduction, ~3% shielding-mass saving). The dramatic ">40% neutron reduction" figure applies to D-T, not D-³He.

### 2. Mass Ratio and Δv

**Rocket equation [HARD]:** Δv = Ve · ln(MR).
- Pure-plasma cruise, Ve = 2,300 km/s, MR = 8: Δv = 2,300 · ln 8 = 2,300 · 2.079 = **4,782 km/s**.
- This is absurdly more than any interplanetary mission needs (a fast Earth–Jupiter transfer is tens to ~100 km/s). **Implication:** at MR 8 the ship is not Δv-limited; it can afford to "waste" most propellant as afterburner reaction mass for thrust.

**Afterburner "gearing" curve [HARD/ENG].** Adding cold propellant mass ṁ_p to the fusion exhaust conserves momentum/energy: for a fixed jet power P, thrust F = √(2·P·ṁ_total)·η and Ve = √(2P/ṁ_total)·η. Doubling total mass flow raises thrust by √2 and cuts Ve by √2. Representative gear points at fixed ~1.37 MW/N jet power budget:

| Mode                       | Ve (km/s) | Δv at MR 8 (km/s) | Relative thrust |
| -------------------------- | --------- | ----------------- | --------------- |
| Pure plasma (cruise)       | 2,300     | 4,780             | 1×              |
| Light afterburner          | 1,000     | 2,080             | ~2.3×           |
| Medium afterburner         | 500       | 1,040             | ~4.6×           |
| Heavy afterburner (combat) | 200       | 416               | ~11.5×          |
| Max thrust burst           | 100       | 208               | ~23×            |

Even in heavy-afterburner combat mode, Δv (~400 km/s) vastly exceeds tactical needs — confirming the drive is thrust-limited and heat-limited, never Δv-limited, at MR 8.

### 3. Cruise Performance — working backward from 10 milligee

**Target:** a = 10 mg = 0.0981 m/s². [HARD]
- Take a fully-fueled 60,000 t (6×10⁷ kg) battleship-scale ship. Required thrust F = m·a = 6×10⁷ · 0.0981 = **5.89×10⁶ N (5.9 MN)**.
- At pure-plasma 1.82 MW of fusion power per newton: P_fusion = 5.89×10⁶ N × 1.82 MW/N = **1.07×10¹³ W ≈ 10.7 TW**. This is the fusion thermal power, of which ~75% (per Atomic Rockets partition) is in charged particles → thrust, the rest brems/neutron loss.
- Propellant (fuel-only, pure plasma) mass flow: at 0.43 mg/s per N total fuel × 5.89×10⁶ N = 2.5 kg/s of D+³He.
- **As fuel burns, mass drops, so at constant thrust acceleration rises** — a 60,000 t ship falling to ~30,000 t doubles its acceleration to ~20 mg, exactly as the setting specifies.

For a smaller 5,000 t corvette at 10 mg: F = 4.9×10⁵ N, P_fusion ≈ 0.89 TW. **Scaling rule:** ~1.8 TW of fusion power per 10,000 t of ship at 10 mg cruise. This is the number the radiators must live with.

### 4. Heat Rejection — the Central Constraint

**Stefan-Boltzmann [HARD]:** Q/A = εσT⁴, σ = 5.67×10⁻⁸ W/m²K⁴.
- **High-temp radiator at 2000 K, ε = 0.9:** Q/A = 0.9 · 5.67×10⁻⁸ · (2000)⁴ = 0.9 · 5.67×10⁻⁸ · 1.6×10¹³ = **816 kW/m²**. At 2500 K it rises to ~2.0 MW/m²; at 1800 K it falls to ~535 kW/m².
- **Low-temp radiator at 500 K, ε = 0.9:** Q/A = 0.9·5.67×10⁻⁸·6.25×10¹⁰ = **3.19 kW/m²** — 256× worse per unit area than the 2000 K loop. This is *why* the low-temp loop must retract and dump to heat sinks in combat.

**How much waste heat must the hot loop shed?** A 10.7 TW fusion reactor loses ~25% (brems + thermal that can't go out the nozzle) as reactor waste heat that must be radiated — call it conservatively ~1–2 TW of genuine radiator load after direct conversion and nozzle losses (the bulk of energy leaves as exhaust kinetic energy, not heat). Even 1 TW / 816 kW/m² = **1.2×10⁶ m² of 2000 K radiator** — impossibly large. **This is the core finding: a warship cannot radiate the full waste heat of a 10-mg battleship-scale reactor continuously.**

**Resolution (the setting's architecture is physically necessary):**
- The reactor is optimized so the overwhelming majority of energy leaves as *directed exhaust*, not heat. Only the brems/synchrotron and structural absorption (a few percent of fusion power) become radiator load. If radiator load is ~1% of 10.7 TW = 107 GW, then hot-radiator area = 107×10⁹ / 816×10³ = **131,000 m²** — still large but distributable as long fin/boom structures or droplet sheets.
- **Realistic hot-radiator mass [ENG]:** carbon-carbon high-temp radiators reach ~2 kg/m² (NASA NTRS 20080048181, goal 2 vs current ~10 kg/m²); refractory-metal panels ~3× a 5 kg/m² aluminum baseline (~15 kg/m²). Liquid-droplet radiators (LDR) achieve ~0.42–1 kg/m² and up to 1.4 kW/kg rejected (Tagliafico & Devia; LDR surveys 100–500 W/kg at 300 K, far higher at 2000 K). At 2 kg/m², 131,000 m² = **262 t of radiator** — a few percent of a battleship's mass, which is consistent.
- **Verdict:** sustained reactor output is capped not by fusion physics but by how much brems/structural heat the hot loop can shed. Ships that run hotter radiators (2500 K refractory/CNT-reinforced) or droplet radiators can support proportionally higher sustained power. **[SPEC]** metamaterial emissivity enhancement (ε→~0.98) and CNT-reinforced refractory panels are plausible but buy only a modest linear improvement — T⁴ dominates, so raising temperature always beats raising emissivity.

**Combat heat sinks — lithium [HARD].** Lithium properties: specific heat 3,560 J/kg·K; heat of fusion 435.4 ± 3.9 J/g (per Novikov et al. 1983, *Int. J. Thermophysics* 4, 227: "The lithium enthalpy of fusion calculated from the experimental data is equal to 435.4±3.9 J·g⁻¹"); melting point 453.61 ± 0.14 K (NIST TN 2273); heat of vaporization 19,600 kJ/kg.
- Warming solid Li from 300 K to melt (454 K): 3,560 · 154 = 548 kJ/kg. Melting: 435 kJ/kg. Warming liquid to ~800 K working temp: 3,560 · 346 ≈ 1,230 kJ/kg. **Total sensible+latent to 800 K ≈ 2.2 MJ/kg.** If allowed to run to ~1,000 K, ~4.6 MJ/kg. Venting as vapor (emergency) adds the enormous 19.6 MJ/kg latent heat of vaporization — which is why venting is the last-ditch measure.
- **Combat endurance math:** A 10 GW point-defense/offensive laser suite that is, say, 40% efficient dumps ~15 GW of waste heat when firing continuously. One tonne of lithium (4.6 MJ/kg → 4.6 GJ) absorbs that for 4.6 GJ / 15 GW = **0.3 seconds of continuous full lasing per tonne**. Ten tonnes = ~3 s continuous, or thousands of short PD pulses. A serious warship therefore carries tens to low-hundreds of tonnes of lithium for minutes of intermittent combat. **This directly ties displacement to combat endurance:** every extra minute of sustained 15 GW waste-heat combat costs ~200 t of lithium.

### 5. Magnetic Field Strength and Reactor Size

**B⁴ scaling [HARD/ENG].** Fusion power density ∝ β²B⁴ (β = plasma/magnetic pressure ratio). MIT/CFS: doubling B gives 16× power density (or same power in 1/16 the volume). Going from ITER-class coils — whose toroidal-field coils are rated at a maximum 11.8 T (ITER Organization: "The toroidal field coils are designed to produce a total magnetic energy of 41 gigajoules and a maximum magnetic field of 11.8 tesla," each weighing 330 t and measuring 9 × 17 m) — to a **sustained 50 T [SPEC — centuries of HTS development]** is a field ratio ~4.2, i.e. (50/11.8)⁴ ≈ **320× the power density**.
- **Reactor core volume:** a DFD-class FRC core is ~2 m diameter × 10 m (Princeton). At ~300× density, a multi-GW core can be on the order of a few m³ of plasma volume. The core plasma is small; the assembly is not.
- **Reactor assembly mass [ENG]:** conceptual magnetic fusion propulsion designs cluster at **1–10 kW(thrust)/kg** (NASA NTRS 20110014263 review; ESA ACT open-magnetic-fusion study 1–9 kW/kg). Taking a mature 5 kW/kg for a thrust-optimized reactor, the *reactor hardware* (magnets + shield + structure) for a warship scales at roughly **hundreds of tonnes per GW of throughput-limited hardware rating**, with HTS magnets, neutron shield (thin, thanks to aneutronic-ish D-³He), and the thrust-bearing structure dominating. For the classes below I budget the reactor+nozzle assembly at ~20% of dry mass.

### 6. Superconducting Power Bus

**[ENG/SPEC].** HTS (REBCO/YBCO) at ~150 K carries >1,000 A/mm² at high field (Nature s41598-021-81559-z). For GW-scale internal transfer:
- A GW at, say, 100 kV bus = 10 kA; at 1,000 A/mm² that is a 10 mm² superconductor cross-section — trivially small in conductor mass. **The mass is in cryogenics and structure, not the conductor.**
- Cryoplant + vacuum jacket + magnetic shielding for a ship-wide 150 K bus is the real cost: budget **~0.5–2 t per GW of bus capacity** for cryocoolers and insulation [ENG estimate], plus the 150 K loop feeds the low-temp radiator/heat-sink system. A battleship moving ~10 GW internally adds ~10–20 t of cryogenic bus infrastructure — small vs radiators and tankage.

### 7. Weapons Integration

**Lasers — flywheel banks [HARD/ENG].** CNT flywheels: literature gives 2,500 Wh/kg (rotor only) and ~900 Wh/kg with housing (De Gruyter micro-FESS study); CNT theoretical up to 8,571 Wh/kg (Wiley *Adv. Mater.* 2019). The setting's 2,000–4,000 Wh/kg is defensible for rotor-dominated mature banks; I use **2,500 Wh/kg system-level**.
- A 10 GW laser at 40% wall-plug efficiency needs 25 GW electrical input; a "full-power shot" of, say, 3 s = 75 GJ electrical = 20,800 kWh. At 2,500 Wh/kg: 20,800 / 2,500 = **8.3 t of flywheel per shot**. For 5–10 shots: **~42–83 t of flywheels** for the main battery alone. Plus capacitors for sub-second pulse shaping (CNT/supercap, higher power density, lower energy density).
- Waste heat per shot (60% of 75 GJ = 45 GJ) is exactly what the lithium sink must swallow — ~10 t of lithium per full main-gun shot. **This couples laser magazine depth to heat-sink mass.**
- **Laser range/aperture [HARD]:** diffraction spot radius r ≈ 0.61·λ·L/R_lens (Atomic Rockets/Rocketpunk). A 30 m aperture main laser at ~1 µm reaches a ~tight spot to hundreds of thousands of km; shorter wavelength (UV) tightens it further. Aperture size, not raw power, sets engagement range — which is why the battleship mounts a 30 m mirror.

**Missiles [ENG].** Metallic hydrogen offensive missiles 500–2,000 kg, interceptors 100–300 kg. Metallic hydrogen density ~0.7 g/cm³ (Silvera & Cole: "We also assume that it has a density of 0.7 gm/cm3 (liquid H2 density is about 0.07 gm/cm3)"), Isp up to 1,700 s pure (needs dilution; realistic ~1,000 s). Storage:
- A magazine of 200 offensive missiles at 1,000 kg avg = **200 t** of ordnance; volume at ~2 m³ each (missile + launch cell + handling) = ~400 m³.
- 400 interceptors at 200 kg = **80 t**, ~200 m³.
- Handling machinery, autoloaders, and blast-hardened cell structure add ~30–50% mass → a large carrier's magazine complex is **300–800 t and 700–1,200 m³**.

**Magnetic launch rails [HARD/ENG].** 200–500 m, 5,000–10,000 g.
- To launch a 1,000 kg missile at 8,000 g (78,500 m/s²) over 300 m: exit velocity v = √(2·a·L) = √(2·78,500·300) = √(4.71×10⁷) = **6,860 m/s**. KE = ½·1000·6,860² = **23.5 GJ** per shot.
- At 20% efficiency (per railgun/coilgun literature at these velocities), electrical draw = ~118 GJ per launch — pulsed from the same flywheel/cap banks. Rail hardware mass scales with peak force F = m·a = 1000·78,500 = 78.5 MN, requiring a massive rigid structure: budget **50–150 t per heavy rail**, comparable to a spinal weapon mount. (Note: rails impart high muzzle velocity but the missile's own metallic-hydrogen drive supplies most terminal Δv; the rail is a "first-stage" launch assist that also reduces the missile's vulnerable low-speed boost phase.)

### 8. Structural Considerations

**CNT primary structure [ENG].** CNT composites: NASA reports up to 200 GPa tensile strength / 1,400 GPa modulus in lab specimens vs ~6.9 GPa for T1100 carbon fiber; realistic bulk CNT fiber today ~4.2 GPa (ScienceDirect). NASA modeling: CNT reinforcement → ~30% launch-vehicle dry-mass reduction. **Effect:** structural mass fraction falls from a conventional ~10–15% toward **~6–10%** of dry mass; the setting's 10,000+ bar CNT pressure vessels are the same material family (CNT COPVs already flight-tested, SubTec-7, 2016).

**Armor — minimal Whipple [ENG].** Since missiles/lasers dominate and kinetic PD is a "poverty weapon," armor is anti-fragment/anti-debris only: multi-layer Whipple bumpers (spaced thin refractory/CNT sheets) + the hot-radiator structure doubling as standoff. Budget **3–8% of dry mass** — far below a "wet-navy" battleship's 30–40%. Atomic Rockets/Children of a Dead Earth note radiators can themselves be armored at an efficiency cost; the honeycomb + thermal-superconductor-mesh scheme (Defenses page) fits the setting.

**Crew and life support [ENG].** ISS-derived closed-loop ECLSS Equivalent System Mass ≈ **730 kg per crew member** (NASA, Mars-transit basis), plus ~0.2 kg/crew-day logistics and ~5 kg/crew-day of consumables if open-loop. For a months-long deployment with ~90% water / ~50–75% O₂ recovery: budget **~1.5–3 t per crew member** including habitat volume (~20–40 m³/person), radiation shelter, and consumables. A 250-crew battleship → ~375–750 t life support + ~6,000 m³ habitable volume.

**Sensors/comms [ENG].** RADAR/LIDAR/IR arrays + comm lasers: a capital-ship suite (large phased arrays, cryo-cooled IR focal planes, meter-class comm-laser apertures) is a small mass fraction — budget **20–80 t** depending on class, dominated by the large IR/optical apertures and their own cooling (which feeds the low-temp loop).

### 9. Additional Design Factors

**Propellant tankage at MR 8 [HARD/ENG].** At MR 8, 87.5% of wet mass is propellant. Two very different storages:
- **Metallic hydrogen afterburner propellant:** if metastable, stored as a dense (~0.7 g/cm³) solid — compact, but if it requires 10,000+ bar CNT pressure-vessel confinement (setting standard), tank mass is significant. Thin-wall pressure-vessel mass scales as m_tank/m_prop ≈ (3/2)·(P/σ)·(ρ_tank/ρ_prop): for CNT at usable ~10 GPa strength storing at 1 GPa (10,000 bar), this gives tank/propellant ratios of a few percent. Budget **tankage ~3–6% of contained propellant mass**.
- **Fusion fuel (D and ³He):** tiny by mass (kg/s burn → tonnes over a campaign) but ³He is a low-density cryogenic gas needing insulated cryotanks; D as cryo-liquid or in metal hydride. Fuel-tank mass is negligible vs afterburner propellant.
- **Consequence:** at MR 8 a 90,000 t dry battleship masses ~720,000 t wet, of which ~630,000 t is propellant. Tankage at 4% = **~25,000 t of tanks** — itself a major structure, and the single biggest argument against very high mass ratios.

**Magnetic nozzle mass [ENG/SPEC].** The superconducting nozzle takes the full thrust load and needs enormous fields. Efficient magnetic nozzles (>85% plume efficiency, DFD studies) use coils comparable in scale to the reactor. Because it bears the thrust and is cryogenic, budget the nozzle at **~30–50% of the drive-assembly mass** — a major element, consistent with the setting's premise. Scaling: nozzle mass rises with thrust rating (∝ field energy ∝ B²·volume needed to redirect the plasma momentum).

**Drive gearing mechanics [ENG/analysis].**
- **Upstream injection** (propellant mixed into plasma before the nozzle): better energy coupling — the cold propellant is heated by the fusion plasma and expands, extracting more of the fusion energy as directed KE. But it contaminates the plasma (raises Z → more bremsstrahlung), and at high injection rates can quench the reaction by cooling the core below ignition. Best for *moderate* thrust boost at good efficiency.
- **Downstream injection** (momentum transfer via collisions in the exhaust plume): simpler, cannot quench the reaction, but thermodynamically worse — cold mass added downstream shares momentum inelastically, so the ISP-to-thrust exchange is less efficient. Best for *maximum* thrust bursts where efficiency is irrelevant.
- **Consequence for the gear curve:** upstream gives a steeper, more efficient ISP↔thrust trade over the mid-range; downstream extends the curve to very high thrust at the cost of a worse exchange rate. A real drive uses upstream for cruise-to-combat downshifts and downstream for emergency max-thrust.

### 10. Ship Classes (internally consistent estimates)

All use MR 8, ~10 mg cruise, ~1.8 TW fusion per 10,000 t, dry-mass allocations: reactor+nozzle ~20%, radiators ~5%, structure ~8%, armor ~5%, weapons/magazines/flywheels per role, crew/life-support/sensors per role, remainder margin. (Tankage is folded into wet mass.)

| Class | Dry mass | Wet mass (MR 8) | Length × beam | Crew | Notes |
|---|---|---|---|---|---|
| **Battleship (laserstar/PD)** | ~90,000 t | ~720,000 t | ~600 × 120 m | ~250 | 1× 30 m/10 GW main laser, 6× 10 m/2 GW secondaries; ~80 t flywheels; ~150 t Li heat sink; heavy interceptor magazine (~400); large hot-radiator booms |
| **Missile carrier** | ~55,000 t | ~440,000 t | ~500 × 90 m | ~180 | 600–800 offensive missiles (~600–800 t ordnance), spinal EWAR array, standard PD; magazine + handling ~800 t |
| **Drone carrier / control ship** | ~50,000 t | ~400,000 t | ~500 × 110 m | ~200 | Hangar volume ~10,000 m³, C4I suite, drone servicing; light armament; large low-temp cooling for electronics |
| **Corvette (screen/picket)** | ~5,000 t | ~40,000 t | ~180 × 40 m | ~25 | Spinal 15 m/5 GW laser; mixed 40-missile loadout; ~20 t flywheels; ~15 t Li; high agility |

**Consistency checks:** A 90,000 t battleship needs ~16 TW fusion for 10 mg — within the 50 T reactor's B⁴-boosted density. Its ~131,000 m² of 2000 K radiator (~262 t) is ~0.3% of dry mass — plausible as deployed booms. Its main-gun magazine (10 shots) demands ~80 t flywheels + ~100 t lithium, matching the endurance premise. At MR 8 its ~630,000 t of propellant in 4% tankage = ~25,000 t of tanks — the dominant single structure, exactly the tradeoff the setting's high mass ratio implies. The corvette scales down cleanly: ~0.9 TW fusion, ~11,000 m² radiator (~22 t), and a magazine/flywheel loadout ~1/10 the battleship's.

## Recommendations

1. **Anchor the setting on two published numbers and cite them in-world:** Atomic Rockets' D-³He figures (Ve = 2,300 km/s, 1.82 MW/N) and Stefan-Boltzmann at 2000 K (~816 kW/m²). Every ship's "sustained power rating" should be quoted as *radiator-limited*, not reactor-limited — this is the most defensible and most dramatic constraint.
2. **Express combat endurance in tonnes of lithium, not minutes.** Adopt the rule ~4.6 MJ/kg (melt + heat to 1,000 K) and ~19.6 MJ/kg if venting. Publish per-class lithium loadouts; this makes "heat exhaustion" a concrete tactical resource.
3. **Downgrade the neutron-suppression claim.** Present spin-polarization as giving a solid ~1.5× cross-section boost and ~+45% burn-up / +1–20% propulsive efficiency, but flag D-D neutron suppression as *unproven* (QSF 0.1–2.5, unmeasured). This keeps the setting honest and gives a plot hook (the PolFusion-descendant experiment that "settled" it in-world).
4. **Treat mass ratio 8 as a soft ceiling.** Tankage mass (~4% of propellant) and the magnetic-nozzle thrust structure mean returns diminish sharply above MR ~6–8; most warships should sit at MR 4–6 and rely on abundant Δv, spending mass budget on radiators, heat sinks, and magazines instead.
5. **Benchmarks that change the design:** if sustained HTS fields >50 T become available, reactors shrink further and the constraint moves entirely to radiators; if droplet radiators mature to ~1 kg/m² at 2000 K, sustained power roughly doubles per tonne and ships can run hotter reactors continuously; if metallic hydrogen proves *non*-metastable, the afterburner reverts to cryo-hydrogen/deuterium at ~10× the tank volume, cutting combat thrust and forcing lower mass ratios.

## Caveats
- **Speculative pillars:** sustained 50 T HTS fields, metastable metallic hydrogen at storable conditions, and CNT bulk properties at 200 GPa are all **[SPEC]** — real lab values are lower (CNT bulk fiber ~4.2 GPa today; metallic hydrogen first observed by Dias & Silvera at 495 ± 13 GPa at ~5.5 K, *Science*, 25 Aug 2017, doi:10.1126/science.aal1579, with metastability still unconfirmed). The setting is defensible *given centuries of development*, but these are the load-bearing assumptions.
- **Radiator load is the softest number.** How much of fusion power becomes radiator heat vs directed exhaust depends on direct-conversion efficiency and nozzle physics that are not settled; I used ~1% of fusion power as radiator load for the area estimates, which is optimistic. If it is 5–10%, hot-radiator area (and ship size) grows proportionally, and the classes above should be read as *lower bounds* on radiator mass.
- **Specific-power figures (1–10 kW/kg) are from conceptual studies** with, in the words of one NASA review, "widely varying assumptions and levels of optimism." Reactor+nozzle mass fractions here (~20%) are engineering judgment, not measured hardware.
- **The DFD data points** (40 N, 56.5 km/s, 0.18 kW/kg for a 1 MW engine) are *current* concept-stage numbers; the setting's warship drives are ~10⁷× more powerful and assume centuries of scaling — legitimate for the timeframe but not verifiable.
- **Δv figures assume ideal impulsive burns and the Atomic Rockets 85% nozzle efficiency;** real gravity losses, cosine losses from thrust vectoring, and afterburner mixing inefficiencies will reduce delivered Δv modestly.
- Where sources conflicted (e.g., propulsive-efficiency gain from polarization), I used the peer-reviewed primary source (Bruhaug & Kish: 1–20%) over secondary summaries.