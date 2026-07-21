use super::{intercept, lidar_pd, CalcResult, InterceptIn, LidarPdIn, LidarWeaponIn};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

const R95_SIGMA: f64 = 2.447_746_830_680_816;
const RANGE_EPSILON_M: f64 = 1e-4;

#[derive(Deserialize, JsonSchema)]
pub struct MissileEngagementIn {
    pub schema_version: String,
    pub scenario_name: String,
    pub clock: EngagementClockIn,
    pub salvo: EngagementSalvoIn,
    /// One complete Lidar/PD input used as the sensor, target, EWAR, and chaff template.
    /// Target, jammer, and chaff positions are scaled with range while their physical
    /// sizes and other properties remain fixed.
    pub lidar_template: LidarPdIn,
    pub sensor_views: Vec<EngagementSensorViewIn>,
    #[serde(default)]
    pub interceptor: Option<EngagementInterceptorIn>,
    #[serde(default)]
    pub weapon_layers: Vec<EngagementWeaponLayerIn>,
    #[serde(default)]
    pub thermal_pools: Vec<EngagementThermalPoolIn>,
    #[serde(default)]
    pub external_events: Vec<EngagementExternalEventIn>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EngagementClockIn {
    /// Seconds after midnight at the initial checkpoint.
    pub start_time_s: f64,
    pub start_range_m: f64,
    /// Range below which checkpoint spacing changes to terminal_step_m.
    pub terminal_phase_range_m: f64,
    pub standoff_range_m: f64,
    pub closure_velocity_m_s: f64,
    pub outer_step_m: f64,
    pub terminal_step_m: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct EngagementSalvoIn {
    pub initial_buses: u32,
    pub torplets_per_bus: u32,
    #[serde(default)]
    pub initial_decoys: u32,
    #[serde(default)]
    pub initial_jammers: u32,
    pub normal_separation_range_m: f64,
    #[serde(default)]
    pub early_release: Option<EngagementEarlyReleaseIn>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EngagementEarlyReleaseIn {
    pub range_m: f64,
    pub buses: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct EngagementSensorViewIn {
    pub id: String,
    pub name: String,
    pub receiver_aperture_m: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct EngagementInterceptorIn {
    pub id: String,
    pub name: String,
    pub launch_count: u32,
    /// The standard intercept calculator derives the nominal merge epoch and range.
    pub trajectory: InterceptIn,
    /// Distributes interceptor arrivals across this much range, centered on the
    /// calculated nominal merge. The inner edge is clipped at bus separation.
    pub merge_window_m: f64,
    /// Fraction of committed interceptors assigned to a real bus rather than a decoy.
    pub real_target_fraction: f64,
    /// Conditional probability of destroying a bus once a real bus is engaged.
    pub kill_probability_if_real: f64,
    /// Conditional probability of destroying a decoy once a decoy is engaged.
    pub kill_probability_if_decoy: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EngagementTargetKind {
    Bus,
    Torplet,
    Jammer,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct EngagementRangeCountIn {
    /// The replacement count applies at this range and all smaller ranges.
    pub at_range_m: f64,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct EngagementRangeEfficiencyIn {
    /// The replacement efficiency applies at this range and all smaller ranges.
    pub at_range_m: f64,
    pub efficiency: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct EngagementWeaponLayerIn {
    pub id: String,
    pub name: String,
    pub target_kind: EngagementTargetKind,
    pub outer_range_m: f64,
    pub inner_range_m: f64,
    pub platforms: u32,
    pub channels_per_platform: u32,
    /// Each channel is evaluated with this weapon definition.
    pub weapon: LidarWeaponIn,
    pub sensor_view_id: String,
    /// Additional target-association/scheduling efficiency not represented by the
    /// single-target Lidar/PD snapshot. Lost capacity is reported, not hidden.
    pub association_efficiency: f64,
    #[serde(default)]
    pub platform_schedule: Vec<EngagementRangeCountIn>,
    #[serde(default)]
    pub efficiency_schedule: Vec<EngagementRangeEfficiencyIn>,
    #[serde(default)]
    pub thermal_pool_id: Option<String>,
    pub wall_plug_efficiency: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct EngagementThermalPoolIn {
    pub id: String,
    pub name: String,
    pub capacity_j: f64,
    #[serde(default)]
    pub initial_stored_j: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct EngagementExternalEventIn {
    pub range_m: f64,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct MissileEngagementOut {
    pub schema_version: String,
    pub scenario_name: String,
    pub summary: EngagementSummaryOut,
    pub interceptor_solution: Option<EngagementInterceptorSolutionOut>,
    pub checkpoints: Vec<EngagementCheckpointOut>,
    pub weapon_totals: Vec<EngagementWeaponTotalOut>,
    pub thermal_pools: Vec<EngagementThermalPoolOut>,
    pub assumptions: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct EngagementSummaryOut {
    pub duration_to_standoff_s: f64,
    pub standoff_clock_s: f64,
    pub standoff_clock_hms: String,
    pub initial_bus_count: u32,
    pub initial_torplet_equivalent: u32,
    pub buses_killed: u32,
    pub torplet_equivalent_removed_with_buses: u32,
    pub buses_early_released: u32,
    pub torplets_created: u32,
    pub torplets_killed: u32,
    pub torplets_at_standoff: u32,
    pub decoys_killed: u32,
    pub decoys_remaining: u32,
    pub jammers_remaining: u32,
    pub interceptors_expended: u32,
    pub conservation_check_passed: bool,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct EngagementInterceptorSolutionOut {
    pub nominal_merge_elapsed_s: f64,
    pub nominal_merge_range_m: f64,
    pub merge_outer_range_m: f64,
    pub merge_inner_range_m: f64,
    pub calculated_delta_v_m_s: f64,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct EngagementInventoryOut {
    pub buses_alive: u32,
    pub torplets_alive: u32,
    pub decoys_alive: u32,
    pub jammers_alive: u32,
    pub interceptors_remaining: u32,
    pub cumulative_bus_kills: u32,
    pub cumulative_torplet_kills: u32,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct EngagementCheckpointOut {
    pub range_m: f64,
    pub elapsed_s: f64,
    pub clock_s: f64,
    pub clock_hms: String,
    pub time_to_standoff_s: f64,
    pub interval_duration_s: f64,
    pub inventory: EngagementInventoryOut,
    pub events: Vec<String>,
    pub sensor_views: Vec<EngagementSensorSnapshotOut>,
    pub weapon_effects: Vec<EngagementWeaponEffectOut>,
    pub thermal_pools: Vec<EngagementThermalPoolOut>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct EngagementSensorSnapshotOut {
    pub id: String,
    pub name: String,
    pub receiver_aperture_m: f64,
    pub detector_state: String,
    pub fire_control_usable: bool,
    pub target_photons: f64,
    pub snr: f64,
    pub jammer_to_signal: f64,
    pub lidar_spot_diameter_m: f64,
    pub clean_centroid_r95_m: f64,
    pub interfered_centroid_r95_m: f64,
    pub centroid_inflation: f64,
    pub centroid_bias_m: f64,
    pub future_aim_r95_m: f64,
    pub body_capture_probability: f64,
    pub patch_capture_probability: f64,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct EngagementWeaponEffectOut {
    pub id: String,
    pub name: String,
    pub target_kind: String,
    pub platforms_available: u32,
    pub channels_available: u32,
    pub association_efficiency: f64,
    pub snapshot_range_m: f64,
    pub detector_state: Option<String>,
    pub fire_control_usable: Option<bool>,
    pub jammer_to_signal: Option<f64>,
    pub interfered_centroid_r95_m: Option<f64>,
    pub weapon_spot_diameter_m: Option<f64>,
    pub useful_central_lobe_power_w_per_channel: Option<f64>,
    pub average_flux_w_m2_per_channel: Option<f64>,
    pub structural_service_time_s: Option<f64>,
    pub gross_service_capacity: f64,
    pub effective_real_target_capacity: f64,
    pub shots_or_services_expended: u32,
    pub buses_killed: u32,
    pub torplets_killed: u32,
    pub decoys_killed: u32,
    pub jammers_killed: u32,
    pub nonthreat_or_failed_services: f64,
    pub heat_added_j: f64,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct EngagementWeaponTotalOut {
    pub id: String,
    pub name: String,
    pub shots_or_services_expended: u32,
    pub buses_killed: u32,
    pub torplets_killed: u32,
    pub decoys_killed: u32,
    pub jammers_killed: u32,
    pub nonthreat_or_failed_services: f64,
    pub heat_added_j: f64,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct EngagementThermalPoolOut {
    pub id: String,
    pub name: String,
    pub stored_j: f64,
    pub capacity_j: f64,
    pub saturation_fraction: f64,
    pub saturated: bool,
}

#[derive(Clone)]
struct ThermalState {
    name: String,
    stored_j: f64,
    capacity_j: f64,
}

#[derive(Default)]
struct WeaponAccumulator {
    fractional_kills: f64,
    credited_kills: u32,
    shots_or_services: u32,
    buses_killed: u32,
    torplets_killed: u32,
    decoys_killed: u32,
    jammers_killed: u32,
    nonthreat_or_failed_services: f64,
    heat_added_j: f64,
}

fn finite_positive(value: f64, path: &str) -> CalcResult<()> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(format!("{path} must be finite and greater than zero"))
    }
}

fn fraction(value: f64, path: &str) -> CalcResult<()> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(format!("{path} must be between zero and one"))
    }
}

fn validate(input: &MissileEngagementIn) -> CalcResult<()> {
    if input.schema_version != "1.0" {
        return Err(format!(
            "schema_version: expected 1.0, got {}",
            input.schema_version
        ));
    }
    let c = &input.clock;
    finite_positive(c.start_range_m, "clock.start_range_m")?;
    finite_positive(c.terminal_phase_range_m, "clock.terminal_phase_range_m")?;
    finite_positive(c.standoff_range_m, "clock.standoff_range_m")?;
    finite_positive(c.closure_velocity_m_s, "clock.closure_velocity_m_s")?;
    finite_positive(c.outer_step_m, "clock.outer_step_m")?;
    finite_positive(c.terminal_step_m, "clock.terminal_step_m")?;
    if !(c.start_range_m > c.terminal_phase_range_m
        && c.terminal_phase_range_m > c.standoff_range_m)
    {
        return Err("clock ranges must descend start > terminal phase > standoff".into());
    }
    if input.salvo.initial_buses == 0 || input.salvo.torplets_per_bus == 0 {
        return Err("salvo bus and torplet counts must be nonzero".into());
    }
    if input.salvo.normal_separation_range_m <= c.standoff_range_m
        || input.salvo.normal_separation_range_m >= c.start_range_m
    {
        return Err("salvo.normal_separation_range_m must lie inside the engagement".into());
    }
    let mut sensor_ids = HashSet::new();
    for sensor in &input.sensor_views {
        if sensor.id.trim().is_empty() || !sensor_ids.insert(sensor.id.clone()) {
            return Err("sensor view ids must be non-empty and unique".into());
        }
        finite_positive(sensor.receiver_aperture_m, "sensor receiver_aperture_m")?;
    }
    if input.sensor_views.is_empty() {
        return Err("at least one sensor view is required".into());
    }
    let mut pool_ids = HashSet::new();
    for pool in &input.thermal_pools {
        if pool.id.trim().is_empty() || !pool_ids.insert(pool.id.clone()) {
            return Err("thermal pool ids must be non-empty and unique".into());
        }
        finite_positive(pool.capacity_j, "thermal pool capacity_j")?;
        if !pool.initial_stored_j.is_finite()
            || pool.initial_stored_j < 0.0
            || pool.initial_stored_j > pool.capacity_j
        {
            return Err("thermal pool initial_stored_j must be within capacity".into());
        }
    }
    let mut weapon_ids = HashSet::new();
    for layer in &input.weapon_layers {
        if layer.id.trim().is_empty() || !weapon_ids.insert(layer.id.clone()) {
            return Err("weapon layer ids must be non-empty and unique".into());
        }
        if !sensor_ids.contains(&layer.sensor_view_id) {
            return Err(format!(
                "weapon layer {} references unknown sensor view {}",
                layer.id, layer.sensor_view_id
            ));
        }
        if let Some(pool) = &layer.thermal_pool_id {
            if !pool_ids.contains(pool) {
                return Err(format!(
                    "weapon layer {} references unknown thermal pool {}",
                    layer.id, pool
                ));
            }
        }
        if layer.outer_range_m <= layer.inner_range_m {
            return Err(format!("weapon layer {} range must descend", layer.id));
        }
        if layer.platforms == 0 || layer.channels_per_platform == 0 {
            return Err(format!(
                "weapon layer {} must have available channels",
                layer.id
            ));
        }
        fraction(
            layer.association_efficiency,
            "weapon layer association_efficiency",
        )?;
        fraction(
            layer.wall_plug_efficiency,
            "weapon layer wall_plug_efficiency",
        )?;
        if layer.wall_plug_efficiency == 0.0 {
            return Err("weapon layer wall_plug_efficiency must be greater than zero".into());
        }
        for point in &layer.efficiency_schedule {
            fraction(point.efficiency, "weapon efficiency schedule")?;
        }
    }
    if let Some(interceptor) = &input.interceptor {
        finite_positive(interceptor.merge_window_m, "interceptor.merge_window_m")?;
        fraction(
            interceptor.real_target_fraction,
            "interceptor.real_target_fraction",
        )?;
        fraction(
            interceptor.kill_probability_if_real,
            "interceptor.kill_probability_if_real",
        )?;
        fraction(
            interceptor.kill_probability_if_decoy,
            "interceptor.kill_probability_if_decoy",
        )?;
    }
    Ok(())
}

fn push_range(ranges: &mut Vec<f64>, range: f64, clock: &EngagementClockIn) {
    if range <= clock.start_range_m + RANGE_EPSILON_M
        && range >= clock.standoff_range_m - RANGE_EPSILON_M
    {
        ranges.push(range.clamp(clock.standoff_range_m, clock.start_range_m));
    }
}

fn checkpoint_ranges(
    input: &MissileEngagementIn,
    interceptor: Option<&EngagementInterceptorSolutionOut>,
) -> Vec<f64> {
    let c = &input.clock;
    let mut ranges = vec![
        c.start_range_m,
        c.terminal_phase_range_m,
        c.standoff_range_m,
    ];
    let mut range = (c.start_range_m / c.outer_step_m).floor() * c.outer_step_m;
    while range > c.terminal_phase_range_m + RANGE_EPSILON_M {
        push_range(&mut ranges, range, c);
        range -= c.outer_step_m;
    }
    range = c.terminal_phase_range_m;
    while range > c.standoff_range_m + RANGE_EPSILON_M {
        push_range(&mut ranges, range, c);
        range -= c.terminal_step_m;
    }
    push_range(&mut ranges, input.salvo.normal_separation_range_m, c);
    if let Some(early) = &input.salvo.early_release {
        push_range(&mut ranges, early.range_m, c);
    }
    if let Some(solution) = interceptor {
        push_range(&mut ranges, solution.nominal_merge_range_m, c);
        push_range(&mut ranges, solution.merge_outer_range_m, c);
        push_range(&mut ranges, solution.merge_inner_range_m, c);
    }
    for layer in &input.weapon_layers {
        push_range(&mut ranges, layer.outer_range_m, c);
        push_range(&mut ranges, layer.inner_range_m, c);
        for point in &layer.platform_schedule {
            push_range(&mut ranges, point.at_range_m, c);
        }
        for point in &layer.efficiency_schedule {
            push_range(&mut ranges, point.at_range_m, c);
        }
    }
    for event in &input.external_events {
        push_range(&mut ranges, event.range_m, c);
    }
    ranges.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));
    ranges.dedup_by(|a, b| (*a - *b).abs() <= RANGE_EPSILON_M);
    ranges
}

fn clock_hms(seconds: f64) -> String {
    let rounded = seconds.round().rem_euclid(86_400.0) as u32;
    format!(
        "{:02}:{:02}:{:02}",
        rounded / 3600,
        (rounded % 3600) / 60,
        rounded % 60
    )
}

fn scaled_lidar_input(
    template: &LidarPdIn,
    range_m: f64,
    receiver_aperture_m: f64,
    weapon: Option<&LidarWeaponIn>,
) -> LidarPdIn {
    let mut input = template.clone();
    let original_range = (template.target.position_m[0].powi(2)
        + template.target.position_m[1].powi(2)
        + template.target.position_m[2].powi(2))
    .sqrt()
    .max(1e-9);
    let scale = range_m / original_range;
    for value in &mut input.target.position_m {
        *value *= scale;
    }
    for jammer in &mut input.jammers {
        for value in &mut jammer.position_m {
            *value *= scale;
        }
    }
    for cloud in &mut input.chaff {
        for value in &mut cloud.position_m {
            *value *= scale;
        }
    }
    input.detector.receiver_aperture_m = receiver_aperture_m;
    if let Some(weapon) = weapon {
        input.weapon = weapon.clone();
    }
    input.scenario_name = format!("{} at {:.3} km", template.scenario_name, range_m / 1000.0);
    input
}

fn sensor_snapshot(
    template: &LidarPdIn,
    sensor: &EngagementSensorViewIn,
    range_m: f64,
) -> CalcResult<EngagementSensorSnapshotOut> {
    let output = lidar_pd(&scaled_lidar_input(
        template,
        range_m,
        sensor.receiver_aperture_m,
        None,
    ))?;
    let clean_r95 = output.detector.photon_centroid_sigma_clean_rad * range_m * R95_SIGMA;
    let actual_r95 = output.summary.measurement_r95_m;
    Ok(EngagementSensorSnapshotOut {
        id: sensor.id.clone(),
        name: sensor.name.clone(),
        receiver_aperture_m: sensor.receiver_aperture_m,
        detector_state: serde_json::to_value(&output.summary.detector_state)
            .ok()
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| "unknown".into()),
        fire_control_usable: output.summary.fire_control_usable,
        target_photons: output.summary.target_photons,
        snr: output.summary.snr,
        jammer_to_signal: output.summary.jammer_to_signal,
        lidar_spot_diameter_m: output.signal.lidar_spot_diameter_m,
        clean_centroid_r95_m: clean_r95,
        interfered_centroid_r95_m: actual_r95,
        centroid_inflation: actual_r95 / clean_r95.max(1e-30),
        centroid_bias_m: output.summary.centroid_bias_m,
        future_aim_r95_m: output.summary.future_aim_r95_m,
        body_capture_probability: output.summary.body_capture_probability,
        patch_capture_probability: output.summary.patch_capture_probability,
    })
}

fn platform_count(layer: &EngagementWeaponLayerIn, range_m: f64) -> u32 {
    let mut count = layer.platforms;
    let mut schedule = layer.platform_schedule.clone();
    schedule.sort_by(|a, b| {
        b.at_range_m
            .partial_cmp(&a.at_range_m)
            .unwrap_or(Ordering::Equal)
    });
    for point in schedule {
        if range_m <= point.at_range_m + RANGE_EPSILON_M {
            count = point.count;
        }
    }
    count
}

fn association_efficiency(layer: &EngagementWeaponLayerIn, range_m: f64) -> f64 {
    let mut efficiency = layer.association_efficiency;
    let mut schedule = layer.efficiency_schedule.clone();
    schedule.sort_by(|a, b| {
        b.at_range_m
            .partial_cmp(&a.at_range_m)
            .unwrap_or(Ordering::Equal)
    });
    for point in schedule {
        if range_m <= point.at_range_m + RANGE_EPSILON_M {
            efficiency = point.efficiency;
        }
    }
    efficiency
}

fn thermal_outputs(states: &HashMap<String, ThermalState>) -> Vec<EngagementThermalPoolOut> {
    let mut pools: Vec<_> = states
        .iter()
        .map(|(id, state)| EngagementThermalPoolOut {
            id: id.clone(),
            name: state.name.clone(),
            stored_j: state.stored_j,
            capacity_j: state.capacity_j,
            saturation_fraction: state.stored_j / state.capacity_j,
            saturated: state.stored_j >= state.capacity_j - 1e-6,
        })
        .collect();
    pools.sort_by(|a, b| a.id.cmp(&b.id));
    pools
}

fn interceptor_solution(
    input: &MissileEngagementIn,
) -> CalcResult<Option<EngagementInterceptorSolutionOut>> {
    let Some(config) = &input.interceptor else {
        return Ok(None);
    };
    let output = intercept(&config.trajectory)?;
    let hit_time = output.t_hit.ok_or_else(|| {
        format!(
            "interceptor trajectory does not hit: {:?}",
            output.miss_reason
        )
    })?;
    let nominal_range = (input.clock.start_range_m - input.clock.closure_velocity_m_s * hit_time)
        .max(input.clock.standoff_range_m);
    let half_window = config.merge_window_m / 2.0;
    let outer = (nominal_range + half_window).min(input.clock.start_range_m);
    let inner = (nominal_range - half_window)
        .max(input.salvo.normal_separation_range_m)
        .max(input.clock.standoff_range_m);
    Ok(Some(EngagementInterceptorSolutionOut {
        nominal_merge_elapsed_s: hit_time,
        nominal_merge_range_m: nominal_range,
        merge_outer_range_m: outer,
        merge_inner_range_m: inner,
        calculated_delta_v_m_s: output.dv_total,
    }))
}

pub fn missile_engagement(input: &MissileEngagementIn) -> CalcResult<MissileEngagementOut> {
    validate(input)?;
    let interceptor_solution = interceptor_solution(input)?;
    let ranges = checkpoint_ranges(input, interceptor_solution.as_ref());
    let c = &input.clock;

    let mut buses_alive = input.salvo.initial_buses;
    let mut torplets_alive = 0u32;
    let mut decoys_alive = input.salvo.initial_decoys;
    let mut jammers_alive = input.salvo.initial_jammers;
    let mut buses_killed = 0u32;
    let mut buses_early_released = 0u32;
    let mut torplets_created = 0u32;
    let mut torplets_killed = 0u32;
    let mut decoys_killed = 0u32;
    let mut interceptors_expended = 0u32;
    let mut interceptor_shot_accumulator = 0.0f64;
    let mut interceptor_kill_accumulator = 0.0f64;
    let mut interceptor_decoy_accumulator = 0.0f64;

    let mut thermal_states: HashMap<String, ThermalState> = input
        .thermal_pools
        .iter()
        .map(|pool| {
            (
                pool.id.clone(),
                ThermalState {
                    name: pool.name.clone(),
                    stored_j: pool.initial_stored_j,
                    capacity_j: pool.capacity_j,
                },
            )
        })
        .collect();
    let mut accumulators: HashMap<String, WeaponAccumulator> = input
        .weapon_layers
        .iter()
        .map(|layer| (layer.id.clone(), WeaponAccumulator::default()))
        .collect();
    if let Some(config) = &input.interceptor {
        accumulators.insert(config.id.clone(), WeaponAccumulator::default());
    }

    let mut checkpoints = Vec::with_capacity(ranges.len());
    for (index, &range_m) in ranges.iter().enumerate() {
        let previous_range = if index == 0 {
            range_m
        } else {
            ranges[index - 1]
        };
        let interval_duration_s = (previous_range - range_m) / c.closure_velocity_m_s;
        let midpoint_range = (previous_range + range_m) / 2.0;
        let elapsed_s = (c.start_range_m - range_m) / c.closure_velocity_m_s;
        let mut events = Vec::new();
        let mut weapon_effects = Vec::new();

        if index > 0 {
            if let (Some(config), Some(solution)) = (&input.interceptor, &interceptor_solution) {
                let overlap_outer = previous_range.min(solution.merge_outer_range_m);
                let overlap_inner = range_m.max(solution.merge_inner_range_m);
                if overlap_outer > overlap_inner + RANGE_EPSILON_M {
                    let window = (solution.merge_outer_range_m - solution.merge_inner_range_m)
                        .max(RANGE_EPSILON_M);
                    interceptor_shot_accumulator +=
                        config.launch_count as f64 * (overlap_outer - overlap_inner) / window;
                    let desired = interceptor_shot_accumulator
                        .floor()
                        .min(config.launch_count as f64) as u32;
                    let shots = desired.saturating_sub(interceptors_expended);
                    interceptors_expended += shots;
                    interceptor_kill_accumulator += shots as f64
                        * config.real_target_fraction
                        * config.kill_probability_if_real;
                    interceptor_decoy_accumulator += shots as f64
                        * (1.0 - config.real_target_fraction)
                        * config.kill_probability_if_decoy;
                    let desired_bus_kills = interceptor_kill_accumulator.floor() as u32;
                    let previous_bus_kills = accumulators[&config.id].buses_killed;
                    let bus_kills = desired_bus_kills
                        .saturating_sub(previous_bus_kills)
                        .min(buses_alive);
                    buses_alive -= bus_kills;
                    buses_killed += bus_kills;
                    let desired_decoy_kills = interceptor_decoy_accumulator.floor() as u32;
                    let previous_decoy_kills = accumulators[&config.id].decoys_killed;
                    let interval_decoy_kills = desired_decoy_kills
                        .saturating_sub(previous_decoy_kills)
                        .min(decoys_alive);
                    decoys_alive -= interval_decoy_kills;
                    decoys_killed += interval_decoy_kills;
                    let accumulator = accumulators.get_mut(&config.id).unwrap();
                    accumulator.shots_or_services += shots;
                    accumulator.buses_killed += bus_kills;
                    accumulator.decoys_killed += interval_decoy_kills;
                    accumulator.nonthreat_or_failed_services +=
                        shots.saturating_sub(bus_kills + interval_decoy_kills) as f64;
                    weapon_effects.push(EngagementWeaponEffectOut {
                        id: config.id.clone(),
                        name: config.name.clone(),
                        target_kind: "bus".into(),
                        platforms_available: 1,
                        channels_available: 1,
                        association_efficiency: config.real_target_fraction,
                        snapshot_range_m: midpoint_range,
                        detector_state: None,
                        fire_control_usable: None,
                        jammer_to_signal: None,
                        interfered_centroid_r95_m: None,
                        weapon_spot_diameter_m: None,
                        useful_central_lobe_power_w_per_channel: None,
                        average_flux_w_m2_per_channel: None,
                        structural_service_time_s: None,
                        gross_service_capacity: shots as f64,
                        effective_real_target_capacity: shots as f64
                            * config.real_target_fraction
                            * config.kill_probability_if_real,
                        shots_or_services_expended: shots,
                        buses_killed: bus_kills,
                        torplets_killed: 0,
                        decoys_killed: interval_decoy_kills,
                        jammers_killed: 0,
                        nonthreat_or_failed_services: shots
                            .saturating_sub(bus_kills + interval_decoy_kills)
                            as f64,
                        heat_added_j: 0.0,
                    });
                    if bus_kills > 0 || interval_decoy_kills > 0 {
                        events.push(format!(
                            "{} destroyed {} buses and {} decoys",
                            config.name, bus_kills, interval_decoy_kills
                        ));
                    }
                }
            }

            for layer in &input.weapon_layers {
                if midpoint_range > layer.outer_range_m + RANGE_EPSILON_M
                    || midpoint_range < layer.inner_range_m - RANGE_EPSILON_M
                {
                    continue;
                }
                let available_targets = match layer.target_kind {
                    EngagementTargetKind::Bus => buses_alive,
                    EngagementTargetKind::Torplet => torplets_alive,
                    EngagementTargetKind::Jammer => jammers_alive,
                };
                if available_targets == 0 {
                    continue;
                }
                let sensor = input
                    .sensor_views
                    .iter()
                    .find(|sensor| sensor.id == layer.sensor_view_id)
                    .unwrap();
                let platforms = platform_count(layer, midpoint_range);
                let channels = platforms.saturating_mul(layer.channels_per_platform);
                if channels == 0 {
                    continue;
                }
                let snapshot = lidar_pd(&scaled_lidar_input(
                    &input.lidar_template,
                    midpoint_range,
                    sensor.receiver_aperture_m,
                    Some(&layer.weapon),
                ))?;
                let service_s = snapshot.point_defense.effective_structural_kill_time_s
                    + layer.weapon.slew_time_s;
                let gross_capacity = interval_duration_s * channels as f64 / service_s.max(1e-30);
                let efficiency = association_efficiency(layer, midpoint_range);
                let mut effective_capacity = gross_capacity * efficiency;
                let average_optical_w =
                    layer.weapon.listed_optical_power_w * layer.weapon.duty_cycle * channels as f64;
                let desired_heat = average_optical_w
                    * (1.0 / layer.wall_plug_efficiency - 1.0)
                    * interval_duration_s;
                let mut heat_added = desired_heat;
                if let Some(pool_id) = &layer.thermal_pool_id {
                    let pool = thermal_states.get_mut(pool_id).unwrap();
                    let remaining = (pool.capacity_j - pool.stored_j).max(0.0);
                    let thermal_fraction = if desired_heat > 0.0 {
                        (remaining / desired_heat).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    effective_capacity *= thermal_fraction;
                    heat_added = desired_heat * thermal_fraction;
                    pool.stored_j += heat_added;
                    if thermal_fraction < 1.0 {
                        events.push(format!("{} was thermally limited", layer.name));
                    }
                }
                let accumulator = accumulators.get_mut(&layer.id).unwrap();
                accumulator.fractional_kills += effective_capacity;
                let desired_total = accumulator.fractional_kills.floor() as u32;
                let requested_kills = desired_total.saturating_sub(accumulator.credited_kills);
                let kills = requested_kills.min(available_targets);
                accumulator.credited_kills += kills;
                let services = gross_capacity.floor() as u32;
                accumulator.shots_or_services += services;
                accumulator.nonthreat_or_failed_services +=
                    (gross_capacity - kills as f64).max(0.0);
                accumulator.heat_added_j += heat_added;
                let (bus_kills, torplet_kills, jammer_kills) = match layer.target_kind {
                    EngagementTargetKind::Bus => {
                        buses_alive -= kills;
                        buses_killed += kills;
                        accumulator.buses_killed += kills;
                        (kills, 0, 0)
                    }
                    EngagementTargetKind::Torplet => {
                        torplets_alive -= kills;
                        torplets_killed += kills;
                        accumulator.torplets_killed += kills;
                        (0, kills, 0)
                    }
                    EngagementTargetKind::Jammer => {
                        jammers_alive -= kills;
                        accumulator.jammers_killed += kills;
                        (0, 0, kills)
                    }
                };
                if kills > 0 {
                    events.push(format!("{} destroyed {} targets", layer.name, kills));
                }
                weapon_effects.push(EngagementWeaponEffectOut {
                    id: layer.id.clone(),
                    name: layer.name.clone(),
                    target_kind: match layer.target_kind {
                        EngagementTargetKind::Bus => "bus",
                        EngagementTargetKind::Torplet => "torplet",
                        EngagementTargetKind::Jammer => "jammer",
                    }
                    .into(),
                    platforms_available: platforms,
                    channels_available: channels,
                    association_efficiency: efficiency,
                    snapshot_range_m: midpoint_range,
                    detector_state: Some(
                        serde_json::to_value(&snapshot.summary.detector_state)
                            .ok()
                            .and_then(|value| value.as_str().map(ToOwned::to_owned))
                            .unwrap_or_else(|| "unknown".into()),
                    ),
                    fire_control_usable: Some(snapshot.summary.fire_control_usable),
                    jammer_to_signal: Some(snapshot.summary.jammer_to_signal),
                    interfered_centroid_r95_m: Some(snapshot.summary.measurement_r95_m),
                    weapon_spot_diameter_m: Some(snapshot.point_defense.weapon_spot_diameter_m),
                    useful_central_lobe_power_w_per_channel: Some(
                        snapshot.point_defense.useful_central_lobe_power_w,
                    ),
                    average_flux_w_m2_per_channel: Some(snapshot.point_defense.average_flux_w_m2),
                    structural_service_time_s: Some(service_s),
                    gross_service_capacity: gross_capacity,
                    effective_real_target_capacity: effective_capacity,
                    shots_or_services_expended: services,
                    buses_killed: bus_kills,
                    torplets_killed: torplet_kills,
                    decoys_killed: 0,
                    jammers_killed: jammer_kills,
                    nonthreat_or_failed_services: (gross_capacity - kills as f64).max(0.0),
                    heat_added_j: heat_added,
                });
                let _ = jammer_kills;
            }

            if let Some(early) = &input.salvo.early_release {
                if (range_m - early.range_m).abs() <= RANGE_EPSILON_M {
                    let released = early.buses.min(buses_alive);
                    buses_alive -= released;
                    buses_early_released += released;
                    let created = released.saturating_mul(input.salvo.torplets_per_bus);
                    torplets_alive += created;
                    torplets_created += created;
                    events.push(format!(
                        "{} buses released {} torplets early",
                        released, created
                    ));
                }
            }
            if (range_m - input.salvo.normal_separation_range_m).abs() <= RANGE_EPSILON_M {
                let separated = buses_alive;
                buses_alive = 0;
                let created = separated.saturating_mul(input.salvo.torplets_per_bus);
                torplets_alive += created;
                torplets_created += created;
                events.push(format!(
                    "{} buses separated into {} torplets",
                    separated, created
                ));
            }
        }

        for event in &input.external_events {
            if (range_m - event.range_m).abs() <= RANGE_EPSILON_M {
                events.push(event.description.clone());
            }
        }
        if (range_m - c.standoff_range_m).abs() <= RANGE_EPSILON_M && torplets_alive > 0 {
            events.push(format!("{} torplets reached standoff", torplets_alive));
        }

        let sensor_views = input
            .sensor_views
            .iter()
            .map(|sensor| sensor_snapshot(&input.lidar_template, sensor, range_m))
            .collect::<CalcResult<Vec<_>>>()?;
        let remaining_interceptors = input
            .interceptor
            .as_ref()
            .map(|config| config.launch_count.saturating_sub(interceptors_expended))
            .unwrap_or(0);
        checkpoints.push(EngagementCheckpointOut {
            range_m,
            elapsed_s,
            clock_s: c.start_time_s + elapsed_s,
            clock_hms: clock_hms(c.start_time_s + elapsed_s),
            time_to_standoff_s: (range_m - c.standoff_range_m) / c.closure_velocity_m_s,
            interval_duration_s,
            inventory: EngagementInventoryOut {
                buses_alive,
                torplets_alive,
                decoys_alive,
                jammers_alive,
                interceptors_remaining: remaining_interceptors,
                cumulative_bus_kills: buses_killed,
                cumulative_torplet_kills: torplets_killed,
            },
            events,
            sensor_views,
            weapon_effects,
            thermal_pools: thermal_outputs(&thermal_states),
        });
    }

    let initial_equivalent = input
        .salvo
        .initial_buses
        .saturating_mul(input.salvo.torplets_per_bus);
    let equivalent_removed = buses_killed.saturating_mul(input.salvo.torplets_per_bus);
    let conservation = initial_equivalent
        == equivalent_removed
            .saturating_add(torplets_killed)
            .saturating_add(torplets_alive);
    let mut weapon_totals = Vec::new();
    if let Some(config) = &input.interceptor {
        let total = &accumulators[&config.id];
        weapon_totals.push(EngagementWeaponTotalOut {
            id: config.id.clone(),
            name: config.name.clone(),
            shots_or_services_expended: total.shots_or_services,
            buses_killed: total.buses_killed,
            torplets_killed: 0,
            decoys_killed: total.decoys_killed,
            jammers_killed: 0,
            nonthreat_or_failed_services: total.nonthreat_or_failed_services,
            heat_added_j: 0.0,
        });
    }
    for layer in &input.weapon_layers {
        let total = &accumulators[&layer.id];
        weapon_totals.push(EngagementWeaponTotalOut {
            id: layer.id.clone(),
            name: layer.name.clone(),
            shots_or_services_expended: total.shots_or_services,
            buses_killed: total.buses_killed,
            torplets_killed: total.torplets_killed,
            decoys_killed: total.decoys_killed,
            jammers_killed: total.jammers_killed,
            nonthreat_or_failed_services: total.nonthreat_or_failed_services,
            heat_added_j: total.heat_added_j,
        });
    }
    let duration = (c.start_range_m - c.standoff_range_m) / c.closure_velocity_m_s;
    let mut warnings = vec![
        "Laser checkpoints call the constant-range Lidar/PD model at each interval midpoint; closure within that interval is not sub-stepped.".into(),
        "Association efficiency and interceptor conditional kill probabilities are scenario inputs, not predictions produced by Lidar/PD.".into(),
        "Expected fractional capacities are accumulated deterministically and credited as whole kills without Monte Carlo randomness.".into(),
    ];
    if !conservation {
        warnings.push("torplet-equivalent conservation check failed".into());
    }
    Ok(MissileEngagementOut {
        schema_version: "1.0".into(),
        scenario_name: input.scenario_name.clone(),
        summary: EngagementSummaryOut {
            duration_to_standoff_s: duration,
            standoff_clock_s: c.start_time_s + duration,
            standoff_clock_hms: clock_hms(c.start_time_s + duration),
            initial_bus_count: input.salvo.initial_buses,
            initial_torplet_equivalent: initial_equivalent,
            buses_killed,
            torplet_equivalent_removed_with_buses: equivalent_removed,
            buses_early_released,
            torplets_created,
            torplets_killed,
            torplets_at_standoff: torplets_alive,
            decoys_killed,
            decoys_remaining: decoys_alive,
            jammers_remaining: jammers_alive,
            interceptors_expended,
            conservation_check_passed: conservation,
        },
        interceptor_solution,
        checkpoints,
        weapon_totals,
        thermal_pools: thermal_outputs(&thermal_states),
        assumptions: vec![
            "Range decreases linearly at the configured closure velocity.".into(),
            "Bus kills remove every carried torplet; release changes inventory form but not torplet-equivalent count.".into(),
            "All weapon layers share the live inventory, so a target can be credited to only one weapon.".into(),
            "Template target, jammer, and chaff positions retain their initial relative geometry as range changes.".into(),
            "Gross laser service capacity is reduced by the explicit association-efficiency schedule and by thermal availability.".into(),
        ],
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> MissileEngagementIn {
        let lidar: LidarPdIn =
            serde_json::from_str(include_str!("../../testdata/lidar_pd_baseline.json")).unwrap();
        MissileEngagementIn {
            schema_version: "1.0".into(),
            scenario_name: "test".into(),
            clock: EngagementClockIn {
                start_time_s: 0.0,
                start_range_m: 25_000_000.0,
                terminal_phase_range_m: 20_000_000.0,
                standoff_range_m: 50_000.0,
                closure_velocity_m_s: 100_000.0,
                outer_step_m: 10_000_000.0,
                terminal_step_m: 500_000.0,
            },
            salvo: EngagementSalvoIn {
                initial_buses: 10,
                torplets_per_bus: 8,
                initial_decoys: 0,
                initial_jammers: 1,
                normal_separation_range_m: 20_000_000.0,
                early_release: Some(EngagementEarlyReleaseIn {
                    range_m: 22_000_000.0,
                    buses: 2,
                }),
            },
            lidar_template: lidar,
            sensor_views: vec![EngagementSensorViewIn {
                id: "sensor".into(),
                name: "Sensor".into(),
                receiver_aperture_m: 1.0,
            }],
            interceptor: None,
            weapon_layers: vec![EngagementWeaponLayerIn {
                id: "laser".into(),
                name: "Laser".into(),
                target_kind: EngagementTargetKind::Torplet,
                outer_range_m: 10_000_000.0,
                inner_range_m: 50_000.0,
                platforms: 1,
                channels_per_platform: 1,
                weapon: LidarWeaponIn {
                    wavelength_m: 5.32e-7,
                    aperture_m: 30.0,
                    m2: 1.0,
                    listed_optical_power_w: 1e9,
                    central_lobe_fraction: 0.84,
                    duty_cycle: 0.5,
                    slew_time_s: 0.05,
                },
                sensor_view_id: "sensor".into(),
                association_efficiency: 0.1,
                platform_schedule: vec![],
                efficiency_schedule: vec![],
                thermal_pool_id: None,
                wall_plug_efficiency: 0.15,
            }],
            thermal_pools: vec![],
            external_events: vec![],
        }
    }

    #[test]
    fn checkpoints_switch_to_terminal_spacing_and_end_at_standoff() {
        let output = missile_engagement(&baseline()).unwrap();
        let ranges: Vec<_> = output
            .checkpoints
            .iter()
            .map(|point| point.range_m)
            .collect();
        assert!(ranges.contains(&20_000_000.0));
        assert!(ranges.contains(&19_500_000.0));
        assert_eq!(*ranges.last().unwrap(), 50_000.0);
    }

    #[test]
    fn inventory_is_conserved_and_kills_are_not_duplicated() {
        let output = missile_engagement(&baseline()).unwrap();
        assert!(output.summary.conservation_check_passed);
        assert_eq!(
            output.summary.initial_torplet_equivalent,
            output.summary.torplet_equivalent_removed_with_buses
                + output.summary.torplets_killed
                + output.summary.torplets_at_standoff
        );
        let attributed: u32 = output
            .weapon_totals
            .iter()
            .map(|weapon| weapon.torplets_killed)
            .sum();
        assert_eq!(attributed, output.summary.torplets_killed);
    }

    #[test]
    fn early_and_normal_release_create_all_surviving_torplets() {
        let mut input = baseline();
        input.weapon_layers.clear();
        let output = missile_engagement(&input).unwrap();
        assert_eq!(output.summary.buses_early_released, 2);
        assert_eq!(output.summary.torplets_created, 80);
        assert_eq!(output.summary.torplets_at_standoff, 80);
    }
}
