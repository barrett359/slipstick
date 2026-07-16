use super::CalcResult;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::f64::consts::PI;

const C: f64 = 299_792_458.0;
const H: f64 = 6.626_070_15e-34;
const EPS: f64 = 1e-30;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LidarPdIn {
    pub schema_version: String,
    pub scenario_name: String,
    pub detector: LidarDetectorIn,
    pub target: LidarTargetIn,
    pub fire_control: LidarFireControlIn,
    pub weapon: LidarWeaponIn,
    #[serde(default)]
    pub jammers: Vec<LidarJammerIn>,
    #[serde(default)]
    pub chaff: Vec<LidarChaffIn>,
    #[serde(default)]
    pub options: LidarOptionsIn,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LidarOptionsIn {
    #[serde(default = "yes")]
    pub include_disabled_entries: bool,
    #[serde(default = "yes")]
    pub return_intermediates: bool,
}

const fn yes() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LidarDetectorIn {
    pub wavelength_m: f64,
    pub transmitter_power_w: f64,
    pub transmitter_aperture_m: f64,
    pub transmitter_m2: f64,
    pub receiver_aperture_m: f64,
    pub integration_time_s: f64,
    pub optical_throughput: f64,
    pub quantum_efficiency: f64,
    pub filter_center_m: f64,
    pub filter_fwhm_m: f64,
    pub gate_width_s: f64,
    pub pulse_repetition_hz: f64,
    pub pixel_scale_rad: f64,
    pub tracking_window_radius_rad: f64,
    pub detector_floor_sigma_rad: f64,
    pub background_photons: f64,
    pub dark_counts: f64,
    pub read_noise_e: f64,
    pub full_well_e: f64,
    pub recovery_time_s: f64,
    pub track_snr_min: f64,
    pub fire_control_snr_min: f64,
    pub ambiguity_ratio_min: f64,
    pub speckle_single_sigma_rad: f64,
    pub speckle_temporal_modes: f64,
    pub speckle_wavelength_modes: f64,
    pub speckle_polarization_modes: f64,
    pub speckle_view_modes: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LidarTargetIn {
    pub position_m: [f64; 3],
    pub projected_area_m2: f64,
    pub characteristic_diameter_m: f64,
    pub body_radius_m: f64,
    pub vulnerable_patch_radius_m: f64,
    pub uv_reflectivity: f64,
    pub aspect_factor: f64,
    pub closure_velocity_m_s: f64,
    pub warhead_standoff_m: f64,
    pub soft_kill_fluence_j_m2: f64,
    pub structural_kill_fluence_j_m2: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LidarFireControlIn {
    pub processing_latency_s: f64,
    pub position_sigma_m: f64,
    pub velocity_sigma_m_s: f64,
    pub acceleration_sigma_m_s2: f64,
    pub maneuver_persistence_s: f64,
    pub boresight_sigma_rad: f64,
    pub platform_sigma_rad: f64,
    pub beam_sigma_rad: f64,
    pub reacquisition_time_s: f64,
    pub minimum_capture_probability: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LidarWeaponIn {
    pub wavelength_m: f64,
    pub aperture_m: f64,
    pub m2: f64,
    pub listed_optical_power_w: f64,
    pub central_lobe_fraction: f64,
    pub duty_cycle: f64,
    pub slew_time_s: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LidarJammerMode {
    Noise,
    FalseSource,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LidarJammerIn {
    pub id: String,
    pub enabled: bool,
    pub mode: LidarJammerMode,
    pub position_m: [f64; 3],
    pub optical_power_w: f64,
    pub aperture_m: f64,
    pub m2: f64,
    pub wavelength_m: f64,
    pub spectral_fwhm_m: f64,
    pub pointing_error_rad: f64,
    pub polarization_overlap: f64,
    pub temporal_overlap: f64,
    pub code_correlation: f64,
    pub range_response: f64,
    pub central_lobe_fraction: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LidarChaffIn {
    pub id: String,
    pub enabled: bool,
    pub position_m: [f64; 3],
    pub width_m: f64,
    pub height_m: f64,
    pub depth_m: f64,
    pub age_s: f64,
    pub expansion_speed_m_s: f64,
    pub optical_depth: f64,
    pub single_scatter_albedo: f64,
    pub backscatter_fraction: f64,
    pub range_response: f64,
    pub clearance_fluence_j_m2: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)] // Invalid/recovering are reserved response states; v1 is a valid single epoch.
pub enum LidarDetectorState {
    Invalid,
    Saturated,
    Recovering,
    Dropped,
    Ambiguous,
    Degraded,
    Tracked,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LidarManeuverRegime {
    Resolved,
    Unresolved,
}

#[derive(Clone, Debug, Serialize)]
pub struct LidarWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LidarPdOut {
    pub schema_version: String,
    pub calculation_id: String,
    pub scenario_name: String,
    pub status: String,
    pub summary: LidarSummaryOut,
    pub geometry: LidarGeometryOut,
    pub signal: LidarSignalOut,
    pub jammers: Vec<LidarJammerOut>,
    pub chaff: Vec<LidarChaffOut>,
    pub detector: LidarDetectorOut,
    pub fire_control: LidarFireControlOut,
    pub point_defense: LidarPointDefenseOut,
    pub warnings: Vec<LidarWarning>,
    pub assumptions: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LidarSummaryOut {
    pub detector_state: LidarDetectorState,
    pub fire_control_usable: bool,
    pub target_photons: f64,
    pub jammer_to_signal: f64,
    pub snr: f64,
    pub measurement_r95_m: f64,
    pub centroid_bias_m: f64,
    pub future_aim_r95_m: f64,
    pub body_capture_probability: f64,
    pub patch_capture_probability: f64,
    pub structural_kill_time_s: f64,
    pub time_to_standoff_s: f64,
    pub structural_kill_feasible: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct LidarGeometryOut {
    pub target_range_m: f64,
    pub target_line_of_sight: [f64; 3],
    pub receiver_area_m2: f64,
    pub receiver_diffraction_rad: f64,
    pub angular_acceptance_rad: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LidarSignalOut {
    pub lidar_spot_diameter_m: f64,
    pub lidar_spot_area_m2: f64,
    pub target_effective_area_m2: f64,
    pub intercepted_power_w: f64,
    pub target_received_power_clean_w: f64,
    pub target_received_power_actual_w: f64,
    pub photon_energy_j: f64,
    pub target_photons_clean: f64,
    pub target_photons_actual: f64,
    pub target_transmittance: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LidarJammerOut {
    pub id: String,
    pub enabled: bool,
    pub mode: LidarJammerMode,
    pub source_range_m: f64,
    pub angular_separation_rad: f64,
    pub tangent_offset_rad: [f64; 2],
    pub divergence_rad: f64,
    pub footprint_diameter_m: f64,
    pub pointing_weight: f64,
    pub angular_overlap_weight: f64,
    pub spectral_overlap_weight: f64,
    pub polarization_overlap: f64,
    pub temporal_overlap: f64,
    pub code_correlation: f64,
    pub range_response: f64,
    pub pre_processing_photons: f64,
    pub post_processing_photons: f64,
    pub jammer_to_signal: f64,
    pub inside_target_cell: bool,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LidarChaffOut {
    pub id: String,
    pub enabled: bool,
    pub current_dimensions_m: [f64; 3],
    pub current_optical_depth: f64,
    pub cloud_range_m: f64,
    pub angular_offset_rad: f64,
    pub tangent_offset_rad: [f64; 2],
    pub lidar_beam_overlap: f64,
    pub target_transmittance: f64,
    pub accepted_chaff_photons: f64,
    pub centroid_offset_rad: [f64; 2],
    pub lidar_clearance_time_s: Option<f64>,
    pub weapon_clearance_time_s: Option<f64>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LidarDetectorOut {
    pub state: LidarDetectorState,
    pub primary_cause: String,
    pub causes: Vec<String>,
    pub total_pre_processing_counts: f64,
    pub full_well_utilization: f64,
    pub target_photons_clean: f64,
    pub target_photons_actual: f64,
    pub background_photons: f64,
    pub noise_jammer_photons: f64,
    pub structured_photons: f64,
    pub read_noise_variance: f64,
    pub snr: f64,
    pub photon_centroid_sigma_clean_rad: f64,
    pub photon_centroid_sigma_actual_rad: f64,
    pub speckle_angular_scale_rad: f64,
    pub speckle_cell_m: f64,
    pub speckle_spatial_modes: f64,
    pub speckle_total_modes: f64,
    pub speckle_residual_sigma_rad: f64,
    pub measurement_sigma_rad: f64,
    pub centroid_bias_vector_rad: [f64; 2],
    pub centroid_bias_magnitude_rad: f64,
    pub random_r50_m: f64,
    pub random_r90_m: f64,
    pub random_r95_m: f64,
    pub bias_inclusive_r95_m: f64,
    pub projected_recovery_time_s: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LidarFireControlOut {
    pub causal_delay_s: f64,
    pub maneuver_regime: LidarManeuverRegime,
    pub position_contribution_m: f64,
    pub velocity_contribution_m: f64,
    pub maneuver_contribution_m: f64,
    pub measurement_contribution_m: f64,
    pub boresight_contribution_m: f64,
    pub platform_contribution_m: f64,
    pub beam_contribution_m: f64,
    pub random_aim_sigma_m: f64,
    pub systematic_bias_m: f64,
    pub equivalent_aim_sigma_m: f64,
    pub future_r95_m: f64,
    pub body_capture_probability: f64,
    pub patch_capture_probability: f64,
    pub fire_control_usable: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct LidarPointDefenseOut {
    pub weapon_spot_diameter_m: f64,
    pub useful_central_lobe_power_w: f64,
    pub average_flux_w_m2: f64,
    pub clean_soft_kill_time_s: f64,
    pub clean_structural_kill_time_s: f64,
    pub body_capture_factor: f64,
    pub patch_capture_factor: f64,
    pub applied_reacquisition_delay_s: f64,
    pub effective_soft_kill_time_s: f64,
    pub effective_structural_kill_time_s: f64,
    pub time_to_standoff_s: f64,
    pub soft_kill_feasible: bool,
    pub structural_kill_feasible: bool,
    pub model_limitation: String,
}

fn finite(v: f64, path: &str) -> CalcResult<()> {
    if v.is_finite() {
        Ok(())
    } else {
        Err(format!("{path}: value must be finite"))
    }
}

fn positive(v: f64, path: &str) -> CalcResult<()> {
    finite(v, path)?;
    if v > 0.0 {
        Ok(())
    } else {
        Err(format!("{path}: value must be greater than zero"))
    }
}

fn nonnegative(v: f64, path: &str) -> CalcResult<()> {
    finite(v, path)?;
    if v >= 0.0 {
        Ok(())
    } else {
        Err(format!("{path}: value must not be negative"))
    }
}

fn fraction(v: f64, path: &str) -> CalcResult<()> {
    finite(v, path)?;
    if (0.0..=1.0).contains(&v) {
        Ok(())
    } else {
        Err(format!("{path}: value must be between 0 and 1"))
    }
}

fn vec_finite(v: [f64; 3], path: &str) -> CalcResult<()> {
    for (n, x) in v.into_iter().enumerate() {
        finite(x, &format!("{path}[{n}]"))?;
    }
    if norm(v) <= EPS {
        return Err(format!("{path}: position vector must not be zero"));
    }
    Ok(())
}

pub fn validate_lidar_pd(i: &LidarPdIn) -> CalcResult<()> {
    if i.schema_version.split('.').next() != Some("1") {
        return Err(format!(
            "schema_version: unsupported major version {} (expected 1.x)",
            i.schema_version
        ));
    }
    if i.scenario_name.chars().count() > 120 {
        return Err("scenario_name: must be at most 120 characters".into());
    }
    if i.jammers.len() > 32 {
        return Err("jammers: at most 32 entries are supported".into());
    }
    if i.chaff.len() > 16 {
        return Err("chaff: at most 16 entries are supported".into());
    }
    let d = &i.detector;
    for (v, p) in [
        (d.wavelength_m, "detector.wavelength_m"),
        (d.transmitter_power_w, "detector.transmitter_power_w"),
        (d.transmitter_aperture_m, "detector.transmitter_aperture_m"),
        (d.receiver_aperture_m, "detector.receiver_aperture_m"),
        (d.integration_time_s, "detector.integration_time_s"),
        (d.filter_center_m, "detector.filter_center_m"),
        (d.filter_fwhm_m, "detector.filter_fwhm_m"),
        (d.gate_width_s, "detector.gate_width_s"),
        (d.pulse_repetition_hz, "detector.pulse_repetition_hz"),
        (d.full_well_e, "detector.full_well_e"),
        (d.track_snr_min, "detector.track_snr_min"),
        (d.fire_control_snr_min, "detector.fire_control_snr_min"),
        (d.ambiguity_ratio_min, "detector.ambiguity_ratio_min"),
        (d.speckle_temporal_modes, "detector.speckle_temporal_modes"),
        (
            d.speckle_wavelength_modes,
            "detector.speckle_wavelength_modes",
        ),
        (
            d.speckle_polarization_modes,
            "detector.speckle_polarization_modes",
        ),
        (d.speckle_view_modes, "detector.speckle_view_modes"),
    ] {
        positive(v, p)?;
    }
    if !d.transmitter_m2.is_finite() || d.transmitter_m2 < 1.0 {
        return Err("detector.transmitter_m2: value must be at least 1".into());
    }
    for (v, p) in [
        (d.optical_throughput, "detector.optical_throughput"),
        (d.quantum_efficiency, "detector.quantum_efficiency"),
    ] {
        fraction(v, p)?;
    }
    for (v, p) in [
        (d.pixel_scale_rad, "detector.pixel_scale_rad"),
        (
            d.tracking_window_radius_rad,
            "detector.tracking_window_radius_rad",
        ),
        (
            d.detector_floor_sigma_rad,
            "detector.detector_floor_sigma_rad",
        ),
        (d.background_photons, "detector.background_photons"),
        (d.dark_counts, "detector.dark_counts"),
        (d.read_noise_e, "detector.read_noise_e"),
        (d.recovery_time_s, "detector.recovery_time_s"),
        (
            d.speckle_single_sigma_rad,
            "detector.speckle_single_sigma_rad",
        ),
    ] {
        nonnegative(v, p)?;
    }

    let t = &i.target;
    vec_finite(t.position_m, "target.position_m")?;
    for (v, p) in [
        (t.projected_area_m2, "target.projected_area_m2"),
        (
            t.characteristic_diameter_m,
            "target.characteristic_diameter_m",
        ),
        (t.body_radius_m, "target.body_radius_m"),
        (t.closure_velocity_m_s, "target.closure_velocity_m_s"),
        (t.soft_kill_fluence_j_m2, "target.soft_kill_fluence_j_m2"),
        (
            t.structural_kill_fluence_j_m2,
            "target.structural_kill_fluence_j_m2",
        ),
    ] {
        positive(v, p)?;
    }
    positive(
        t.vulnerable_patch_radius_m,
        "target.vulnerable_patch_radius_m",
    )?;
    if t.vulnerable_patch_radius_m > t.body_radius_m {
        return Err("target.vulnerable_patch_radius_m: value must not exceed body radius".into());
    }
    fraction(t.uv_reflectivity, "target.uv_reflectivity")?;
    fraction(t.aspect_factor, "target.aspect_factor")?;
    nonnegative(t.warhead_standoff_m, "target.warhead_standoff_m")?;

    let f = &i.fire_control;
    for (v, p) in [
        (f.processing_latency_s, "fire_control.processing_latency_s"),
        (f.position_sigma_m, "fire_control.position_sigma_m"),
        (f.velocity_sigma_m_s, "fire_control.velocity_sigma_m_s"),
        (
            f.acceleration_sigma_m_s2,
            "fire_control.acceleration_sigma_m_s2",
        ),
        (
            f.maneuver_persistence_s,
            "fire_control.maneuver_persistence_s",
        ),
        (f.boresight_sigma_rad, "fire_control.boresight_sigma_rad"),
        (f.platform_sigma_rad, "fire_control.platform_sigma_rad"),
        (f.beam_sigma_rad, "fire_control.beam_sigma_rad"),
        (f.reacquisition_time_s, "fire_control.reacquisition_time_s"),
    ] {
        nonnegative(v, p)?;
    }
    fraction(
        f.minimum_capture_probability,
        "fire_control.minimum_capture_probability",
    )?;

    let w = &i.weapon;
    for (v, p) in [
        (w.wavelength_m, "weapon.wavelength_m"),
        (w.aperture_m, "weapon.aperture_m"),
        (w.listed_optical_power_w, "weapon.listed_optical_power_w"),
    ] {
        positive(v, p)?;
    }
    if !w.m2.is_finite() || w.m2 < 1.0 {
        return Err("weapon.m2: value must be at least 1".into());
    }
    fraction(w.central_lobe_fraction, "weapon.central_lobe_fraction")?;
    fraction(w.duty_cycle, "weapon.duty_cycle")?;
    nonnegative(w.slew_time_s, "weapon.slew_time_s")?;

    let mut ids = HashSet::new();
    for (n, j) in i.jammers.iter().enumerate() {
        let root = format!("jammers[{n}]");
        if j.id.trim().is_empty() || !ids.insert(format!("j:{}", j.id)) {
            return Err(format!("{root}.id: identifier must be nonempty and unique"));
        }
        vec_finite(j.position_m, &format!("{root}.position_m"))?;
        for (v, p) in [
            (j.optical_power_w, "optical_power_w"),
            (j.aperture_m, "aperture_m"),
            (j.wavelength_m, "wavelength_m"),
            (j.spectral_fwhm_m, "spectral_fwhm_m"),
        ] {
            positive(v, &format!("{root}.{p}"))?;
        }
        if !j.m2.is_finite() || j.m2 < 1.0 {
            return Err(format!("{root}.m2: value must be at least 1"));
        }
        nonnegative(j.pointing_error_rad, &format!("{root}.pointing_error_rad"))?;
        for (v, p) in [
            (j.polarization_overlap, "polarization_overlap"),
            (j.temporal_overlap, "temporal_overlap"),
            (j.code_correlation, "code_correlation"),
            (j.range_response, "range_response"),
            (j.central_lobe_fraction, "central_lobe_fraction"),
        ] {
            fraction(v, &format!("{root}.{p}"))?;
        }
    }
    for (n, c) in i.chaff.iter().enumerate() {
        let root = format!("chaff[{n}]");
        if c.id.trim().is_empty() || !ids.insert(format!("c:{}", c.id)) {
            return Err(format!("{root}.id: identifier must be nonempty and unique"));
        }
        vec_finite(c.position_m, &format!("{root}.position_m"))?;
        for (v, p) in [
            (c.width_m, "width_m"),
            (c.height_m, "height_m"),
            (c.depth_m, "depth_m"),
            (c.clearance_fluence_j_m2, "clearance_fluence_j_m2"),
        ] {
            positive(v, &format!("{root}.{p}"))?;
        }
        for (v, p) in [
            (c.age_s, "age_s"),
            (c.expansion_speed_m_s, "expansion_speed_m_s"),
            (c.optical_depth, "optical_depth"),
        ] {
            nonnegative(v, &format!("{root}.{p}"))?;
        }
        for (v, p) in [
            (c.single_scatter_albedo, "single_scatter_albedo"),
            (c.backscatter_fraction, "backscatter_fraction"),
            (c.range_response, "range_response"),
        ] {
            fraction(v, &format!("{root}.{p}"))?;
        }
    }
    Ok(())
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(v: [f64; 3]) -> f64 {
    dot(v, v).sqrt()
}

fn scale(v: [f64; 3], x: f64) -> [f64; 3] {
    [v[0] * x, v[1] * x, v[2] * x]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn tangent_basis(los: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let reference = if los[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let e1_raw = cross(reference, los);
    let e1 = scale(e1_raw, 1.0 / norm(e1_raw));
    (e1, cross(los, e1))
}

fn direction_geometry(
    v: [f64; 3],
    los: [f64; 3],
    e1: [f64; 3],
    e2: [f64; 3],
) -> (f64, f64, [f64; 2]) {
    let r = norm(v);
    let u = scale(v, 1.0 / r);
    let along = dot(u, los).clamp(-1.0, 1.0);
    let alpha = along.acos();
    let t = [dot(u, e1).atan2(along), dot(u, e2).atan2(along)];
    (r, alpha, t)
}

fn spot_diameter(m2: f64, wavelength_m: f64, range_m: f64, aperture_m: f64) -> f64 {
    1.22 * m2 * wavelength_m * range_m / aperture_m
}

fn circle_rectangle_overlap(radius: f64, width: f64, height: f64, cx: f64, cy: f64) -> f64 {
    // Deterministic 64x64 area sample. Convergence is tested against the
    // centered cloud limits; this avoids geometry branches at grazing overlap.
    const N: usize = 64;
    let mut disk = 0usize;
    let mut both = 0usize;
    for ix in 0..N {
        let x = -radius + 2.0 * radius * (ix as f64 + 0.5) / N as f64;
        for iy in 0..N {
            let y = -radius + 2.0 * radius * (iy as f64 + 0.5) / N as f64;
            if x * x + y * y <= radius * radius {
                disk += 1;
                if (x - cx).abs() <= width / 2.0 && (y - cy).abs() <= height / 2.0 {
                    both += 1;
                }
            }
        }
    }
    if disk == 0 {
        0.0
    } else {
        both as f64 / disk as f64
    }
}

fn spectral_overlap(center_a: f64, fwhm_a: f64, center_b: f64, fwhm_b: f64) -> f64 {
    let k = 2.0 * (2.0_f64.ln()).sqrt();
    let a = fwhm_a / k;
    let b = fwhm_b / k;
    let sum = a * a + b * b;
    ((2.0 * a * b / sum).sqrt() * (-(center_a - center_b).powi(2) / (2.0 * sum)).exp())
        .clamp(0.0, 1.0)
}

fn capture(radius: f64, sigma: f64) -> f64 {
    if sigma <= EPS {
        1.0
    } else {
        (1.0 - (-radius * radius / (2.0 * sigma * sigma)).exp()).clamp(0.0, 1.0)
    }
}

fn warning(code: &str, message: impl Into<String>) -> LidarWarning {
    LidarWarning {
        code: code.into(),
        message: message.into(),
    }
}

pub fn lidar_pd(i: &LidarPdIn) -> CalcResult<LidarPdOut> {
    validate_lidar_pd(i)?;
    let d = &i.detector;
    let t = &i.target;
    let f = &i.fire_control;
    let w = &i.weapon;
    let target_range = norm(t.position_m);
    let los = scale(t.position_m, 1.0 / target_range);
    let (e1, e2) = tangent_basis(los);
    let receiver_area = PI * (d.receiver_aperture_m / 2.0).powi(2);
    let receiver_diffraction = 1.22 * d.wavelength_m / d.receiver_aperture_m;
    let angular_acceptance = ((receiver_diffraction / 2.0).powi(2)
        + d.pixel_scale_rad.powi(2)
        + d.tracking_window_radius_rad.powi(2))
    .sqrt();
    let photon_energy = H * C / d.wavelength_m;

    let lidar_spot_diameter = spot_diameter(
        d.transmitter_m2,
        d.wavelength_m,
        target_range,
        d.transmitter_aperture_m,
    );
    let lidar_spot_area = PI * (lidar_spot_diameter / 2.0).powi(2);
    let target_effective_area = t.projected_area_m2 * t.aspect_factor;
    let intercepted_power =
        d.transmitter_power_w * (target_effective_area / lidar_spot_area).min(1.0);
    let target_received_power_clean = intercepted_power * t.uv_reflectivity * receiver_area
        / (PI * target_range.powi(2))
        * d.optical_throughput;
    let target_photons_clean =
        target_received_power_clean * d.integration_time_s * d.quantum_efficiency / photon_energy;

    let weapon_spot = spot_diameter(w.m2, w.wavelength_m, target_range, w.aperture_m);
    let weapon_lobe_power = w.listed_optical_power_w * w.central_lobe_fraction;
    let weapon_flux = weapon_lobe_power * w.duty_cycle / (PI * (weapon_spot / 2.0).powi(2));

    let mut warnings = Vec::new();
    if target_range <= t.warhead_standoff_m {
        warnings.push(warning(
            "INSIDE_STANDOFF",
            "Target is already at or inside the requested warhead standoff range.",
        ));
    }

    let mut total_transmittance = 1.0;
    let mut chaff_out = Vec::with_capacity(i.chaff.len());
    let mut chaff_structured: Vec<(f64, [f64; 2])> = Vec::new();
    for c in &i.chaff {
        let (range, alpha, tangent) = direction_geometry(c.position_m, los, e1, e2);
        let width = c.width_m + 2.0 * c.expansion_speed_m_s * c.age_s;
        let height = c.height_m + 2.0 * c.expansion_speed_m_s * c.age_s;
        let depth = c.depth_m + 2.0 * c.expansion_speed_m_s * c.age_s;
        let tau = c.optical_depth * c.width_m * c.height_m / (width * height);
        let beam_diameter = spot_diameter(
            d.transmitter_m2,
            d.wavelength_m,
            range,
            d.transmitter_aperture_m,
        );
        let center = [tangent[0].tan() * range, tangent[1].tan() * range];
        let overlap = if c.enabled {
            circle_rectangle_overlap(beam_diameter / 2.0, width, height, center[0], center[1])
        } else {
            0.0
        };
        let trans = if c.enabled {
            (1.0 - overlap) + overlap * (-2.0 * tau).exp()
        } else {
            1.0
        };
        total_transmittance *= trans;
        let p_scattered =
            d.transmitter_power_w * overlap * (1.0 - (-tau).exp()) * c.single_scatter_albedo;
        let p_rx = if c.enabled {
            p_scattered * c.backscatter_fraction * receiver_area / (PI * range.powi(2))
                * d.optical_throughput
                * c.range_response
        } else {
            0.0
        };
        let photons = p_rx * d.integration_time_s * d.quantum_efficiency / photon_energy;
        if photons > 0.0 {
            chaff_structured.push((photons, tangent));
        }
        let lidar_flux = d.transmitter_power_w / (PI * (beam_diameter / 2.0).powi(2));
        let weapon_cloud_spot = spot_diameter(w.m2, w.wavelength_m, range, w.aperture_m);
        let weapon_cloud_flux =
            weapon_lobe_power * w.duty_cycle / (PI * (weapon_cloud_spot / 2.0).powi(2));
        let mut item_warnings = Vec::new();
        if c.enabled && overlap <= 0.0 {
            item_warnings.push(
                "Cloud is outside the transmitted lidar footprint and has no modeled effect."
                    .into(),
            );
            warnings.push(warning(
                "CHAFF_OUTSIDE_FOOTPRINT",
                format!("Chaff {} is outside the lidar footprint.", c.id),
            ));
        }
        let lidar_clear = if c.enabled && overlap > 0.0 {
            Some(c.clearance_fluence_j_m2 / lidar_flux.max(EPS))
        } else {
            None
        };
        let weapon_clear = if c.enabled && overlap > 0.0 {
            Some(c.clearance_fluence_j_m2 / weapon_cloud_flux.max(EPS))
        } else {
            None
        };
        chaff_out.push(LidarChaffOut {
            id: c.id.clone(),
            enabled: c.enabled,
            current_dimensions_m: [width, height, depth],
            current_optical_depth: tau,
            cloud_range_m: range,
            angular_offset_rad: alpha,
            tangent_offset_rad: tangent,
            lidar_beam_overlap: overlap,
            target_transmittance: trans,
            accepted_chaff_photons: photons,
            centroid_offset_rad: tangent,
            lidar_clearance_time_s: lidar_clear,
            weapon_clearance_time_s: weapon_clear,
            warnings: item_warnings,
        });
    }
    let target_received_power_actual = target_received_power_clean * total_transmittance;
    let target_photons = target_photons_clean * total_transmittance;
    if target_photons < 1.0 {
        warnings.push(warning(
            "SUB_PHOTON_RETURN",
            "The expected target return is below one detected photon.",
        ));
    }

    let mut jammer_out = Vec::with_capacity(i.jammers.len());
    let mut jammer_pre = 0.0;
    let mut jammer_noise = 0.0;
    let mut structured: Vec<(f64, [f64; 2])> = chaff_structured.clone();
    let mut strongest_false: f64 = 0.0;
    for j in &i.jammers {
        let (range, alpha, tangent) = direction_geometry(j.position_m, los, e1, e2);
        let footprint = spot_diameter(j.m2, j.wavelength_m, range, j.aperture_m);
        let divergence = footprint / range;
        let ideal = j.optical_power_w
            * j.central_lobe_fraction
            * (receiver_area / (PI * (footprint / 2.0).powi(2))).min(1.0);
        let beam_sigma = (divergence / 2.355).max(EPS);
        let pointing_weight = (-0.5 * (j.pointing_error_rad / beam_sigma).powi(2)).exp();
        let angular_weight = (-0.5 * (alpha / angular_acceptance.max(EPS)).powi(2)).exp();
        let spectral_weight = spectral_overlap(
            j.wavelength_m,
            j.spectral_fwhm_m,
            d.filter_center_m,
            d.filter_fwhm_m,
        );
        let pre_power = if j.enabled {
            ideal
                * pointing_weight
                * angular_weight
                * spectral_weight
                * j.polarization_overlap
                * j.temporal_overlap
        } else {
            0.0
        };
        let post_power = pre_power * j.code_correlation * j.range_response;
        let jammer_photon_energy = H * C / j.wavelength_m;
        let pre = pre_power * d.integration_time_s * d.quantum_efficiency / jammer_photon_energy;
        let post = post_power * d.integration_time_s * d.quantum_efficiency / jammer_photon_energy;
        jammer_pre += pre;
        match j.mode {
            LidarJammerMode::Noise => jammer_noise += post,
            LidarJammerMode::FalseSource => {
                strongest_false = strongest_false.max(post);
                if post > 0.0 {
                    structured.push((post, tangent));
                }
            }
        }
        let mut item_warnings = Vec::new();
        if j.enabled && angular_weight < 0.01 {
            item_warnings.push("Jammer is well outside the target tracking cell.".into());
            warnings.push(warning(
                "JAMMER_OUTSIDE_CELL",
                format!("Jammer {} has negligible angular overlap.", j.id),
            ));
        }
        if j.enabled && spectral_weight < 0.01 {
            item_warnings
                .push("Jammer wavelength is strongly rejected by the receiver filter.".into());
        }
        jammer_out.push(LidarJammerOut {
            id: j.id.clone(),
            enabled: j.enabled,
            mode: j.mode.clone(),
            source_range_m: range,
            angular_separation_rad: alpha,
            tangent_offset_rad: tangent,
            divergence_rad: divergence,
            footprint_diameter_m: footprint,
            pointing_weight,
            angular_overlap_weight: angular_weight,
            spectral_overlap_weight: spectral_weight,
            polarization_overlap: j.polarization_overlap,
            temporal_overlap: j.temporal_overlap,
            code_correlation: j.code_correlation,
            range_response: j.range_response,
            pre_processing_photons: pre,
            post_processing_photons: post,
            jammer_to_signal: post / target_photons.max(EPS),
            inside_target_cell: alpha <= angular_acceptance,
            warnings: item_warnings,
        });
    }

    let chaff_photons: f64 = chaff_structured.iter().map(|x| x.0).sum();
    let false_photons: f64 = structured.iter().map(|x| x.0).sum::<f64>() - chaff_photons;
    let structured_photons = chaff_photons + false_photons;
    let noise_counts = d.background_photons + d.dark_counts + jammer_noise;
    let variance_counts =
        target_photons + structured_photons + noise_counts + d.read_noise_e.powi(2);
    let snr = target_photons / variance_counts.sqrt().max(EPS);
    let jammer_to_signal = jammer_out
        .iter()
        .map(|x| x.post_processing_photons)
        .sum::<f64>()
        / target_photons.max(EPS);
    let target_angular_extent = t.characteristic_diameter_m / target_range;
    let theta_effective = (receiver_diffraction.powi(2) + target_angular_extent.powi(2)).sqrt();
    let photon_clean = theta_effective / target_photons_clean.max(1.0).sqrt();
    let photon_actual = theta_effective * variance_counts.sqrt() / target_photons.max(EPS);
    let theta_speckle = d.wavelength_m / t.characteristic_diameter_m;
    let speckle_cell = theta_speckle * target_range;
    let spatial_modes = (d.receiver_aperture_m / speckle_cell).powi(2).max(1.0);
    let total_modes = spatial_modes
        * d.speckle_temporal_modes
        * d.speckle_wavelength_modes
        * d.speckle_polarization_modes
        * d.speckle_view_modes;
    let speckle_sigma = d.speckle_single_sigma_rad / total_modes.sqrt();
    let measure_sigma =
        (photon_actual.powi(2) + speckle_sigma.powi(2) + d.detector_floor_sigma_rad.powi(2)).sqrt();
    let centroid_den = (target_photons + structured_photons).max(EPS);
    let bias = structured.iter().fold([0.0, 0.0], |mut a, (n, off)| {
        a[0] += n * off[0] / centroid_den;
        a[1] += n * off[1] / centroid_den;
        a
    });
    let bias_mag = bias[0].hypot(bias[1]);
    let radius = |p: f64| target_range * measure_sigma * (-2.0 * (1.0_f64 - p).ln()).sqrt();
    let r50 = radius(0.50);
    let r90 = radius(0.90);
    let r95 = radius(0.95);
    let r95_bias = target_range * bias_mag + r95;

    let pre_counts =
        target_photons + chaff_photons + jammer_pre + d.background_photons + d.dark_counts;
    let saturated = pre_counts > d.full_well_e;
    if saturated && pre_counts > d.full_well_e * 10.0 {
        warnings.push(warning(
            "SEVERE_SATURATION",
            "Detector counts exceed full well by more than an order of magnitude.",
        ));
    }
    let ambiguous = strongest_false / target_photons.max(EPS) >= d.ambiguity_ratio_min;
    let dropped = snr < d.track_snr_min;

    let tau = 2.0 * target_range / C + f.processing_latency_s;
    let (regime, maneuver) = if f.maneuver_persistence_s >= tau {
        (
            LidarManeuverRegime::Resolved,
            0.5 * f.acceleration_sigma_m_s2 * tau.powi(2),
        )
    } else {
        (
            LidarManeuverRegime::Unresolved,
            (f.acceleration_sigma_m_s2.powi(2) * f.maneuver_persistence_s * tau.powi(3) / 3.0)
                .sqrt(),
        )
    };
    let velocity_component = tau * f.velocity_sigma_m_s;
    let measurement_component = target_range * measure_sigma;
    let boresight_component = target_range * f.boresight_sigma_rad;
    let platform_component = target_range * f.platform_sigma_rad;
    let beam_component = target_range * f.beam_sigma_rad;
    let random_aim = (f.position_sigma_m.powi(2)
        + velocity_component.powi(2)
        + maneuver.powi(2)
        + measurement_component.powi(2)
        + boresight_component.powi(2)
        + platform_component.powi(2)
        + beam_component.powi(2))
    .sqrt();
    let systematic_bias = target_range * bias_mag;
    let equivalent_aim = (random_aim.powi(2) + systematic_bias.powi(2) / 2.0).sqrt();
    let future_r95 = equivalent_aim * (-2.0 * 0.05_f64.ln()).sqrt();
    let body_capture = capture(t.body_radius_m + weapon_spot / 2.0, equivalent_aim);
    let patch_absolute = capture(
        t.vulnerable_patch_radius_m + weapon_spot / 2.0,
        equivalent_aim,
    );
    let patch_capture = (patch_absolute / body_capture.max(EPS)).clamp(0.0, 1.0);
    let fire_control_usable = !saturated
        && !dropped
        && !ambiguous
        && snr >= d.fire_control_snr_min
        && body_capture >= f.minimum_capture_probability;
    let state = if saturated {
        LidarDetectorState::Saturated
    } else if dropped {
        LidarDetectorState::Dropped
    } else if ambiguous {
        LidarDetectorState::Ambiguous
    } else if !fire_control_usable {
        LidarDetectorState::Degraded
    } else {
        LidarDetectorState::Tracked
    };
    let mut causes: Vec<String> = Vec::new();
    if saturated {
        causes.push("pre-processing counts exceed detector full well".into());
    }
    if dropped {
        causes.push("target SNR is below the tracking threshold".into());
    }
    if ambiguous {
        causes.push("a structured false source rivals the target return".into());
    }
    if snr < d.fire_control_snr_min {
        causes.push("SNR is below the fire-control threshold".into());
    }
    if body_capture < f.minimum_capture_probability {
        causes.push("body capture probability is below the configured minimum".into());
    }
    if causes.is_empty() {
        causes.push("all configured tracking and fire-control thresholds are met".into());
    }
    let primary_cause = causes[0].clone();

    let clean_soft = t.soft_kill_fluence_j_m2 / weapon_flux.max(EPS);
    let clean_structural = t.structural_kill_fluence_j_m2 / weapon_flux.max(EPS);
    let reacq = match state {
        LidarDetectorState::Saturated
        | LidarDetectorState::Dropped
        | LidarDetectorState::Ambiguous => f.reacquisition_time_s,
        _ => 0.0,
    };
    let efficiency = (body_capture * patch_capture).max(EPS);
    let effective_soft = clean_soft / efficiency + reacq;
    let effective_structural = clean_structural / efficiency + reacq;
    let time_to_standoff =
        ((target_range - t.warhead_standoff_m) / t.closure_velocity_m_s).max(0.0);
    let soft_feasible = effective_soft + w.slew_time_s <= time_to_standoff;
    let structural_feasible = effective_structural + w.slew_time_s <= time_to_standoff;
    warnings.push(warning("SNAPSHOT_PD_MODEL", "Point-defense feasibility uses current-range flux and does not integrate increasing flux during closure."));
    warnings.push(warning(
        "EQUIVALENT_BIAS_MODEL",
        "Structured bias is folded into an equivalent isotropic error for capture probability.",
    ));
    if warnings.len() > 100 {
        warnings.truncate(100);
    }

    let detector = LidarDetectorOut {
        state: state.clone(),
        primary_cause,
        causes,
        total_pre_processing_counts: pre_counts,
        full_well_utilization: pre_counts / d.full_well_e,
        target_photons_clean,
        target_photons_actual: target_photons,
        background_photons: d.background_photons + d.dark_counts,
        noise_jammer_photons: jammer_noise,
        structured_photons,
        read_noise_variance: d.read_noise_e.powi(2),
        snr,
        photon_centroid_sigma_clean_rad: photon_clean,
        photon_centroid_sigma_actual_rad: photon_actual,
        speckle_angular_scale_rad: theta_speckle,
        speckle_cell_m: speckle_cell,
        speckle_spatial_modes: spatial_modes,
        speckle_total_modes: total_modes,
        speckle_residual_sigma_rad: speckle_sigma,
        measurement_sigma_rad: measure_sigma,
        centroid_bias_vector_rad: bias,
        centroid_bias_magnitude_rad: bias_mag,
        random_r50_m: r50,
        random_r90_m: r90,
        random_r95_m: r95,
        bias_inclusive_r95_m: r95_bias,
        projected_recovery_time_s: if saturated { d.recovery_time_s } else { 0.0 },
    };
    let fire_control = LidarFireControlOut {
        causal_delay_s: tau,
        maneuver_regime: regime,
        position_contribution_m: f.position_sigma_m,
        velocity_contribution_m: velocity_component,
        maneuver_contribution_m: maneuver,
        measurement_contribution_m: measurement_component,
        boresight_contribution_m: boresight_component,
        platform_contribution_m: platform_component,
        beam_contribution_m: beam_component,
        random_aim_sigma_m: random_aim,
        systematic_bias_m: systematic_bias,
        equivalent_aim_sigma_m: equivalent_aim,
        future_r95_m: future_r95,
        body_capture_probability: body_capture,
        patch_capture_probability: patch_capture,
        fire_control_usable,
    };
    let point_defense = LidarPointDefenseOut {
        weapon_spot_diameter_m: weapon_spot, useful_central_lobe_power_w: weapon_lobe_power,
        average_flux_w_m2: weapon_flux, clean_soft_kill_time_s: clean_soft,
        clean_structural_kill_time_s: clean_structural, body_capture_factor: body_capture,
        patch_capture_factor: patch_capture, applied_reacquisition_delay_s: reacq,
        effective_soft_kill_time_s: effective_soft, effective_structural_kill_time_s: effective_structural,
        time_to_standoff_s: time_to_standoff, soft_kill_feasible: soft_feasible,
        structural_kill_feasible: structural_feasible,
        model_limitation: "Constant-range snapshot: flux growth during closure, time-varying maneuvers, detector recovery, and target service rate are not integrated.".into(),
    };
    let summary = LidarSummaryOut {
        detector_state: state,
        fire_control_usable,
        target_photons,
        jammer_to_signal,
        snr,
        measurement_r95_m: r95,
        centroid_bias_m: systematic_bias,
        future_aim_r95_m: future_r95,
        body_capture_probability: body_capture,
        patch_capture_probability: patch_capture,
        structural_kill_time_s: effective_structural,
        time_to_standoff_s: time_to_standoff,
        structural_kill_feasible: structural_feasible,
    };
    Ok(LidarPdOut {
        schema_version: "1.0".into(),
        calculation_id: String::new(),
        scenario_name: i.scenario_name.clone(),
        status: "ok".into(),
        summary,
        geometry: LidarGeometryOut {
            target_range_m: target_range,
            target_line_of_sight: los,
            receiver_area_m2: receiver_area,
            receiver_diffraction_rad: receiver_diffraction,
            angular_acceptance_rad: angular_acceptance,
        },
        signal: LidarSignalOut {
            lidar_spot_diameter_m: lidar_spot_diameter,
            lidar_spot_area_m2: lidar_spot_area,
            target_effective_area_m2: target_effective_area,
            intercepted_power_w: intercepted_power,
            target_received_power_clean_w: target_received_power_clean,
            target_received_power_actual_w: target_received_power_actual,
            photon_energy_j: photon_energy,
            target_photons_clean,
            target_photons_actual: target_photons,
            target_transmittance: total_transmittance,
        },
        jammers: jammer_out,
        chaff: chaff_out,
        detector,
        fire_control,
        point_defense,
        warnings,
        assumptions: vec![
            "Diffuse Lambertian target return is an engineering approximation.".into(),
            "Chaff uses a rectangular cloud and deterministic sampled beam overlap.".into(),
            "Speckle severity and diversity counts are user assumptions.".into(),
            "Capture uses an equivalent isotropic Gaussian approximation.".into(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> LidarPdIn {
        serde_json::from_str(include_str!("../../testdata/lidar_pd_baseline.json")).unwrap()
    }

    fn close(a: f64, b: f64, rel: f64) -> bool {
        (a - b).abs() <= rel * a.abs().max(b.abs()).max(1.0)
    }

    #[test]
    fn golden_clean_lidar_and_causal_delay() {
        let out = lidar_pd(&baseline()).unwrap();
        assert!(close(out.signal.lidar_spot_diameter_m, 3.2452, 2e-3));
        assert!(out.signal.target_photons_clean > 1.0e4);
        assert!(close(out.fire_control.causal_delay_s, 0.0677, 2e-2));
    }

    #[test]
    fn remaining_golden_optical_cases() {
        let base = baseline();
        let ten = lidar_pd(&base).unwrap();
        assert!(close(ten.signal.target_photons_clean, 2.02e4, 0.02));
        let mut five_input = base.clone();
        five_input.target.position_m[0] = 5.0e6;
        let five = lidar_pd(&five_input).unwrap();
        assert!(close(five.signal.lidar_spot_diameter_m, 1.6226, 2e-3));
        assert!(close(five.signal.target_photons_clean, 3.24e5, 0.03));
        assert!(close(
            five.detector.photon_centroid_sigma_clean_rad,
            0.57e-9,
            0.08
        ));
        assert!(close(
            five.point_defense.weapon_spot_diameter_m,
            0.108_173,
            2e-3
        ));
    }

    #[test]
    fn angular_weight_reference_points() {
        let theta = 1e-6;
        let weight = |alpha: f64| (-0.5 * (alpha / theta).powi(2)).exp();
        assert!(close(weight(0.0), 1.0, 1e-12));
        assert!(close(weight(theta), 0.606_530_66, 1e-8));
        assert!(close(weight(3.0 * theta), 0.011_108_997, 1e-8));
    }

    #[test]
    fn receiver_area_scales_signal_and_disabled_entries_are_neutral() {
        let base = baseline();
        let a = lidar_pd(&base).unwrap();
        let mut big = base.clone();
        big.detector.receiver_aperture_m = 30.0;
        let b = lidar_pd(&big).unwrap();
        assert!(close(
            b.signal.target_photons_clean / a.signal.target_photons_clean,
            900.0,
            1e-10
        ));
        let mut disabled = base;
        disabled.jammers.push(LidarJammerIn {
            id: "off".into(),
            enabled: false,
            mode: LidarJammerMode::Noise,
            position_m: [1e7, 0.0, 0.0],
            optical_power_w: 1e9,
            aperture_m: 1.0,
            m2: 1.0,
            wavelength_m: 266e-9,
            spectral_fwhm_m: 1e-11,
            pointing_error_rad: 0.0,
            polarization_overlap: 1.0,
            temporal_overlap: 1.0,
            code_correlation: 1.0,
            range_response: 1.0,
            central_lobe_fraction: 1.0,
        });
        let c = lidar_pd(&disabled).unwrap();
        assert!(close(a.summary.snr, c.summary.snr, 1e-12));
    }

    #[test]
    fn countermeasures_and_error_growth_are_monotonic() {
        let mut weak = baseline();
        weak.jammers[0].optical_power_w = 1.0;
        let weak_out = lidar_pd(&weak).unwrap();
        let mut strong = weak.clone();
        strong.jammers[0].optical_power_w = 100.0;
        let strong_out = lidar_pd(&strong).unwrap();
        assert!(strong_out.detector.snr <= weak_out.detector.snr);

        let mut poor_aim = weak.clone();
        poor_aim.fire_control.beam_sigma_rad *= 1000.0;
        let poor_out = lidar_pd(&poor_aim).unwrap();
        assert!(
            poor_out.fire_control.body_capture_probability
                <= weak_out.fire_control.body_capture_probability
        );

        let mut hard = weak;
        hard.target.structural_kill_fluence_j_m2 *= 2.0;
        let hard_out = lidar_pd(&hard).unwrap();
        assert!(
            hard_out.point_defense.clean_structural_kill_time_s
                >= weak_out.point_defense.clean_structural_kill_time_s
        );
    }

    #[test]
    fn pre_processing_rejection_does_not_prevent_saturation() {
        let mut i = baseline();
        i.detector.full_well_e = 100.0;
        i.jammers[0].code_correlation = 0.0;
        let out = lidar_pd(&i).unwrap();
        assert!(matches!(out.detector.state, LidarDetectorState::Saturated));
        assert_eq!(out.jammers[0].post_processing_photons, 0.0);
    }

    #[test]
    fn chaff_outside_footprint_is_neutral_and_warns() {
        let mut i = baseline();
        let clean = lidar_pd(&i).unwrap();
        i.chaff.push(LidarChaffIn {
            id: "far".into(),
            enabled: true,
            position_m: [1e7, 1e6, 0.0],
            width_m: 10.0,
            height_m: 10.0,
            depth_m: 10.0,
            age_s: 0.0,
            expansion_speed_m_s: 0.0,
            optical_depth: 10.0,
            single_scatter_albedo: 1.0,
            backscatter_fraction: 1.0,
            range_response: 1.0,
            clearance_fluence_j_m2: 1e6,
        });
        let out = lidar_pd(&i).unwrap();
        assert!(close(
            clean.signal.target_photons_actual,
            out.signal.target_photons_actual,
            1e-12
        ));
        assert!(out
            .warnings
            .iter()
            .any(|x| x.code == "CHAFF_OUTSIDE_FOOTPRINT"));
    }

    #[test]
    fn centered_chaff_attenuates_and_false_source_biases_centroid() {
        let mut with_chaff = baseline();
        with_chaff.jammers.clear();
        with_chaff.chaff.push(LidarChaffIn {
            id: "center".into(),
            enabled: true,
            position_m: [9.9e6, 0.0, 0.0],
            width_m: 10.0,
            height_m: 10.0,
            depth_m: 10.0,
            age_s: 0.0,
            expansion_speed_m_s: 0.0,
            optical_depth: 1.0,
            single_scatter_albedo: 0.8,
            backscatter_fraction: 0.1,
            range_response: 1.0,
            clearance_fluence_j_m2: 1e7,
        });
        let chaff = lidar_pd(&with_chaff).unwrap();
        assert!(chaff.signal.target_transmittance < 0.2);
        assert!(chaff.chaff[0].accepted_chaff_photons > 0.0);
        assert!(chaff.chaff[0].lidar_clearance_time_s.is_some());

        let mut deceptive = baseline();
        deceptive.jammers[0].mode = LidarJammerMode::FalseSource;
        deceptive.jammers[0].position_m = [1.2e7, 1.0, 0.0];
        deceptive.jammers[0].optical_power_w = 1e-4;
        deceptive.jammers[0].code_correlation = 1.0;
        let out = lidar_pd(&deceptive).unwrap();
        assert!(out.detector.centroid_bias_magnitude_rad > 0.0);
        assert!(out.fire_control.systematic_bias_m > 0.0);
    }

    #[test]
    fn validation_reports_field_paths() {
        let mut i = baseline();
        i.target.closure_velocity_m_s = 0.0;
        assert!(lidar_pd(&i)
            .unwrap_err()
            .starts_with("target.closure_velocity_m_s:"));
        i.target.closure_velocity_m_s = 1.0;
        i.schema_version = "2.0".into();
        assert!(lidar_pd(&i).unwrap_err().starts_with("schema_version:"));
    }

    #[test]
    fn overlap_sampler_has_expected_centered_limits() {
        assert!(circle_rectangle_overlap(1.0, 4.0, 4.0, 0.0, 0.0) > 0.999);
        assert!(circle_rectangle_overlap(1.0, 0.1, 0.1, 10.0, 0.0) == 0.0);
    }
}
