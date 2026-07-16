# Space Combat Reference

This document captures the physics, weapon systems, and tactical doctrine established through worldbuilding discussions. Everything here is grounded in real or defensibly extrapolated physics unless explicitly noted.

---

## Core Principles

### Heat Is the Master Constraint

Every weapon, every sensor, every reactor produces waste heat. The only way to get rid of heat in space is to radiate it, which requires massive radiator arrays. These radiators are structurally fragile, impossible to armor meaningfully, and visible to any infrared sensor at absurd range. A ship's combat endurance is ultimately determined by its thermal budget — how much heat it can absorb and radiate before systems begin to fail.

This has three immediate consequences. First, stealth is impossible for any ship with an operating reactor. Even current-era infrared sensors could detect a ship's heat signature at several light-minutes. Internal heat sinks (as seen in The Expanse or Starship Mage) buy minutes at best, not hours. Second, radiators are the primary vulnerability of every ship. Most kills in space combat result from radiator damage rather than hull penetration. Third, sustained combat degrades capability over time as thermal budgets are consumed. A fleet that has been fighting for days is measurably weaker than one that hasn't, even if no ships have been destroyed.

### Space Combat Is Consensual

It is nearly impossible to force an engagement on an opponent who doesn't want one. A ship that applies lateral delta-v early compounds its positional displacement over time — every second of head start translates to meters of offset per second, accumulating continuously. A pursuer who delays response must correct both the velocity mismatch and the accumulated displacement, and by the time they do, the evader has burned again. The evader's delta-v investment appreciates; the pursuer's depreciates.

The only way to force a battle is to threaten something the enemy can't abandon: a planet, a station, a wormhole gate, a civilian population. This shapes all strategic thinking. Battles happen at fixed assets. Open-space engagements only occur when both sides believe they can win, or when one side has no choice.

### Battles Take Days to Weeks

Ship accelerations in this setting are measured in milligees to low gees. Engagement ranges for primary weapons are measured in megameters. Closing from detection range to weapon range takes hours to days. Maneuvering for positional advantage — aspect ratio, vector matching, formation geometry — takes longer still. The battle itself (the exchange of fire) may last minutes to hours, but the approach and aftermath extend the total engagement to days or weeks.

---

## Weapon Systems

### Ship-Mounted Lasers (Primary Weapon)

Ship-mounted lasers are the backbone of both offense and defense. They deliver energy at lightspeed (no dodging), have effectively infinite magazine depth (limited only by thermal budget and power supply), and scale with the ship's ability to manage waste heat.

**Wavelength and Optics:** Green (532 nm) or near-UV (~200 nm) lasers with large conventional mirrors are the most physically defensible choice. Shorter wavelengths give tighter beams via the diffraction limit, but X-ray and gamma wavelengths cannot be focused with conventional optics.

X-ray focusing requires grazing-incidence reflection — photons skipping off surfaces at angles of a degree or two. This works for telescopes (Chandra) but scaling to weapon-grade power throughput is deeply speculative. The surface quality requirements are sub-angstrom over the entire optical path, and gigawatts of power hitting those surfaces creates extreme thermal management problems in the optics themselves.

Near-UV (~200 nm) provides roughly 2.5x better diffraction performance than green while remaining within conventional optics territory. This is likely the sweet spot for hard sci-fi credibility.

**Range:** Governed by the diffraction limit. Spot diameter at range R ≈ 1.22 × λ × R / D, where λ is wavelength and D is aperture diameter. A weapon is lethal as long as flux density at the spot exceeds the target's damage threshold.

Example: A 100 GW green laser (532 nm) with a 100m segmented primary mirror at 5 Gm produces a ~32m spot delivering ~120 MW/m². This is sufficient to rapidly destroy radiators and burn through structural elements. The same laser at 10 Gm produces a ~65m spot at ~30 MW/m² — still damaging to radiators but no longer an instant kill. At 20 Gm the flux drops to marginal levels.

Practical maximum engagement range for a capital ship laser with a very large mirror: approximately 5-10 Gm for decisive damage, extending to perhaps 15-20 Gm for harassment and radiator attrition.

**Aspect Ratio Warfare:** Because radiators are the primary vulnerability, fleet formations exist primarily to control aspect ratio — keeping radiators edge-on to the enemy while maintaining firing arcs. The slow geometric maneuvering to achieve favorable aspect ratios is the primary activity during the approach phase of a battle. This is where the "weeks of positioning for minutes of energy exchange" dynamic comes from.

