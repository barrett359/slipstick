# TF Sahara reconstruction — design gate and first engagement slices

Draft: `tf-sahara-v0-8-reconstruction-1784497821`

Status: all ship and missile designs validate; the live fleet is unchanged.

## 1. Validated designs

| Design | Dry mass | Full-load wet mass | Full-load propellant | Dry accel | Wet accel | Retracted sink endurance |
|---|---:|---:|---:|---:|---:|---:|
| Solarian battleship | 90,000 t | 540,000 t | 450,000 t | 10.09 mg | 1.68 mg | 129.25 s |
| Solarian missile carrier | 55,000 t | 330,000 t | 275,000 t | 10.32 mg | 1.72 mg | 488.01 s |
| Solarian drone carrier | 50,000 t | 300,000 t | 250,000 t | 9.08 mg | 1.51 mg | not laser-limited in report |
| Solarian corvette | 5,000 t | 20,000 t | 15,000 t | 11.35 mg | 2.84 mg | 34.03 s |
| CATHEDRAL bracket | 180,000 t | 900,000 t | 720,000 t | 7.56 mg | 1.51 mg | 132.80 s |
| Chanduran heavy bracket | 35,000 t | 175,000 t | 140,000 t | 9.73 mg | 1.95 mg | 132.43 s |
| Chanduran light bracket | 8,000 t | 32,000 t | 24,000 t | 9.93 mg | 2.48 mg | 57.57 s |

Accelerations above are calculator outputs converted from m/s² to milligee. The
commissioned scenario states carry roughly half full propellant, so their actual
starting acceleration lies between the dry and full-load columns. Nothing in the
validated baseline supports the chapter's two-thirds-g maximum gearing claim.

### Weapon designs

- Javelin: 12,000 kg; 9,257 kg propellant; 92 km/s bus exhaust; 135.8 km/s
  calculated total delta-v; six 25 kg-dry / 30 km/s torplets.
- SICKLE bracket: 27,000 kg Lancer-family bus; 17,362 kg propellant; 137 km/s
  bus exhaust; 141.1 km/s calculated total delta-v; eight 50 kg-dry / 36 km/s
  torplets.
- Counter-missile baseline: 200 kg MH vehicle; 34.73 km/s delta-v. This is
  underconstrained by project knowledge and must not be promoted to canon.

Casaba effect-energy fields are combat-model calibration proxies. Project knowledge
does not specify a delivered Casaba energy or coupling model.

## 2. Corrected first-salvo accounting

- 96 buses x 8 torplets = 768 initial torplets.
- 41 bus kills leave 55 buses and 440 torplets.
- Releasing 11 buses early changes timing and accuracy, not total torplet count.
- The chapter's 392 terminal threats require 47 bus kills, six more than narrated.
- The chapter's 328 kills / 64 leakers is internally consistent only with 392,
  not with the narrated 41 bus kills.

At 138 km/s, 10,000 km to a 50 km standoff lasts 72.10 s. A 115 s terminal
window begins at about 15,920 km. Separation at 20,000 km would leave 144.57 s.

## 3. Lasers v3 terminal-defense integral

The calculation uses the v3 range gates:

1. 10,000–3,000 km: uncertainty-matched concentrated BB-main fire.
2. 3,000–500 km: sequential main, secondary, and corvette fire.
3. 500–50 km: distributed groups and close-in PDL service.

Every snapshot uses a 0.5 m / 0.25 m² torplet, 138 km/s closure, 1 GJ/m²
structural threshold, and a separate 50 ms service/slew term. The integral is an
upper scheduling bound with perfect target allocation.

| Case | Fleet service-capacity upper bound |
|---|---:|
| Clean, dedicated 1 m receivers | 9,330 |
| 100 W aligned jammer | 6,530 |
| 1 kW aligned jammer | 6,172 |
| 100 W jammer plus a fresh 2 m chaff puff every service | 3,468 |
| 100 W jammer with weapon-aperture receive | 6,782 |

