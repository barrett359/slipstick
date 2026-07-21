# Gabriel orbital mechanics and TF Sahara maneuver report

This report replaces every orbital time asserted by `TF_Sahara_v0.8.md`. It
uses the validated ship designs and commissioned propellant states in the
isolated Slipstick draft `tf-sahara-v0-8-reconstruction-1784497821`. The live
fleet document is unchanged.

All propulsion, burn, and circular-orbit figures below were run through the
native Rust calculators (`gear`, `burn`, `burn_for_dv`, and `orbit_v`). The
small number of conic-transfer and Hill-sphere values not directly exposed by
those calculators are identified as derived textbook mechanics.

## 1. Reference model

### Gabriel

| Quantity | Adopted value |
|---|---:|
| Mass | 1.30 Io = 1.16115194e23 kg |
| Mean radius | 2,500 km |
| Mean density | 1,774 kg/m3 |
| Gravitational parameter, mu | 7.749876393e12 m3/s2 |
| Surface gravity | 1.240 m/s2 = 0.1264 g |
| Surface escape speed | 2.490 km/s |

The 2,500 km radius interprets the chapter's "five thousand kilometres of rock
and ice" as Gabriel's diameter. A 1.3-Io-mass body with Io's actual radius
would produce a very different occultation interval and is not used here.

### Veritas dependency and Hill stability

The reference folder does not specify Veritas's mass, Gabriel's orbit around
Veritas, or that orbit's plane. Exact system trajectories therefore cannot yet
be calculated. For a stability check only, this report brackets Veritas as
Jupiter-mass and Gabriel's semimajor axis as Io-like, 421,700 km.

Under that bracket:

| Quantity | Result |
|---|---:|
| Gabriel orbital period about Veritas | 42.462 h |
| Gabriel Hill radius | 11,521 km from Gabriel's center |
| Approximate stable prograde limit | 5,639 km center radius |
| Approximate stable retrograde limit | 10,725 km center radius |
| Minimum Gabriel-Veritas separation for a stable 4,500 km prograde orbit | 336,500 km |

The proposed 4,500 km battle orbit is stable, but it is not deep inside the
Hill sphere. At 152,000 km, Sahara is about thirteen Hill radii from Gabriel.
Its long approach is a Veritas-centric trajectory, not a Gabriel capture
ellipse. Gabriel-centered two-body mechanics become a useful patched-conic
approximation only close to the moon.

## 2. Chanduran reference orbit

The reference battle orbit is 2,000 km over the cloud tops, or 4,500 km from
Gabriel's center.

| Quantity | Result |
|---|---:|
| Circular speed | 1.312324 km/s |
| Escape speed at orbit | 1.855907 km/s |
| Prograde escape burn | 0.543582 km/s |
| Orbital period | 21,545.24 s = 5 h 59 min 5.2 s |
| Maximum edge-on occultation | 67 min 19.6 s |
| Visible near-side arc | 4 h 51 min 45.6 s |

The orbit repeats in approximately six hours. Any battle lasting longer than
one orbit necessarily contains repeated occultations and repeated predictable
rise geometries unless the Chanduran fleet maneuvers.

## 3. TF Sahara propulsion state

The commissioned Solarian ships carry approximately half of full propellant.
At maximum-thrust gearing the calculator selects about 21 km/s effective
exhaust velocity. The force is limited by installed reactor/nozzle design, not
by a generic two-thirds-g setting.

| Ship type | Current mass | Maximum thrust | Initial maximum acceleration | Delta-v in an 18 min full burn | Displacement during that burn |
|---|---:|---:|---:|---:|---:|
| Tiberius | 315,000 t | 975.3 MN | 3.096 m/s2 (0.316 g) | 3.642 km/s | 1,910 km |
| Luna's Dream | 192,500 t | 600.0 MN | 3.117 m/s2 (0.318 g) | 3.669 km/s | 1,924 km |
| Bonaparte | 175,000 t | 487.7 MN | 2.787 m/s2 (0.284 g) | 3.248 km/s | 1,709 km |
| Corvette | 12,500 t | 60.96 MN | 4.877 m/s2 (0.497 g) | 6.064 km/s | 3,117 km |