**Tactical Role:** Offensive fire against enemy radiators and ship systems. Point defense against incoming missiles (the primary defense layer — see Point Defense Doctrine below). Sustained thermal pressure that degrades enemy combat capability over time.

### Lasestars (Dedicated Laser Platforms)

The Chanduran Lasestar is a capital ship built entirely around its primary laser weapon. Its defining feature is an enormous segmented primary mirror (50-100m class) and the massive radiator arrays required to sustain continuous high-power lasing. These radiator arrays give Lasestars a dual advantage: devastating long-range firepower and, out of combat, significantly higher acceleration than other capital ships (more radiator area means more waste heat capacity means more reactor output for thrust).

The primary laser can be diverted through multiple smaller secondary lenses for point defense, giving the Lasestar excellent mid-range PD capability in addition to its long-range offensive role.

The vulnerability is inherent in the design: the same radiators that enable the weapon make the ship fatally fragile if anything gets close enough to damage them. Lasestars fight at extreme range and die if they can't maintain it.

Estimated maximum effective range for a Lasestar's main battery: approximately 5 Gm for full lethality, consistent with the Engagement Snippet reference.

### Missiles (Primary Kill Delivery)

Missiles are the primary mechanism for delivering damage to enemy ships. They succeed not by physically reaching the target but by getting close enough to employ their warheads at standoff range.

**Drive Systems:** Metallic hydrogen provides the primary missile propellant. Missile delta-v at a mass ratio of 20 is approximately 50 km/s. Acceleration curves of 3-9+ gees are referenced in existing material. Missiles maneuver aggressively during approach, corkscrewing and jinking to defeat point defense tracking.

**Warhead: Casaba Howitzer (Primary):** A nuclear shaped charge that focuses the energy of a nuclear detonation into a directed jet of superheated metal plasma. Real physics — the concept was tested during Project Orion. The jet maintains coherence over significant distance, delivering extreme thermal and kinetic energy to a narrow cone.

Effective standoff range depends on yield and liner design. Conservative estimates: 20-50 km for small tactical warheads on cluster torplets. Aggressive estimates: up to 300 km for large strategic warheads on primary missiles like the Asmodeus class. This standoff range is the critical parameter that drives point defense doctrine — you must kill the missile before it reaches Casaba range, not before it reaches your hull.

**Warhead: Bomb-Pumped Laser (Secondary/Specialty):** A nuclear device pumps multiple X-ray lasing rods. Each rod fires a single X-ray laser pulse before the device self-destructs. The X-ray pulse propagates at lightspeed and the lasing event completes in ~10 nanoseconds — mechanical perturbation from the detonation literally cannot affect the rods before they've already fired.

The real limitations are beam divergence (set by rod aspect ratio, likely microradians — still meters of spot size at hundreds of km) and pointing accuracy (must know exactly where the target is at moment of detonation, no correction after firing). Effective range: potentially farther than Casaba howitzers (hundreds of km to low thousands) but with worse precision.

Bomb-pumped lasers may be a Chanduran specialty, fitting their laser-centric doctrine. Solarian doctrine likely favors Casaba howitzers for their simplicity and reliability.

**Missile Architecture:** Large missiles (Asmodeus class) carry a primary Casaba warhead plus a cluster of smaller submunitions (torplets) that separate during the terminal phase. The torplets disperse to saturate point defense, each carrying its own smaller Casaba warhead. Electronic countermeasures (jamming, chaff, flares, LIDAR dazzlers) are integral to all missile designs.

**Counter-Missiles:** Defensive missiles launched to intercept incoming salvos at long range (thousands of km). These carry proximity-fused fragmentation warheads — they detonate near the incoming missile and shower it with high-velocity fragments (depleted uranium ball bearings, tungsten penetrators). The goal is to damage guidance, propulsion, or the warhead itself before it reaches Casaba range.

### Particle Beams (Radiation Kill Weapon)

Particle beam weapons deliver relativistic particles that penetrate deep into target structures, depositing energy throughout the depth via ionization. The damage mechanism is fundamentally different from lasers: rather than surface ablation, particle beams produce intense radiation that kills crew, fries electronics, and disrupts computer systems. A ship hit by a particle beam may remain structurally intact but suffer complete mission kill.

**Charged vs. Neutral Beams:** Charged particle beams (proton, electron, heavy ion) suffer from Coulomb repulsion — like charges push apart, spreading the beam. At relativistic velocities the beam's own magnetic field partially counteracts this, but practical range is limited to tens to low hundreds of km. Additionally, charged beams can be deflected by magnetic shielding.

