use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FleetDocument {
    /// Persisted fleet document schema version.
    pub schema_version: u32,
    /// Canon and engineering settings used by every calculator.
    pub settings: Settings,
    /// Target and structure materials available to laser calculations.
    pub materials: Vec<Material>,
    /// Reusable staged missile and interceptor designs.
    pub missiles: Vec<Missile>,
    /// Reusable ship designs.
    pub designs: Vec<Design>,
    /// Commissioned ship state.
    pub states: Vec<ShipState>,
    /// Human and calculator generated fleet history.
    pub events: Vec<FleetEvent>,
    /// System-map epoch, bodies, and ship navigation state.
    pub system: SystemState,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Settings {
    /// Fraction of fusion power delivered to exhaust.
    pub f_exh: f64,
    /// Nozzle conversion efficiency.
    pub eta_noz: f64,
    /// Maximum exhaust velocity in metres per second.
    pub ve_max_m_s: f64,
    /// Minimum selectable exhaust velocity in metres per second.
    pub ve_gear_min_m_s: f64,
    /// Standard gravity in metres per second squared.
    pub g: f64,
    /// Stefan-Boltzmann constant in watts per square metre kelvin to the fourth.
    pub sigma: f64,
    /// Lithium heat capacity in megajoules per kilogram.
    pub li_sink_mj_per_kg: f64,
    /// Heat dumped per kilogram of vented lithium in megajoules.
    pub li_vent_mj_per_kg: f64,
    /// Flywheel storage in megajoules per tonne.
    pub flywheel_mj_per_t: f64,
    /// Propellant mass carried per unit tank mass.
    pub tank_prop_per_mass: f64,
    /// Laser drilling cutoff speed in millimetres per second.
    pub laser_cutoff_mm_s: f64,
    /// Effective laser drilling efficiency.
    pub eta_drill: f64,
    /// Default wall-plug efficiency for lasers.
    pub laser_eta_wall: f64,
    /// Default missile-kill pulse duration in seconds.
    pub pulse_missile_s: f64,
    /// Default ship-kill pulse duration in seconds.
    pub pulse_ship_s: f64,
    /// Default penetration threshold in millimetres.
    pub kill_threshold_mm: f64,
    /// Doctrine multiplier from kill range to open-fire range.
    pub open_fire_factor: f64,
    /// Metallic-hydrogen exhaust velocity in metres per second.
    pub prop_mh_ve_m_s: f64,
    /// Antimatter-thermal specific impulse in seconds.
    pub prop_am_isp_s: f64,
    /// Fusion-bus specific impulse in seconds.
    pub prop_fusion_isp_s: f64,
    /// Auto-size target acceleration in milligee.
    pub as_target_accel_mg: f64,
    /// Auto-size reactor mass in tonnes per terawatt.
    pub as_reactor_t_per_tw: f64,
    /// Auto-size nozzle thrust-cap margin.
    pub as_nozzle_cap_factor: f64,
    /// Auto-size nozzle mass in tonnes per meganewton.
    pub as_nozzle_t_per_mn: f64,
    /// Auto-size hot-radiator reactor load fraction.
    pub as_rad_load_frac: f64,
    /// Auto-size hot-radiator temperature in kelvin.
    pub as_hot_t_k: f64,
    /// Auto-size hot-radiator emissivity.
    pub as_hot_eps: f64,
    /// Auto-size hot-radiator specific rejection in megawatts per kilogram.
    pub as_hot_mw_per_kg: f64,
    /// Low-temperature radiator area relative to hot-radiator area.
    pub as_low_area_frac: f64,
    /// Auto-size low-radiator temperature in kelvin.
    pub as_low_t_k: f64,
    /// Auto-size low-radiator emissivity.
    pub as_low_eps: f64,
    /// Auto-size low-radiator specific rejection in megawatts per kilogram.
    pub as_low_mw_per_kg: f64,
    /// Auto-size structure fraction of non-structure dry mass.
    pub as_structure_frac: f64,
    /// Auto-size heat-sink endurance in minutes.
    pub as_sink_endurance_min: f64,
    /// Additional installed sink mass factor.
    pub as_sink_extra_mass_factor: f64,
    /// Auto-size flywheel firing duration in seconds.
    pub as_flywheel_fire_s: f64,
    /// Newtonian gravitational constant in SI units.
    pub g_const: f64,
    /// Speed of light in metres per second.
    pub c_m_s: f64,
    /// Default map step in seconds.
    pub map_tick_s: f64,
    /// Maximum map integration substep in seconds.
    pub map_substep_s: f64,
    /// Default projected-course duration in days.
    pub map_project_d: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Material {
    /// Human-readable material name.
    pub name: String,
    /// Density in kilograms per cubic metre.
    pub rho: f64,
    /// Effective vaporization energy in megajoules per kilogram.
    pub e_vap_mj: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Missile {
    /// Stable missile design identifier.
    pub id: String,
    /// Human-readable missile name.
    pub name: String,
    /// Bus payload or submunition mass in kilograms.
    pub payload_kg: f64,
    /// Ordered propulsion stages.
    pub stages: Vec<MissileStage>,
    /// Default burn/coast doctrine.
    pub default_phases: Vec<MissilePhase>,
    /// Design notes and assumptions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Optional terminal-effect model used only by combat simulations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_effect: Option<TerminalEffect>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct MissileStage {
    /// Stable stage identifier.
    pub id: String,
    /// Human-readable stage name.
    pub name: String,
    /// Stage dry mass in kilograms.
    pub dry_mass_kg: f64,
    /// Stage propellant mass in kilograms.
    pub propellant_kg: f64,
    /// Propulsion preset: mh, antimatter, fusion, or custom.
    pub propulsion: String,
    /// Initial acceleration in standard gravities.
    pub a0_g: f64,
    /// Whether the stage dry mass is discarded after burnout.
    pub jettison: bool,
    /// Specific impulse in seconds for applicable propulsion presets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isp_s: Option<f64>,
    /// Custom exhaust velocity in metres per second.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ve_m_s: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct MissilePhase {
    /// Stage used by this phase.
    pub stage_id: String,
    /// Fraction of the stage propellant allocated to this phase.
    pub prop_frac: f64,
    /// Optional range in metres to coast to before the next phase.
    pub coast_to_range_m: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TerminalEffect {
    /// Delivered non-kinetic effect energy in joules.
    pub effect_energy_j: f64,
    /// One-sigma terminal miss distance in metres.
    pub accuracy_sigma_m: f64,
    /// Optional relative targeting weights keyed by combat component role.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub component_bias: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Design {
    /// Stable ship-design identifier.
    pub id: String,
    /// Human-readable design name.
    pub name: String,
    /// Narrative ship class or role.
    pub class: String,
    /// Wet-to-dry mass ratio at full load.
    pub mr: f64,
    /// Explicit structure mass in tonnes when automatic structure is disabled.
    pub structure_t: f64,
    /// Whether structure mass is automatically sized.
    pub structure_auto: bool,
    /// Design notes and assumptions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Installed ship components.
    pub components: Vec<Component>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Component {
    /// Stable component identifier unique within the design.
    pub id: String,
    /// Component kind used by design and combat calculations.
    pub kind: String,
    /// Human-readable component name.
    pub name: String,
    /// Installed mass in tonnes where the kind uses explicit mass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass_t: Option<f64>,
    /// Whether auto-sizing owns this component's size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto: Option<bool>,
    /// Whether the designer should use mass_t instead of deriving mass from the component sizing fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass_override: Option<bool>,
    /// Fusion power in watts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_fusion_w: Option<f64>,
    /// Fraction of reactor waste heat assigned to the hot radiator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rad_load_frac: Option<f64>,
    /// Nozzle thrust cap in newtons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub f_max_n: Option<f64>,
    /// Radiator rejection area in square metres.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_m2: Option<f64>,
    /// Radiator temperature in kelvin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_k: Option<f64>,
    /// Radiator emissivity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eps: Option<f64>,
    /// Component specific power or radiator specific rejection in megawatts per kilogram.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mw_per_kg: Option<f64>,
    /// Radiator sizing method: specific_power or areal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radiator_mode: Option<String>,
    /// Radiator areal density in kilograms per square metre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kg_per_m2: Option<f64>,
    /// Radiator rejection in megawatts per square metre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mw_per_m2: Option<f64>,
    /// Fraction of total laser waste heat assigned to a low-temperature radiator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub laser_waste_frac: Option<f64>,
    /// Requested storage or heat-acceptance endurance in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endurance_s: Option<f64>,
    /// Selected storage material name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
    /// Storage energy or heat capacity in megajoules per kilogram.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_mj_per_kg: Option<f64>,
    /// Installed heat-sink mass divided by active heat-storage material mass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installed_mass_factor: Option<f64>,
    /// Tank structure mass divided by carried propellant mass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tank_structure_frac: Option<f64>,
    /// Loaded missile mass divided by magazine structure mass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missile_mass_ratio: Option<f64>,
    /// Crew complement supported by a crew compartment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crew_count: Option<u32>,
    /// Compartment tonnes per supported crew member.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tonnes_per_crew: Option<f64>,
    /// Structure mass divided by the mass of the rest of the dry ship.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structure_frac: Option<f64>,
    /// Wet acceleration target used by reactor sizing actions, in milligee.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_accel_mg: Option<f64>,
    /// Lithium inventory in tonnes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub li_t: Option<f64>,
    /// Number of identical installed units.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    /// Laser optical beam power in watts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_beam_w: Option<f64>,
    /// Laser aperture diameter in metres.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aperture_m: Option<f64>,
    /// Laser wavelength in metres.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lambda_m: Option<f64>,
    /// Laser wall-plug efficiency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eta_wall: Option<f64>,
    /// Laser pulse duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_pulse_s: Option<f64>,
    /// Saved laser damage profiles.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<LaserProfile>,
    /// Magazine missile design identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missile_id: Option<String>,
    /// Magazine capacity in rounds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u32>,
    /// Component notes and assumptions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Optional functional combat-damage model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat: Option<ComponentCombatProfile>,
    /// Forward-compatible component fields not yet known to this binary.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct LaserProfile {
    /// Stable profile identifier.
    pub id: String,
    /// Human-readable profile name.
    pub name: String,
    /// Target material name.
    pub material: String,
    /// Pulse duration in seconds.
    pub t_pulse_s: f64,
    /// Required penetration in millimetres.
    pub threshold_mm: f64,
    /// Optional writer-selected reference range in metres.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen_range_m: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ComponentCombatProfile {
    /// Functional role such as drive, sensor, weapon, radiator, magazine, or structure.
    pub role: String,
    /// Relative probability of being exposed to a successful attack, from zero to one.
    pub exposure: f64,
    /// Relative damage sensitivity; one is the baseline.
    pub vulnerability: f64,
    /// Components in the same redundancy group can substitute for one another.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redundancy_group: Option<String>,
    /// Integrity at or below which the component becomes degraded.
    pub degraded_at: f64,
    /// Integrity at or below which the component becomes disabled.
    pub disabled_at: f64,
    /// Integrity at or below which the component is destroyed.
    pub destroyed_at: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ShipState {
    /// Stable commissioned-ship identifier.
    pub id: String,
    /// Human-readable commissioned-ship name.
    pub name: String,
    /// Ship design identifier.
    pub design_id: String,
    /// Current propellant in tonnes.
    pub propellant_t: f64,
    /// Compatibility scalar velocity in kilometres per second.
    pub velocity_kms: f64,
    /// Current heat stored in the lithium sink in megajoules.
    pub sink_mj: f64,
    /// Remaining lithium sink capacity in megajoules.
    pub sink_capacity_mj: f64,
    /// Current flywheel energy in megajoules.
    pub flywheel_mj: f64,
    /// Hot-radiator integrity as a percentage.
    pub radiator_hot_pct: f64,
    /// Low-temperature-radiator integrity as a percentage.
    pub radiator_low_pct: f64,
    /// Current rounds keyed by magazine component ID.
    pub magazines: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FleetEvent {
    pub id: String,
    pub ship_id: String,
    pub date: String,
    pub kind: String,
    pub note: String,
    /// Event-specific state deltas.
    pub deltas: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SystemState {
    /// Seconds from the map scenario epoch.
    pub epoch_s: f64,
    /// Gravitating system bodies.
    pub bodies: Vec<SystemBody>,
    /// Navigation state keyed by ship ID.
    pub nav: BTreeMap<String, NavState>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SystemBody {
    pub id: String,
    pub name: String,
    pub mass_kg: f64,
    pub radius_m: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase0_deg: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NavState {
    /// Inertial x coordinate in metres.
    pub x: f64,
    /// Inertial y coordinate in metres.
    pub y: f64,
    /// Inertial x velocity in metres per second.
    pub vx: f64,
    /// Inertial y velocity in metres per second.
    pub vy: f64,
    /// Body ID when landed; null in flight.
    pub landed_on: Option<String>,
    /// Optional currently programmed finite burn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burn: Option<NavBurnState>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NavBurnState {
    pub thrust_n: f64,
    pub mdot_kg_s: f64,
    pub ve_m_s: f64,
    pub mode: String,
    pub angle_deg: f64,
    pub target_body: Option<String>,
    pub t_start_s: f64,
    pub t_remaining_s: f64,
    pub prop_drawn_t: f64,
    pub dv_gained: f64,
}

impl FleetDocument {
    pub fn from_value(value: Value) -> Result<Self, String> {
        serde_json::from_value(value).map_err(|e| format!("fleet schema: {e}"))
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.schema_version != 2 {
            errors.push(format!(
                "schema_version: expected 2, got {}",
                self.schema_version
            ));
        }
        for (name, value) in [
            ("f_exh", self.settings.f_exh),
            ("eta_noz", self.settings.eta_noz),
            ("eta_drill", self.settings.eta_drill),
            ("laser_eta_wall", self.settings.laser_eta_wall),
            ("as_rad_load_frac", self.settings.as_rad_load_frac),
            ("as_hot_eps", self.settings.as_hot_eps),
            ("as_low_area_frac", self.settings.as_low_area_frac),
            ("as_low_eps", self.settings.as_low_eps),
            ("as_structure_frac", self.settings.as_structure_frac),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                errors.push(format!("settings.{name}: must be between 0 and 1"));
            }
        }
        for (name, value) in [
            ("ve_max_m_s", self.settings.ve_max_m_s),
            ("ve_gear_min_m_s", self.settings.ve_gear_min_m_s),
            ("g", self.settings.g),
            ("sigma", self.settings.sigma),
            ("g_const", self.settings.g_const),
            ("c_m_s", self.settings.c_m_s),
            ("map_tick_s", self.settings.map_tick_s),
            ("map_substep_s", self.settings.map_substep_s),
        ] {
            if !value.is_finite() || value <= 0.0 {
                errors.push(format!("settings.{name}: must be positive"));
            }
        }
        let missile_ids = unique_ids(
            self.missiles.iter().map(|x| x.id.as_str()),
            "missile",
            &mut errors,
        );
        let design_ids = unique_ids(
            self.designs.iter().map(|x| x.id.as_str()),
            "design",
            &mut errors,
        );
        let state_ids = unique_ids(
            self.states.iter().map(|x| x.id.as_str()),
            "ship state",
            &mut errors,
        );

        for design in &self.designs {
            if !design.mr.is_finite() || design.mr <= 1.0 {
                errors.push(format!("design {}: mr must be greater than 1", design.id));
            }
            if !design.structure_t.is_finite() || design.structure_t < 0.0 {
                errors.push(format!(
                    "design {}: structure_t must be non-negative",
                    design.id
                ));
            }
            let mut component_ids = BTreeSet::new();
            for component in &design.components {
                if !component_ids.insert(&component.id) {
                    errors.push(format!(
                        "design {}: duplicate component id {}",
                        design.id, component.id
                    ));
                }
                if let Some(missile_id) = &component.missile_id {
                    if !missile_ids.contains(missile_id.as_str()) {
                        errors.push(format!(
                            "design {} component {}: unknown missile {}",
                            design.id, component.id, missile_id
                        ));
                    }
                }
                if let Some(combat) = &component.combat {
                    if !(0.0..=1.0).contains(&combat.exposure)
                        || !(0.0..=1.0).contains(&combat.degraded_at)
                        || !(0.0..=1.0).contains(&combat.disabled_at)
                        || !(0.0..=1.0).contains(&combat.destroyed_at)
                        || combat.destroyed_at > combat.disabled_at
                        || combat.disabled_at > combat.degraded_at
                    {
                        errors.push(format!(
                            "design {} component {}: invalid combat thresholds or exposure",
                            design.id, component.id
                        ));
                    }
                }
                for (field, value) in [
                    ("mass_t", component.mass_t),
                    ("p_fusion_w", component.p_fusion_w),
                    ("f_max_n", component.f_max_n),
                    ("area_m2", component.area_m2),
                    ("t_k", component.t_k),
                    ("mw_per_kg", component.mw_per_kg),
                    ("kg_per_m2", component.kg_per_m2),
                    ("mw_per_m2", component.mw_per_m2),
                    ("endurance_s", component.endurance_s),
                    ("energy_mj_per_kg", component.energy_mj_per_kg),
                    ("installed_mass_factor", component.installed_mass_factor),
                    ("tank_structure_frac", component.tank_structure_frac),
                    ("missile_mass_ratio", component.missile_mass_ratio),
                    ("tonnes_per_crew", component.tonnes_per_crew),
                    ("structure_frac", component.structure_frac),
                    ("target_accel_mg", component.target_accel_mg),
                    ("li_t", component.li_t),
                    ("p_beam_w", component.p_beam_w),
                    ("aperture_m", component.aperture_m),
                    ("lambda_m", component.lambda_m),
                    ("t_pulse_s", component.t_pulse_s),
                ] {
                    if value.is_some_and(|value| !value.is_finite() || value < 0.0) {
                        errors.push(format!(
                            "design {} component {}: {field} must be finite and non-negative",
                            design.id, component.id
                        ));
                    }
                }
                for (field, value) in [
                    ("rad_load_frac", component.rad_load_frac),
                    ("eps", component.eps),
                    ("eta_wall", component.eta_wall),
                    ("laser_waste_frac", component.laser_waste_frac),
                ] {
                    if value
                        .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
                    {
                        errors.push(format!(
                            "design {} component {}: {field} must be between 0 and 1",
                            design.id, component.id
                        ));
                    }
                }
            }
        }
        for material in &self.materials {
            if !material.rho.is_finite()
                || material.rho <= 0.0
                || !material.e_vap_mj.is_finite()
                || material.e_vap_mj <= 0.0
            {
                errors.push(format!(
                    "material {}: density and vaporization energy must be positive",
                    material.name
                ));
            }
        }
        for missile in &self.missiles {
            if !missile.payload_kg.is_finite() || missile.payload_kg < 0.0 {
                errors.push(format!(
                    "missile {}: payload_kg must be non-negative",
                    missile.id
                ));
            }
            let mut stage_ids: BTreeSet<&str> = BTreeSet::new();
            for stage in &missile.stages {
                if !stage_ids.insert(stage.id.as_str()) {
                    errors.push(format!(
                        "missile {}: duplicate stage id {}",
                        missile.id, stage.id
                    ));
                }
                if !stage.dry_mass_kg.is_finite()
                    || !stage.propellant_kg.is_finite()
                    || !stage.a0_g.is_finite()
                    || stage.dry_mass_kg <= 0.0
                    || stage.propellant_kg < 0.0
                    || stage.a0_g <= 0.0
                {
                    errors.push(format!(
                        "missile {} stage {}: invalid mass or acceleration",
                        missile.id, stage.id
                    ));
                }
            }
            for phase in &missile.default_phases {
                if !stage_ids.contains(phase.stage_id.as_str()) {
                    errors.push(format!(
                        "missile {}: phase uses unknown stage {}",
                        missile.id, phase.stage_id
                    ));
                }
                if !phase.prop_frac.is_finite() || !(0.0..=1.0).contains(&phase.prop_frac) {
                    errors.push(format!(
                        "missile {} phase {}: prop_frac must be between 0 and 1",
                        missile.id, phase.stage_id
                    ));
                }
            }
            if let Some(effect) = &missile.terminal_effect {
                if !effect.effect_energy_j.is_finite()
                    || effect.effect_energy_j < 0.0
                    || !effect.accuracy_sigma_m.is_finite()
                    || effect.accuracy_sigma_m < 0.0
                {
                    errors.push(format!("missile {}: invalid terminal effect", missile.id));
                }
            }
        }
        for state in &self.states {
            if !design_ids.contains(state.design_id.as_str()) {
                errors.push(format!(
                    "ship state {}: unknown design {}",
                    state.id, state.design_id
                ));
            }
            if [
                state.propellant_t,
                state.sink_mj,
                state.sink_capacity_mj,
                state.flywheel_mj,
            ]
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
            {
                errors.push(format!(
                    "ship state {}: resource values must be non-negative",
                    state.id
                ));
            }
            if !(0.0..=100.0).contains(&state.radiator_hot_pct)
                || !(0.0..=100.0).contains(&state.radiator_low_pct)
            {
                errors.push(format!(
                    "ship state {}: radiator integrity must be 0 to 100 percent",
                    state.id
                ));
            }
        }
        for event in &self.events {
            if !state_ids.contains(event.ship_id.as_str()) {
                errors.push(format!(
                    "event {}: unknown ship {}",
                    event.id, event.ship_id
                ));
            }
        }
        for ship_id in self.system.nav.keys() {
            if !state_ids.contains(ship_id.as_str()) {
                errors.push(format!("system nav: unknown ship {ship_id}"));
            }
        }
        for body in &self.system.bodies {
            if !body.mass_kg.is_finite()
                || body.mass_kg <= 0.0
                || !body.radius_m.is_finite()
                || body.radius_m <= 0.0
            {
                errors.push(format!(
                    "system body {}: mass and radius must be positive",
                    body.id
                ));
            }
        }
        for (ship_id, nav) in &self.system.nav {
            if [nav.x, nav.y, nav.vx, nav.vy]
                .iter()
                .any(|value| !value.is_finite())
            {
                errors.push(format!(
                    "system nav {ship_id}: position and velocity must be finite"
                ));
            }
        }
        errors
    }
}

fn unique_ids<'a>(
    ids: impl Iterator<Item = &'a str>,
    label: &str,
    errors: &mut Vec<String>,
) -> BTreeSet<&'a str> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if id.is_empty() {
            errors.push(format!("{label}: empty id"));
        } else if !seen.insert(id) {
            errors.push(format!("{label}: duplicate id {id}"));
        }
    }
    seen
}

pub fn revision(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_fleet_is_typed_and_valid() {
        let fleet: FleetDocument =
            serde_json::from_str(include_str!("default_fleet.json")).expect("typed default fleet");
        assert_eq!(fleet.schema_version, 2);
        assert!(fleet.validate().is_empty(), "{:?}", fleet.validate());
    }

    #[test]
    fn typed_round_trip_preserves_default_document() {
        let original: Value = serde_json::from_str(include_str!("default_fleet.json")).unwrap();
        let fleet = FleetDocument::from_value(original.clone()).unwrap();
        let round_trip = serde_json::to_value(fleet).unwrap();
        assert_json_semantically_equal(&round_trip, &original);
    }

    fn assert_json_semantically_equal(left: &Value, right: &Value) {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => assert_eq!(a.as_f64(), b.as_f64()),
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len());
                for (a, b) in a.iter().zip(b) {
                    assert_json_semantically_equal(a, b);
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                assert_eq!(a.len(), b.len());
                for (key, a) in a {
                    let b = b.get(key).unwrap_or_else(|| panic!("missing key {key}"));
                    assert_json_semantically_equal(a, b);
                }
            }
            _ => assert_eq!(left, right),
        }
    }
}
