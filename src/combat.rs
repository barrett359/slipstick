use crate::model::{Component, ComponentCombatProfile, FleetDocument, NavState};
use crate::physics;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CombatScenario {
    /// Version of this standalone combat-scenario contract.
    pub schema_version: String,
    /// Human-readable scenario name.
    pub name: String,
    /// Maximum simulated duration in seconds.
    pub duration_s: f64,
    /// Maximum interval between navigation and engagement updates in seconds.
    #[serde(default = "default_step")]
    pub step_s: f64,
    /// Reproducible seed for the representative run and ensemble.
    #[serde(default = "default_seed")]
    pub seed: u64,
    /// Number of seeded runs in the outcome ensemble.
    #[serde(default = "default_samples")]
    pub samples: usize,
    /// Narrative objective used in the deterministic report.
    #[serde(default)]
    pub objective: String,
    /// Optional scenario-local map positions and velocities keyed by ship ID.
    /// Values here override the draft System Map without mutating it.
    #[serde(default)]
    pub initial_nav: BTreeMap<String, NavState>,
    /// Participating commissioned ships and their doctrine.
    pub participants: Vec<CombatParticipant>,
}

fn default_step() -> f64 {
    10.0
}
fn default_seed() -> u64 {
    1
}
fn default_samples() -> usize {
    100
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CombatParticipant {
    /// Commissioned ship ID from fleet states and system.nav.
    pub ship_id: String,
    /// Team identifier used for victory resolution.
    pub team: String,
    /// Editable rule-based combat doctrine.
    #[serde(default)]
    pub doctrine: CombatDoctrine,
    /// Optional full Lidar/PD calculator template. Geometry is replaced each update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lidar_pd: Option<physics::LidarPdIn>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CombatDoctrine {
    /// hold, return_fire, or weapons_free.
    pub rules_of_engagement: String,
    /// Maximum range at which the ship attempts active detection.
    pub sensor_range_m: f64,
    /// Interval between sensor attempts in seconds.
    #[serde(default = "default_sensor_cadence")]
    pub sensor_cadence_s: f64,
    /// Maximum range at which missiles may be launched.
    pub missile_range_m: f64,
    /// Rounds launched at once against a selected target.
    pub missile_salvo: u32,
    /// Rounds retained in each magazine for later defense or escalation.
    pub defensive_reserve: u32,
    /// Whether offensive laser fire is permitted.
    pub laser_fire: bool,
    /// Mission-integrity fraction at which the ship attempts to disengage.
    pub retreat_integrity: f64,
    /// Optional ordered target IDs; remaining enemies follow by range.
    #[serde(default)]
    pub target_priority: Vec<String>,
}

fn default_sensor_cadence() -> f64 {
    10.0
}

impl Default for CombatDoctrine {
    fn default() -> Self {
        Self {
            rules_of_engagement: "weapons_free".into(),
            sensor_range_m: 100_000_000.0,
            sensor_cadence_s: default_sensor_cadence(),
            missile_range_m: 100_000_000.0,
            missile_salvo: 1,
            defensive_reserve: 0,
            laser_fire: true,
            retreat_integrity: 0.2,
            target_priority: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CombatEvent {
    pub time_s: f64,
    pub kind: String,
    pub actor: Option<String>,
    pub target: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub data: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ComponentOutcome {
    pub ship_id: String,
    pub component_id: String,
    pub role: String,
    pub integrity: f64,
    pub condition: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RepresentativeOutcome {
    pub seed: u64,
    pub winner: Option<String>,
    pub end_time_s: f64,
    pub ammunition_expended: BTreeMap<String, u32>,
    pub resources: Vec<ResourceOutcome>,
    pub components: Vec<ComponentOutcome>,
    pub events: Vec<CombatEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResourceOutcome {
    pub ship_id: String,
    pub propellant_t: f64,
    pub heat_mj: f64,
    pub sink_capacity_mj: f64,
    pub flywheel_mj: f64,
    pub tracks: usize,
    pub retreated: bool,
    pub ammunition: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TimingDistribution {
    pub samples: usize,
    pub mean_s: Option<f64>,
    pub median_s: Option<f64>,
    pub p90_s: Option<f64>,
    pub min_s: Option<f64>,
    pub max_s: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnsembleSummary {
    pub samples: usize,
    pub wins: BTreeMap<String, usize>,
    pub draws: usize,
    pub win_probability: BTreeMap<String, f64>,
    pub mean_end_time_s: f64,
    /// First-event timing distributions keyed by detection, fire, hit, and kill.
    pub timing: BTreeMap<String, TimingDistribution>,
    /// Mean rounds expended keyed by ship and magazine.
    pub mean_ammunition_expended: BTreeMap<String, f64>,
    pub component_disable_rate: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CombatRun {
    pub scenario: String,
    pub assumptions: Vec<String>,
    pub representative: RepresentativeOutcome,
    pub ensemble: EnsembleSummary,
}

#[derive(Clone)]
struct RuntimeComponent {
    id: String,
    profile: ComponentCombatProfile,
    integrity: f64,
}

#[derive(Clone)]
struct RuntimeShip {
    id: String,
    team: String,
    doctrine: CombatDoctrine,
    lidar_pd: Option<physics::LidarPdIn>,
    design_id: String,
    nav: NavState,
    mass_kg: f64,
    floor_kg: f64,
    components: Vec<RuntimeComponent>,
    ammunition: BTreeMap<String, u32>,
    initial_ammunition: BTreeMap<String, u32>,
    tracks: BTreeSet<String>,
    last_sensor_s: BTreeMap<String, f64>,
    fired_at: BTreeSet<String>,
    missiles_launched_at: BTreeSet<String>,
    propellant_t: f64,
    heat_mj: f64,
    sink_capacity_mj: f64,
    flywheel_mj: f64,
    retreated: bool,
    defeat_reported: bool,
}

#[derive(Clone)]
struct PendingImpact {
    time_s: f64,
    attacker: String,
    target: String,
    missile_id: String,
    count: u32,
    terminal_v_m_s: f64,
}

pub fn validate_scenario(fleet: &FleetDocument, scenario: &CombatScenario) -> Vec<String> {
    let mut errors = Vec::new();
    if scenario.schema_version.split('.').next() != Some("1") {
        errors.push("combat schema_version must be 1.x".into());
    }
    if !scenario.duration_s.is_finite() || scenario.duration_s <= 0.0 {
        errors.push("duration_s must be positive".into());
    }
    if !scenario.step_s.is_finite() || scenario.step_s <= 0.0 {
        errors.push("step_s must be positive".into());
    }
    if scenario.samples == 0 || scenario.samples > 10_000 {
        errors.push("samples must be between 1 and 10000".into());
    }
    let mut ids = BTreeSet::new();
    let mut teams = BTreeSet::new();
    for p in &scenario.participants {
        if !ids.insert(&p.ship_id) {
            errors.push(format!("duplicate participant {}", p.ship_id));
        }
        teams.insert(&p.team);
        if !fleet.states.iter().any(|s| s.id == p.ship_id) {
            errors.push(format!("unknown participant ship {}", p.ship_id));
        }
        if !scenario.initial_nav.contains_key(&p.ship_id)
            && !fleet.system.nav.contains_key(&p.ship_id)
        {
            errors.push(format!(
                "participant {} is not placed on the System Map",
                p.ship_id
            ));
        }
        if !(0.0..=1.0).contains(&p.doctrine.retreat_integrity) {
            errors.push(format!(
                "participant {} has invalid retreat_integrity",
                p.ship_id
            ));
        }
        if p.doctrine.missile_salvo == 0 {
            errors.push(format!(
                "participant {} missile_salvo must be positive",
                p.ship_id
            ));
        }
        if !p.doctrine.sensor_cadence_s.is_finite() || p.doctrine.sensor_cadence_s <= 0.0 {
            errors.push(format!(
                "participant {} sensor_cadence_s must be positive",
                p.ship_id
            ));
        }
        if !matches!(
            p.doctrine.rules_of_engagement.as_str(),
            "hold" | "return_fire" | "weapons_free"
        ) {
            errors.push(format!(
                "participant {} has unsupported rules_of_engagement",
                p.ship_id
            ));
        }
    }
    if teams.len() < 2 {
        errors.push("combat requires at least two teams".into());
    }
    errors
}

pub fn run(fleet: &FleetDocument, scenario: &CombatScenario) -> Result<CombatRun, String> {
    let errors = validate_scenario(fleet, scenario);
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    let representative = run_one(fleet, scenario, scenario.seed)?;
    let mut wins = BTreeMap::<String, usize>::new();
    let mut draws = 0usize;
    let mut total_time = 0.0;
    let mut disabled = BTreeMap::<String, usize>::new();
    let mut timing_samples = ["detection", "fire", "hit", "kill"]
        .into_iter()
        .map(|kind| (kind.to_string(), Vec::new()))
        .collect::<BTreeMap<String, Vec<f64>>>();
    let mut ammunition_totals = BTreeMap::<String, u64>::new();
    for sample in 0..scenario.samples {
        let seed = scenario
            .seed
            .wrapping_add((sample as u64).wrapping_mul(0x9e3779b97f4a7c15));
        let outcome = if sample == 0 && seed == representative.seed {
            representative.clone()
        } else {
            run_one(fleet, scenario, seed)?
        };
        total_time += outcome.end_time_s;
        match outcome.winner {
            Some(ref team) => *wins.entry(team.clone()).or_default() += 1,
            None => draws += 1,
        }
        for c in outcome
            .components
            .iter()
            .filter(|c| c.condition == "disabled" || c.condition == "destroyed")
        {
            *disabled
                .entry(format!("{}:{}", c.ship_id, c.role))
                .or_default() += 1;
        }
        collect_first_event_time(
            &outcome,
            "detection",
            &["track_acquired"],
            &mut timing_samples,
        );
        collect_first_event_time(
            &outcome,
            "fire",
            &["laser_fire", "missile_launch"],
            &mut timing_samples,
        );
        collect_first_event_time(
            &outcome,
            "hit",
            &["laser_hit", "missile_hit"],
            &mut timing_samples,
        );
        collect_first_event_time(
            &outcome,
            "kill",
            &["ship_defeated", "retreat"],
            &mut timing_samples,
        );
        for (magazine, count) in &outcome.ammunition_expended {
            *ammunition_totals.entry(magazine.clone()).or_default() += u64::from(*count);
        }
    }
    let samples_f = scenario.samples as f64;
    let win_probability = wins
        .iter()
        .map(|(team, n)| (team.clone(), *n as f64 / samples_f))
        .collect();
    let component_disable_rate = disabled
        .into_iter()
        .map(|(key, n)| (key, n as f64 / samples_f))
        .collect();
    let timing = timing_samples
        .into_iter()
        .map(|(kind, samples)| (kind, distribution(samples)))
        .collect();
    let mean_ammunition_expended = ammunition_totals
        .into_iter()
        .map(|(key, count)| (key, count as f64 / samples_f))
        .collect();
    Ok(CombatRun {
        scenario: scenario.name.clone(),
        assumptions: vec![
            "System Map gravity and ship kinematics are authoritative between engagement updates.".into(),
            "When no Lidar/PD template is supplied, detection uses an explicit linear range falloff.".into(),
            "Laser damage uses saved kill profiles; missile damage combines terminal kinetic and configured effect energy.".into(),
            "Missile terminal accuracy uses the configured one-sigma error against a mass-scaled effective target radius; legacy missiles without a profile assume 70 percent accuracy.".into(),
            "Damage resolves functional component condition, not armor, fragmentation, or internal geometry.".into(),
        ],
        representative,
        ensemble: EnsembleSummary {
            samples: scenario.samples,
            wins,
            draws,
            win_probability,
            mean_end_time_s: total_time / samples_f,
            timing,
            mean_ammunition_expended,
            component_disable_rate,
        },
    })
}

fn collect_first_event_time(
    outcome: &RepresentativeOutcome,
    label: &str,
    kinds: &[&str],
    samples: &mut BTreeMap<String, Vec<f64>>,
) {
    if let Some(event) = outcome
        .events
        .iter()
        .find(|event| kinds.contains(&event.kind.as_str()))
    {
        samples.entry(label.into()).or_default().push(event.time_s);
    }
}

fn distribution(mut samples: Vec<f64>) -> TimingDistribution {
    samples.sort_by(f64::total_cmp);
    let count = samples.len();
    let percentile = |fraction: f64| {
        (!samples.is_empty()).then(|| {
            let index = ((samples.len() - 1) as f64 * fraction).round() as usize;
            samples[index]
        })
    };
    TimingDistribution {
        samples: count,
        mean_s: (!samples.is_empty()).then(|| samples.iter().sum::<f64>() / count as f64),
        median_s: percentile(0.5),
        p90_s: percentile(0.9),
        min_s: samples.first().copied(),
        max_s: samples.last().copied(),
    }
}

fn run_one(
    fleet: &FleetDocument,
    scenario: &CombatScenario,
    seed: u64,
) -> Result<RepresentativeOutcome, String> {
    let mut rng = SplitMix64(seed);
    let mut ships = build_runtime(fleet, scenario)?;
    let mut events = Vec::new();
    let mut impacts = Vec::<PendingImpact>::new();
    let mut time = 0.0;
    let mut winner = None;
    while time <= scenario.duration_s {
        resolve_impacts(fleet, &mut ships, &mut impacts, time, &mut rng, &mut events)?;
        let pairs = hostile_pairs(&ships);
        for (attacker_id, target_id) in pairs {
            if ship_defeated(&ships[&attacker_id]) || ship_defeated(&ships[&target_id]) {
                continue;
            }
            let (range, closing) = geometry(&ships[&attacker_id].nav, &ships[&target_id].nav);
            update_track(
                &mut ships,
                &attacker_id,
                &target_id,
                range,
                closing,
                time,
                &mut rng,
                &mut events,
            );
            if !ships[&attacker_id].tracks.contains(&target_id) {
                continue;
            }
            fire_lasers(
                fleet,
                &mut ships,
                &attacker_id,
                &target_id,
                range,
                time,
                &mut rng,
                &mut events,
            )?;
            launch_missiles(
                fleet,
                &mut ships,
                &attacker_id,
                &target_id,
                range,
                closing,
                time,
                &mut impacts,
                &mut events,
            )?;
        }
        update_ship_statuses(&mut ships, time, &mut events);
        if let Some(team) = sole_surviving_team(&ships) {
            winner = Some(team);
            break;
        }
        if time >= scenario.duration_s {
            break;
        }
        let dt = scenario.step_s.min(scenario.duration_s - time);
        advance_map(fleet, &mut ships, time, dt)?;
        time += dt;
    }
    let mut ammunition_expended = BTreeMap::new();
    let mut resources = Vec::new();
    let mut components = Vec::new();
    for ship in ships.values() {
        for (mag, initial) in &ship.initial_ammunition {
            let remaining = ship.ammunition.get(mag).copied().unwrap_or(0);
            ammunition_expended.insert(format!("{}:{}", ship.id, mag), initial - remaining);
        }
        for c in &ship.components {
            components.push(ComponentOutcome {
                ship_id: ship.id.clone(),
                component_id: c.id.clone(),
                role: c.profile.role.clone(),
                integrity: c.integrity,
                condition: condition(c).into(),
            });
        }
        resources.push(ResourceOutcome {
            ship_id: ship.id.clone(),
            propellant_t: ship.propellant_t,
            heat_mj: ship.heat_mj,
            sink_capacity_mj: ship.sink_capacity_mj,
            flywheel_mj: ship.flywheel_mj,
            tracks: ship.tracks.len(),
            retreated: ship.retreated,
            ammunition: ship.ammunition.clone(),
        });
    }
    Ok(RepresentativeOutcome {
        seed,
        winner,
        end_time_s: time,
        ammunition_expended,
        resources,
        components,
        events,
    })
}

fn build_runtime(
    fleet: &FleetDocument,
    scenario: &CombatScenario,
) -> Result<BTreeMap<String, RuntimeShip>, String> {
    let mut out = BTreeMap::new();
    for p in &scenario.participants {
        let state = fleet.states.iter().find(|s| s.id == p.ship_id).unwrap();
        let design = fleet
            .designs
            .iter()
            .find(|d| d.id == state.design_id)
            .ok_or_else(|| format!("ship {} uses unknown design {}", state.id, state.design_id))?;
        let dry_t =
            design.structure_t + design.components.iter().map(component_mass_t).sum::<f64>();
        let mut components: Vec<RuntimeComponent> = design
            .components
            .iter()
            .map(|c| RuntimeComponent {
                id: c.id.clone(),
                profile: c.combat.clone().unwrap_or_else(|| default_combat(c)),
                integrity: 1.0,
            })
            .collect();
        components.push(RuntimeComponent {
            id: "__structure".into(),
            profile: ComponentCombatProfile {
                role: "structure".into(),
                exposure: 1.0,
                vulnerability: 0.65,
                redundancy_group: None,
                degraded_at: 0.7,
                disabled_at: 0.2,
                destroyed_at: 0.02,
            },
            integrity: 1.0,
        });
        out.insert(
            p.ship_id.clone(),
            RuntimeShip {
                id: p.ship_id.clone(),
                team: p.team.clone(),
                doctrine: p.doctrine.clone(),
                lidar_pd: p.lidar_pd.clone(),
                design_id: design.id.clone(),
                nav: scenario
                    .initial_nav
                    .get(&p.ship_id)
                    .or_else(|| fleet.system.nav.get(&p.ship_id))
                    .expect("scenario validation guarantees map state")
                    .clone(),
                mass_kg: (dry_t + state.propellant_t) * 1000.0,
                floor_kg: dry_t.max(0.001) * 1000.0,
                components,
                ammunition: state.magazines.clone(),
                initial_ammunition: state.magazines.clone(),
                tracks: BTreeSet::new(),
                last_sensor_s: BTreeMap::new(),
                fired_at: BTreeSet::new(),
                missiles_launched_at: BTreeSet::new(),
                propellant_t: state.propellant_t,
                heat_mj: state.sink_mj,
                sink_capacity_mj: state.sink_capacity_mj,
                flywheel_mj: state.flywheel_mj,
                retreated: false,
                defeat_reported: false,
            },
        );
    }
    Ok(out)
}

fn component_mass_t(c: &Component) -> f64 {
    let count = if c.kind == "laser" {
        c.count.unwrap_or(1) as f64
    } else {
        1.0
    };
    count
        * c.mass_t.unwrap_or_else(|| {
            if c.kind == "radiator_hot" || c.kind == "radiator_low" {
                let q = c.area_m2.unwrap_or(0.0)
                    * c.eps.unwrap_or(0.0)
                    * 5.670_374_419e-8
                    * c.t_k.unwrap_or(0.0).powi(4);
                q / (c.mw_per_kg.unwrap_or(1.0) * 1e9)
            } else {
                0.0
            }
        })
}

fn default_combat(c: &Component) -> ComponentCombatProfile {
    let role = match c.kind.as_str() {
        "reactor" | "nozzle" | "tank" => "drive",
        "laser" => "weapon",
        "magazine" => "magazine",
        "radiator_hot" | "radiator_low" | "heat_sink" => "radiator",
        other => other,
    };
    ComponentCombatProfile {
        role: role.into(),
        exposure: if role == "radiator" { 0.9 } else { 0.5 },
        vulnerability: 1.0,
        redundancy_group: None,
        degraded_at: 0.75,
        disabled_at: 0.25,
        destroyed_at: 0.05,
    }
}

fn hostile_pairs(ships: &BTreeMap<String, RuntimeShip>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for a in ships.values() {
        let mut targets: Vec<&RuntimeShip> = ships
            .values()
            .filter(|b| a.team != b.team && !ship_defeated(b))
            .collect();
        targets.sort_by(|x, y| {
            let xp = a.doctrine.target_priority.iter().position(|id| id == &x.id);
            let yp = a.doctrine.target_priority.iter().position(|id| id == &y.id);
            xp.cmp(&yp).then_with(|| {
                geometry(&a.nav, &x.nav)
                    .0
                    .total_cmp(&geometry(&a.nav, &y.nav).0)
            })
        });
        if let Some(target) = targets.first() {
            out.push((a.id.clone(), target.id.clone()));
        }
    }
    out
}

fn geometry(a: &NavState, b: &NavState) -> (f64, f64) {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let range = dx.hypot(dy).max(1.0);
    let closing = -((b.vx - a.vx) * dx + (b.vy - a.vy) * dy) / range;
    (range, closing)
}

#[allow(clippy::too_many_arguments)]
fn update_track(
    ships: &mut BTreeMap<String, RuntimeShip>,
    attacker_id: &str,
    target_id: &str,
    range: f64,
    closing: f64,
    time: f64,
    rng: &mut SplitMix64,
    events: &mut Vec<CombatEvent>,
) {
    let attacker = &ships[attacker_id];
    let last_sensor = attacker
        .last_sensor_s
        .get(target_id)
        .copied()
        .unwrap_or(f64::NEG_INFINITY);
    if time - last_sensor + f64::EPSILON < attacker.doctrine.sensor_cadence_s {
        return;
    }
    let mut p = (1.0 - range / attacker.doctrine.sensor_range_m.max(1.0)).clamp(0.0, 1.0);
    if let Some(mut template) = attacker.lidar_pd.clone() {
        template.target.position_m = [range, 0.0, 0.0];
        template.target.closure_velocity_m_s = closing.max(0.1);
        if let Ok(result) = physics::lidar_pd(&template) {
            p = result.summary.body_capture_probability.clamp(0.0, 1.0);
        }
    }
    let had = attacker.tracks.contains(target_id);
    let has = rng.unit() < p;
    {
        let a = ships.get_mut(attacker_id).unwrap();
        a.last_sensor_s.insert(target_id.into(), time);
        if has != had {
            if has {
                a.tracks.insert(target_id.into());
            } else {
                a.tracks.remove(target_id);
            }
            events.push(event(
                time,
                if has { "track_acquired" } else { "track_lost" },
                Some(attacker_id),
                Some(target_id),
                format!(
                    "{} {} track on {} at {:.3} Mm",
                    attacker_id,
                    if has { "acquired" } else { "lost" },
                    target_id,
                    range / 1e6
                ),
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fire_lasers(
    fleet: &FleetDocument,
    ships: &mut BTreeMap<String, RuntimeShip>,
    attacker_id: &str,
    target_id: &str,
    range: f64,
    time: f64,
    rng: &mut SplitMix64,
    events: &mut Vec<CombatEvent>,
) -> Result<(), String> {
    let doctrine = &ships[attacker_id].doctrine;
    if !doctrine.laser_fire
        || doctrine.rules_of_engagement == "hold"
        || (doctrine.rules_of_engagement == "return_fire"
            && !ships[target_id].fired_at.contains(attacker_id))
    {
        return Ok(());
    }
    let design = fleet
        .designs
        .iter()
        .find(|d| d.id == ships[attacker_id].design_id)
        .unwrap();
    for laser in design.components.iter().filter(|c| c.kind == "laser") {
        let weapon_available = ships[attacker_id]
            .components
            .iter()
            .find(|component| component.id == laser.id)
            .is_some_and(|component| !matches!(condition(component), "disabled" | "destroyed"));
        if !weapon_available {
            continue;
        }
        let mut inputs = Vec::new();
        for profile in &laser.profiles {
            if let Some(material) = fleet.materials.iter().find(|m| m.name == profile.material) {
                inputs.push(physics::LaserProfileIn {
                    name: profile.name.clone(),
                    rho: material.rho,
                    e_vap_mj: material.e_vap_mj,
                    t_pulse_s: profile.t_pulse_s,
                    threshold_mm: profile.threshold_mm,
                });
            }
        }
        if inputs.is_empty() {
            continue;
        }
        let result = physics::laser_profiles(&physics::LaserProfilesIn {
            p_beam: laser.p_beam_w.unwrap_or(0.0),
            aperture: laser.aperture_m.unwrap_or(0.0),
            lambda: laser.lambda_m.unwrap_or(0.0),
            eta_drill: fleet.settings.eta_drill,
            open_fire_factor: fleet.settings.open_fire_factor,
            profiles: inputs,
            n: Some(16),
        })?;
        let open = result.profiles.iter().map(|p| p.r_open).fold(0.0, f64::max);
        let kill = result.profiles.iter().map(|p| p.r_kill).fold(0.0, f64::max);
        let pulse_s = laser.t_pulse_s.unwrap_or(fleet.settings.pulse_ship_s);
        let beam_w = laser.p_beam_w.unwrap_or(0.0) * f64::from(laser.count.unwrap_or(1));
        let eta = laser
            .eta_wall
            .unwrap_or(fleet.settings.laser_eta_wall)
            .max(1e-6);
        let heat_mj = beam_w * (1.0 / eta - 1.0) * pulse_s / 1e6;
        let electric_mj = beam_w / eta * pulse_s / 1e6;
        let can_fire = {
            let attacker = &ships[attacker_id];
            attacker.heat_mj + heat_mj <= attacker.sink_capacity_mj
                && attacker.flywheel_mj >= electric_mj
        };
        if range <= open && can_fire && rng.unit() < 0.85 {
            {
                let attacker = ships.get_mut(attacker_id).unwrap();
                attacker.heat_mj += heat_mj;
                attacker.flywheel_mj -= electric_mj;
                attacker.fired_at.insert(target_id.into());
            }
            let damage = (0.12 * (kill / range.max(1.0)).powi(2)).clamp(0.02, 0.8);
            apply_damage(ships, target_id, damage, rng, None, time, events, "laser");
            events.push(event(
                time,
                "laser_fire",
                Some(attacker_id),
                Some(target_id),
                format!("{} fired {} at {}", attacker_id, laser.name, target_id),
            ));
            events.push(event(
                time,
                "laser_hit",
                Some(attacker_id),
                Some(target_id),
                format!("{} delivered laser fluence to {}", attacker_id, target_id),
            ));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn launch_missiles(
    fleet: &FleetDocument,
    ships: &mut BTreeMap<String, RuntimeShip>,
    attacker_id: &str,
    target_id: &str,
    range: f64,
    closing: f64,
    time: f64,
    impacts: &mut Vec<PendingImpact>,
    events: &mut Vec<CombatEvent>,
) -> Result<(), String> {
    let doctrine = &ships[attacker_id].doctrine;
    if range > doctrine.missile_range_m
        || ships[attacker_id].missiles_launched_at.contains(target_id)
        || doctrine.rules_of_engagement == "hold"
        || (doctrine.rules_of_engagement == "return_fire"
            && !ships[target_id].fired_at.contains(attacker_id))
    {
        return Ok(());
    }
    let design = fleet
        .designs
        .iter()
        .find(|d| d.id == ships[attacker_id].design_id)
        .unwrap();
    let magazine = design.components.iter().find(|c| {
        c.kind == "magazine"
            && ships[attacker_id]
                .ammunition
                .get(&c.id)
                .copied()
                .unwrap_or(0)
                > ships[attacker_id].doctrine.defensive_reserve
    });
    let Some(magazine) = magazine else {
        return Ok(());
    };
    let missile_id = magazine
        .missile_id
        .as_ref()
        .ok_or("magazine has no missile_id")?;
    let missile = fleet
        .missiles
        .iter()
        .find(|m| &m.id == missile_id)
        .ok_or("unknown magazine missile")?;
    let available =
        ships[attacker_id].ammunition[&magazine.id] - ships[attacker_id].doctrine.defensive_reserve;
    let salvo = ships[attacker_id].doctrine.missile_salvo.min(available);
    let stages = missile
        .stages
        .iter()
        .map(|s| physics::MissileStageIn {
            id: s.id.clone(),
            name: s.name.clone(),
            dry_mass_kg: s.dry_mass_kg,
            propellant_kg: s.propellant_kg,
            ve: stage_ve(fleet, s),
            a0_g: s.a0_g,
            jettison: s.jettison,
        })
        .collect::<Vec<_>>();
    let phases = missile
        .default_phases
        .iter()
        .map(|p| physics::PhaseIn {
            stage_id: p.stage_id.clone(),
            prop_frac: p.prop_frac,
            coast_to_range: p.coast_to_range_m,
        })
        .collect();
    let result = physics::intercept(&physics::InterceptIn {
        range,
        v_close0: closing,
        payload_kg: missile.payload_kg,
        stages,
        g: fleet.settings.g,
        phases,
    })?;
    let attacker = ships.get_mut(attacker_id).unwrap();
    *attacker.ammunition.get_mut(&magazine.id).unwrap() -= salvo;
    attacker.fired_at.insert(target_id.into());
    attacker.missiles_launched_at.insert(target_id.into());
    events.push(event(
        time,
        "missile_launch",
        Some(attacker_id),
        Some(target_id),
        format!(
            "{} launched {} × {} at {}",
            attacker_id, salvo, missile.name, target_id
        ),
    ));
    if result.hit {
        events.push(event(
            time,
            "missile_guidance",
            Some(attacker_id),
            Some(target_id),
            format!(
                "{} guidance predicts intercept in {:.1} s",
                missile.name,
                result.t_hit.unwrap_or(0.0)
            ),
        ));
        impacts.push(PendingImpact {
            time_s: time + result.t_hit.unwrap_or(0.0),
            attacker: attacker_id.into(),
            target: target_id.into(),
            missile_id: missile.id.clone(),
            count: salvo,
            terminal_v_m_s: result.v_terminal.unwrap_or(result.v_burnout).abs(),
        });
    }
    Ok(())
}

fn resolve_impacts(
    fleet: &FleetDocument,
    ships: &mut BTreeMap<String, RuntimeShip>,
    impacts: &mut Vec<PendingImpact>,
    time: f64,
    rng: &mut SplitMix64,
    events: &mut Vec<CombatEvent>,
) -> Result<(), String> {
    let mut later = Vec::new();
    for impact in impacts.drain(..) {
        if impact.time_s > time {
            later.push(impact);
            continue;
        }
        let missile = fleet
            .missiles
            .iter()
            .find(|m| m.id == impact.missile_id)
            .ok_or("impact uses unknown missile")?;
        let (range, closing) = geometry(&ships[&impact.target].nav, &ships[&impact.attacker].nav);
        let mut pd_probability: f64 = 0.0;
        if let Some(mut template) = ships[&impact.target].lidar_pd.clone() {
            template.target.position_m = [range.max(1.0), 0.0, 0.0];
            template.target.closure_velocity_m_s = closing.abs().max(0.1);
            if let Ok(pd) = physics::lidar_pd(&template) {
                pd_probability = if pd.summary.structural_kill_feasible {
                    pd.summary.body_capture_probability
                } else {
                    pd.summary.body_capture_probability * 0.2
                };
            }
        }
        let survivors = (0..impact.count)
            .filter(|_| rng.unit() >= pd_probability.clamp(0.0, 0.98))
            .count() as u32;
        if survivors == 0 {
            events.push(event(
                time,
                "point_defense_kill",
                Some(&impact.target),
                Some(&impact.attacker),
                format!(
                    "{} defeated the incoming {} salvo",
                    impact.target, missile.name
                ),
            ));
            continue;
        }
        let accuracy_probability = missile.terminal_effect.as_ref().map_or(0.7, |effect| {
            let target_radius_m = (ships[&impact.target].mass_kg / 1_000.0).cbrt().max(1.0);
            let sigma = effect.accuracy_sigma_m.max(1e-6);
            (1.0 - (-target_radius_m.powi(2) / (2.0 * sigma.powi(2))).exp()).clamp(0.0, 1.0)
        });
        let hits = (0..survivors)
            .filter(|_| rng.unit() < accuracy_probability)
            .count() as u32;
        if hits == 0 {
            events.push(event(
                time,
                "missile_miss",
                Some(&impact.attacker),
                Some(&impact.target),
                format!(
                    "{} surviving {} rounds missed {} at terminal guidance",
                    survivors, missile.name, impact.target
                ),
            ));
            continue;
        }
        let effect = missile
            .terminal_effect
            .as_ref()
            .map(|x| x.effect_energy_j)
            .unwrap_or(1e9);
        let kinetic = 0.5 * missile.payload_kg.max(1.0) * impact.terminal_v_m_s.powi(2);
        let target_mass = ships[&impact.target].mass_kg.max(1.0);
        let damage = (hits as f64 * (effect + kinetic) / (target_mass * 5e6)).clamp(0.15, 1.5);
        let bias = missile.terminal_effect.as_ref().map(|x| &x.component_bias);
        apply_damage(
            ships,
            &impact.target,
            damage,
            rng,
            bias,
            time,
            events,
            "missile",
        );
        events.push(event(
            time,
            "missile_hit",
            Some(&impact.attacker),
            Some(&impact.target),
            format!("{} {} rounds hit {}", hits, missile.name, impact.target),
        ));
    }
    *impacts = later;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn apply_damage(
    ships: &mut BTreeMap<String, RuntimeShip>,
    target_id: &str,
    damage: f64,
    rng: &mut SplitMix64,
    bias: Option<&BTreeMap<String, f64>>,
    time: f64,
    events: &mut Vec<CombatEvent>,
    source: &str,
) {
    let target = ships.get_mut(target_id).unwrap();
    let weights: Vec<f64> = target
        .components
        .iter()
        .map(|c| {
            c.profile.exposure.max(0.0)
                * c.profile.vulnerability.max(0.0)
                * bias
                    .and_then(|b| b.get(&c.profile.role))
                    .copied()
                    .unwrap_or(1.0)
        })
        .collect();
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return;
    }
    let mut pick = rng.unit() * total;
    let mut index = 0;
    for (i, weight) in weights.iter().enumerate() {
        if pick <= *weight {
            index = i;
            break;
        }
        pick -= weight;
    }
    let component = &mut target.components[index];
    let before = condition(component).to_string();
    component.integrity = (component.integrity - damage * component.profile.vulnerability).max(0.0);
    let after = condition(component).to_string();
    if before != after {
        events.push(event(
            time,
            "component_condition",
            None,
            Some(target_id),
            format!(
                "{} changed {} from {} to {} after {} damage",
                target_id, component.id, before, after, source
            ),
        ));
    }
}

fn condition(c: &RuntimeComponent) -> &'static str {
    if c.integrity <= c.profile.destroyed_at {
        "destroyed"
    } else if c.integrity <= c.profile.disabled_at {
        "disabled"
    } else if c.integrity <= c.profile.degraded_at {
        "degraded"
    } else {
        "intact"
    }
}

fn ship_defeated(ship: &RuntimeShip) -> bool {
    ship.retreated
        || ship.components.iter().any(|c| {
            c.profile.role == "structure" && matches!(condition(c), "disabled" | "destroyed")
        })
        || !ship
            .components
            .iter()
            .any(|c| c.profile.role == "drive" && !matches!(condition(c), "disabled" | "destroyed"))
}

fn update_ship_statuses(
    ships: &mut BTreeMap<String, RuntimeShip>,
    time: f64,
    events: &mut Vec<CombatEvent>,
) {
    for ship in ships.values_mut() {
        let functional_defeat = ship.components.iter().any(|component| {
            component.profile.role == "structure"
                && matches!(condition(component), "disabled" | "destroyed")
        }) || !ship.components.iter().any(|component| {
            component.profile.role == "drive"
                && !matches!(condition(component), "disabled" | "destroyed")
        });
        if functional_defeat && !ship.defeat_reported {
            ship.defeat_reported = true;
            events.push(event(
                time,
                "ship_defeated",
                None,
                Some(&ship.id),
                format!("{} lost mission-critical function", ship.id),
            ));
            continue;
        }
        let mission_integrity = ship
            .components
            .iter()
            .map(|component| component.integrity)
            .sum::<f64>()
            / ship.components.len().max(1) as f64;
        if !ship.retreated && mission_integrity <= ship.doctrine.retreat_integrity {
            ship.retreated = true;
            events.push(event(
                time,
                "retreat",
                Some(&ship.id),
                None,
                format!(
                    "{} disengaged at {:.1}% mission integrity",
                    ship.id,
                    mission_integrity * 100.0
                ),
            ));
        }
    }
}

fn sole_surviving_team(ships: &BTreeMap<String, RuntimeShip>) -> Option<String> {
    let teams: BTreeSet<String> = ships
        .values()
        .filter(|s| !ship_defeated(s))
        .map(|s| s.team.clone())
        .collect();
    if teams.len() == 1 {
        teams.into_iter().next()
    } else {
        None
    }
}

fn advance_map(
    fleet: &FleetDocument,
    ships: &mut BTreeMap<String, RuntimeShip>,
    time: f64,
    dt: f64,
) -> Result<(), String> {
    let bodies = fleet
        .system
        .bodies
        .iter()
        .map(|b| physics::NavBody {
            id: b.id.clone(),
            mass_kg: b.mass_kg,
            radius_m: b.radius_m,
            a_m: b.a_m.unwrap_or(0.0),
            phase0_deg: b.phase0_deg.unwrap_or(0.0),
            parent: b.parent.clone(),
        })
        .collect();
    let nav_ships = ships
        .values()
        .map(|s| physics::NavShipIn {
            id: s.id.clone(),
            x: s.nav.x,
            y: s.nav.y,
            vx: s.nav.vx,
            vy: s.nav.vy,
            mass_kg: s.mass_kg,
            m_floor_kg: s.floor_kg,
            landed_on: s.nav.landed_on.clone(),
            burn: s.nav.burn.as_ref().map(|burn| physics::NavBurnIn {
                thrust: burn.thrust_n,
                mdot: burn.mdot_kg_s,
                t_remaining_s: burn.t_remaining_s,
                mode: burn.mode.clone(),
                angle_deg: burn.angle_deg,
                target_body: burn.target_body.clone(),
                t_start_s: burn.t_start_s,
            }),
        })
        .collect();
    let output = physics::nav_tick(&physics::NavTickIn {
        g_const: fleet.settings.g_const,
        epoch_s: fleet.system.epoch_s + time,
        dt_s: dt,
        substep_s: Some(fleet.settings.map_substep_s.min(dt).max(0.5)),
        bodies,
        ships: nav_ships,
        path_points: Some(2),
    })?;
    for s in output.ships {
        if let Some(runtime) = ships.get_mut(&s.id) {
            runtime.nav.x = s.x;
            runtime.nav.y = s.y;
            runtime.nav.vx = s.vx;
            runtime.nav.vy = s.vy;
            runtime.nav.landed_on = s.landed_on;
            runtime.mass_kg = s.mass_kg;
            runtime.propellant_t = ((s.mass_kg - runtime.floor_kg) / 1000.0).max(0.0);
            if let Some(burn) = runtime.nav.burn.as_mut() {
                burn.t_remaining_s = s.burn_t_remaining_s;
                burn.t_start_s = s.burn_t_start_remaining_s;
                burn.prop_drawn_t += s.prop_used_kg / 1000.0;
                burn.dv_gained += s.dv_spent;
                if burn.t_remaining_s <= 0.0 {
                    runtime.nav.burn = None;
                }
            }
        }
    }
    Ok(())
}

fn stage_ve(fleet: &FleetDocument, stage: &crate::model::MissileStage) -> f64 {
    match stage.propulsion.as_str() {
        "am" => stage.isp_s.unwrap_or(fleet.settings.prop_am_isp_s) * fleet.settings.g,
        "fusion" => stage.isp_s.unwrap_or(fleet.settings.prop_fusion_isp_s) * fleet.settings.g,
        "custom" => stage.ve_m_s.unwrap_or(fleet.settings.prop_mh_ve_m_s),
        _ => fleet.settings.prop_mh_ve_m_s,
    }
}

fn event(
    time_s: f64,
    kind: &str,
    actor: Option<&str>,
    target: Option<&str>,
    message: String,
) -> CombatEvent {
    CombatEvent {
        time_s,
        kind: kind.into(),
        actor: actor.map(str::to_string),
        target: target.map(str::to_string),
        message,
        data: BTreeMap::new(),
    }
}

#[derive(Clone)]
struct SplitMix64(u64);
impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / ((1u64 << 53) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (FleetDocument, CombatScenario) {
        let mut fleet: FleetDocument =
            serde_json::from_str(include_str!("default_fleet.json")).unwrap();
        let mut other = fleet.states[0].clone();
        other.id = "opfor".into();
        other.name = "OPFOR".into();
        fleet.states.push(other.clone());
        let mut nav = fleet
            .system
            .nav
            .get(&fleet.states[0].id)
            .cloned()
            .unwrap_or(NavState {
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                landed_on: None,
                burn: None,
            });
        fleet
            .system
            .nav
            .insert(fleet.states[0].id.clone(), nav.clone());
        nav.x += 5_000_000.0;
        nav.vx = -1000.0;
        fleet.system.nav.insert(other.id.clone(), nav);
        let participants = vec![
            CombatParticipant {
                ship_id: fleet.states[0].id.clone(),
                team: "blue".into(),
                doctrine: CombatDoctrine::default(),
                lidar_pd: None,
            },
            CombatParticipant {
                ship_id: other.id,
                team: "red".into(),
                doctrine: CombatDoctrine::default(),
                lidar_pd: None,
            },
        ];
        (
            fleet,
            CombatScenario {
                schema_version: "1.0".into(),
                name: "fixture".into(),
                duration_s: 30.0,
                step_s: 10.0,
                seed: 7,
                samples: 3,
                objective: String::new(),
                initial_nav: BTreeMap::new(),
                participants,
            },
        )
    }

    #[test]
    fn seeded_ensemble_is_reproducible() {
        let (fleet, scenario) = fixture();
        let first = run(&fleet, &scenario).unwrap();
        assert_eq!(first.ensemble.timing.len(), 4);
        let a = serde_json::to_value(first).unwrap();
        let b = serde_json::to_value(run(&fleet, &scenario).unwrap()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn simulation_does_not_mutate_fleet() {
        let (fleet, scenario) = fixture();
        let before = serde_json::to_value(&fleet).unwrap();
        run(&fleet, &scenario).unwrap();
        assert_eq!(before, serde_json::to_value(&fleet).unwrap());
    }

    #[test]
    fn ammunition_is_conserved() {
        let (fleet, scenario) = fixture();
        let outcome = run(&fleet, &scenario).unwrap().representative;
        for resources in &outcome.resources {
            let state = fleet
                .states
                .iter()
                .find(|state| state.id == resources.ship_id)
                .unwrap();
            for (magazine, initial) in &state.magazines {
                let remaining = resources.ammunition.get(magazine).copied().unwrap_or(0);
                let expended = outcome
                    .ammunition_expended
                    .get(&format!("{}:{magazine}", resources.ship_id))
                    .copied()
                    .unwrap_or(0);
                assert_eq!(*initial, remaining + expended);
            }
        }
    }

    #[test]
    fn component_thresholds_cover_all_functional_states() {
        let profile = ComponentCombatProfile {
            role: "sensor".into(),
            exposure: 1.0,
            vulnerability: 1.0,
            redundancy_group: None,
            degraded_at: 0.75,
            disabled_at: 0.25,
            destroyed_at: 0.05,
        };
        let mut component = RuntimeComponent {
            id: "sensor".into(),
            profile,
            integrity: 1.0,
        };
        assert_eq!(condition(&component), "intact");
        component.integrity = 0.7;
        assert_eq!(condition(&component), "degraded");
        component.integrity = 0.2;
        assert_eq!(condition(&component), "disabled");
        component.integrity = 0.0;
        assert_eq!(condition(&component), "destroyed");
    }

    #[test]
    fn detection_can_be_lost_and_reacquired_on_sensor_cadence() {
        let (fleet, scenario) = fixture();
        let mut ships = build_runtime(&fleet, &scenario).unwrap();
        let blue = fleet.states[0].id.clone();
        let red = "opfor";
        let mut events = Vec::new();
        let mut rng = SplitMix64(1);
        update_track(&mut ships, &blue, red, 1.0, 0.0, 0.0, &mut rng, &mut events);
        assert!(ships[&blue].tracks.contains(red));
        update_track(
            &mut ships,
            &blue,
            red,
            1.0e12,
            0.0,
            10.0,
            &mut rng,
            &mut events,
        );
        assert!(!ships[&blue].tracks.contains(red));
        update_track(
            &mut ships,
            &blue,
            red,
            1.0,
            0.0,
            20.0,
            &mut rng,
            &mut events,
        );
        assert!(ships[&blue].tracks.contains(red));
        assert_eq!(
            events
                .iter()
                .filter(|event| event.kind == "track_acquired")
                .count(),
            2
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| event.kind == "track_lost")
                .count(),
            1
        );
    }

    #[test]
    fn laser_and_missile_engagements_use_existing_physics() {
        let (fleet, scenario) = fixture();
        let mut ships = build_runtime(&fleet, &scenario).unwrap();
        let blue = fleet.states[0].id.clone();
        let red = "opfor";
        let mut events = Vec::new();
        let mut rng = SplitMix64(3);
        fire_lasers(
            &fleet,
            &mut ships,
            &blue,
            red,
            1_000_000.0,
            0.0,
            &mut rng,
            &mut events,
        )
        .unwrap();
        assert!(events.iter().any(|event| event.kind == "laser_hit"));
        let mut impacts = Vec::new();
        launch_missiles(
            &fleet,
            &mut ships,
            &blue,
            red,
            5_000_000.0,
            1_000.0,
            0.0,
            &mut impacts,
            &mut events,
        )
        .unwrap();
        assert!(events.iter().any(|event| event.kind == "missile_launch"));
        assert!(!impacts.is_empty());
    }

    #[test]
    fn retreat_changes_victory_condition() {
        let (fleet, scenario) = fixture();
        let mut ships = build_runtime(&fleet, &scenario).unwrap();
        ships.get_mut("opfor").unwrap().doctrine.retreat_integrity = 1.0;
        let mut events = Vec::new();
        update_ship_statuses(&mut ships, 0.0, &mut events);
        assert!(ships["opfor"].retreated);
        assert_eq!(sole_surviving_team(&ships).as_deref(), Some("blue"));
        assert!(events.iter().any(|event| event.kind == "retreat"));
    }

    #[test]
    fn lidar_point_defense_can_defeat_an_incoming_round() {
        let (fleet, scenario) = fixture();
        let mut ships = build_runtime(&fleet, &scenario).unwrap();
        let mut lidar: physics::LidarPdIn =
            serde_json::from_str(include_str!("../testdata/lidar_pd_baseline.json")).unwrap();
        lidar.target.position_m = [5_000_000.0, 0.0, 0.0];
        lidar.target.closure_velocity_m_s = 1_000.0;
        let result = physics::lidar_pd(&lidar).unwrap();
        let probability = if result.summary.structural_kill_feasible {
            result.summary.body_capture_probability
        } else {
            result.summary.body_capture_probability * 0.2
        }
        .clamp(0.0, 0.98);
        assert!(probability > 0.0);
        ships.get_mut("opfor").unwrap().lidar_pd = Some(lidar);
        let seed = (0..1_000_000)
            .find(|seed| SplitMix64(*seed).unit() < probability)
            .expect("a reproducible point-defense success seed");
        let mut rng = SplitMix64(seed);
        let mut impacts = vec![PendingImpact {
            time_s: 0.0,
            attacker: fleet.states[0].id.clone(),
            target: "opfor".into(),
            missile_id: "mh164".into(),
            count: 1,
            terminal_v_m_s: 50_000.0,
        }];
        let mut events = Vec::new();
        resolve_impacts(&fleet, &mut ships, &mut impacts, 0.0, &mut rng, &mut events).unwrap();
        assert!(events
            .iter()
            .any(|event| event.kind == "point_defense_kill"));
        assert!(impacts.is_empty());
    }
}