Bonaparte is the intact fleet's acceleration bottleneck. A maneuver that keeps
the capitals together is limited initially to about 0.284 g, rising as
propellant is consumed. The corvettes can create several thousand kilometres
of relative baseline in eighteen minutes, but doing so sacrifices formation,
mutual point defense, and common injection geometry.

## 4. Sahara's powered arrival and hard injection burn

### 4.1 What 152,000 km at 6 km/s means

Even if Gabriel were isolated, local escape speed at 152,000 km is only 0.319
km/s. A ship moving 6 km/s relative to Gabriel is emphatically unbound. In the
Gabriel-only approximation it has:

- hyperbolic excess speed: 5.9915 km/s;
- target periapsis: 4,500 km;
- periapsis speed without prior braking: 6.2724 km/s;
- circular speed at periapsis: 1.3123 km/s;
- impulsive capture requirement: 4.9600 km/s.

The two-body coast time from 152,000 km to that periapsis is approximately
7.012 h, but this time is illustrative only because most of the path lies
outside Gabriel's Hill sphere.

### 4.2 Emergency raw capture

If Sahara reaches periapsis with roughly 6 km/s hyperbolic excess, the fleet
must remove 4.960 km/s. Maximum-thrust calculator results are:

| Ship type | Propellant consumed | Burn duration |
|---|---:|---:|
| Tiberius | 66,267 t | 23 min 46.8 s |
| Luna's Dream | 40,496 t | 23 min 37.4 s |
| Bonaparte | 36,815 t | 26 min 25.4 s |
| Corvette | 2,630 t | 15 min 5.9 s |

This is not a credible impulsive periapsis burn. Bonaparte's burn lasts 26.4
minutes while the target orbit itself takes only 359 minutes. The ship travels
thousands of kilometres during the maneuver, so a finite-burn targeting solver
must begin well before nominal periapsis. A delayed ignition risks either a
flyby or atmospheric impact. This is an emergency recovery profile, not the
planned arrival.

### 4.3 Recommended powered approach

A defensible arrival uses the drive's gearing instead of carrying almost all
6 km/s into the moon's Hill sphere. The fleet brakes at efficient medium gear
during the Veritas-centric approach, targeting a Gabriel-relative hyperbolic
excess speed near 1.0 km/s.

Bonaparte, the fleet limiter, produces at 500 km/s exhaust velocity:

- thrust: 20.482 MN;
- initial acceleration: 0.1170 m/s2 = 0.01193 g;
- mass flow: 40.963 kg/s.

Removing approximately 4.950 km/s at this gear takes 42,088 s, or 11 h 41 min
28 s, and consumes only 1,724 t of Bonaparte's propellant. The simple
one-dimensional distance integral is approximately 148,000 km, which is why
this gearing is a good first design point for a 152,000 km powered approach.
The exact thrust direction and ignition epoch require the missing Veritas and
Gabriel state vectors.

At 1.0 km/s hyperbolic excess, the final local capture becomes:

| Quantity | Result |
|---|---:|
| Periapsis speed | 2.1082 km/s |
| Circular speed | 1.3123 km/s |
| Hard injection delta-v | 0.79585 km/s |

Maximum-thrust injection results:

| Ship type | Propellant consumed | Burn duration |
|---|---:|---:|
| Tiberius | 11,714 t | 4 min 12.2 s |
| Luna's Dream | 7,159 t | 4 min 10.6 s |
| Bonaparte | 6,508 t | 4 min 40.3 s |
| Corvette | 465 t | 2 min 40.1 s |

This is a genuine hard injection burn: brief compared with the orbit, large
enough to be operationally dramatic, and comfortably inside every ship's
remaining propellant budget. It is still a finite burn. The formation should
ignite on staggered calculated nodes so that every hull reaches the same
post-burn osculating orbit rather than all ships receiving the same clock-time
throttle command.

The recommended canonical sequence is therefore:

1. Veritas-centric powered braking while Sahara is still far outside Gabriel's
   Hill sphere.
2. Combat maneuver corrections folded into the arrival solution.
3. Hill-sphere entry at approximately 1 km/s hyperbolic excess.
4. A four-to-five-minute hard retrograde injection centered near the 4,500 km
   periapsis.