Neutral particle beams solve both problems. Ions are accelerated then neutralized before exit. No self-repulsion, no magnetic deflection. Beam divergence is locked in by the thermal spread of particles at the neutralization point — typically microradians for a high-quality accelerator. At 1,000 km: ~1m spot (excellent). At 100,000 km: ~100m spot (marginal against ship targets). At 1 Gm: useless. Practical maximum range: roughly 300,000-500,000 km.

**Damage by Range:**

- Close range (high flux, >8 Gy whole-body dose): Cardiovascular and CNS syndrome. Incapacitation within minutes. Death within days. Immediate electronics damage. Immediate mission kill.
- Medium range (1-8 Gy): Hematopoietic and GI syndrome. Acute symptoms are severe — projectile vomiting, bloody diarrhea, cognitive impairment. Death over days to weeks. The ship may retain some combat capability briefly, but the crew is already fatally irradiated.
- Long range (0.1-1 Gy): No acute symptoms. Elevated cancer risk. Not tactically decisive in a single engagement but cumulative exposure across a campaign becomes a serious medical and morale problem.

**Tactical Role:** Particle beams occupy a unique niche as a mission-kill weapon. They are devastating against crewed ships but less effective against hardened unmanned systems and drones. Their most significant effect may be psychological — radiation death is culturally understood as uniquely horrific. A fleet that identifies particle beam platforms in the opposing formation and has the option to disengage may choose to do so immediately.

Because space combat is consensual, particle beam ships serve as potent deterrents in open-space engagements. Their effectiveness is reduced in defensive scenarios where the enemy must hold position regardless (defending a gate, station, or planet).

**Hardware:** The accelerator requires tens to hundreds of meters of beamline, massive power supplies, and large superconducting magnets. This is strictly a capital ship weapon.

### Drones (Reusable Deployed Assets)

Drones occupy the tactical space between disposable missiles and permanent ship assets. They are autonomous or semi-autonomous platforms deployed from a carrier or mothership, designed to operate independently and potentially be recovered after an engagement.

The traditional objection to space fighters and carriers (the round-trip delta-v problem) does not apply to drones. A drone deployed to a station doesn't need return fuel — it operates until its thermal budget or power supply is exhausted, then is either recovered or abandoned. This makes drones far more mass-efficient than manned fighters while providing capabilities missiles cannot.

**Tactical Roles:**

- *Counter-missile escort:* Laser drones mixed into friendly missile salvos provide PD for the missiles themselves, protecting them from incoming counter-missiles. The missiles don't need to waste mass on defensive systems because the drones handle it.
- *Command relay:* When jamming breaks the link between the launching ship and its missiles, drones closer to the engagement can maintain contact and relay targeting data through the ECM environment.
- *Forward PD screen:* Drones parked between the fleet and the threat axis engage incoming missiles at longer range than ship-mounted PD can achieve, extending the defensive envelope.
- *Direct fire harassment:* A drone with a small laser strafing a ship is less decisive than a missile strike but forces the target to spend thermal budget on PD and evasive maneuvering, degrading their ability to fight the main engagement.
- *Post-battle recovery:* Unlike missiles, drones that survive can be collected and reused. Over a multi-engagement campaign, this cost advantage compounds.

### Kinetic Point Defense (Marginal/Poverty Weapon)

Kinetic PD — autocannons firing projectiles at incoming missiles — is the least effective defense layer. Its limitations are fundamental.