All bounds exceed either 392 or the corrected 440 terminal threats. Therefore the
current calculator does **not** predict 64 leakers from simple jammer power and
chaff alone. To reproduce the chapter's two possible accounting paths, the missing
association/scheduling layer must retain only:

- 328 / 3,468 = 9.46% effective capacity for the chapter's 392-threat count; or
- 376 / 3,468 = 10.84% effective capacity for the corrected 440-threat count.

That bracket is close to the Sensors v3 severe-contest sensitivity band (~10%),
but it is not a derived result. It represents unmodeled false associations, decoy
queue load, CATHEDRAL dazzle, picket loss, cross-platform scheduling, and kill
confirmation. The draft outcome is plausible only as a severe-contest assumption.

## 4. Counter-missile geometry

The conservative 200 kg MH interceptor launched against a 138 km/s inbound front
at 144,000 km reaches merge after 846.9 s, roughly 27,127 km from the defending
fleet. It does not reproduce the chapter's 60,000–80,000 km merge. A 6.7 km/s rail
kick would not close that difference. The outer-intercept passage needs either a
fusion-heated interceptor, substantially more delta-v, or a much later merge.

## 5. Rise-point alpha strike at 158,000 km

Calculator snapshots for one Tiberius main, five secondaries, and nine corvette
spinals give:

| Weapon | Count | Spot diameter | Avg central-lobe flux each |
|---|---:|---:|---:|
| Tiberius main | 1 | 3.42 m | 45.77 MW/m² |
| Tiberius secondary | 5 | 10.25 m | 1.02 MW/m² |
| Corvette spinal | 9 | 6.84 m | 5.72 MW/m² |

If every beam perfectly overlaps one patch, the combined average flux is
102.34 MW/m². That yields 0.31 GJ/m² in 3 s, 0.51 GJ/m² in 5 s, 1.02 GJ/m²
in 10 s, and 9.31 GJ/m² in 91 s. A 3–10 s precomputed strike can plausibly
soft-kill exposed metrology/cooling and approach the 1 GJ/m² structural threshold.
Ninety-one seconds is unnecessary and requires the enemy not to return fire.

At 50% duty, Tiberius's main plus five secondaries produce 5.67 GW average laser
waste heat. Ninety-one seconds consumes about 23.9% of the validated 5,000 t
lithium sink, before other loads. The chapter's 30% to 40% sink change is too small
for this design.

The chapter's stated 4.5 GW is also not the v3 beam sum. The participating weapons
total 6.5 GW listed optical, 5.46 GW central-lobe peak, or 2.73 GW central-lobe
average at 50% duty.

## 6. Solarian return-wave timing

The validated Javelin crosses 175,000 km from rest in 5,297.86 s (88.30 min),
arriving under power at 76.56 km/s. The chapter's 89-minute flight time is therefore
credible only for a path near 175,000 km, not the separately asserted 158,000 km
snapshot (which the calculator crosses in about 84.47 min).

## 7. Why this is not yet a full chapter simulation

The current combat runner cannot honestly chain the complete narrative because:

- component damage is not persisted into a later scenario run;
- one ship selects one target per update, so a distributed fleetwide missile wall
  cannot be represented as one launch event;
- Lidar/PD is a single-epoch snapshot and does not own a shared torplet service queue;
- missile PD at impact uses participant geometry, which is not necessarily the
  missile-front geometry;
- armor facings, internal geometry, fragmentation, blast propagation, and
  sympathetic magazine/tank failure are outside schema v1;
- Gabriel's authoritative radius, mass, and observer geometry are absent;
- authoritative Chanduran ship specifications and Casaba coupling/yield are absent;
- the 101-drone deployment requires a propulsion and predeployment history that
  the chapter does not provide.

The next defensible slice is the post-alpha CATHEDRAL/escort defense against 216
Javelins, run as a sensitivity matrix over array damage, heat remaining, terminal
separation range, and association efficiency. It should not inherit the chapter's
35–40% array damage or 67-minute heat estimate as facts.