5. A later trim burn of tens of metres per second after formation and damage
   assessment, rather than pretending the finite injection produces a perfect
   circle.

## 5. Solarian maneuver options during the missile battle

### Fleet dogleg

A common lateral burn is limited by Bonaparte. At the initial 2.787 m/s2:

| Burn time | Approximate lateral delta-v | Approximate displacement during burn |
|---|---:|---:|
| 60 s | 0.167 km/s | 5.0 km |
| 120 s | 0.334 km/s | 20.1 km |
| 300 s | 0.836 km/s | 125 km |
| 1,080 s, calculated finite-mass result | 3.248 km/s | 1,709 km |

These moves are significant against a precomputed aimpoint but do not outrun
torplets carrying about 36 km/s of terminal delta-v. Their purpose is to force
late target reassociation and correction, not to leave the threat envelope.
Short, irregular burns are preferable to one continuous lateral sprint because
they preserve propellant, formation baselines, and the arrival solution.

### Capture-plane consequence

Maneuver direction matters more than raw displacement:

- Retrograde components help the already-planned approach braking.
- Prograde components increase the final injection burden.
- Out-of-plane or lateral components move the periapsis and must be removed or
  deliberately retained as orbital inclination.
- A full eighteen-minute Bonaparte-limited lateral burn adds 3.248 km/s. If the
  planned residual excess speed was 1.0 km/s, the combined excess becomes
  3.399 km/s and circular capture rises from 0.796 to 2.560 km/s.

After that full combat burn, Bonaparte's revised injection consumes another
17,207 t and lasts 12 min 21.0 s. Capture remains possible, but the fleet has
turned a clean arrival into another long finite-burn problem.

### Formation split

Over eighteen minutes a corvette at maximum thrust travels about 3,117 km from
its original tangent while Bonaparte travels 1,709 km. Same-direction burns can
open about 1,408 km of relative baseline; opposing burns can open about 4,826
km. Neither is desirable during terminal defense. Useful combat baselines are
more likely tens to low hundreds of kilometres, with the faster corvettes
reserving thrust to rejoin the capital injection corridor.

### Abort capture or change the flyby

Because Sahara is far outside the Hill sphere when attacked, it is not trapped
into meeting Gabriel. Approximately 100 m/s of transverse velocity accumulated
over the illustrative seven-hour approach shifts the uncorrected aimpoint by
about 2,500 km. Roughly 0.2-0.3 km/s is enough to turn a 4,500 km periapsis into
a clean miss in the simple model. Breaking contact is therefore much cheaper
than capturing; the reason to continue inward must be strategic, not kinematic.

## 6. Chanduran maneuver options during the battle

The Chanduran fleet begins with the tactical advantage of an established,
predictable orbit and the strategic disadvantage that the orbit is predictable.
Its bracketed starting maximum accelerations are:

| Ship type | Initial maximum acceleration |
|---|---:|
| CATHEDRAL | 2.709 m/s2 = 0.276 g |
| Heavy escort | 3.483 m/s2 = 0.355 g |
| Light escort | 4.267 m/s2 = 0.435 g |

CATHEDRAL can gain 3.151 km/s and move 1,659 km during an eighteen-minute full
burn, but it consumes 75,239 t of propellant doing so. That maneuver completely
abandons the original orbit and is not compatible with descriptions of the
fleet quietly holding station.

### Break orbit and withdraw

Only 0.5436 km/s prograde is required to reach local escape. CATHEDRAL can make
that burn in 198.1 s while consuming 13,798 t of propellant. Escape from Gabriel
is easy; escape from Sahara's weapons and from Veritas's larger gravity field is
a separate intercept problem.

### Shift the predicted rise point

At 1.312 km/s orbital speed, impulsive plane changes cost:

| Plane change | Delta-v | Maximum cross-track displacement at 4,500 km radius |
|---|---:|---:|
| 1 degree | 22.9 m/s | 78.5 km |
| 5 degrees | 114.5 m/s | 392 km |
| 10 degrees | 228.8 m/s | 781 km |
| 30 degrees | 679.4 m/s | 2,250 km |