**Mass Budget Problem:** Every kilogram of ammunition requires 10-30 kg of fuel to transport (depending on the ship's mass ratio). Even small-caliber rounds (10-15mm, 0.05-0.1 kg each) impose meaningful mass penalties at scale. A credible kinetic PD loadout of thousands of rounds translates to tons of ammunition requiring tens of tons of fuel.

**Range Problem:** Metallic hydrogen propellant (theoretical specific energy ~50 MJ/kg, roughly 14x gunpowder) enables muzzle velocities of perhaps 4-7 km/s — significantly better than chemical propellant. But even at 7 km/s, flight time to a target at 100 km is ~14 seconds. A maneuvering missile dodges easily. Effective engagement range is realistically tens of km at most, against targets that have already exhausted most of their maneuvering fuel.

**Casaba Standoff Problem:** If incoming missiles carry Casaba howitzer warheads with 50-300 km standoff range, kinetic PD is engaging targets that may have already fired their warheads. The defense layer arrives too late.

**Where Kinetic PD Survives:** Ships too small or too cheap to mount adequate laser PD: pirates, corvettes, armed civilian ships. It becomes a poverty weapon — better than nothing, low mass per gun if not per engagement, and there are always ships that can't afford better. This is an interesting worldbuilding detail: the quality of your point defense correlates directly with your economic resources. Metallic hydrogen propellant justifies why anyone bothers at all — the rounds are fast enough to provide marginal utility inside a few tens of km against fuel-depleted terminal-phase torplets.

Caliber should be small (10-15mm) to minimize per-round mass. Tungsten penetrators rather than explosive shells — against unarmored missiles you just need to punch holes in guidance or propulsion.

### Railguns / Mass Drivers (Fixed Installations Only)

Unguided projectile weapons fired from ships are effectively useless against maneuvering targets at any meaningful range. Even at c-fractional velocities, targets at light-seconds of distance have enough time to displace laterally and dodge. RCS packages on rounds provide minimal correction relative to the closure velocity.

Railguns are viable only on fixed installations — stations, asteroids, orbital platforms — where barrel length and power supply are not constrained by ship mass budgets. In this role they serve as anti-infrastructure weapons (targeting ships on predictable approach vectors to stations or gates) or area denial (forcing approaching ships to spend maneuvering fuel dodging, degrading their combat capability before they arrive).

---

## Point Defense Doctrine

Point defense is organized in three layers, each engaging at progressively shorter range with decreasing effectiveness:

**Outer Layer — Counter-Missiles (10,000+ km):** Defensive missiles launched to intercept incoming salvos well before they reach Casaba warhead range. These carry proximity-fused fragmentation warheads (depleted uranium or tungsten ball bearings) that detonate near incoming missiles and shower them with high-velocity debris. Supplemented by escort drones providing additional PD for the counter-missile swarm itself.

**Inner Layer — Laser PD (100-1,000+ km):** The primary real defense. Ship-mounted lasers (or the Lasestar's secondary lens array) burn through missile buses and torplets before they reach Casaba range. This is where the bulk of incoming missiles should die. Effectiveness is limited by the ship's thermal budget — sustained PD lasing generates enormous waste heat, which degrades the ship's ability to fire offensively.

**Last Ditch — Kinetic PD (<50 km):** Small-caliber metallic hydrogen autocannons firing tungsten penetrators at fuel-depleted torplets that leaked through both outer layers. Marginal effectiveness. Present on ships that can't afford better laser PD, or as a "we're dead anyway" layer on larger ships. The thin margin of utility is offset by low mass per gun.

---

## Doctrinal Asymmetry: Solarian vs. Chanduran

The two superpowers have developed distinct military doctrines that reflect their cultures and economic systems.

**Solarian Doctrine — Mass and Expendability:** Corporate mass-production. Missile-heavy salvos with Casaba howitzer warheads. Quantity over individual unit quality. Ships are built to be replaceable. The Solarian approach treats ordnance as consumable — the economic engine of Sol can manufacture more. Kinetic PD is more common in the Solarian fleet due to cost-conscious shipbuilding.

**Chanduran Doctrine — Precision and Reusability:** AI-optimized quality over quantity. Laser-centric platforms (Lasestars as the backbone). Drone swarms for flexible, reusable force projection. Bomb-pumped laser warheads on missiles (fitting the laser-centric philosophy). Particle beam platforms as a potential specialty — their radiation kill mechanism is particularly effective against crewed Solarian ships while Chanduran forces can field more hardened unmanned systems.

This asymmetry creates interesting tactical matchups. Solarian fleets drown Chanduran formations in missile salvos, forcing them to spend thermal budget on PD. Chanduran fleets use superior laser range and drone screens to attrit Solarian formations before they can close to effective missile range. Neither doctrine is strictly superior — outcomes depend on positioning, numbers, and the specific engagement geometry.

---

## Open Questions

- **Exact Casaba howitzer yield-to-range relationship** for different warhead classes (tactical torplet vs. strategic Asmodeus). Needs detailed physics work to establish specific numbers.
- **Drone recovery logistics** — how does post-battle drone collection work? Delta-v cost of retrieval, vulnerability during recovery operations, economic breakeven vs. building new drones.
- **Particle beam ship design** — how large is the accelerator? What does a ship built around a hundred-meter beamline look like? How does it protect itself given it must close to shorter range than a Lasestar?
- **ECM/ECCM depth** — jamming, spoofing, and counter-jamming doctrine needs development. The Minerva fragments show aggressive jamming environments. How does this interact with drone command relaying?
- **Antimatter warheads** — available post-Demetry Ring, prohibitively expensive for standard ordnance, but what about strategic weapons? A single antimatter warhead might justify its cost against high-value targets.
- **Lasestar mirror design** — segmented vs. monolithic, deployment/retraction during combat, vulnerability of the mirror itself as a critical system.
