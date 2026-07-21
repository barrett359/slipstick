# TF Sahara reconstruction design basis

This workspace is an isolated Slipstick draft. It does not alter `data/fleet.json`.

## Authority and exclusions

1. `setting_reference_files/Lasers_v3.md` is authoritative for laser wavelength,
   module power, duty cycle, fire-control modes, and laser heat-sink endurance.
   The older 200 nm / 10 GW defaults and all v2.5 engagement ladders are excluded.
2. `setting_reference_files/Systems_Quick_Reference.md` supplies reactor specific
   power, radiator performance, crew budgets, sensor architecture, and EWAR.
3. `setting_reference_files/missiles_v2.md` supplies Javelin and Lancer-family
   bus mass, propulsion, payload, and torplet counts.
4. `setting_reference_files/Ship Design Research.md` supplies class-scale dry
   masses and tankage/structure fractions, except where its older laser numbers
   conflict with Lasers v3.
5. Numbers asserted only by `TF_Sahara_v0.8.md` are hypotheses, not design inputs.

## Explicit engineering choices

- Full-load ship mass ratio is 6 for capital ships, 4 for corvettes/light escorts,
  and 5 for the estimated Chanduran lasestar/heavy escort. This follows the ship
  research recommendation to treat MR 4-6 as normal and MR 8 as a soft ceiling.
- Commissioned ships begin at roughly half of full propellant capacity to model an
  end-of-transit force without importing the chapter's inconsistent acceleration.
- Solarian battleship: 90,000 t dry; 1 x 1 GW / 30 m main; 6 x 200 MW / 10 m
  secondaries; 12 x 150 MW / 3 m PDL clusters.
- Corvette: 5,000 t dry; 1 x 500 MW / 15 m spinal; 2 x standard PDL clusters.
- Missile carrier: 55,000 t dry, 600 Javelins, 200 interceptors, spinal EWAR.
- Drone carrier: 50,000 t dry, 101 sensor/PD drones represented as hangar/C4I mass;
  the current fleet schema has no persistent drone entity type.
- Javelin is the 12 t reference design: 1 g, 92 km/s bus exhaust, 135 km/s bus
  delta-v, six 25 kg-dry / 30 km/s torplets.
- SICKLE uses the 27 t Lancer physical design because the references say Chanduran
  doctrine favors Tempest/Lancer quality and the chapter's only non-outcome clue is
  an eight-torplet 27 t bus. Its name and exact inventory are scenario labels.
- Casaba delivered energy is not specified in project knowledge. The terminal-effect
  values are transparent combat-model calibration parameters, not canon yields.
- CATHEDRAL, Chanduran heavy escort, and Chanduran light escort are bracket models;
  no authoritative Chanduran ship specification exists in the reference folder.
  Their values must remain visibly provisional in every report.

## Simulator boundaries that affect interpretation

- Combat damage is functional component damage, not armor-facing, fragmentation,
  internal geometry, sympathetic detonation, or blast propagation.
- Each ship chooses one hostile target per update. A fleetwide missile wall must be
  represented by controlled scenario slices rather than assumed to emerge from one
  monolithic run.
- Lidar/PD is a deterministic single-epoch calculation embedded in a time-stepped
  ship engagement. It does not yet integrate full salvo service queues, distributed
  group scheduling, or torplet-by-torplet covariance over closure.
- The current combat resolver evaluates PD at impact using participant geometry.
  Missile-front geometry therefore requires dedicated terminal slices; launcher
  range is not a valid substitute.

## Initial roster model

- TF Sahara: Tiberius, Luna's Dream, Bonaparte, fifteen corvettes. Thirteen
  corvette names appear in the chapter; two are retained as `Sahara-14` and
  `Sahara-15` rather than silently inventing canon names.
- Bogey Epsilon: CATHEDRAL, two heavy escorts, ten light escorts. This is the only
  roster that closes the chapter's 13 initial contacts, three light losses behind
  Gabriel, ten contacts at second rise, and final one-heavy/four-light remnant.