A 100 m/s normal burn inclines the orbit about 4.36 degrees and shifts the
maximum rise latitude by roughly 342 km. CATHEDRAL can execute that burn in
about forty seconds at maximum thrust. A pre-laid alpha strike aimed at one
precise limb point is therefore fragile unless Sahara observes and updates on
the hidden burn or covers a broad uncertainty region.

### Change phase and occultation timing

Small tangential burns strongly change this shallow moon orbit:

| Burn at 4,500 km | Resulting other apsis | New period | Period change |
|---|---:|---:|---:|
| 50 m/s retrograde | 3,874 km | 322.28 min | -36.80 min |
| 100 m/s retrograde | 3,349 km | 292.47 min | -66.61 min |
| 50 m/s prograde | 5,258 km | 405.38 min | +46.29 min |
| 100 m/s prograde | 6,191 km | 464.94 min | +105.85 min |

These are not instantaneous shifts in rise time; they change the subsequent
coast arc. A burn made early in occultation can move the next emergence by many
minutes. Retrograde burns also lower periapsis toward Gabriel and become unsafe
quickly: 200 m/s retrograde would put the opposite apsis only about 23 km above
the assumed cloud-top radius.

### Move the whole battle orbit

Two-impulse Hohmann transfers from the 4,500 km orbit are:

| Destination radius | Total delta-v | Transfer time |
|---|---:|---:|
| 4,000 km | 79.5 m/s | 2 h 44 min 47 s |
| 5,000 km | 67.3 m/s | 3 h 14 min 43 s |
| 5,500 km | 125.0 m/s | 3 h 30 min 17 s |
| 6,500 km | 218.6 m/s | 4 h 2 min 36 s |
| 3,000 km | 292.0 m/s | 2 h 16 min 35 s |

The first burn is cheap, but the transfer takes hours. Such moves are useful
for the next orbit or the next missile wave, not for dodging a laser shot that
will arrive in half a second.

### Formation maneuver

The escorts can create a screen ahead of CATHEDRAL with tens of metres per
second and minutes of coast. They can also separate in orbit phase so that one
ship rises before another. What they cannot do while remaining cold is spread
thousands of kilometres in tens of minutes. Any rapid large-baseline maneuver
uses the fusion drives, is thermally unmistakable, and changes every predicted
limb crossing.

## 7. Engagement-level consequences

1. Sahara's first missile defense occurs during a powered Veritas-centric
   arrival, many hours before the recommended Gabriel injection burn.
2. A six-hour Chanduran orbit means a long battle contains multiple complete
   revolutions. The enemy does not disappear once and then return according to
   a one-off dramatic clock.
3. Sahara should keep combat maneuvers mostly retrograde and modestly lateral if
   it still intends to capture. A maximum lateral sprint remains physically
   possible but exacts a large injection penalty.
4. The Chanduran fleet's best defense against a predicted rise-point strike is
   a hidden normal or tangential burn, not passively following the same circle.
5. Sahara can predict a new orbit as soon as it measures the post-burn state,
   but cannot predict an unobserved burn from the old orbit alone. "Orbits don't
   lie" is true only after the maneuver has been constrained.
6. Both fleets' fusion burns are conspicuous. Maneuver can change geometry;
   it cannot preserve stealth except while Gabriel physically blocks the
   observer.
7. If Sahara's strategic objective does not require Gabriel capture, a small
   early transverse burn produces a safe flyby far more cheaply than fighting
   for the 4,500 km orbit.

## 8. Inputs still required for a full system trajectory

The following values are necessary before the calculator can produce an exact
Veritas-frame state history rather than the local patched-conic report above:

- Veritas mass and radius;
- Gabriel semimajor axis, eccentricity, and orbital plane around Veritas;
- Sahara's position and full three-component velocity in the Veritas frame;
- intended Gabriel orbit inclination and direction;
- exact injection periapsis and permitted atmospheric clearance;
- whether the Chanduran 4,500 km orbit is prograde or retrograde relative to
  Gabriel's orbit around Veritas.

Until those are fixed, the local orbit, burn budgets, maneuver costs, and Hill
stability conclusions are reliable; exact system-clock epochs and inertial
directions are not.
