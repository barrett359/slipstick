//! All physics for Slipstick. The frontend never computes physics; every
//! number a user sees that isn't a straight sum of masses comes from here.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::model::{Component, Design, Missile, Settings};

mod lidar_pd;
pub use lidar_pd::*;
mod missile_engagement;
pub use missile_engagement::*;

pub type CalcResult<T> = Result<T, String>;

fn require_pos(v: f64, name: &str) -> CalcResult<f64> {
    if !v.is_finite() || v <= 0.0 {
        Err(format!("{} must be positive (got {})", name, v))
    } else {
        Ok(v)
    }
}

// ---------------------------------------------------------------------------
// Drive gearing — fixed jet power model.
//
// P_jet = P_fusion * f_exh * eta_noz
// At gear setting Ve: F = 2 P_jet / Ve, mdot = F / Ve.
// Reactor fuel throughput: mdot_fuel = 2 P_jet_used / Ve_max^2 (pure-plasma
// flow; the reactor throttles when the nozzle thrust cap binds).
// Afterburner propellant = mdot - mdot_fuel.
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct GearIn {
    pub p_fusion: f64,
    pub f_exh: f64,
    pub eta_noz: f64,
    /// Afterburner propellant energy, J/kg. 0 for inert propellant.
    #[serde(default)]
    pub e_afterburner: f64,
    pub ve: f64,
    pub ve_max: f64,
    #[serde(default)]
    pub f_cap: Option<f64>,
    #[serde(default)]
    pub mass_kg: Option<f64>,
    #[serde(default)]
    pub duration_s: Option<f64>,
}

#[derive(Serialize, JsonSchema)]
pub struct GearOut {
    pub p_jet: f64,
    pub thrust: f64,
    pub capped: bool,
    pub ve_cap: Option<f64>,
    /// Ve below which afterburner propellant energy alone exceeds exhaust KE.
    pub ve_floor: Option<f64>,
    pub mdot: f64,
    pub mdot_fuel: f64,
    pub mdot_afterburner: f64,
    pub accel: Option<f64>,
    pub prop_used_kg: Option<f64>,
    pub fuel_used_kg: Option<f64>,
    pub afterburner_used_kg: Option<f64>,
}

pub fn gear(i: &GearIn) -> CalcResult<GearOut> {
    require_pos(i.p_fusion, "p_fusion")?;
    require_pos(i.ve, "ve")?;
    require_pos(i.ve_max, "ve_max")?;

    let p_jet = i.p_fusion * i.f_exh * i.eta_noz;

    let ve_sq = i.ve * i.ve;
    let ve_max_sq = i.ve_max * i.ve_max;

    // Afterburner propellant recombination energy → velocity floor
    // e_afterburner: J/kg (0 for inert propellant, ~216e6 for MH)
    let ve_mh_sq = 2.0 * i.e_afterburner;

    if ve_sq <= ve_mh_sq {
        return Err(format!(
            "ve ({:.1} km/s) is at or below afterburner floor ({:.1} km/s)",
            i.ve * 1e-3,
            ve_mh_sq.sqrt() * 1e-3
        ));
    }

    // Correction factor: ≈1 when ve_max >> ve_mh (always true for MH)
    let corr = 1.0 - ve_mh_sq / ve_max_sq;

    // Thrust including afterburner energy contribution.
    // When e_afterburner = 0: reduces to 2·p_jet/ve.
    // Diverges as ve → ve_mh (propellant energy dominates).
    let f_raw = 2.0 * p_jet * i.ve * corr / (ve_sq - ve_mh_sq);

    let (thrust, capped) = match i.f_cap {
        Some(cap) if cap > 0.0 && f_raw > cap => (cap, true),
        _ => (f_raw, false),
    };

    // Ve where nozzle thrust cap binds.
    // Solving cap = 2·p_jet·ve·corr/(ve²−ve_mh²) gives a quadratic:
    //   cap·ve² − 2·p_jet·corr·ve − cap·ve_mh² = 0
    let ve_cap = i.f_cap.filter(|c| *c > 0.0).map(|cap| {
        let pc = p_jet * corr;
        (pc + (pc * pc + cap * cap * ve_mh_sq).sqrt()) / cap
    });

    let ve_floor = if ve_mh_sq > 0.0 {
        Some(ve_mh_sq.sqrt())
    } else {
        None
    };

    let mdot = thrust / i.ve;

    // Fuel/afterburner split accounting for recombination energy.
    // Same formula whether capped or not — the split depends on
    // the energy ratio, not on how thrust was determined.
    let denom = ve_max_sq - ve_mh_sq;
    let mdot_fuel = mdot * (ve_sq - ve_mh_sq) / denom;
    let mdot_afterburner = (mdot * (ve_max_sq - ve_sq) / denom).max(0.0);

    Ok(GearOut {
        p_jet,
        thrust,
        capped,
        ve_cap,
        ve_floor,
        mdot,
        mdot_fuel,
        mdot_afterburner,
        accel: i.mass_kg.map(|m| thrust / m),
        prop_used_kg: i.duration_s.map(|t| mdot * t),
        fuel_used_kg: i.duration_s.map(|t| mdot_fuel * t),
        afterburner_used_kg: i.duration_s.map(|t| mdot_afterburner * t),
    })
}

// ---------------------------------------------------------------------------
// Delta-v.  dv = Ve ln(m_wet / m_floor), m_floor = m_dry exp(dv_reserve / Ve).
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct DeltavIn {
    pub ve: f64,
    pub m_wet: f64,
    pub m_dry: f64,
    #[serde(default)]
    pub dv_reserve: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct DeltavOut {
    pub m_floor: f64,
    pub dv: f64,
}

pub fn deltav(i: &DeltavIn) -> CalcResult<DeltavOut> {
    require_pos(i.ve, "ve")?;
    require_pos(i.m_wet, "m_wet")?;
    require_pos(i.m_dry, "m_dry")?;
    let m_floor = i.m_dry * (i.dv_reserve / i.ve).exp();
    let dv = i.ve * (i.m_wet / m_floor).ln();
    Ok(DeltavOut { m_floor, dv })
}

// ---------------------------------------------------------------------------
// Drive curve: thrust and delta-v sampled across the gear range, for plotting.
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct DriveCurveIn {
    pub p_fusion: f64,
    pub f_exh: f64,
    pub eta_noz: f64,
    #[serde(default)]
    pub e_afterburner: f64,
    pub ve_min: f64,
    pub ve_max: f64,
    #[serde(default)]
    pub f_cap: Option<f64>,
    #[serde(default)]
    pub m_wet: Option<f64>,
    #[serde(default)]
    pub m_dry: Option<f64>,
    #[serde(default)]
    pub dv_reserve: f64,
    #[serde(default)]
    pub n: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
pub struct DriveCurveOut {
    pub ve: Vec<f64>,
    pub thrust: Vec<f64>,
    pub dv: Option<Vec<f64>>,
    pub ve_cap: Option<f64>,
}

pub fn drive_curve(i: &DriveCurveIn) -> CalcResult<DriveCurveOut> {
    require_pos(i.ve_min, "ve_min")?;
    require_pos(i.ve_max, "ve_max")?;
    if i.ve_min >= i.ve_max {
        return Err("ve_min must be < ve_max".into());
    }
    let n = i.n.unwrap_or(160).clamp(8, 2000);
    let ratio = i.ve_max / i.ve_min;
    let mut ve = Vec::with_capacity(n);
    let mut thrust = Vec::with_capacity(n);
    let want_dv = i.m_wet.is_some() && i.m_dry.is_some();
    let mut dv = if want_dv {
        Some(Vec::with_capacity(n))
    } else {
        None
    };
    let mut ve_cap = None;
    for k in 0..n {
        let v = i.ve_min * ratio.powf(k as f64 / (n - 1) as f64);
        let g = gear(&GearIn {
            p_fusion: i.p_fusion,
            f_exh: i.f_exh,
            eta_noz: i.eta_noz,
            e_afterburner: i.e_afterburner,
            ve: v,
            ve_max: i.ve_max,
            f_cap: i.f_cap,
            mass_kg: None,
            duration_s: None,
        })?;
        ve_cap = g.ve_cap;
        ve.push(v);
        thrust.push(g.thrust);
        if let Some(dvv) = dv.as_mut() {
            let d = deltav(&DeltavIn {
                ve: v,
                m_wet: i.m_wet.unwrap(),
                m_dry: i.m_dry.unwrap(),
                dv_reserve: i.dv_reserve,
            })?;
            dvv.push(d.dv.max(0.0));
        }
    }
    Ok(DriveCurveOut {
        ve,
        thrust,
        dv,
        ve_cap,
    })
}

// ---------------------------------------------------------------------------
// Travel — analytic flip-and-burn, no timestep integration.
//
// Constant-thrust burn with tau = m0/mdot:
//   v(t) = v0 + Ve ln(m0/m(t))
//   x(t) = v0 t + Ve [t + (tau - t) ln(1 - t/tau)]
// Bisect on burn-1 duration for total distance = target, under the hard
// constraint m_flip >= sqrt(m0 * m_floor).
// ---------------------------------------------------------------------------

/// Distance covered under an accelerating burn from velocity v0.
fn burn_x(ve: f64, v0: f64, tau: f64, t: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    let frac = 1.0 - t / tau;
    let log_term = if frac <= 1e-300 {
        0.0
    } else {
        (tau - t) * frac.ln()
    };
    v0 * t + ve * (t + log_term)
}

/// Velocity gained by burn at time t (added for accel, subtracted for decel).
fn burn_dv(ve: f64, tau: f64, t: f64) -> f64 {
    let frac = 1.0 - t / tau;
    if frac <= 1e-300 {
        f64::INFINITY
    } else {
        -ve * frac.ln()
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct TravelIn {
    pub distance: f64,
    pub ve: f64,
    pub thrust: f64,
    pub mdot: f64,
    pub m0: f64,
    pub m_dry: f64,
    #[serde(default)]
    pub dv_reserve: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct TravelOut {
    pub feasible: bool,
    /// Farthest full flip-and-burn distance at this gear and mass.
    pub max_distance: f64,
    pub t_flip: f64,
    pub t_total: f64,
    pub peak_v: f64,
    pub prop_used_kg: f64,
    pub m_arrival: f64,
    pub m_floor: f64,
    pub dv_spent: f64,
    /// Sampled [t, x, v, m] rows across both burns.
    pub profile: Vec<[f64; 4]>,
}

struct FlipSolution {
    t1: f64,
    t2: f64,
    x_total: f64,
    v_peak: f64,
    m_end: f64,
}

fn flip_solution(ve: f64, mdot: f64, m0: f64, t1: f64) -> FlipSolution {
    let tau1 = m0 / mdot;
    let m1 = m0 - mdot * t1;
    let v1 = burn_dv(ve, tau1, t1);
    let x1 = burn_x(ve, 0.0, tau1, t1);
    // Decel burn kills v1: ends at m_end = m1 * exp(-v1/ve) = m1^2/m0.
    let m_end = m1 * m1 / m0;
    let t2 = (m1 - m_end) / mdot;
    let tau2 = m1 / mdot;
    // x2 = v1 t - ve[t + (tau-t)ln(1-t/tau)] evaluated at t2
    let x2 = v1 * t2 - (burn_x(ve, 0.0, tau2, t2));
    FlipSolution {
        t1,
        t2,
        x_total: x1 + x2,
        v_peak: v1,
        m_end,
    }
}

pub fn travel(i: &TravelIn) -> CalcResult<TravelOut> {
    require_pos(i.distance, "distance")?;
    require_pos(i.thrust, "thrust")?;
    require_pos(i.mdot, "mdot")?;
    require_pos(i.m0, "m0")?;
    require_pos(i.m_dry, "m_dry")?;
    require_pos(i.ve, "ve")?;
    let ve_eff = i.thrust / i.mdot; // equals gear Ve by construction
    let m_floor = i.m_dry * (i.dv_reserve / ve_eff).exp();
    if m_floor >= i.m0 {
        return Err(format!(
            "no burnable propellant: mass floor {:.1} t (dry + reserve) >= current mass {:.1} t",
            m_floor / 1000.0,
            i.m0 / 1000.0
        ));
    }
    // Hard constraint: m_flip >= sqrt(m0 * m_floor).
    let m_flip_min = (i.m0 * m_floor).sqrt();
    let t1_max = (i.m0 - m_flip_min) / i.mdot;
    let max_sol = flip_solution(ve_eff, i.mdot, i.m0, t1_max);
    let max_distance = max_sol.x_total;

    let (feasible, sol) = if max_distance < i.distance {
        (false, max_sol)
    } else {
        // Bisect t1 in (0, t1_max]; distance is monotonic in t1.
        let mut lo = 0.0f64;
        let mut hi = t1_max;
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if flip_solution(ve_eff, i.mdot, i.m0, mid).x_total < i.distance {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (true, flip_solution(ve_eff, i.mdot, i.m0, 0.5 * (lo + hi)))
    };

    // Sample the profile: burn 1 then burn 2, ~160 points total.
    let t_total = sol.t1 + sol.t2;
    let n = 160usize;
    let mut profile = Vec::with_capacity(n + 2);
    let tau1 = i.m0 / i.mdot;
    let m1 = i.m0 - i.mdot * sol.t1;
    let v1 = burn_dv(ve_eff, tau1, sol.t1);
    let x1 = burn_x(ve_eff, 0.0, tau1, sol.t1);
    let tau2 = m1 / i.mdot;
    for k in 0..=n {
        let t = t_total * k as f64 / n as f64;
        let (x, v, m) = if t <= sol.t1 {
            (
                burn_x(ve_eff, 0.0, tau1, t),
                burn_dv(ve_eff, tau1, t),
                i.m0 - i.mdot * t,
            )
        } else {
            let td = t - sol.t1;
            (
                x1 + v1 * td - burn_x(ve_eff, 0.0, tau2, td),
                v1 - burn_dv(ve_eff, tau2, td),
                m1 - i.mdot * td,
            )
        };
        profile.push([t, x, v, m]);
    }

    Ok(TravelOut {
        feasible,
        max_distance,
        t_flip: sol.t1,
        t_total,
        peak_v: sol.v_peak,
        prop_used_kg: i.m0 - sol.m_end,
        m_arrival: sol.m_end,
        m_floor,
        dv_spent: ve_eff * (i.m0 / sol.m_end).ln(),
        profile,
    })
}

// ---------------------------------------------------------------------------
// Timed burn: gear + duration → Δv, end velocity, distance covered.
// direction +1 = prograde (accelerate), −1 = retrograde (decelerate).
// ---------------------------------------------------------------------------

fn one() -> f64 {
    1.0
}

#[derive(Deserialize, JsonSchema)]
pub struct BurnIn {
    pub v0: f64,
    pub duration_s: f64,
    pub thrust: f64,
    pub mdot: f64,
    pub m0: f64,
    pub m_floor: f64,
    #[serde(default = "one")]
    pub direction: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct BurnOut {
    /// Actual burn time (clamped at the propellant floor).
    pub t: f64,
    pub clamped: bool,
    pub dv: f64,
    pub v_end: f64,
    pub distance: f64,
    pub prop_used_kg: f64,
    pub m_end: f64,
    /// [t, x, v, m] samples.
    pub profile: Vec<[f64; 4]>,
}

pub fn timed_burn(i: &BurnIn) -> CalcResult<BurnOut> {
    require_pos(i.duration_s, "duration_s")?;
    require_pos(i.thrust, "thrust")?;
    require_pos(i.mdot, "mdot")?;
    require_pos(i.m0, "m0")?;
    if i.m_floor >= i.m0 {
        return Err(format!(
            "no burnable propellant: mass floor {:.1} t >= current mass {:.1} t",
            i.m_floor / 1000.0,
            i.m0 / 1000.0
        ));
    }
    let t_max = (i.m0 - i.m_floor) / i.mdot;
    let t = i.duration_s.min(t_max);
    let clamped = t < i.duration_s * (1.0 - 1e-12);
    let ve_eff = i.thrust / i.mdot;
    let tau = i.m0 / i.mdot;
    let dir = if i.direction < 0.0 { -1.0 } else { 1.0 };
    let dv = burn_dv(ve_eff, tau, t);
    let n = 60usize;
    let profile = (0..=n)
        .map(|k| {
            let s = t * k as f64 / n as f64;
            [
                s,
                i.v0 * s + dir * burn_x(ve_eff, 0.0, tau, s),
                i.v0 + dir * burn_dv(ve_eff, tau, s),
                i.m0 - i.mdot * s,
            ]
        })
        .collect();
    Ok(BurnOut {
        t,
        clamped,
        dv,
        v_end: i.v0 + dir * dv,
        distance: i.v0 * t + dir * burn_x(ve_eff, 0.0, tau, t),
        prop_used_kg: i.mdot * t,
        m_end: i.m0 - i.mdot * t,
        profile,
    })
}

// ---------------------------------------------------------------------------
// Sprint intercept: burn everything down to the floor accelerating, then
// coast to the target. No deceleration — a flyby/intercept profile.
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct SprintIn {
    pub distance: f64,
    #[serde(default)]
    pub v0: f64,
    pub thrust: f64,
    pub mdot: f64,
    pub m0: f64,
    pub m_floor: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct SprintOut {
    pub hit: bool,
    pub t_total: Option<f64>,
    pub t_burn: f64,
    pub v_arrival: Option<f64>,
    pub dv_spent: f64,
    pub prop_used_kg: f64,
    pub m_end: f64,
    pub miss_reason: Option<String>,
    /// [t, x, v, m] samples.
    pub profile: Vec<[f64; 4]>,
}

pub fn sprint(i: &SprintIn) -> CalcResult<SprintOut> {
    require_pos(i.distance, "distance")?;
    require_pos(i.thrust, "thrust")?;
    require_pos(i.mdot, "mdot")?;
    require_pos(i.m0, "m0")?;
    if i.m_floor >= i.m0 {
        return Err("no burnable propellant above the reserve floor".into());
    }
    let ve_eff = i.thrust / i.mdot;
    let tau = i.m0 / i.mdot;
    let t_b = (i.m0 - i.m_floor) / i.mdot;
    let x_at = |t: f64| burn_x(ve_eff, i.v0, tau, t);
    let v_at = |t: f64| i.v0 + burn_dv(ve_eff, tau, t);

    // Crossing inside the burn?
    let n = 400;
    let mut cross: Option<f64> = None;
    let (mut prev_s, mut prev_x) = (0.0f64, 0.0f64);
    for k in 1..=n {
        let s = t_b * k as f64 / n as f64;
        let xr = x_at(s);
        if xr >= i.distance && prev_x < i.distance {
            let (mut lo, mut hi) = (prev_s, s);
            for _ in 0..60 {
                let mid = 0.5 * (lo + hi);
                if x_at(mid) < i.distance {
                    lo = mid
                } else {
                    hi = mid
                }
            }
            cross = Some(0.5 * (lo + hi));
            break;
        }
        prev_s = s;
        prev_x = xr;
    }

    let x_b = x_at(t_b);
    let v_b = v_at(t_b);
    let (hit, t_total, v_arrival, t_end, miss_reason) = match cross {
        Some(s) => (true, Some(s), Some(v_at(s)), s, None),
        None if v_b > 0.0 => {
            let t = t_b + (i.distance - x_b) / v_b;
            (true, Some(t), Some(v_b), t, None)
        }
        None => (
            false,
            None,
            None,
            t_b * 1.5,
            Some("velocity never goes positive — distance keeps opening".into()),
        ),
    };

    let burn_end = t_end.min(t_b);
    let mut profile: Vec<[f64; 4]> = Vec::with_capacity(120);
    let np = 80;
    for k in 0..=np {
        let s = burn_end * k as f64 / np as f64;
        profile.push([s, x_at(s), v_at(s), i.m0 - i.mdot * s]);
    }
    if t_end > t_b {
        for k in 1..=30 {
            let s = t_b + (t_end - t_b) * k as f64 / 30.0;
            profile.push([s, x_b + v_b * (s - t_b), v_b, i.m_floor]);
        }
    }

    let prop_used = i.mdot * burn_end;
    Ok(SprintOut {
        hit,
        t_total,
        t_burn: t_b,
        v_arrival,
        dv_spent: burn_dv(ve_eff, tau, burn_end),
        prop_used_kg: prop_used,
        m_end: i.m0 - prop_used,
        miss_reason,
        profile,
    })
}

// ---------------------------------------------------------------------------
// Auto-size: given a target wet acceleration at Ve_max and per-thrust scaling
// parameters, solve the linear fixed point for dry mass. Drive, radiator, and
// tank masses all scale with the thrust they must support, so
//   dry = payload + k·dry  with  k = a·MR·c_F + tank + structure terms;
// feasible iff k < 1, and a_max falls out in closed form.
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct AutoLaser {
    pub p_beam: f64,
    pub eta_wall: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct AutosizeIn {
    /// Target wet acceleration at Ve_max, m/s².
    pub a_target: f64,
    pub mr: f64,
    /// Pinned components + ordnance (+ structure if pinned), tonnes.
    pub payload_t: f64,
    /// Set when the reactor is pinned: size everything else around it.
    #[serde(default)]
    pub p_fusion_pinned: Option<f64>,
    pub auto_nozzle: bool,
    pub auto_rad_hot: bool,
    pub auto_rad_low: bool,
    pub auto_tank: bool,
    pub auto_sink: bool,
    pub auto_flywheel: bool,
    pub auto_structure: bool,
    pub ve_max: f64,
    pub f_exh: f64,
    pub eta_noz: f64,
    pub sigma: f64,
    pub reactor_t_per_tw: f64,
    pub nozzle_cap_factor: f64,
    pub nozzle_t_per_mn: f64,
    pub rad_load_frac: f64,
    pub hot_t_k: f64,
    pub hot_eps: f64,
    /// Hot-loop specific power, MW rejected per kg of radiator.
    pub hot_mw_per_kg: f64,
    pub low_area_frac: f64,
    pub low_t_k: f64,
    pub low_eps: f64,
    /// Low-loop specific power, MW rejected per kg of radiator.
    pub low_mw_per_kg: f64,
    /// Tank tonnes per tonne of propellant.
    pub tank_mass_per_prop: f64,
    /// Structure as a fraction of dry mass.
    pub structure_frac: f64,
    pub sink_endurance_s: f64,
    pub sink_extra_mass_factor: f64,
    pub li_sink_mj_per_kg: f64,
    pub flywheel_fire_s: f64,
    pub flywheel_mj_per_t: f64,
    #[serde(default)]
    pub lasers: Vec<AutoLaser>,
}

#[derive(Serialize, JsonSchema)]
pub struct AutosizeOut {
    pub feasible: bool,
    /// Max wet acceleration at this MR with these scaling parameters, m/s².
    pub a_max: f64,
    pub a_achieved: f64,
    pub dry_t: f64,
    pub wet_t: f64,
    pub thrust_n: f64,
    pub p_fusion_w: f64,
    pub reactor_t: Option<f64>,
    pub nozzle_cap_n: Option<f64>,
    pub nozzle_t: Option<f64>,
    pub hot_area_m2: Option<f64>,
    pub hot_t: Option<f64>,
    pub low_area_m2: Option<f64>,
    pub low_t: Option<f64>,
    pub tank_t: Option<f64>,
    pub tank_cap_t: Option<f64>,
    pub sink_li_t: Option<f64>,
    pub sink_t: Option<f64>,
    pub flywheel_t: Option<f64>,
    pub structure_t: Option<f64>,
    pub laser_waste_w: f64,
    pub laser_elec_w: f64,
}

pub fn autosize(i: &AutosizeIn) -> CalcResult<AutosizeOut> {
    require_pos(i.mr, "mr")?;
    require_pos(i.payload_t, "payload_t")?;
    require_pos(i.ve_max, "ve_max")?;
    if i.p_fusion_pinned.is_none() {
        require_pos(i.a_target, "a_target")?;
    }
    // Fusion watts needed per newton of thrust at Ve_max.
    let w_per_n = i.ve_max / (2.0 * i.f_exh * i.eta_noz);

    let laser_waste_w: f64 = i
        .lasers
        .iter()
        .filter(|l| l.eta_wall > 0.0)
        .map(|l| l.p_beam * (1.0 / l.eta_wall - 1.0))
        .sum();
    let laser_elec_w: f64 = i
        .lasers
        .iter()
        .filter(|l| l.eta_wall > 0.0)
        .map(|l| l.p_beam / l.eta_wall)
        .sum();

    // Sink and flywheels size against the laser suite, not thrust: constants.
    let sink_li_kg = if i.auto_sink && i.li_sink_mj_per_kg > 0.0 {
        laser_waste_w * i.sink_endurance_s / (i.li_sink_mj_per_kg * 1e6)
    } else {
        0.0
    };
    let sink_kg = sink_li_kg * i.sink_extra_mass_factor;
    let fly_kg = if i.auto_flywheel && i.flywheel_mj_per_t > 0.0 {
        laser_elec_w * i.flywheel_fire_s / (i.flywheel_mj_per_t * 1e3)
    } else {
        0.0
    };
    let payload_kg = i.payload_t * 1000.0 + sink_kg + fly_kg;

    // kg of drive hardware per newton of thrust, for the auto categories.
    if i.auto_rad_hot || i.auto_rad_low {
        require_pos(i.hot_t_k, "hot_t_k")?;
        require_pos(i.hot_eps, "hot_eps")?;
    }
    if i.auto_rad_hot {
        require_pos(i.hot_mw_per_kg, "hot_mw_per_kg")?;
    }
    if i.auto_rad_low {
        require_pos(i.low_t_k, "low_t_k")?;
        require_pos(i.low_eps, "low_eps")?;
        require_pos(i.low_mw_per_kg, "low_mw_per_kg")?;
    }

    let hot_flux_w_m2 = i.hot_eps * i.sigma * i.hot_t_k.powi(4);
    let low_flux_w_m2 = i.low_eps * i.sigma * i.low_t_k.powi(4);
    let hot_area_per_w = i.rad_load_frac / hot_flux_w_m2;
    let low_area_per_w = hot_area_per_w * i.low_area_frac;
    let hot_kg_per_w = i.rad_load_frac / (i.hot_mw_per_kg * 1e6);
    let low_kg_per_w = low_area_per_w * low_flux_w_m2 / (i.low_mw_per_kg * 1e6);
    let mut c_f = 0.0; // kg per N
    if i.p_fusion_pinned.is_none() {
        c_f += w_per_n * i.reactor_t_per_tw * 1e3 / 1e12;
    }
    if i.auto_nozzle {
        c_f += i.nozzle_cap_factor * i.nozzle_t_per_mn * 1e3 / 1e6;
    }
    if i.auto_rad_hot {
        c_f += w_per_n * hot_kg_per_w;
    }
    if i.auto_rad_low {
        c_f += w_per_n * low_kg_per_w;
    }
    let k_other = if i.auto_tank {
        i.tank_mass_per_prop * (i.mr - 1.0)
    } else {
        0.0
    } + if i.auto_structure {
        i.structure_frac
    } else {
        0.0
    };
    if k_other >= 1.0 {
        return Err(format!(
            "tank + structure fractions alone consume the whole dry mass (k = {:.2}) — lower MR or the scaling parameters",
            k_other
        ));
    }

    let (feasible, a_eff, a_max, dry_kg, thrust_n, p_fusion) = match i.p_fusion_pinned {
        None => {
            let a_max = (1.0 - k_other) / (i.mr * c_f);
            let feasible = i.a_target < a_max;
            let a_eff = if feasible { i.a_target } else { a_max * 0.98 };
            let k = a_eff * i.mr * c_f + k_other;
            let dry_kg = payload_kg / (1.0 - k);
            let thrust = a_eff * i.mr * dry_kg;
            (feasible, a_eff, a_max, dry_kg, thrust, thrust * w_per_n)
        }
        Some(p) => {
            // Reactor pinned (its mass is inside payload_t): thrust is fixed,
            // the ∝-thrust hardware becomes a constant, and only tank/structure
            // stay proportional.
            let p_jet = p * i.f_exh * i.eta_noz;
            let thrust = 2.0 * p_jet / i.ve_max;
            let fixed_kg = c_f * thrust;
            let dry_kg = (payload_kg + fixed_kg) / (1.0 - k_other);
            let a = thrust / (i.mr * dry_kg);
            (true, a, a, dry_kg, thrust, p)
        }
    };

    let wet_kg = dry_kg * i.mr;
    let prop_kg = wet_kg - dry_kg;
    Ok(AutosizeOut {
        feasible,
        a_max,
        a_achieved: a_eff,
        dry_t: dry_kg / 1000.0,
        wet_t: wet_kg / 1000.0,
        thrust_n,
        p_fusion_w: p_fusion,
        reactor_t: i
            .p_fusion_pinned
            .is_none()
            .then(|| p_fusion / 1e12 * i.reactor_t_per_tw),
        nozzle_cap_n: i.auto_nozzle.then_some(thrust_n * i.nozzle_cap_factor),
        nozzle_t: i
            .auto_nozzle
            .then(|| thrust_n * i.nozzle_cap_factor / 1e6 * i.nozzle_t_per_mn),
        hot_area_m2: i.auto_rad_hot.then_some(p_fusion * hot_area_per_w),
        hot_t: i.auto_rad_hot.then(|| p_fusion * hot_kg_per_w / 1000.0),
        low_area_m2: i.auto_rad_low.then_some(p_fusion * low_area_per_w),
        low_t: i.auto_rad_low.then(|| p_fusion * low_kg_per_w / 1000.0),
        tank_t: i.auto_tank.then(|| prop_kg * i.tank_mass_per_prop / 1000.0),
        tank_cap_t: i.auto_tank.then(|| prop_kg / 1000.0),
        sink_li_t: i.auto_sink.then(|| sink_li_kg / 1000.0),
        sink_t: i.auto_sink.then(|| sink_kg / 1000.0),
        flywheel_t: i.auto_flywheel.then(|| fly_kg / 1000.0),
        structure_t: i.auto_structure.then(|| dry_kg * i.structure_frac / 1000.0),
        laser_waste_w,
        laser_elec_w,
    })
}

// ---------------------------------------------------------------------------
// Designer component sizing. This is the authoritative implementation used by
// both the HTTP UI and the provider-neutral agent calculator. The returned
// design persists derived mass_t values so reports, saves, and simulations all
// consume the same snapshot without reimplementing these formulas.
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct DesignerActionIn {
    /// Component identifier within the supplied design.
    pub component_id: String,
    /// Sizing action: reactor-min, reactor-max, nozzle, radiator-hot,
    /// radiator-low, heat-sink, flywheel, tank, magazine, or structure.
    pub mode: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct DesignerIn {
    /// Shared engineering settings used by designer sizing.
    pub settings: Settings,
    /// Missile definitions used to derive loaded magazine and ordnance mass.
    #[serde(default)]
    pub missiles: Vec<Missile>,
    /// Ship design to normalize or resize.
    pub design: Design,
    /// Optional component-local sizing action. Omit to normalize all masses.
    #[serde(default)]
    pub action: Option<DesignerActionIn>,
}

#[derive(Clone, Serialize, JsonSchema)]
pub struct DesignerSummary {
    pub component_mass_t: f64,
    pub ordnance_t: f64,
    pub dry_t: f64,
    pub wet_t: f64,
    pub propellant_t: f64,
    pub tank_capacity_t: f64,
    pub reactor_fusion_w: f64,
    pub reactor_waste_w: f64,
    pub laser_waste_w: f64,
    pub laser_electrical_w: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct DesignerActionOut {
    pub component_id: String,
    pub mode: String,
    pub feasible: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub achieved_accel_mg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ceiling_accel_mg: Option<f64>,
}

#[derive(Serialize, JsonSchema)]
pub struct DesignerOut {
    pub design: Design,
    pub summary: DesignerSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<DesignerActionOut>,
}

fn missile_wet_kg(missile: &Missile) -> f64 {
    missile.payload_kg
        + missile
            .stages
            .iter()
            .map(|stage| stage.dry_mass_kg + stage.propellant_kg)
            .sum::<f64>()
}

fn designer_laser_count(component: &Component) -> f64 {
    component.count.unwrap_or(1).max(1) as f64
}

fn designer_component_mass_t(component: &Component) -> f64 {
    let count = if component.kind == "laser" {
        designer_laser_count(component)
    } else {
        1.0
    };
    component.mass_t.unwrap_or(0.0).max(0.0) * count
}

fn designer_laser_totals(design: &Design, settings: &Settings) -> (f64, f64) {
    design
        .components
        .iter()
        .filter(|component| component.kind == "laser")
        .fold((0.0, 0.0), |(waste, electrical), component| {
            let eta = component.eta_wall.unwrap_or(settings.laser_eta_wall);
            let beam = component.p_beam_w.unwrap_or(0.0) * designer_laser_count(component);
            if eta > 0.0 {
                (waste + beam * (1.0 / eta - 1.0), electrical + beam / eta)
            } else {
                (waste, electrical)
            }
        })
}

fn designer_reactor_totals(design: &Design, settings: &Settings) -> (f64, f64) {
    design
        .components
        .iter()
        .filter(|component| component.kind == "reactor")
        .fold((0.0, 0.0), |(power, waste), component| {
            let fusion = component.p_fusion_w.unwrap_or(0.0);
            (
                power + fusion,
                waste + fusion * component.rad_load_frac.unwrap_or(settings.as_rad_load_frac),
            )
        })
}

fn designer_ordnance_t(design: &Design, missile_masses: &BTreeMap<String, f64>) -> f64 {
    design
        .components
        .iter()
        .filter(|component| component.kind == "magazine")
        .map(|component| {
            component.capacity.unwrap_or(0) as f64
                * component
                    .missile_id
                    .as_ref()
                    .and_then(|id| missile_masses.get(id))
                    .copied()
                    .unwrap_or(0.0)
                / 1000.0
        })
        .sum()
}

fn designer_summary(
    design: &Design,
    settings: &Settings,
    missile_masses: &BTreeMap<String, f64>,
) -> DesignerSummary {
    let component_mass_t = design
        .components
        .iter()
        .map(designer_component_mass_t)
        .sum();
    let ordnance_t = designer_ordnance_t(design, missile_masses);
    let dry_t = component_mass_t + ordnance_t + design.structure_t.max(0.0);
    let wet_t = dry_t * design.mr;
    let tank_capacity_t = design
        .components
        .iter()
        .filter(|component| component.kind == "tank")
        .map(|component| {
            component.mass_t.unwrap_or(0.0)
                / component
                    .tank_structure_frac
                    .filter(|fraction| *fraction > 0.0)
                    .unwrap_or(1.0 / settings.tank_prop_per_mass)
        })
        .sum();
    let (reactor_fusion_w, reactor_waste_w) = designer_reactor_totals(design, settings);
    let (laser_waste_w, laser_electrical_w) = designer_laser_totals(design, settings);
    DesignerSummary {
        component_mass_t,
        ordnance_t,
        dry_t,
        wet_t,
        propellant_t: wet_t - dry_t,
        tank_capacity_t,
        reactor_fusion_w,
        reactor_waste_w,
        laser_waste_w,
        laser_electrical_w,
    }
}

fn designer_radiator_power_w(component: &Component, settings: &Settings) -> f64 {
    let area = component.area_m2.unwrap_or(0.0).max(0.0);
    if let Some(surface) = component.mw_per_m2.filter(|value| *value > 0.0) {
        area * surface * 1e6
    } else {
        area * component.eps.unwrap_or(0.9) * settings.sigma * component.t_k.unwrap_or(0.0).powi(4)
    }
}

fn designer_prepare_legacy(
    design: &mut Design,
    settings: &Settings,
    missile_masses: &BTreeMap<String, f64>,
) {
    let (laser_waste, laser_electrical) = designer_laser_totals(design, settings);
    let (fusion, _) = designer_reactor_totals(design, settings);
    let jet_power = fusion * settings.f_exh * settings.eta_noz;
    for component in &mut design.components {
        if component.mass_override.is_none()
            && matches!(
                component.kind.as_str(),
                "reactor"
                    | "nozzle"
                    | "radiator_hot"
                    | "radiator_low"
                    | "heat_sink"
                    | "flywheel"
                    | "laser"
                    | "magazine"
                    | "tank"
                    | "crew"
                    | "structure"
            )
        {
            component.mass_override = Some(component.auto == Some(false));
        }
        match component.kind.as_str() {
            "reactor" => {
                component.mw_per_kg.get_or_insert_with(|| {
                    component.p_fusion_w.unwrap_or(0.0)
                        / (component.mass_t.unwrap_or(0.0).max(1e-12) * 1e9)
                });
                component
                    .target_accel_mg
                    .get_or_insert(settings.as_target_accel_mg);
            }
            "nozzle" => {
                component
                    .mw_per_kg
                    .get_or_insert(jet_power / (component.mass_t.unwrap_or(0.0).max(1e-12) * 1e9));
            }
            "radiator_hot" | "radiator_low" => {
                let t = component
                    .t_k
                    .unwrap_or(if component.kind == "radiator_hot" {
                        settings.as_hot_t_k
                    } else {
                        settings.as_low_t_k
                    });
                let eps = component.eps.unwrap_or(0.9);
                let surface = component
                    .mw_per_m2
                    .get_or_insert(eps * settings.sigma * t.powi(4) / 1e6);
                let specific =
                    component
                        .mw_per_kg
                        .get_or_insert(if component.kind == "radiator_hot" {
                            settings.as_hot_mw_per_kg
                        } else {
                            settings.as_low_mw_per_kg
                        });
                component
                    .kg_per_m2
                    .get_or_insert(*surface / specific.max(1e-12));
                component
                    .radiator_mode
                    .get_or_insert_with(|| "specific_power".into());
                if component.kind == "radiator_low" && component.laser_waste_frac.is_none() {
                    component.laser_waste_frac = Some(if laser_waste > 0.0 {
                        (designer_radiator_power_w(component, settings) / laser_waste).min(1.0)
                    } else {
                        1.0
                    });
                }
                if component.mass_t.is_none() {
                    let power = designer_radiator_power_w(component, settings);
                    component.mass_t =
                        Some(if component.radiator_mode.as_deref() == Some("areal") {
                            component.area_m2.unwrap_or(0.0) * component.kg_per_m2.unwrap_or(0.0)
                                / 1000.0
                        } else {
                            power / (component.mw_per_kg.unwrap_or(0.0).max(1e-12) * 1e9)
                        });
                }
            }
            "heat_sink" => {
                let capacity = *component
                    .energy_mj_per_kg
                    .get_or_insert(settings.li_sink_mj_per_kg);
                component.installed_mass_factor.get_or_insert(
                    component.mass_t.unwrap_or(0.0) / component.li_t.unwrap_or(0.0).max(1e-12),
                );
                component.endurance_s.get_or_insert(if laser_waste > 0.0 {
                    component.li_t.unwrap_or(0.0) * 1e9 * capacity / laser_waste
                } else {
                    settings.as_sink_endurance_min * 60.0
                });
            }
            "flywheel" => {
                component
                    .material
                    .get_or_insert_with(|| "Carbon-fiber composite".into());
                let density = *component
                    .energy_mj_per_kg
                    .get_or_insert(settings.flywheel_mj_per_t / 1000.0);
                component
                    .endurance_s
                    .get_or_insert(if laser_electrical > 0.0 {
                        component.mass_t.unwrap_or(0.0) * 1e9 * density / laser_electrical
                    } else {
                        settings.as_flywheel_fire_s
                    });
            }
            "laser" => {
                component.mw_per_kg.get_or_insert(
                    component.p_beam_w.unwrap_or(0.0)
                        / (component.mass_t.unwrap_or(0.0).max(1e-12) * 1e9),
                );
            }
            "magazine" => {
                let loaded_t = component.capacity.unwrap_or(0) as f64
                    * component
                        .missile_id
                        .as_ref()
                        .and_then(|id| missile_masses.get(id))
                        .copied()
                        .unwrap_or(0.0)
                    / 1000.0;
                component
                    .missile_mass_ratio
                    .get_or_insert(loaded_t / component.mass_t.unwrap_or(0.0).max(1e-12));
            }
            "tank" => {
                component
                    .tank_structure_frac
                    .get_or_insert(1.0 / settings.tank_prop_per_mass);
            }
            _ => {}
        }
        component.auto = None;
    }

    if !design
        .components
        .iter()
        .any(|component| component.kind == "structure")
    {
        let old_structure = design.structure_t.max(0.0);
        let rest = design
            .components
            .iter()
            .map(designer_component_mass_t)
            .sum::<f64>()
            + designer_ordnance_t(design, missile_masses);
        design.components.push(Component {
            id: format!("{}-structure", design.id),
            kind: "structure".into(),
            name: "Primary structure".into(),
            mass_t: Some(old_structure),
            auto: None,
            mass_override: Some(!design.structure_auto),
            structure_frac: Some(old_structure / rest.max(1e-12)),
            p_fusion_w: None,
            rad_load_frac: None,
            f_max_n: None,
            area_m2: None,
            t_k: None,
            eps: None,
            mw_per_kg: None,
            radiator_mode: None,
            kg_per_m2: None,
            mw_per_m2: None,
            laser_waste_frac: None,
            endurance_s: None,
            material: None,
            energy_mj_per_kg: None,
            installed_mass_factor: None,
            tank_structure_frac: None,
            missile_mass_ratio: None,
            crew_count: None,
            tonnes_per_crew: None,
            target_accel_mg: None,
            li_t: None,
            count: None,
            p_beam_w: None,
            aperture_m: None,
            lambda_m: None,
            eta_wall: None,
            t_pulse_s: None,
            profiles: Vec::new(),
            missile_id: None,
            capacity: None,
            note: None,
            combat: None,
            extra: BTreeMap::new(),
        });
        design.structure_t = 0.0;
        design.structure_auto = false;
    }
}

fn designer_normalize(
    design: &mut Design,
    settings: &Settings,
    missile_masses: &BTreeMap<String, f64>,
) -> DesignerSummary {
    let (_, laser_electrical) = designer_laser_totals(design, settings);
    let (fusion, _) = designer_reactor_totals(design, settings);
    let auto_nozzles = design
        .components
        .iter()
        .filter(|component| component.kind == "nozzle" && component.mass_override != Some(true))
        .count()
        .max(1) as f64;
    let jet_power = fusion * settings.f_exh * settings.eta_noz;

    for component in &mut design.components {
        if component.mass_override == Some(true) {
            continue;
        }
        let mass = match component.kind.as_str() {
            "reactor" => {
                component.p_fusion_w.unwrap_or(0.0)
                    / (component.mw_per_kg.unwrap_or(0.0).max(1e-12) * 1e9)
            }
            "nozzle" => {
                jet_power / auto_nozzles / (component.mw_per_kg.unwrap_or(0.0).max(1e-12) * 1e9)
            }
            "radiator_hot" | "radiator_low" => {
                let power = designer_radiator_power_w(component, settings);
                if component.radiator_mode.as_deref() == Some("areal") {
                    component.area_m2.unwrap_or(0.0) * component.kg_per_m2.unwrap_or(0.0) / 1000.0
                } else {
                    power / (component.mw_per_kg.unwrap_or(0.0).max(1e-12) * 1e9)
                }
            }
            "heat_sink" => {
                component.li_t.unwrap_or(0.0)
                    * component
                        .installed_mass_factor
                        .unwrap_or(settings.as_sink_extra_mass_factor)
            }
            "flywheel" => {
                laser_electrical * component.endurance_s.unwrap_or(0.0)
                    / (component.energy_mj_per_kg.unwrap_or(0.0).max(1e-12) * 1e9)
            }
            "laser" => {
                component.p_beam_w.unwrap_or(0.0)
                    / (component.mw_per_kg.unwrap_or(0.0).max(1e-12) * 1e9)
            }
            "magazine" => {
                let loaded_t = component.capacity.unwrap_or(0) as f64
                    * component
                        .missile_id
                        .as_ref()
                        .and_then(|id| missile_masses.get(id))
                        .copied()
                        .unwrap_or(0.0)
                    / 1000.0;
                loaded_t / component.missile_mass_ratio.unwrap_or(0.0).max(1e-12)
            }
            "crew" => {
                component.crew_count.unwrap_or(0) as f64 * component.tonnes_per_crew.unwrap_or(0.0)
            }
            _ => continue,
        };
        component.mass_t = Some(mass.max(0.0));
        if matches!(component.kind.as_str(), "radiator_hot" | "radiator_low") {
            let eps = component.eps.unwrap_or(0.9);
            if let Some(surface) = component.mw_per_m2.filter(|value| *value > 0.0) {
                component.t_k = Some((surface * 1e6 / (eps * settings.sigma)).powf(0.25));
                component.eps = Some(eps);
            }
        }
    }

    let auto_tanks = design
        .components
        .iter()
        .filter(|component| component.kind == "tank" && component.mass_override != Some(true))
        .count();
    let auto_structures = design
        .components
        .iter()
        .filter(|component| component.kind == "structure" && component.mass_override != Some(true))
        .count();
    for _ in 0..50 {
        let before: f64 = design
            .components
            .iter()
            .filter(|component| {
                matches!(component.kind.as_str(), "tank" | "structure")
                    && component.mass_override != Some(true)
            })
            .map(|component| component.mass_t.unwrap_or(0.0))
            .sum();
        let current = designer_summary(design, settings, missile_masses);
        let propellant = current.dry_t * (design.mr - 1.0).max(0.0);
        if auto_tanks > 0 {
            for component in design.components.iter_mut().filter(|component| {
                component.kind == "tank" && component.mass_override != Some(true)
            }) {
                component.mass_t = Some(
                    propellant * component.tank_structure_frac.unwrap_or(0.0) / auto_tanks as f64,
                );
            }
        }
        let rest = design
            .components
            .iter()
            .filter(|component| component.kind != "structure")
            .map(designer_component_mass_t)
            .sum::<f64>()
            + designer_ordnance_t(design, missile_masses);
        if auto_structures > 0 {
            for component in design.components.iter_mut().filter(|component| {
                component.kind == "structure" && component.mass_override != Some(true)
            }) {
                component.mass_t =
                    Some(rest * component.structure_frac.unwrap_or(0.0) / auto_structures as f64);
            }
        }
        let after: f64 = design
            .components
            .iter()
            .filter(|component| {
                matches!(component.kind.as_str(), "tank" | "structure")
                    && component.mass_override != Some(true)
            })
            .map(|component| component.mass_t.unwrap_or(0.0))
            .sum();
        if (after - before).abs() < 1e-8_f64.max(after * 1e-10) {
            break;
        }
    }
    if design
        .components
        .iter()
        .any(|component| component.kind == "structure")
    {
        design.structure_t = 0.0;
        design.structure_auto = false;
    }
    designer_summary(design, settings, missile_masses)
}

pub fn designer(i: &DesignerIn) -> CalcResult<DesignerOut> {
    require_pos(i.design.mr, "design.mr")?;
    require_pos(i.settings.g, "settings.g")?;
    require_pos(i.settings.sigma, "settings.sigma")?;
    let missile_masses = i
        .missiles
        .iter()
        .map(|missile| (missile.id.clone(), missile_wet_kg(missile)))
        .collect::<BTreeMap<_, _>>();
    let mut design = i.design.clone();
    designer_prepare_legacy(&mut design, &i.settings, &missile_masses);
    let mut summary = designer_normalize(&mut design, &i.settings, &missile_masses);
    let mut action_out = None;

    if let Some(action) = &i.action {
        let index = design
            .components
            .iter()
            .position(|component| component.id == action.component_id)
            .ok_or_else(|| format!("component {} not found", action.component_id))?;
        let kind = design.components[index].kind.clone();
        let mut feasible = true;
        let mut achieved_accel_mg = None;
        let mut ceiling_accel_mg = None;
        let message;
        match action.mode.as_str() {
            "reactor-min" | "reactor-max" => {
                if kind != "reactor" {
                    return Err(format!("{} requires a reactor component", action.mode));
                }
                let ve = if action.mode == "reactor-min" {
                    i.settings.ve_gear_min_m_s
                } else {
                    i.settings.ve_max_m_s
                };
                require_pos(ve, "reactor sizing exhaust velocity")?;
                let target_mg = design.components[index]
                    .target_accel_mg
                    .unwrap_or(i.settings.as_target_accel_mg);
                let target = target_mg * i.settings.g * 1e-3;
                let accel_at = |power: f64, design: &mut Design| {
                    design.components[index].p_fusion_w = Some(power);
                    let report = designer_normalize(design, &i.settings, &missile_masses);
                    2.0 * report.reactor_fusion_w * i.settings.f_exh * i.settings.eta_noz
                        / ve
                        / (report.wet_t * 1000.0).max(1.0)
                };
                let mut lo = 0.0;
                let mut hi = design.components[index].p_fusion_w.unwrap_or(1e9).max(1e9);
                while accel_at(hi, &mut design) < target && hi < 1e22 {
                    hi *= 2.0;
                }
                let ceiling = accel_at(hi, &mut design);
                feasible = ceiling >= target;
                let solved_target = if feasible { target } else { ceiling * 0.98 };
                if solved_target <= 0.0 {
                    return Err("current design has no positive reactor sizing solution".into());
                }
                for _ in 0..80 {
                    let mid = (lo + hi) / 2.0;
                    if accel_at(mid, &mut design) < solved_target {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                design.components[index].p_fusion_w = Some(hi);
                designer_normalize(&mut design, &i.settings, &missile_masses);
                let achieved = solved_target / (i.settings.g * 1e-3);
                achieved_accel_mg = Some(achieved);
                ceiling_accel_mg = (!feasible).then_some(ceiling / (i.settings.g * 1e-3));
                message = if feasible {
                    format!("sized reactor to {:.3} MW for {:.3} mg", hi / 1e6, achieved)
                } else {
                    format!(
                        "target infeasible; sized to {:.3} mg, 98% of the mass-scaling ceiling",
                        achieved
                    )
                };
            }
            "nozzle" => {
                if kind != "nozzle" {
                    return Err("nozzle sizing requires a nozzle component".into());
                }
                let nozzles = design
                    .components
                    .iter()
                    .filter(|component| component.kind == "nozzle")
                    .count()
                    .max(1) as f64;
                design.components[index].f_max_n = Some(
                    2.0 * summary.reactor_fusion_w * i.settings.f_exh * i.settings.eta_noz
                        / i.settings.ve_gear_min_m_s
                        / nozzles,
                );
                message = "sized nozzle not to bottleneck at minimum exhaust velocity".into();
            }
            "radiator-hot" | "radiator-low" => {
                let expected = if action.mode == "radiator-hot" {
                    "radiator_hot"
                } else {
                    "radiator_low"
                };
                if kind != expected {
                    return Err(format!("{} requires a {} component", action.mode, expected));
                }
                let target_w = if action.mode == "radiator-hot" {
                    summary.reactor_waste_w
                } else {
                    summary.laser_waste_w * design.components[index].laser_waste_frac.unwrap_or(1.0)
                };
                let surface = require_pos(
                    design.components[index].mw_per_m2.unwrap_or(0.0),
                    "radiator mw_per_m2",
                )?;
                design.components[index].area_m2 = Some(target_w / (surface * 1e6));
                message = format!("sized radiator to reject {:.3} MW", target_w / 1e6);
            }
            "heat-sink" => {
                if kind != "heat_sink" {
                    return Err("heat-sink sizing requires a heat_sink component".into());
                }
                let capacity = design.components[index]
                    .energy_mj_per_kg
                    .unwrap_or(i.settings.li_sink_mj_per_kg)
                    .max(1e-12);
                design.components[index].li_t = Some(
                    summary.laser_waste_w * design.components[index].endurance_s.unwrap_or(0.0)
                        / (capacity * 1e9),
                );
                message = "sized heat storage for the requested laser operating time".into();
            }
            "flywheel" | "tank" | "magazine" | "structure" => {
                let expected = match action.mode.as_str() {
                    "flywheel" => "flywheel",
                    "tank" => "tank",
                    "magazine" => "magazine",
                    _ => "structure",
                };
                if kind != expected {
                    return Err(format!("{} requires a {} component", action.mode, expected));
                }
                message = format!("recalculated {} mass", expected);
            }
            other => return Err(format!("unknown designer sizing action {other}")),
        }
        summary = designer_normalize(&mut design, &i.settings, &missile_masses);
        action_out = Some(DesignerActionOut {
            component_id: action.component_id.clone(),
            mode: action.mode.clone(),
            feasible,
            message,
            achieved_accel_mg,
            ceiling_accel_mg,
        });
    }

    Ok(DesignerOut {
        design,
        summary,
        action: action_out,
    })
}

// ---------------------------------------------------------------------------
// Laser kill profiles: penetration *per pulse* against a material, with the
// kill range where it crosses the profile's threshold. Same drilling model,
// different bookkeeping — each profile carries its own pulse length.
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct LaserProfileIn {
    pub name: String,
    pub rho: f64,
    pub e_vap_mj: f64,
    pub t_pulse_s: f64,
    pub threshold_mm: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct LaserProfilesIn {
    pub p_beam: f64,
    pub aperture: f64,
    pub lambda: f64,
    pub eta_drill: f64,
    /// Doctrine multiplier for the "open fire" range (Lasers.md: 1.5×).
    #[serde(default = "default_open_fire")]
    pub open_fire_factor: f64,
    pub profiles: Vec<LaserProfileIn>,
    #[serde(default)]
    pub n: Option<usize>,
}

fn default_open_fire() -> f64 {
    1.5
}

#[derive(Serialize, JsonSchema)]
pub struct LaserProfileOut {
    pub name: String,
    /// Range where penetration-per-pulse equals the threshold, m.
    pub r_kill: f64,
    pub r_open: f64,
    /// Total energy in one pulse (J); constant with range in vacuum.
    pub pulse_energy_j: f64,
    /// Pulse energy per illuminated area (J/m²) on the shared range grid.
    pub fluence_j_m2: Vec<f64>,
    /// Penetration per pulse (mm) on the shared range grid.
    pub pen_mm: Vec<f64>,
}

#[derive(Serialize, JsonSchema)]
pub struct LaserProfilesOut {
    pub range_m: Vec<f64>,
    /// Airy-disc diameter (m) on the shared range grid.
    pub spot_diameter_m: Vec<f64>,
    /// Beam power per illuminated area (W/m²) on the shared range grid.
    pub irradiance_w_m2: Vec<f64>,
    /// Total beam power (W); constant with range in the vacuum model.
    pub beam_power_w: f64,
    pub profiles: Vec<LaserProfileOut>,
}

pub fn laser_profiles(i: &LaserProfilesIn) -> CalcResult<LaserProfilesOut> {
    require_pos(i.p_beam, "p_beam")?;
    require_pos(i.aperture, "aperture")?;
    require_pos(i.lambda, "lambda")?;
    require_pos(i.eta_drill, "eta_drill")?;
    if i.profiles.is_empty() {
        return Err("no profiles given".into());
    }
    let r_kill_of = |p: &LaserProfileIn| -> CalcResult<f64> {
        require_pos(p.rho, "rho")?;
        require_pos(p.e_vap_mj, "e_vap_mj")?;
        require_pos(p.t_pulse_s, "t_pulse_s")?;
        require_pos(p.threshold_mm, "threshold_mm")?;
        let rate_needed = p.threshold_mm / 1000.0 / p.t_pulse_s; // m/s
        let phi_min = rate_needed * p.rho * p.e_vap_mj * 1e6 / i.eta_drill;
        Ok(
            i.aperture * (4.0 * i.p_beam / (std::f64::consts::PI * phi_min)).sqrt()
                / (1.22 * i.lambda),
        )
    };
    let mut r_hi = 0.0f64;
    for p in &i.profiles {
        r_hi = r_hi.max(r_kill_of(p)?);
    }
    r_hi *= 1.15;
    let n = i.n.unwrap_or(140).clamp(8, 2000);
    let r_lo = r_hi / 400.0;
    let ratio = r_hi / r_lo;
    let range_m: Vec<f64> = (0..n)
        .map(|k| r_lo * ratio.powf(k as f64 / (n - 1) as f64))
        .collect();
    let spot_diameter_m: Vec<f64> = range_m
        .iter()
        .map(|r| 1.22 * i.lambda * r / i.aperture)
        .collect();
    let irradiance_w_m2: Vec<f64> = spot_diameter_m
        .iter()
        .map(|d| 4.0 * i.p_beam / (std::f64::consts::PI * d * d))
        .collect();
    let profiles = i
        .profiles
        .iter()
        .map(|p| {
            let r_kill = r_kill_of(p).unwrap();
            let pen_mm = range_m
                .iter()
                .zip(irradiance_w_m2.iter())
                .map(|(_r, flux)| {
                    let rate = i.eta_drill * flux / (p.rho * p.e_vap_mj * 1e6); // m/s
                    rate * p.t_pulse_s * 1000.0
                })
                .collect();
            let pulse_energy_j = i.p_beam * p.t_pulse_s;
            let fluence_j_m2 = irradiance_w_m2
                .iter()
                .map(|phi| phi * p.t_pulse_s)
                .collect();
            LaserProfileOut {
                name: p.name.clone(),
                r_kill,
                r_open: r_kill * i.open_fire_factor,
                pulse_energy_j,
                fluence_j_m2,
                pen_mm,
            }
        })
        .collect();
    Ok(LaserProfilesOut {
        range_m,
        spot_diameter_m,
        irradiance_w_m2,
        beam_power_w: i.p_beam,
        profiles,
    })
}

// ---------------------------------------------------------------------------
// Laser damage.
//
// Spot d = 1.22 lambda R / D;  flux = 4P/(pi d^2);
// penetration = eta_drill * flux / (rho * E_vap).
// R_max from the cutoff penetration rate.
// Pure vaporization-drilling model — ignores melt ejection and lateral
// thermal bleed (stated honestly in the UI).
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone, JsonSchema)]
pub struct LaserMaterial {
    pub name: String,
    pub rho: f64,
    pub e_vap_mj: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct LaserIn {
    pub p_beam: f64,
    pub aperture: f64,
    pub lambda: f64,
    pub eta_drill: f64,
    /// Penetration cutoff defining "effective", mm/s.
    pub cutoff_mm_s: f64,
    #[serde(default)]
    pub materials: Vec<LaserMaterial>,
    #[serde(default)]
    pub eta_wall: Option<f64>,
    #[serde(default)]
    pub t_pulse: Option<f64>,
    #[serde(default)]
    pub n: Option<usize>,
    /// Optional power/heat context for the shot panel.
    #[serde(default)]
    pub flywheel_mj: Option<f64>,
    #[serde(default)]
    pub sink_mj: Option<f64>,
    /// Low-temp loop rejection available while firing, W.
    #[serde(default)]
    pub q_low_w: Option<f64>,
}

#[derive(Serialize, JsonSchema)]
pub struct LaserMaterialOut {
    pub name: String,
    pub r_max: f64,
    pub phi_min: f64,
    /// Penetration rate (mm/s) sampled on the shared range grid.
    pub rate_mm_s: Vec<f64>,
}

#[derive(Serialize, JsonSchema)]
pub struct LaserShot {
    pub beam_mj: f64,
    pub electrical_mj: f64,
    pub waste_mj: f64,
    pub shots_per_bank: Option<f64>,
}

#[derive(Serialize, JsonSchema)]
pub struct LaserSustain {
    /// Continuous-fire wall-plug waste, W.
    pub waste_w: f64,
    /// Seconds until the sink saturates with the low loop retracted / deployed.
    /// None = indefinite (rejection keeps up).
    pub endurance_retracted_s: Option<f64>,
    pub endurance_deployed_s: Option<f64>,
}

#[derive(Serialize, JsonSchema)]
pub struct LaserOut {
    pub r_max_global: f64,
    /// Shared range grid (m) for all material curves.
    pub range_m: Vec<f64>,
    pub materials: Vec<LaserMaterialOut>,
    pub shot: Option<LaserShot>,
    pub sustain: Option<LaserSustain>,
}

pub fn laser(i: &LaserIn) -> CalcResult<LaserOut> {
    require_pos(i.p_beam, "p_beam")?;
    require_pos(i.aperture, "aperture")?;
    require_pos(i.lambda, "lambda")?;
    require_pos(i.eta_drill, "eta_drill")?;
    require_pos(i.cutoff_mm_s, "cutoff_mm_s")?;
    let cutoff_m_s = i.cutoff_mm_s / 1000.0;

    let r_max_of = |m: &LaserMaterial| -> f64 {
        let e_vap = m.e_vap_mj * 1e6;
        let phi_min = cutoff_m_s * m.rho * e_vap / i.eta_drill;
        i.aperture * (4.0 * i.p_beam / (std::f64::consts::PI * phi_min)).sqrt() / (1.22 * i.lambda)
    };

    let r_max_global = i.materials.iter().map(&r_max_of).fold(0.0f64, f64::max);

    let n = i.n.unwrap_or(120).clamp(8, 2000);
    let mut range_m = Vec::with_capacity(n);
    if r_max_global > 0.0 {
        let r_lo = r_max_global / 200.0;
        let r_hi = r_max_global * 1.02;
        for k in 0..n {
            range_m.push(r_lo + (r_hi - r_lo) * k as f64 / (n - 1) as f64);
        }
    }

    let materials = i
        .materials
        .iter()
        .map(|m| {
            let e_vap = m.e_vap_mj * 1e6;
            let phi_min = cutoff_m_s * m.rho * e_vap / i.eta_drill;
            let rate_mm_s = range_m
                .iter()
                .map(|r| {
                    let d = 1.22 * i.lambda * r / i.aperture;
                    let flux = 4.0 * i.p_beam / (std::f64::consts::PI * d * d);
                    1000.0 * i.eta_drill * flux / (m.rho * e_vap)
                })
                .collect();
            LaserMaterialOut {
                name: m.name.clone(),
                r_max: r_max_of(m),
                phi_min,
                rate_mm_s,
            }
        })
        .collect();

    let shot = match (i.eta_wall, i.t_pulse) {
        (Some(ew), Some(tp)) if ew > 0.0 && tp > 0.0 => {
            let beam = i.p_beam * tp / 1e6;
            let electrical = beam / ew;
            Some(LaserShot {
                beam_mj: beam,
                electrical_mj: electrical,
                waste_mj: electrical - beam,
                shots_per_bank: i.flywheel_mj.filter(|f| *f > 0.0).map(|f| f / electrical),
            })
        }
        _ => None,
    };

    let sustain = i.eta_wall.filter(|ew| *ew > 0.0).map(|ew| {
        let waste_w = i.p_beam * (1.0 / ew - 1.0);
        // None = indefinite; the client only asks when it supplies a sink.
        let endurance = |q_reject: f64| -> Option<f64> {
            let net = waste_w - q_reject;
            match i.sink_mj {
                Some(sink) if sink > 0.0 && net > 0.0 => Some(sink * 1e6 / net),
                _ => None,
            }
        };
        LaserSustain {
            waste_w,
            endurance_retracted_s: endurance(0.0),
            endurance_deployed_s: endurance(i.q_low_w.unwrap_or(0.0)),
        }
    });

    Ok(LaserOut {
        r_max_global,
        range_m,
        materials,
        shot,
        sustain,
    })
}

// ---------------------------------------------------------------------------
// Heat.
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct RadiatorIn {
    pub area: f64,
    pub t_k: f64,
    pub eps: f64,
    pub sigma: f64,
    /// 0..100
    #[serde(default = "hundred")]
    pub integrity_pct: f64,
}

fn hundred() -> f64 {
    100.0
}

#[derive(Serialize, JsonSchema)]
pub struct RadiatorOut {
    pub q_w: f64,
}

pub fn radiator(i: &RadiatorIn) -> CalcResult<RadiatorOut> {
    require_pos(i.area, "area")?;
    require_pos(i.t_k, "t_k")?;
    Ok(RadiatorOut {
        q_w: i.eps * i.sigma * i.t_k.powi(4) * i.area * (i.integrity_pct / 100.0).clamp(0.0, 1.0),
    })
}

#[derive(Deserialize, JsonSchema)]
pub struct VentIn {
    pub heat_mj: f64,
    /// Heat dumped per kg of lithium expelled (MJ/kg), canon 19.6.
    pub vent_mj_per_kg: f64,
    /// Sink capacity per kg (MJ/kg), canon 4.6.
    pub sink_mj_per_kg: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct VentOut {
    pub li_kg: f64,
    /// Permanent sink capacity loss from the vented mass.
    pub capacity_lost_mj: f64,
}

pub fn vent(i: &VentIn) -> CalcResult<VentOut> {
    require_pos(i.heat_mj, "heat_mj")?;
    require_pos(i.vent_mj_per_kg, "vent_mj_per_kg")?;
    require_pos(i.sink_mj_per_kg, "sink_mj_per_kg")?;
    let li_kg = i.heat_mj / i.vent_mj_per_kg;
    Ok(VentOut {
        li_kg,
        capacity_lost_mj: li_kg * i.sink_mj_per_kg,
    })
}

// ---------------------------------------------------------------------------
// Staged missiles. Each stage burns at constant thrust. The stage's a0 is the
// ignition acceleration of the complete stack that remains at that point.
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone, JsonSchema)]
pub struct MissileStageIn {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub dry_mass_kg: f64,
    pub propellant_kg: f64,
    pub ve: f64,
    pub a0_g: f64,
    #[serde(default)]
    pub jettison: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct MissileIn {
    #[serde(default)]
    pub payload_kg: f64,
    pub stages: Vec<MissileStageIn>,
    pub g: f64,
}

#[derive(Serialize, Clone, JsonSchema)]
pub struct MissileStageOut {
    pub id: String,
    pub name: String,
    pub ignition_mass_kg: f64,
    pub burnout_mass_kg: f64,
    pub post_jettison_mass_kg: f64,
    pub dry_mass_kg: f64,
    pub propellant_kg: f64,
    pub ve: f64,
    pub thrust: f64,
    pub mdot: f64,
    pub t_ignition: f64,
    pub t_burn: f64,
    pub dv: f64,
    pub accel_ignition_g: f64,
    pub accel_burnout_g: f64,
    pub jettison: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct MissileSampleOut {
    pub t: f64,
    pub accel_g: f64,
    pub velocity: f64,
    pub mass_kg: f64,
    pub stage_id: String,
    pub event: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct MissileOut {
    pub m_wet: f64,
    pub m_dry: f64,
    pub m_prop: f64,
    pub dv: f64,
    /// Compatibility aggregate: first-stage thrust and flow.
    pub thrust: f64,
    pub mdot: f64,
    pub t_burn: f64,
    pub a_burnout_g: f64,
    /// Compatibility plot data: [time, acceleration].
    pub accel_profile: Vec<[f64; 2]>,
    pub stage_reports: Vec<MissileStageOut>,
    pub profile: Vec<MissileSampleOut>,
}

pub fn missile(i: &MissileIn) -> CalcResult<MissileOut> {
    require_pos(i.g, "g")?;
    if !i.payload_kg.is_finite() || i.payload_kg < 0.0 {
        return Err("payload_kg must be finite and non-negative".into());
    }
    if i.stages.is_empty() {
        return Err("at least one missile stage is required".into());
    }
    let mut ids = std::collections::HashSet::new();
    for s in &i.stages {
        if s.id.trim().is_empty() || !ids.insert(s.id.clone()) {
            return Err("missile stage ids must be non-empty and unique".into());
        }
        if !s.dry_mass_kg.is_finite() || s.dry_mass_kg < 0.0 {
            return Err(format!("stage {} dry_mass_kg must be non-negative", s.id));
        }
        require_pos(s.propellant_kg, &format!("stage {} propellant_kg", s.id))?;
        require_pos(s.ve, &format!("stage {} ve", s.id))?;
        require_pos(s.a0_g, &format!("stage {} a0_g", s.id))?;
    }

    let m_prop: f64 = i.stages.iter().map(|s| s.propellant_kg).sum();
    let inert: f64 = i.stages.iter().map(|s| s.dry_mass_kg).sum();
    let m_wet = i.payload_kg + inert + m_prop;
    require_pos(m_wet, "missile wet mass")?;

    let mut mass = m_wet;
    let mut time = 0.0;
    let mut velocity = 0.0;
    let mut reports = Vec::with_capacity(i.stages.len());
    let mut profile = Vec::new();
    let mut accel_profile = Vec::new();
    for stage in &i.stages {
        if mass <= stage.propellant_kg {
            return Err(format!(
                "stage {} propellant exceeds the remaining stack mass",
                stage.id
            ));
        }
        let ignition_mass = mass;
        let thrust = stage.a0_g * i.g * ignition_mass;
        let mdot = thrust / stage.ve;
        let t_burn = stage.propellant_kg / mdot;
        let burnout_mass = mass - stage.propellant_kg;
        let dv = stage.ve * (ignition_mass / burnout_mass).ln();
        let a_burnout = thrust / burnout_mass / i.g;
        let t0 = time;
        for k in 0..=32 {
            let f = k as f64 / 32.0;
            let ts = t_burn * f;
            let mk = ignition_mass - stage.propellant_kg * f;
            let vk = velocity + stage.ve * (ignition_mass / mk).ln();
            let ak = thrust / mk / i.g;
            profile.push(MissileSampleOut {
                t: t0 + ts,
                accel_g: ak,
                velocity: vk,
                mass_kg: mk,
                stage_id: stage.id.clone(),
                event: (k == 32).then(|| "burnout".into()),
            });
            accel_profile.push([t0 + ts, ak]);
        }
        time += t_burn;
        velocity += dv;
        mass = burnout_mass;
        if stage.jettison {
            if mass < stage.dry_mass_kg {
                return Err(format!("stage {} dry mass exceeds burnout mass", stage.id));
            }
            mass -= stage.dry_mass_kg;
            profile.push(MissileSampleOut {
                t: time,
                accel_g: 0.0,
                velocity,
                mass_kg: mass,
                stage_id: stage.id.clone(),
                event: Some("jettison".into()),
            });
        }
        reports.push(MissileStageOut {
            id: stage.id.clone(),
            name: if stage.name.is_empty() {
                stage.id.clone()
            } else {
                stage.name.clone()
            },
            ignition_mass_kg: ignition_mass,
            burnout_mass_kg: burnout_mass,
            post_jettison_mass_kg: mass,
            dry_mass_kg: stage.dry_mass_kg,
            propellant_kg: stage.propellant_kg,
            ve: stage.ve,
            thrust,
            mdot,
            t_ignition: t0,
            t_burn,
            dv,
            accel_ignition_g: stage.a0_g,
            accel_burnout_g: a_burnout,
            jettison: stage.jettison,
        });
    }
    let first = reports.first().unwrap();
    let final_accel = reports.last().unwrap().accel_burnout_g;
    Ok(MissileOut {
        m_wet,
        m_dry: m_wet - m_prop,
        m_prop,
        dv: velocity,
        thrust: first.thrust,
        mdot: first.mdot,
        t_burn: time,
        a_burnout_g: final_accel,
        accel_profile,
        stage_reports: reports,
        profile,
    })
}

// ---------------------------------------------------------------------------
// Two-stage missile optimizer. The "bus" carries a payload of complete MH
// submunitions and chooses the exhaust velocity that maximizes bus delta-v
// after reactor, radiator, tank, and guidance mass are accounted for.
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct MissileOptimizeIn {
    pub total_mass_kg: f64,
    pub a0_g: f64,
    pub reactor_specific_power_mw_kg: f64,
    pub radiator_specific_power_mw_kg: f64,
    pub waste_heat_fraction: f64,
    pub mh_ve: f64,
    pub h2_cooling_j_kg: f64,
    pub mh_cooling_j_kg: f64,
    pub n_submunitions: u32,
    pub submunition_dv: f64,
    pub tank_fraction: f64,
    pub guidance_mass_kg: f64,
    pub reference_sub_dry_kg: f64,
    pub g: f64,
}

#[derive(Serialize, Clone, JsonSchema)]
pub struct MissileOptimizedDesignOut {
    pub ve: f64,
    pub dv: f64,
    pub mass_ratio: f64,
    pub reactor_power_w: f64,
    pub reactor_mass_kg: f64,
    pub radiator_mass_kg: f64,
    pub power_system_mass_kg: f64,
    pub submunitions_wet_kg: f64,
    pub submunition_wet_each_kg: f64,
    pub bus_dry_mass_kg: f64,
    pub propellant_kg: f64,
    pub tank_mass_kg: f64,
    pub burn_time_s: f64,
    pub final_accel_g: f64,
    pub mdot_kg_s: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct MissileOptimizeSweepOut {
    pub sub_dry_kg: f64,
    pub sub_wet_kg: f64,
    pub fusion_ve: Option<f64>,
    pub fusion_dv: Option<f64>,
    pub fusion_mass_ratio: Option<f64>,
    pub fusion_power_w: Option<f64>,
    pub h2_fusion_dv: Option<f64>,
    pub pure_mh_dv: Option<f64>,
    pub pure_mh_mass_ratio: Option<f64>,
    pub fusion_advantage: Option<f64>,
}

#[derive(Serialize, JsonSchema)]
pub struct MissileOptimizeOut {
    pub submunition_mass_ratio: f64,
    pub thrust_n: f64,
    pub h2_critical_ve: f64,
    pub reference_fusion: Option<MissileOptimizedDesignOut>,
    pub reference_h2_fusion: Option<MissileOptimizedDesignOut>,
    pub reference_pure_mh: Option<MissileOptimizedDesignOut>,
    pub sweep: Vec<MissileOptimizeSweepOut>,
}

#[derive(Clone, Copy, JsonSchema)]
enum OptimizerPropulsion {
    MhFusion,
    H2Fusion,
    PureMh,
}

fn optimized_missile_design(
    i: &MissileOptimizeIn,
    sub_dry_kg: f64,
    thrust_n: f64,
    sub_mr: f64,
    mode: OptimizerPropulsion,
) -> Option<MissileOptimizedDesignOut> {
    let eta = 1.0 - i.waste_heat_fraction;
    let reactor_alpha = i.reactor_specific_power_mw_kg * 1e6;
    let radiator_alpha = i.radiator_specific_power_mw_kg * 1e6;
    let submunitions = i.n_submunitions as f64 * sub_dry_kg * sub_mr;

    let power_masses = |ve: f64| -> (f64, f64, f64) {
        match mode {
            OptimizerPropulsion::PureMh => (0.0, 0.0, 0.0),
            OptimizerPropulsion::MhFusion => {
                let de = ve * ve - i.mh_ve * i.mh_ve;
                if de <= 0.0 {
                    return (0.0, 0.0, 0.0);
                }
                let waste_per_kg = i.waste_heat_fraction * de / 2.0;
                let power = if waste_per_kg <= i.mh_cooling_j_kg {
                    thrust_n * de / (2.0 * ve)
                } else {
                    thrust_n * (ve * ve / 2.0 - i.mh_ve * i.mh_ve / 2.0 - i.mh_cooling_j_kg)
                        / (eta * ve)
                };
                let radiator_heat =
                    (i.waste_heat_fraction * power - (thrust_n / ve) * i.mh_cooling_j_kg).max(0.0);
                (power / reactor_alpha, radiator_heat / radiator_alpha, power)
            }
            OptimizerPropulsion::H2Fusion => {
                let critical = (2.0 * i.h2_cooling_j_kg / i.waste_heat_fraction).sqrt();
                let power = if ve <= critical {
                    thrust_n * ve / 2.0
                } else {
                    thrust_n * (ve * ve / 2.0 - i.h2_cooling_j_kg) / (eta * ve)
                };
                let radiator_heat =
                    (i.waste_heat_fraction * power - (thrust_n / ve) * i.h2_cooling_j_kg).max(0.0);
                (power / reactor_alpha, radiator_heat / radiator_alpha, power)
            }
        }
    };

    let design_at = |ve: f64| -> Option<MissileOptimizedDesignOut> {
        let (reactor_mass, radiator_mass, reactor_power) = power_masses(ve);
        let power_mass = reactor_mass + radiator_mass;
        // tank_mass = tank_fraction * propellant_mass; solve the fixed point.
        let dry_total =
            (submunitions + i.guidance_mass_kg + power_mass + i.tank_fraction * i.total_mass_kg)
                / (1.0 + i.tank_fraction);
        if !dry_total.is_finite() || dry_total <= 0.0 || dry_total >= i.total_mass_kg {
            return None;
        }
        let propellant = i.total_mass_kg - dry_total;
        let tank_mass = i.tank_fraction * propellant;
        let mass_ratio = i.total_mass_kg / dry_total;
        let dv = ve * mass_ratio.ln();
        let mdot = thrust_n / ve;
        Some(MissileOptimizedDesignOut {
            ve,
            dv,
            mass_ratio,
            reactor_power_w: reactor_power,
            reactor_mass_kg: reactor_mass,
            radiator_mass_kg: radiator_mass,
            power_system_mass_kg: power_mass,
            submunitions_wet_kg: submunitions,
            submunition_wet_each_kg: sub_dry_kg * sub_mr,
            bus_dry_mass_kg: i.guidance_mass_kg + power_mass + tank_mass,
            propellant_kg: propellant,
            tank_mass_kg: tank_mass,
            burn_time_s: propellant / mdot,
            final_accel_g: thrust_n / dry_total / i.g,
            mdot_kg_s: mdot,
        })
    };

    if matches!(mode, OptimizerPropulsion::PureMh) {
        return design_at(i.mh_ve);
    }
    let mut best: Option<MissileOptimizedDesignOut> = None;
    for n in 1..=((400_000.0 - i.mh_ve).max(0.0) / 500.0).floor() as u32 {
        let ve = i.mh_ve + n as f64 * 500.0;
        if let Some(candidate) = design_at(ve) {
            if best.as_ref().is_none_or(|b| candidate.dv > b.dv) {
                best = Some(candidate);
            }
        }
    }
    if let Some(coarse) = &best {
        let lo = (i.mh_ve + 100.0).max(coarse.ve - 1000.0);
        let hi = coarse.ve + 1000.0;
        let mut ve = lo;
        while ve <= hi {
            if let Some(candidate) = design_at(ve) {
                if best.as_ref().is_none_or(|b| candidate.dv > b.dv) {
                    best = Some(candidate);
                }
            }
            ve += 50.0;
        }
    }
    best
}

pub fn optimize_missile(i: &MissileOptimizeIn) -> CalcResult<MissileOptimizeOut> {
    require_pos(i.total_mass_kg, "total_mass_kg")?;
    require_pos(i.a0_g, "a0_g")?;
    require_pos(
        i.reactor_specific_power_mw_kg,
        "reactor_specific_power_mw_kg",
    )?;
    require_pos(
        i.radiator_specific_power_mw_kg,
        "radiator_specific_power_mw_kg",
    )?;
    require_pos(i.mh_ve, "mh_ve")?;
    require_pos(i.reference_sub_dry_kg, "reference_sub_dry_kg")?;
    require_pos(i.g, "g")?;
    if !i.waste_heat_fraction.is_finite()
        || i.waste_heat_fraction <= 0.0
        || i.waste_heat_fraction >= 1.0
    {
        return Err("waste_heat_fraction must be between 0 and 1".into());
    }
    if i.n_submunitions == 0 {
        return Err("n_submunitions must be at least 1".into());
    }
    for (value, name) in [
        (i.h2_cooling_j_kg, "h2_cooling_j_kg"),
        (i.mh_cooling_j_kg, "mh_cooling_j_kg"),
        (i.submunition_dv, "submunition_dv"),
        (i.guidance_mass_kg, "guidance_mass_kg"),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(format!("{} must be finite and non-negative", name));
        }
    }
    if !i.tank_fraction.is_finite() || i.tank_fraction < 0.0 || i.tank_fraction >= 1.0 {
        return Err("tank_fraction must be at least 0 and less than 1".into());
    }

    let thrust_n = i.total_mass_kg * i.a0_g * i.g;
    let sub_mr = (i.submunition_dv / i.mh_ve).exp();
    if !sub_mr.is_finite() {
        return Err("submunition mass ratio is not finite".into());
    }
    let reference_fusion = optimized_missile_design(
        i,
        i.reference_sub_dry_kg,
        thrust_n,
        sub_mr,
        OptimizerPropulsion::MhFusion,
    );
    let reference_h2_fusion = optimized_missile_design(
        i,
        i.reference_sub_dry_kg,
        thrust_n,
        sub_mr,
        OptimizerPropulsion::H2Fusion,
    );
    let reference_pure_mh = optimized_missile_design(
        i,
        i.reference_sub_dry_kg,
        thrust_n,
        sub_mr,
        OptimizerPropulsion::PureMh,
    );

    let max_sub_dry = (i.total_mass_kg * 0.95 / (i.n_submunitions as f64 * sub_mr)).floor();
    let step = (max_sub_dry / 30.0 / 5.0).round().max(1.0) * 5.0;
    let mut sweep = Vec::new();
    let mut sub_dry = 10.0;
    while sub_dry <= max_sub_dry && sweep.len() < 200 {
        let fusion =
            optimized_missile_design(i, sub_dry, thrust_n, sub_mr, OptimizerPropulsion::MhFusion);
        let h2 =
            optimized_missile_design(i, sub_dry, thrust_n, sub_mr, OptimizerPropulsion::H2Fusion);
        let mh =
            optimized_missile_design(i, sub_dry, thrust_n, sub_mr, OptimizerPropulsion::PureMh);
        sweep.push(MissileOptimizeSweepOut {
            sub_dry_kg: sub_dry,
            sub_wet_kg: sub_dry * sub_mr,
            fusion_ve: fusion.as_ref().map(|d| d.ve),
            fusion_dv: fusion.as_ref().map(|d| d.dv),
            fusion_mass_ratio: fusion.as_ref().map(|d| d.mass_ratio),
            fusion_power_w: fusion.as_ref().map(|d| d.reactor_power_w),
            h2_fusion_dv: h2.as_ref().map(|d| d.dv),
            pure_mh_dv: mh.as_ref().map(|d| d.dv),
            pure_mh_mass_ratio: mh.as_ref().map(|d| d.mass_ratio),
            fusion_advantage: fusion.as_ref().zip(mh.as_ref()).map(|(f, m)| f.dv / m.dv),
        });
        sub_dry += step;
    }

    Ok(MissileOptimizeOut {
        submunition_mass_ratio: sub_mr,
        thrust_n,
        h2_critical_ve: (2.0 * i.h2_cooling_j_kg / i.waste_heat_fraction).sqrt(),
        reference_fusion,
        reference_h2_fusion,
        reference_pure_mh,
        sweep,
    })
}

// ---------------------------------------------------------------------------
// Intercept: a stage-aware sequence of burn/coast phases.
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone, JsonSchema)]
pub struct PhaseIn {
    #[serde(default)]
    pub stage_id: String,
    /// Fraction of this stage's original propellant burned in this phase.
    pub prop_frac: f64,
    #[serde(default)]
    pub coast_to_range: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct InterceptIn {
    pub range: f64,
    pub v_close0: f64,
    #[serde(default)]
    pub payload_kg: f64,
    pub stages: Vec<MissileStageIn>,
    pub g: f64,
    #[serde(default)]
    pub phases: Vec<PhaseIn>,
}

#[derive(Serialize, JsonSchema)]
pub struct PhaseOut {
    pub kind: String,
    pub stage_id: Option<String>,
    pub t0: f64,
    pub t1: f64,
    pub x1: f64,
    pub v1: f64,
    pub dv: f64,
    pub mass_kg: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct InterceptSampleOut {
    pub t: f64,
    pub distance: f64,
    pub velocity: f64,
    pub mass_kg: f64,
    pub stage_id: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct InterceptOut {
    pub hit: bool,
    pub phase: String,
    pub t_hit: Option<f64>,
    pub t_burn: f64,
    pub x_burnout: f64,
    pub v_burnout: f64,
    pub v_terminal: Option<f64>,
    pub dv_spent: Option<f64>,
    pub dv_total: f64,
    pub timeline: Vec<PhaseOut>,
    pub miss_reason: Option<String>,
    /// Compatibility plot data: [time, distance closed, closing velocity].
    pub profile: Vec<[f64; 3]>,
    pub stage_profile: Vec<InterceptSampleOut>,
}

pub fn intercept(i: &InterceptIn) -> CalcResult<InterceptOut> {
    require_pos(i.range, "range")?;
    let m = missile(&MissileIn {
        payload_kg: i.payload_kg,
        stages: i.stages.clone(),
        g: i.g,
    })?;
    let defaults: Vec<PhaseIn> = i
        .stages
        .iter()
        .map(|s| PhaseIn {
            stage_id: s.id.clone(),
            prop_frac: 1.0,
            coast_to_range: None,
        })
        .collect();
    let phases = if i.phases.is_empty() {
        &defaults
    } else {
        &i.phases
    };

    let mut allocated = std::collections::HashMap::<String, f64>::new();
    for p in phases {
        if !p.prop_frac.is_finite() || p.prop_frac < 0.0 {
            return Err("phase propellant fraction must be non-negative".into());
        }
        let id = if p.stage_id.is_empty() {
            &i.stages[0].id
        } else {
            &p.stage_id
        };
        if !i.stages.iter().any(|s| &s.id == id) {
            return Err(format!("unknown missile stage {}", id));
        }
        let sum = allocated.entry(id.clone()).or_default();
        *sum += p.prop_frac;
        if *sum > 1.0 + 1e-9 {
            return Err(format!(
                "burn phases allocate more than 100% of stage {}",
                id
            ));
        }
    }

    let (mut t, mut x, mut v, mut mass) = (0.0, 0.0, i.v_close0, m.m_wet);
    let mut stage_idx = 0usize;
    let mut remaining: Vec<f64> = i.stages.iter().map(|s| s.propellant_kg).collect();
    let mut dv_spent_acc = 0.0;
    let mut timeline = Vec::new();
    let mut profile = vec![[0.0, 0.0, v]];
    let mut stage_profile = vec![InterceptSampleOut {
        t: 0.0,
        distance: 0.0,
        velocity: v,
        mass_kg: mass,
        stage_id: Some(i.stages[0].id.clone()),
    }];
    let mut hit_state: Option<(f64, f64, f64)> = None;
    let mut hit_during_burn = false;
    let mut miss_reason = None;
    let (mut x_burnout, mut v_burnout) = (0.0, v);

    'schedule: for ph in phases {
        while stage_idx < i.stages.len() && remaining[stage_idx] <= 1e-9 {
            stage_idx += 1;
        }
        if stage_idx >= i.stages.len() {
            break;
        }
        let stage = &i.stages[stage_idx];
        let requested = if ph.stage_id.is_empty() {
            &stage.id
        } else {
            &ph.stage_id
        };
        if requested != &stage.id {
            return Err(format!(
                "stage {} cannot ignite before stage {} is exhausted",
                requested, stage.id
            ));
        }
        let report = &m.stage_reports[stage_idx];
        let dm = (ph.prop_frac * stage.propellant_kg).min(remaining[stage_idx]);
        if dm > 1e-12 {
            let tau = mass / report.mdot;
            let t_b = dm / report.mdot;
            let mut cross = None;
            let (mut prev_s, mut prev_x) = (0.0, 0.0);
            for k in 1..=240 {
                let s = t_b * k as f64 / 240.0;
                let xr = burn_x(stage.ve, v, tau, s);
                if x + xr >= i.range && x + prev_x < i.range {
                    let (mut lo, mut hi) = (prev_s, s);
                    for _ in 0..60 {
                        let mid = 0.5 * (lo + hi);
                        if x + burn_x(stage.ve, v, tau, mid) < i.range {
                            lo = mid
                        } else {
                            hi = mid
                        }
                    }
                    cross = Some(0.5 * (lo + hi));
                    break;
                }
                prev_s = s;
                prev_x = xr;
            }
            let s_end = cross.unwrap_or(t_b);
            for k in 1..=40 {
                let s = s_end * k as f64 / 40.0;
                let xp = x + burn_x(stage.ve, v, tau, s);
                let vp = v + burn_dv(stage.ve, tau, s);
                let mp = mass - report.mdot * s;
                profile.push([t + s, xp, vp]);
                stage_profile.push(InterceptSampleOut {
                    t: t + s,
                    distance: xp,
                    velocity: vp,
                    mass_kg: mp,
                    stage_id: Some(stage.id.clone()),
                });
            }
            let dvb = burn_dv(stage.ve, tau, s_end);
            x += burn_x(stage.ve, v, tau, s_end);
            v += dvb;
            let used = report.mdot * s_end;
            mass -= used;
            remaining[stage_idx] = (remaining[stage_idx] - used).max(0.0);
            t += s_end;
            dv_spent_acc += dvb;
            timeline.push(PhaseOut {
                kind: "burn".into(),
                stage_id: Some(stage.id.clone()),
                t0: t - s_end,
                t1: t,
                x1: x,
                v1: v,
                dv: dvb,
                mass_kg: mass,
            });
            x_burnout = x;
            v_burnout = v;
            if cross.is_some() {
                hit_state = Some((t, v, dv_spent_acc));
                hit_during_burn = true;
                break 'schedule;
            }
            if remaining[stage_idx] <= 1e-9 && stage.jettison {
                mass -= stage.dry_mass_kg;
                timeline.push(PhaseOut {
                    kind: "jettison".into(),
                    stage_id: Some(stage.id.clone()),
                    t0: t,
                    t1: t,
                    x1: x,
                    v1: v,
                    dv: 0.0,
                    mass_kg: mass,
                });
                stage_profile.push(InterceptSampleOut {
                    t,
                    distance: x,
                    velocity: v,
                    mass_kg: mass,
                    stage_id: Some(stage.id.clone()),
                });
            }
        }
        if let Some(rc) = ph.coast_to_range {
            let x_target = (i.range - rc.max(0.0)).min(i.range);
            if x_target > x {
                if v <= 0.0 {
                    miss_reason = Some(
                        "stalled during coast — closing velocity ≤ 0 with distance to go".into(),
                    );
                    break 'schedule;
                }
                let t_c = (x_target - x) / v;
                for k in 1..=8 {
                    let s = t_c * k as f64 / 8.0;
                    profile.push([t + s, x + v * s, v]);
                    stage_profile.push(InterceptSampleOut {
                        t: t + s,
                        distance: x + v * s,
                        velocity: v,
                        mass_kg: mass,
                        stage_id: Some(stage.id.clone()),
                    });
                }
                t += t_c;
                x = x_target;
                timeline.push(PhaseOut {
                    kind: "coast".into(),
                    stage_id: Some(stage.id.clone()),
                    t0: t - t_c,
                    t1: t,
                    x1: x,
                    v1: v,
                    dv: 0.0,
                    mass_kg: mass,
                });
                if x >= i.range - 1e-6 {
                    hit_state = Some((t, v, dv_spent_acc));
                    break 'schedule;
                }
            }
        }
    }

    if hit_state.is_none() && miss_reason.is_none() {
        if v > 0.0 {
            let t_c = (i.range - x) / v;
            for k in 1..=12 {
                let s = t_c * k as f64 / 12.0;
                profile.push([t + s, x + v * s, v]);
                stage_profile.push(InterceptSampleOut {
                    t: t + s,
                    distance: x + v * s,
                    velocity: v,
                    mass_kg: mass,
                    stage_id: None,
                });
            }
            t += t_c;
            timeline.push(PhaseOut {
                kind: "terminal coast".into(),
                stage_id: None,
                t0: t - t_c,
                t1: t,
                x1: i.range,
                v1: v,
                dv: 0.0,
                mass_kg: mass,
            });
            hit_state = Some((t, v, dv_spent_acc));
        } else {
            miss_reason = Some("closing velocity never goes positive before burnout".into());
        }
    }

    let (hit, phase, t_hit, v_terminal, dv_spent) = match hit_state {
        Some((th, vt, dv)) => (
            true,
            if hit_during_burn {
                "powered".into()
            } else {
                "coast".into()
            },
            Some(th),
            Some(vt),
            Some(dv),
        ),
        None => (false, "miss".into(), None, None, None),
    };
    Ok(InterceptOut {
        hit,
        phase,
        t_hit,
        t_burn: m.t_burn,
        x_burnout,
        v_burnout,
        v_terminal,
        dv_spent,
        dv_total: m.dv,
        timeline,
        miss_reason,
        profile,
        stage_profile,
    })
}

// ---------------------------------------------------------------------------
// Design report: all Designer-tab consistency numbers in one call.
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct RadiatorSpec {
    pub area: f64,
    pub t_k: f64,
    pub eps: f64,
    #[serde(default = "hundred")]
    pub integrity_pct: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct LaserSpec {
    pub p_beam: f64,
    pub eta_wall: f64,
    pub t_pulse: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReportIn {
    pub p_fusion: f64,
    pub f_exh: f64,
    pub eta_noz: f64,
    #[serde(default)]
    pub e_afterburner: f64,
    pub ve_max: f64,
    #[serde(default)]
    pub f_cap: Option<f64>,
    pub m_dry: f64,
    pub m_wet: f64,
    #[serde(default)]
    pub dv_reserve: f64,
    pub rad_load_frac: f64,
    pub sigma: f64,
    #[serde(default)]
    pub rad_hot: Vec<RadiatorSpec>,
    #[serde(default)]
    pub rad_low: Vec<RadiatorSpec>,
    #[serde(default)]
    pub sink_mj: f64,
    #[serde(default)]
    pub flywheel_mj: f64,
    #[serde(default)]
    pub lasers: Vec<LaserSpec>,
}

#[derive(Serialize, JsonSchema)]
pub struct LaserShotReport {
    pub beam_mj: f64,
    pub electrical_mj: f64,
    pub waste_mj: f64,
    pub shots_per_bank: f64,
    /// Continuous-fire wall-plug waste, W.
    pub waste_w: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct ReportOut {
    pub p_jet: f64,
    pub thrust_max: f64,
    pub capped_at_ve_max: bool,
    pub ve_cap: Option<f64>,
    pub accel_wet: f64,
    pub accel_dry: f64,
    pub dv_plasma: f64,
    pub waste_heat_w: f64,
    pub hot_reject_w: f64,
    pub hot_margin_w: f64,
    pub low_reject_w: f64,
    pub laser_waste_w: f64,
    /// Seconds of continuous all-laser fire before the sink saturates
    /// (None = indefinite: low loop keeps up).
    pub sink_endurance_s: Option<f64>,
    /// Same, with the low-temp loop retracted (combat condition).
    pub sink_endurance_retracted_s: Option<f64>,
    pub laser_shots: Vec<LaserShotReport>,
}

pub fn design_report(i: &ReportIn) -> CalcResult<ReportOut> {
    let g = gear(&GearIn {
        p_fusion: i.p_fusion,
        f_exh: i.f_exh,
        eta_noz: i.eta_noz,
        e_afterburner: i.e_afterburner,
        ve: i.ve_max,
        ve_max: i.ve_max,
        f_cap: i.f_cap,
        mass_kg: None,
        duration_s: None,
    })?;
    let dv = deltav(&DeltavIn {
        ve: i.ve_max,
        m_wet: i.m_wet,
        m_dry: i.m_dry,
        dv_reserve: i.dv_reserve,
    })?;
    let sum_rad = |rads: &[RadiatorSpec]| -> f64 {
        rads.iter()
            .map(|r| {
                r.eps * i.sigma * r.t_k.powi(4) * r.area * (r.integrity_pct / 100.0).clamp(0.0, 1.0)
            })
            .sum()
    };
    let hot_reject_w = sum_rad(&i.rad_hot);
    let low_reject_w = sum_rad(&i.rad_low);
    let waste_heat_w = i.p_fusion * i.rad_load_frac;
    let laser_shots: Vec<LaserShotReport> = i
        .lasers
        .iter()
        .map(|l| {
            let beam = l.p_beam * l.t_pulse / 1e6;
            let electrical = if l.eta_wall > 0.0 {
                beam / l.eta_wall
            } else {
                beam
            };
            LaserShotReport {
                beam_mj: beam,
                electrical_mj: electrical,
                waste_mj: electrical - beam,
                shots_per_bank: if electrical > 0.0 {
                    i.flywheel_mj / electrical
                } else {
                    0.0
                },
                waste_w: if l.eta_wall > 0.0 {
                    l.p_beam * (1.0 / l.eta_wall - 1.0)
                } else {
                    0.0
                },
            }
        })
        .collect();
    let laser_waste_w: f64 = laser_shots.iter().map(|s| s.waste_w).sum();
    let endurance = |q_reject: f64| -> Option<f64> {
        let net = laser_waste_w - q_reject;
        if net > 0.0 && i.sink_mj > 0.0 {
            Some(i.sink_mj * 1e6 / net)
        } else {
            None
        }
    };
    let sink_endurance_s = endurance(low_reject_w);
    let sink_endurance_retracted_s = endurance(0.0);
    Ok(ReportOut {
        p_jet: g.p_jet,
        thrust_max: g.thrust,
        capped_at_ve_max: g.capped,
        ve_cap: g.ve_cap,
        accel_wet: g.thrust / i.m_wet,
        accel_dry: g.thrust / i.m_dry,
        dv_plasma: dv.dv,
        waste_heat_w,
        hot_reject_w,
        hot_margin_w: hot_reject_w - waste_heat_w,
        low_reject_w,
        laser_waste_w,
        sink_endurance_s,
        sink_endurance_retracted_s,
        laser_shots,
    })
}

// ---------------------------------------------------------------------------
// System map: bodies on circular rails, ships integrated under gravity plus
// active nav burns. Leapfrog (kick-drift-kick) with analytic body positions
// per substep — symplectic, so orbits don't decay over long ticks.
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone, JsonSchema)]
pub struct NavBody {
    pub id: String,
    pub mass_kg: f64,
    pub radius_m: f64,
    /// Circular orbit radius around the parent; 0 (or no parent) = fixed root.
    #[serde(default)]
    pub a_m: f64,
    #[serde(default)]
    pub phase0_deg: f64,
    #[serde(default)]
    pub parent: Option<String>,
}

#[derive(Deserialize, Clone, JsonSchema)]
pub struct NavBurnIn {
    pub thrust: f64,
    pub mdot: f64,
    pub t_remaining_s: f64,
    /// "prograde" | "retrograde" | "angle" | "body"
    pub mode: String,
    #[serde(default)]
    pub angle_deg: f64,
    #[serde(default)]
    pub target_body: Option<String>,
    /// Ignition delay from the start of this tick (maneuver nodes).
    #[serde(default)]
    pub t_start_s: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct NavShipIn {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub mass_kg: f64,
    /// Burn cuts off here (dry mass or dry + reserve).
    pub m_floor_kg: f64,
    #[serde(default)]
    pub landed_on: Option<String>,
    #[serde(default)]
    pub burn: Option<NavBurnIn>,
}

#[derive(Deserialize, JsonSchema)]
pub struct NavTickIn {
    pub g_const: f64,
    pub epoch_s: f64,
    /// Seconds to advance; 0 = just report positions.
    pub dt_s: f64,
    #[serde(default)]
    pub substep_s: Option<f64>,
    pub bodies: Vec<NavBody>,
    #[serde(default)]
    pub ships: Vec<NavShipIn>,
    /// Sample each ship's trajectory into a path of about this many points.
    #[serde(default)]
    pub path_points: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
pub struct NavBodyOut {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    /// [t_rel, x, y] samples over the tick (for frame-relative path drawing).
    pub path: Vec<[f64; 3]>,
}

#[derive(Serialize, JsonSchema)]
pub struct NavShipOut {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub mass_kg: f64,
    pub prop_used_kg: f64,
    pub dv_spent: f64,
    pub burn_t_remaining_s: f64,
    pub burn_t_start_remaining_s: f64,
    pub landed_on: Option<String>,
    pub notes: Vec<String>,
    /// [t_rel, x, y] samples.
    pub path: Vec<[f64; 3]>,
}

#[derive(Serialize, JsonSchema)]
pub struct NavTickOut {
    pub epoch_s: f64,
    pub bodies: Vec<NavBodyOut>,
    pub ships: Vec<NavShipOut>,
}

struct Eph {
    mu: f64,
    radius: f64,
    a: f64,
    n: f64, // mean motion, rad/s
    phase0: f64,
    parent: Option<usize>,
}

/// Resolve parents and mean motions. Bodies may appear in any order.
fn build_eph(g_const: f64, bodies: &[NavBody]) -> CalcResult<Vec<Eph>> {
    let idx_of = |id: &str| bodies.iter().position(|b| b.id == id);
    let mut eph = Vec::with_capacity(bodies.len());
    for b in bodies {
        let parent = match &b.parent {
            Some(p) if !p.is_empty() => {
                Some(idx_of(p).ok_or_else(|| format!("body {}: unknown parent {}", b.id, p))?)
            }
            _ => None,
        };
        let mu = g_const * b.mass_kg;
        let n = match parent {
            Some(pi) if b.a_m > 0.0 => {
                let mu_p = g_const * bodies[pi].mass_kg;
                (mu_p / b.a_m.powi(3)).sqrt()
            }
            _ => 0.0,
        };
        eph.push(Eph {
            mu,
            radius: b.radius_m,
            a: b.a_m,
            n,
            phase0: b.phase0_deg.to_radians(),
            parent,
        });
    }
    // Reject parent cycles (walk each chain with a depth cap).
    for (i, _) in eph.iter().enumerate() {
        let (mut cur, mut depth) = (i, 0);
        while let Some(p) = eph[cur].parent {
            cur = p;
            depth += 1;
            if depth > bodies.len() {
                return Err(format!("body {}: parent cycle", bodies[i].id));
            }
        }
    }
    Ok(eph)
}

/// Positions and velocities of every body at absolute time t.
fn body_states(eph: &[Eph], t: f64) -> Vec<[f64; 4]> {
    let mut out = vec![[f64::NAN; 4]; eph.len()];
    fn resolve(eph: &[Eph], out: &mut Vec<[f64; 4]>, i: usize, t: f64) -> [f64; 4] {
        if !out[i][0].is_nan() {
            return out[i];
        }
        let e = &eph[i];
        let base = match e.parent {
            Some(p) => resolve(eph, out, p, t),
            None => [0.0, 0.0, 0.0, 0.0],
        };
        let s = if e.a > 0.0 && e.parent.is_some() {
            let th = e.phase0 + e.n * t;
            let (sin, cos) = th.sin_cos();
            [
                base[0] + e.a * cos,
                base[1] + e.a * sin,
                base[2] - e.a * e.n * sin,
                base[3] + e.a * e.n * cos,
            ]
        } else {
            base
        };
        out[i] = s;
        s
    }
    for i in 0..eph.len() {
        resolve(eph, &mut out, i, t);
    }
    out
}

pub fn nav_tick(i: &NavTickIn) -> CalcResult<NavTickOut> {
    require_pos(i.g_const, "g_const")?;
    if i.dt_s < 0.0 {
        return Err("dt_s must be >= 0".into());
    }
    let eph = build_eph(i.g_const, &i.bodies)?;
    let idx_of = |id: &str| i.bodies.iter().position(|b| b.id == id);

    let t0 = i.epoch_s;
    let t1 = i.epoch_s + i.dt_s;
    let h_req = i.substep_s.unwrap_or(60.0).max(0.5);
    let steps = if i.dt_s > 0.0 {
        (i.dt_s / h_req).ceil() as usize
    } else {
        0
    };
    if steps > 3_000_000 {
        return Err(format!(
            "tick needs {} substeps — raise the substep or shorten the tick",
            steps
        ));
    }
    let h = if steps > 0 {
        i.dt_s / steps as f64
    } else {
        0.0
    };
    let path_every = steps.max(1) / i.path_points.unwrap_or(160).clamp(2, 2000) + 1;

    let mut ships_out = Vec::with_capacity(i.ships.len());
    for sh in &i.ships {
        let mut notes: Vec<String> = Vec::new();
        let m0 = sh.mass_kg;
        require_pos(m0, "mass_kg")?;

        // Landed and staying landed: ride the body.
        let landed_idx = sh.landed_on.as_deref().and_then(idx_of);
        if let Some(bi) = landed_idx {
            if sh.burn.is_none() {
                let b0 = body_states(&eph, t0)[bi];
                let b1 = body_states(&eph, t1)[bi];
                let off = [sh.x - b0[0], sh.y - b0[1]];
                ships_out.push(NavShipOut {
                    id: sh.id.clone(),
                    x: b1[0] + off[0],
                    y: b1[1] + off[1],
                    vx: b1[2],
                    vy: b1[3],
                    mass_kg: m0,
                    prop_used_kg: 0.0,
                    dv_spent: 0.0,
                    burn_t_remaining_s: 0.0,
                    burn_t_start_remaining_s: 0.0,
                    landed_on: sh.landed_on.clone(),
                    notes,
                    path: vec![],
                });
                continue;
            }
            notes.push(format!(
                "Lifting off from {}",
                sh.landed_on.clone().unwrap()
            ));
        }

        let (mut x, mut y) = (sh.x, sh.y);
        // A landed ship taking off starts with its body's velocity.
        let (mut vx, mut vy) = match landed_idx {
            Some(bi) => {
                let b0 = body_states(&eph, t0)[bi];
                (b0[2], b0[3])
            }
            None => (sh.vx, sh.vy),
        };
        let mut m = m0;
        let mut burn = sh.burn.clone();
        let burn_target = burn
            .as_ref()
            .and_then(|b| b.target_body.as_deref().and_then(idx_of));
        let mut dv_spent = 0.0f64;
        let mut landed: Option<usize> = None;
        let mut path: Vec<[f64; 3]> = vec![[0.0, x, y]];

        let thrust_acc = |burn: &Option<NavBurnIn>,
                          m: f64,
                          vx: f64,
                          vy: f64,
                          x: f64,
                          y: f64,
                          bstates: &[[f64; 4]]|
         -> [f64; 2] {
            let Some(b) = burn else { return [0.0, 0.0] };
            if b.t_remaining_s <= 0.0 || b.t_start_s > 0.0 {
                return [0.0, 0.0];
            }
            let a = b.thrust / m;
            let dir = match b.mode.as_str() {
                "prograde" | "retrograde" => {
                    let sp = (vx * vx + vy * vy).sqrt();
                    let sign = if b.mode == "retrograde" { -1.0 } else { 1.0 };
                    if sp < 1e-9 {
                        [sign, 0.0]
                    } else {
                        [sign * vx / sp, sign * vy / sp]
                    }
                }
                "body" => match burn_target {
                    Some(bi) => {
                        let d = [bstates[bi][0] - x, bstates[bi][1] - y];
                        let r = (d[0] * d[0] + d[1] * d[1]).sqrt().max(1.0);
                        [d[0] / r, d[1] / r]
                    }
                    None => [1.0, 0.0],
                },
                _ => {
                    let th = b.angle_deg.to_radians();
                    [th.cos(), th.sin()]
                }
            };
            [a * dir[0], a * dir[1]]
        };

        let gravity = |x: f64, y: f64, bstates: &[[f64; 4]]| -> [f64; 2] {
            let mut ax = 0.0;
            let mut ay = 0.0;
            for (bi, e) in eph.iter().enumerate() {
                if e.mu <= 0.0 {
                    continue;
                }
                let dx = x - bstates[bi][0];
                let dy = y - bstates[bi][1];
                let r2 = dx * dx + dy * dy;
                let r = r2.sqrt();
                if r < 1.0 {
                    continue;
                }
                let f = -e.mu / (r2 * r);
                ax += f * dx;
                ay += f * dy;
            }
            [ax, ay]
        };

        let mut bstates = body_states(&eph, t0);
        'steps: for k in 0..steps {
            // kick
            let ag = gravity(x, y, &bstates);
            let at = thrust_acc(&burn, m, vx, vy, x, y, &bstates);
            vx += (ag[0] + at[0]) * h / 2.0;
            vy += (ag[1] + at[1]) * h / 2.0;
            // drift
            x += vx * h;
            y += vy * h;
            // burn bookkeeping over this substep; the second kick uses the
            // pre-bookkeeping burn state so a burn ending exactly on a substep
            // boundary still delivers its full impulse
            let burn_pre = burn.clone();
            if let Some(b) = burn.as_mut() {
                if b.t_start_s > 0.0 {
                    // still coasting toward ignition
                    b.t_start_s = (b.t_start_s - h).max(0.0);
                    if b.t_start_s <= 0.0 {
                        notes.push("Ignition".into());
                    }
                } else if b.t_remaining_s > 0.0 {
                    let dt_burn = b.t_remaining_s.min(h);
                    dv_spent += b.thrust / m * dt_burn;
                    m -= b.mdot * dt_burn;
                    b.t_remaining_s -= dt_burn;
                    if b.t_remaining_s <= 0.0 {
                        notes.push("Burn complete".into());
                    }
                    if m <= sh.m_floor_kg {
                        m = sh.m_floor_kg;
                        b.t_remaining_s = 0.0;
                        notes.push("Burn cut off at the propellant floor".into());
                    }
                }
            }
            bstates = body_states(&eph, t0 + (k + 1) as f64 * h);
            // kick
            let ag2 = gravity(x, y, &bstates);
            let at2 = thrust_acc(&burn_pre, m, vx, vy, x, y, &bstates);
            vx += (ag2[0] + at2[0]) * h / 2.0;
            vy += (ag2[1] + at2[1]) * h / 2.0;

            // Contact check: inside a body and moving toward it → land there.
            for (bi, e) in eph.iter().enumerate() {
                if e.radius <= 0.0 {
                    continue;
                }
                let dx = x - bstates[bi][0];
                let dy = y - bstates[bi][1];
                let r = (dx * dx + dy * dy).sqrt();
                if r < e.radius {
                    let vrx = vx - bstates[bi][2];
                    let vry = vy - bstates[bi][3];
                    let v_rel = (vrx * vrx + vry * vry).sqrt();
                    let scale = if r > 1.0 { e.radius / r } else { 1.0 };
                    x = bstates[bi][0] + dx * scale;
                    y = bstates[bi][1] + dy * scale;
                    vx = bstates[bi][2];
                    vy = bstates[bi][3];
                    landed = Some(bi);
                    notes.push(if v_rel > 200.0 {
                        format!(
                            "IMPACT on {} at {:.1} km/s relative — that was not a landing",
                            i.bodies[bi].id,
                            v_rel / 1000.0
                        )
                    } else {
                        format!(
                            "Landed on {} (contact at {:.1} m/s)",
                            i.bodies[bi].id, v_rel
                        )
                    });
                    if let Some(b) = burn.as_mut() {
                        b.t_remaining_s = 0.0;
                    }
                    path.push([(k + 1) as f64 * h, x, y]);
                    break 'steps;
                }
            }
            if (k + 1) % path_every == 0 {
                path.push([(k + 1) as f64 * h, x, y]);
            }
        }
        if landed.is_none() {
            path.push([i.dt_s, x, y]);
        }

        let (b_rem, b_start_rem) = burn
            .map(|b| (b.t_remaining_s.max(0.0), b.t_start_s.max(0.0)))
            .unwrap_or((0.0, 0.0));
        ships_out.push(NavShipOut {
            id: sh.id.clone(),
            x,
            y,
            vx,
            vy,
            mass_kg: m,
            prop_used_kg: m0 - m,
            dv_spent,
            burn_t_remaining_s: b_rem,
            burn_t_start_remaining_s: b_start_rem,
            landed_on: landed.map(|bi| i.bodies[bi].id.clone()),
            notes,
            path,
        });
    }

    // Body paths on the same sample cadence, so ship paths can be re-expressed
    // relative to any body (local reference frames).
    let mut sample_ts: Vec<f64> = vec![0.0];
    for k in 0..steps {
        if (k + 1) % path_every == 0 {
            sample_ts.push((k + 1) as f64 * h);
        }
    }
    if *sample_ts.last().unwrap() < i.dt_s {
        sample_ts.push(i.dt_s);
    }
    let mut body_paths: Vec<Vec<[f64; 3]>> = vec![Vec::with_capacity(sample_ts.len()); eph.len()];
    for t in &sample_ts {
        let bs = body_states(&eph, t0 + t);
        for (bi, s) in bs.iter().enumerate() {
            body_paths[bi].push([*t, s[0], s[1]]);
        }
    }

    let bodies = body_states(&eph, t1)
        .into_iter()
        .zip(&i.bodies)
        .zip(body_paths)
        .map(|((s, b), path)| NavBodyOut {
            id: b.id.clone(),
            x: s[0],
            y: s[1],
            vx: s[2],
            vy: s[3],
            path,
        })
        .collect();

    Ok(NavTickOut {
        epoch_s: t1,
        bodies,
        ships: ships_out,
    })
}

// ---------------------------------------------------------------------------
// Circular-orbit helper for placing ships: v_circ, v_esc, period at radius r.
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct OrbitVIn {
    pub g_const: f64,
    pub mass_kg: f64,
    pub r_m: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct OrbitVOut {
    pub v_circ: f64,
    pub v_esc: f64,
    pub period_s: f64,
}

pub fn orbit_v(i: &OrbitVIn) -> CalcResult<OrbitVOut> {
    require_pos(i.g_const, "g_const")?;
    require_pos(i.mass_kg, "mass_kg")?;
    require_pos(i.r_m, "r_m")?;
    let mu = i.g_const * i.mass_kg;
    Ok(OrbitVOut {
        v_circ: (mu / i.r_m).sqrt(),
        v_esc: (2.0 * mu / i.r_m).sqrt(),
        period_s: 2.0 * std::f64::consts::PI * (i.r_m.powi(3) / mu).sqrt(),
    })
}

// ---------------------------------------------------------------------------
// Burn time for a requested Δv: inverse rocket equation at constant thrust.
//   t = (m0/mdot)·(1 − exp(−Δv/ve_eff)),  ve_eff = F/mdot
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct BurnForDvIn {
    pub thrust: f64,
    pub mdot: f64,
    pub m0: f64,
    pub m_floor: f64,
    pub dv: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct BurnForDvOut {
    pub t_burn_s: f64,
    pub prop_kg: f64,
    /// Δv actually delivered (clamped at the propellant floor).
    pub dv_delivered: f64,
    pub dv_possible: f64,
    pub clamped: bool,
}

pub fn burn_for_dv(i: &BurnForDvIn) -> CalcResult<BurnForDvOut> {
    require_pos(i.thrust, "thrust")?;
    require_pos(i.mdot, "mdot")?;
    require_pos(i.m0, "m0")?;
    require_pos(i.dv, "dv")?;
    if i.m_floor >= i.m0 {
        return Err("no burnable propellant above the floor".into());
    }
    let ve = i.thrust / i.mdot;
    let dv_possible = ve * (i.m0 / i.m_floor).ln();
    let clamped = i.dv > dv_possible;
    let dv_eff = i.dv.min(dv_possible);
    let t = i.m0 / i.mdot * (1.0 - (-dv_eff / ve).exp());
    Ok(BurnForDvOut {
        t_burn_s: t,
        prop_kg: i.mdot * t,
        dv_delivered: dv_eff,
        dv_possible,
        clamped,
    })
}

// ---------------------------------------------------------------------------
// Intercept planner: "get me there spending no more than this much Δv."
// Impulsive-burn search over departure time × heading × Δv against the target's
// future position, integrated under gravity from every body, then refined and
// converted to a finite burn for the nav machinery. Ships burn for minutes and
// coast for weeks, so the impulsive approximation is honest here.
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
pub struct InterceptTargetShip {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct NavInterceptIn {
    pub g_const: f64,
    /// Current sim clock — body rails are evaluated from here, not from T+0.
    #[serde(default)]
    pub epoch_s: f64,
    pub bodies: Vec<NavBody>,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub mass_kg: f64,
    pub m_floor_kg: f64,
    /// Thrust and flow at the chosen gear, for the finite-burn conversion.
    pub thrust: f64,
    pub mdot: f64,
    #[serde(default)]
    pub target_body: Option<String>,
    #[serde(default)]
    pub target_ship: Option<InterceptTargetShip>,
    pub dv_max: f64,
    pub depart_max_s: f64,
    pub horizon_s: f64,
    pub capture_radius_m: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct NavInterceptOut {
    pub feasible: bool,
    pub t_depart_s: f64,
    pub heading_deg: f64,
    pub dv: f64,
    pub t_arrival_s: Option<f64>,
    pub transit_s: Option<f64>,
    pub v_rel_arrival: Option<f64>,
    /// Closest approach of the best candidate (= capture distance on a hit).
    pub closest_m: f64,
    pub t_burn_s: f64,
    pub t_start_s: f64,
    pub prop_kg: f64,
    pub dv_possible: f64,
    /// Best trajectory, [t, x, y].
    pub path: Vec<[f64; 3]>,
}

pub fn nav_intercept(i: &NavInterceptIn) -> CalcResult<NavInterceptOut> {
    require_pos(i.g_const, "g_const")?;
    require_pos(i.dv_max, "dv_max")?;
    require_pos(i.horizon_s, "horizon_s")?;
    require_pos(i.capture_radius_m, "capture_radius_m")?;
    require_pos(i.thrust, "thrust")?;
    require_pos(i.mdot, "mdot")?;
    if i.depart_max_s < 0.0 || i.depart_max_s >= i.horizon_s {
        return Err("departure window must sit inside the horizon".into());
    }
    let eph = build_eph(i.g_const, &i.bodies)?;

    let h = (i.horizon_s / 1500.0).clamp(60.0, 21600.0);
    let steps = (i.horizon_s / h).ceil() as usize;
    if steps > 30_000 {
        return Err("horizon too long for the intercept search — shorten it".into());
    }

    // Precompute one shared timeline of body states.
    let timeline: Vec<Vec<[f64; 4]>> = (0..=steps)
        .map(|k| body_states(&eph, i.epoch_s + k as f64 * h))
        .collect();

    let gravity = |x: f64, y: f64, bs: &[[f64; 4]]| -> [f64; 2] {
        let mut ax = 0.0;
        let mut ay = 0.0;
        for (bi, e) in eph.iter().enumerate() {
            if e.mu <= 0.0 {
                continue;
            }
            let dx = x - bs[bi][0];
            let dy = y - bs[bi][1];
            let r2 = dx * dx + dy * dy;
            let r = r2.sqrt();
            if r < 1.0 {
                continue;
            }
            let f = -e.mu / (r2 * r);
            ax += f * dx;
            ay += f * dy;
        }
        [ax, ay]
    };
    let step_state = |s: &mut [f64; 4], k: usize| {
        let a1 = gravity(s[0], s[1], &timeline[k]);
        s[2] += a1[0] * h / 2.0;
        s[3] += a1[1] * h / 2.0;
        s[0] += s[2] * h;
        s[1] += s[3] * h;
        let a2 = gravity(s[0], s[1], &timeline[k + 1]);
        s[2] += a2[0] * h / 2.0;
        s[3] += a2[1] * h / 2.0;
    };

    // Target positions per step: a body on rails, or a ship on a ballistic arc.
    let target_pos: Vec<[f64; 2]> = match (&i.target_body, &i.target_ship) {
        (Some(id), _) => {
            let bi = i
                .bodies
                .iter()
                .position(|b| &b.id == id)
                .ok_or_else(|| format!("unknown target body {}", id))?;
            timeline.iter().map(|bs| [bs[bi][0], bs[bi][1]]).collect()
        }
        (None, Some(ts)) => {
            let mut s = [ts.x, ts.y, ts.vx, ts.vy];
            let mut out = Vec::with_capacity(steps + 1);
            out.push([s[0], s[1]]);
            for k in 0..steps {
                step_state(&mut s, k);
                out.push([s[0], s[1]]);
            }
            out
        }
        _ => return Err("give either target_body or target_ship".into()),
    };

    let d0 = ((i.x - target_pos[0][0]).powi(2) + (i.y - target_pos[0][1]).powi(2)).sqrt();
    if d0 <= i.capture_radius_m {
        return Err("already inside the capture radius of the target".into());
    }

    // Our own ballistic coast: candidate burns depart from points along it.
    let mut coast: Vec<[f64; 4]> = Vec::with_capacity(steps + 1);
    let mut s = [i.x, i.y, i.vx, i.vy];
    coast.push(s);
    for k in 0..steps {
        step_state(&mut s, k);
        coast.push(s);
    }

    let ve = i.thrust / i.mdot;
    let dv_possible = ve * (i.mass_kg / i.m_floor_kg).ln();
    let dv_hi = i.dv_max.min(dv_possible * 0.98);
    if dv_hi <= 0.0 {
        return Err("no Δv available above the propellant floor".into());
    }

    // score: hit → arrival time (earlier wins); miss → huge + closest approach.
    let simulate = |kd: usize, theta: f64, dv: f64| -> (f64, f64, Option<(usize, f64)>) {
        let mut st = coast[kd];
        st[2] += dv * theta.cos();
        st[3] += dv * theta.sin();
        let mut closest = f64::INFINITY;
        for k in kd..steps {
            step_state(&mut st, k);
            let dx = st[0] - target_pos[k + 1][0];
            let dy = st[1] - target_pos[k + 1][1];
            let d = (dx * dx + dy * dy).sqrt();
            if d < closest {
                closest = d;
            }
            if d <= i.capture_radius_m {
                let tp = &timeline[k + 1];
                // relative speed vs the target at contact
                let (tvx, tvy) = match &i.target_body {
                    Some(id) => {
                        let bi = i.bodies.iter().position(|b| &b.id == id).unwrap();
                        (tp[bi][2], tp[bi][3])
                    }
                    None => {
                        // finite-difference the ship target's path
                        let p0 = target_pos[k];
                        let p1 = target_pos[k + 1];
                        ((p1[0] - p0[0]) / h, (p1[1] - p0[1]) / h)
                    }
                };
                let vrel = ((st[2] - tvx).powi(2) + (st[3] - tvy).powi(2)).sqrt();
                return ((k + 1) as f64 * h, closest, Some((k + 1, vrel)));
            }
        }
        (1e18 + closest, closest, None)
    };

    let kd_max = ((i.depart_max_s / h) as usize)
        .min(steps.saturating_sub(2))
        .max(0);
    let mut best = (
        f64::INFINITY,
        0usize,
        0.0f64,
        dv_hi,
        f64::INFINITY,
        None::<(usize, f64)>,
    );
    let n_kd = 16usize;
    let n_th = 24usize;
    let dv_grid = [0.25, 0.5, 0.75, 1.0];
    for a in 0..=n_kd {
        let kd = kd_max * a / n_kd;
        for b in 0..n_th {
            let theta = 2.0 * std::f64::consts::PI * b as f64 / n_th as f64;
            for f in dv_grid {
                let dv = dv_hi * f;
                let (score, closest, hit) = simulate(kd, theta, dv);
                if score < best.0 {
                    best = (score, kd, theta, dv, closest, hit);
                }
            }
        }
    }

    // Coordinate refinement with shrinking deltas.
    let mut dkd = (kd_max / 16).max(1) as isize;
    let mut dth = 2.0 * std::f64::consts::PI / n_th as f64;
    let mut ddv = dv_hi / 8.0;
    for _ in 0..8 {
        let mut improved = true;
        while improved {
            improved = false;
            let (_, kd, th, dv, _, _) = best;
            let cands = [
                (kd as isize - dkd, th, dv),
                (kd as isize + dkd, th, dv),
                (kd as isize, th - dth, dv),
                (kd as isize, th + dth, dv),
                (kd as isize, th, (dv - ddv).max(ddv * 0.1)),
                (kd as isize, th, (dv + ddv).min(dv_hi)),
            ];
            for (ck, cth, cdv) in cands {
                if ck < 0 || ck as usize > kd_max {
                    continue;
                }
                let (score, closest, hit) = simulate(ck as usize, cth, cdv);
                if score < best.0 {
                    best = (score, ck as usize, cth, cdv, closest, hit);
                    improved = true;
                }
            }
        }
        dkd = (dkd / 2).max(1);
        dth /= 2.0;
        ddv /= 2.0;
    }

    let (_, kd, theta, dv, closest, hit) = best;
    // Re-run the winner to record its path.
    let mut path: Vec<[f64; 3]> = Vec::with_capacity(320);
    {
        let every = (steps / 280).max(1);
        for (k, point) in coast.iter().enumerate().take(kd) {
            if k % every == 0 {
                path.push([k as f64 * h, point[0], point[1]]);
            }
        }
        let mut st = coast[kd];
        st[2] += dv * theta.cos();
        st[3] += dv * theta.sin();
        path.push([kd as f64 * h, st[0], st[1]]);
        let k_end = hit.map(|(k, _)| k).unwrap_or(steps);
        for k in kd..k_end {
            step_state(&mut st, k);
            if (k + 1) % every == 0 || k + 1 == k_end {
                path.push([(k + 1) as f64 * h, st[0], st[1]]);
            }
        }
    }

    let conv = burn_for_dv(&BurnForDvIn {
        thrust: i.thrust,
        mdot: i.mdot,
        m0: i.mass_kg,
        m_floor: i.m_floor_kg,
        dv,
    })?;
    let t_depart = kd as f64 * h;
    Ok(NavInterceptOut {
        feasible: hit.is_some(),
        t_depart_s: t_depart,
        heading_deg: theta.to_degrees().rem_euclid(360.0),
        dv,
        t_arrival_s: hit.map(|(k, _)| k as f64 * h),
        transit_s: hit.map(|(k, _)| (k - kd) as f64 * h),
        v_rel_arrival: hit.map(|(_, v)| v),
        closest_m: closest,
        t_burn_s: conv.t_burn_s,
        t_start_s: (t_depart - conv.t_burn_s / 2.0).max(0.0),
        prop_kg: conv.prop_kg,
        dv_possible,
        path,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol * b.abs()
    }

    fn asmodeus_stage() -> MissileStageIn {
        MissileStageIn {
            id: "main".into(),
            name: "Main stage".into(),
            dry_mass_kg: 100.0,
            propellant_kg: 1900.0,
            ve: 16700.0,
            a0_g: 10.0,
            jettison: false,
        }
    }

    fn asmodeus_intercept(range: f64, v_close0: f64, phases: Vec<PhaseIn>) -> InterceptIn {
        InterceptIn {
            range,
            v_close0,
            payload_kg: 0.0,
            stages: vec![asmodeus_stage()],
            g: 9.81,
            phases,
        }
    }

    /// Canon consistency check: 1.82 MW fusion at Ve = 2,300 km/s
    /// -> 1.013 N thrust, 0.44 mg/s flow, within 2%.
    #[test]
    fn gear_matches_canon_budget() {
        let g = gear(&GearIn {
            p_fusion: 1.82e6,
            f_exh: 0.753,
            eta_noz: 0.85,
            e_afterburner: 0.0,
            ve: 2.3e6,
            ve_max: 2.3e6,
            f_cap: None,
            mass_kg: None,
            duration_s: None,
        })
        .unwrap();
        assert!(close(g.thrust, 1.013, 0.02), "thrust {} N", g.thrust);
        assert!(close(g.mdot, 0.44e-6, 0.02), "mdot {} kg/s", g.mdot);
        // Pure plasma at Ve_max: no afterburner flow.
        assert!(g.mdot_afterburner < 1e-12);
    }

    #[test]
    fn gear_afterburner_splits_flow() {
        let g = gear(&GearIn {
            p_fusion: 16e12,
            f_exh: 0.753,
            eta_noz: 0.85,
            e_afterburner: 0.0,
            ve: 500e3,
            ve_max: 2.3e6,
            f_cap: None,
            mass_kg: None,
            duration_s: None,
        })
        .unwrap();
        // mdot_fuel is the constant pure-plasma flow.
        let p_jet = 16e12 * 0.753 * 0.85;
        assert!(close(g.mdot_fuel, 2.0 * p_jet / (2.3e6f64 * 2.3e6), 1e-9));
        assert!(close(g.mdot, g.mdot_fuel + g.mdot_afterburner, 1e-12));
        assert!(g.mdot_afterburner > 0.0);
    }

    #[test]
    fn gear_nozzle_cap_binds() {
        let g = gear(&GearIn {
            p_fusion: 16e12,
            f_exh: 0.753,
            eta_noz: 0.85,
            e_afterburner: 0.0,
            ve: 50e3,
            ve_max: 2.3e6,
            f_cap: Some(1.0e8),
            mass_kg: None,
            duration_s: None,
        })
        .unwrap();
        assert!(g.capped);
        assert_eq!(g.thrust, 1.0e8);
        // Throttled reactor: fuel flow below the pure-plasma max.
        let full_fuel = 2.0 * g.p_jet / (2.3e6f64 * 2.3e6);
        assert!(g.mdot_fuel < full_fuel);
    }

    /// MH-164 Asmodeus canon: 2 t wet, MR 20, Ve 16.7 km/s -> 50 km/s dv.
    #[test]
    fn asmodeus_matches_canon() {
        let m = missile(&MissileIn {
            payload_kg: 0.0,
            stages: vec![asmodeus_stage()],
            g: 9.81,
        })
        .unwrap();
        assert!(close(m.dv, 50000.0, 0.002), "dv {}", m.dv);
        assert!(close(m.a_burnout_g, 200.0, 1e-9));
    }

    #[test]
    fn mixed_propulsion_staging_accounts_for_jettison() {
        let out = missile(&MissileIn {
            payload_kg: 100.0,
            g: 9.81,
            stages: vec![
                MissileStageIn {
                    id: "booster".into(),
                    name: "MH booster".into(),
                    dry_mass_kg: 100.0,
                    propellant_kg: 800.0,
                    ve: 10_000.0,
                    a0_g: 10.0,
                    jettison: true,
                },
                MissileStageIn {
                    id: "sustainer".into(),
                    name: "Fusion sustainer".into(),
                    dry_mass_kg: 50.0,
                    propellant_kg: 450.0,
                    ve: 30_000.0,
                    a0_g: 5.0,
                    jettison: false,
                },
            ],
        })
        .unwrap();
        let expected = 10_000.0 * (1500.0f64 / 700.0).ln() + 30_000.0 * (600.0f64 / 150.0).ln();
        assert!(close(out.dv, expected, 1e-12));
        assert!(close(
            out.stage_reports[0].post_jettison_mass_kg,
            600.0,
            1e-12
        ));
        assert!(out
            .profile
            .iter()
            .any(|p| p.event.as_deref() == Some("jettison")));
    }

    fn optimizer_baseline() -> MissileOptimizeIn {
        MissileOptimizeIn {
            total_mass_kg: 27_000.0,
            a0_g: 0.5,
            reactor_specific_power_mw_kg: 1.1,
            radiator_specific_power_mw_kg: 1.6,
            waste_heat_fraction: 0.05,
            mh_ve: 16_700.0,
            h2_cooling_j_kg: 7e6,
            mh_cooling_j_kg: 0.4e6,
            n_submunitions: 10,
            submunition_dv: 50_000.0,
            tank_fraction: 0.04,
            guidance_mass_kg: 250.0,
            reference_sub_dry_kg: 50.0,
            g: 9.81,
        }
    }

    #[test]
    fn missile_optimizer_improves_on_pure_mh_for_baseline() {
        let out = optimize_missile(&optimizer_baseline()).unwrap();
        let fusion = out.reference_fusion.unwrap();
        let pure = out.reference_pure_mh.unwrap();
        assert!(fusion.dv > pure.dv, "fusion {} pure {}", fusion.dv, pure.dv);
        assert!(fusion.ve > 16_700.0);
        assert!(fusion.reactor_mass_kg > 0.0);
        assert!(fusion.radiator_mass_kg >= 0.0);
        assert!(out.sweep.len() >= 20);
    }

    #[test]
    fn optimized_bus_maps_exactly_to_missile_calculator_schema() {
        let input = optimizer_baseline();
        let optimized = optimize_missile(&input).unwrap().reference_fusion.unwrap();
        let calculated = missile(&MissileIn {
            payload_kg: optimized.submunitions_wet_kg,
            stages: vec![MissileStageIn {
                id: "bus".into(),
                name: "Optimized bus".into(),
                dry_mass_kg: optimized.bus_dry_mass_kg,
                propellant_kg: optimized.propellant_kg,
                ve: optimized.ve,
                a0_g: input.a0_g,
                jettison: false,
            }],
            g: input.g,
        })
        .unwrap();
        assert!(close(calculated.m_wet, input.total_mass_kg, 1e-12));
        assert!(close(calculated.dv, optimized.dv, 1e-12));
        assert!(close(calculated.t_burn, optimized.burn_time_s, 1e-12));
    }

    #[test]
    fn stage_aware_intercept_burns_in_order_across_a_coast() {
        let stages = vec![
            MissileStageIn {
                id: "boost".into(),
                name: "Boost".into(),
                dry_mass_kg: 50.0,
                propellant_kg: 400.0,
                ve: 12_000.0,
                a0_g: 10.0,
                jettison: true,
            },
            MissileStageIn {
                id: "terminal".into(),
                name: "Terminal".into(),
                dry_mass_kg: 50.0,
                propellant_kg: 400.0,
                ve: 30_000.0,
                a0_g: 8.0,
                jettison: false,
            },
        ];
        let out = intercept(&InterceptIn {
            range: 1e9,
            v_close0: 2e3,
            payload_kg: 100.0,
            stages,
            g: 9.81,
            phases: vec![
                PhaseIn {
                    stage_id: "boost".into(),
                    prop_frac: 1.0,
                    coast_to_range: Some(1e8),
                },
                PhaseIn {
                    stage_id: "terminal".into(),
                    prop_frac: 1.0,
                    coast_to_range: None,
                },
            ],
        })
        .unwrap();
        assert!(out.hit, "{:?}", out.miss_reason);
        let kinds: Vec<&str> = out.timeline.iter().map(|p| p.kind.as_str()).collect();
        assert_eq!(
            kinds,
            vec!["burn", "jettison", "coast", "burn", "terminal coast"]
        );
        assert_eq!(out.timeline[0].stage_id.as_deref(), Some("boost"));
        assert_eq!(out.timeline[3].stage_id.as_deref(), Some("terminal"));
    }

    #[test]
    fn intercept_rejects_stage_skipping_and_overallocation() {
        let stages = vec![
            asmodeus_stage(),
            MissileStageIn {
                id: "second".into(),
                name: "Second".into(),
                dry_mass_kg: 10.0,
                propellant_kg: 10.0,
                ve: 20_000.0,
                a0_g: 5.0,
                jettison: false,
            },
        ];
        let skipped = intercept(&InterceptIn {
            range: 1e8,
            v_close0: 0.0,
            payload_kg: 0.0,
            stages: stages.clone(),
            g: 9.81,
            phases: vec![PhaseIn {
                stage_id: "second".into(),
                prop_frac: 1.0,
                coast_to_range: None,
            }],
        });
        assert!(skipped.err().unwrap().contains("cannot ignite"));
        let over = intercept(&InterceptIn {
            range: 1e8,
            v_close0: 0.0,
            payload_kg: 0.0,
            stages,
            g: 9.81,
            phases: vec![
                PhaseIn {
                    stage_id: "main".into(),
                    prop_frac: 0.6,
                    coast_to_range: None,
                },
                PhaseIn {
                    stage_id: "main".into(),
                    prop_frac: 0.5,
                    coast_to_range: None,
                },
            ],
        });
        assert!(over.err().unwrap().contains("more than 100%"));
    }

    #[test]
    fn burn_x_small_t_is_half_a_t_squared() {
        // ve=3000, mdot=1, m0=1e6 -> a0 = 3e-3 m/s^2, tau = 1e6 s
        let x = burn_x(3000.0, 0.0, 1e6, 10.0);
        let expect = 0.5 * (3000.0 * 1.0 / 1e6) * 100.0;
        assert!(close(x, expect, 1e-3), "x {} vs {}", x, expect);
    }

    /// Analytic travel solution must agree with brute-force integration.
    #[test]
    fn travel_matches_numeric_integration() {
        let ve = 2.3e6;
        let thrust = 8.9e6;
        let mdot = thrust / ve;
        let m0 = 7.2e8;
        let m_dry = 9.0e7;
        let t = travel(&TravelIn {
            distance: 1.5e11, // 1 AU
            ve,
            thrust,
            mdot,
            m0,
            m_dry,
            dv_reserve: 0.0,
        })
        .unwrap();
        assert!(t.feasible);
        // Integrate the same flip-and-burn numerically.
        let dt = t.t_total / 2_000_000.0;
        let (mut x, mut v, mut m) = (0.0f64, 0.0f64, m0);
        let mut time = 0.0;
        while time < t.t_total {
            let a = if time < t.t_flip {
                thrust / m
            } else {
                -thrust / m
            };
            v += a * dt;
            x += v * dt;
            m -= mdot * dt;
            time += dt;
        }
        assert!(close(x, 1.5e11, 1e-3), "numeric x {} vs 1.5e11", x);
        assert!(v.abs() < t.peak_v * 1e-2, "arrival v {}", v);
        // Flip constraint honored.
        let m_flip = m0 - mdot * t.t_flip;
        assert!(m_flip >= (m0 * t.m_floor).sqrt() * 0.999);
    }

    #[test]
    fn travel_reports_max_distance_when_infeasible() {
        let ve = 2.3e6;
        let thrust = 8.9e6;
        let mdot = thrust / ve;
        let t = travel(&TravelIn {
            distance: 1e30,
            ve,
            thrust,
            mdot,
            m0: 7.2e8,
            m_dry: 9.0e7,
            dv_reserve: 50e3,
        })
        .unwrap();
        assert!(!t.feasible);
        assert!(t.max_distance > 0.0 && t.max_distance < 1e30);
        assert!(t.m_arrival >= t.m_floor * 0.999);
    }

    #[test]
    fn laser_r_max_consistent_with_curve() {
        let out = laser(&LaserIn {
            p_beam: 3e8,
            aperture: 4.0,
            lambda: 4.5e-7,
            eta_drill: 0.5,
            cutoff_mm_s: 0.5,
            materials: vec![LaserMaterial {
                name: "Steel".into(),
                rho: 7850.0,
                e_vap_mj: 8.2,
            }],
            eta_wall: Some(0.5),
            t_pulse: Some(5.0),
            n: None,
            flywheel_mj: Some(1_800_000.0),
            sink_mj: Some(2_300_000.0),
            q_low_w: None,
        })
        .unwrap();
        let m = &out.materials[0];
        // At exactly R_max the rate equals the cutoff.
        let d = 1.22 * 4.5e-7 * m.r_max / 4.0;
        let flux = 4.0 * 3e8 / (std::f64::consts::PI * d * d);
        let rate = 1000.0 * 0.5 * flux / (7850.0 * 8.2e6);
        assert!(close(rate, 0.5, 1e-9), "rate at r_max {}", rate);
        let shot = out.shot.unwrap();
        assert!(close(shot.beam_mj, 1500.0, 1e-9));
        assert!(close(shot.electrical_mj, 3000.0, 1e-9));
        assert!(close(shot.waste_mj, 1500.0, 1e-9));
    }

    #[test]
    fn vent_scars_capacity() {
        let v = vent(&VentIn {
            heat_mj: 1960.0,
            vent_mj_per_kg: 19.6,
            sink_mj_per_kg: 4.6,
        })
        .unwrap();
        assert!(close(v.li_kg, 100.0, 1e-9));
        assert!(close(v.capacity_lost_mj, 460.0, 1e-9));
    }

    #[test]
    fn intercept_coast_hit() {
        // Long range: burnout then coast.
        let out = intercept(&asmodeus_intercept(1e8, 10e3, vec![])).unwrap();
        assert!(out.hit);
        assert_eq!(out.phase, "coast");
        let expect = out.t_burn + (1e8 - out.x_burnout) / out.v_burnout;
        assert!(close(out.t_hit.unwrap(), expect, 1e-9));
        assert!(close(out.v_terminal.unwrap(), 10e3 + 50000.0, 0.002));
    }

    #[test]
    fn intercept_flags_miss_when_target_outruns() {
        let out = intercept(&asmodeus_intercept(1e8, -60e3, vec![])).unwrap();
        assert!(!out.hit);
        assert_eq!(out.phase, "miss");
        assert!(out.miss_reason.is_some());
    }

    #[test]
    fn intercept_powered_hit_at_short_range() {
        let out = intercept(&asmodeus_intercept(100e3, 5e3, vec![])).unwrap();
        assert!(out.hit);
        assert_eq!(out.phase, "powered");
        assert!(out.t_hit.unwrap() < out.t_burn);
        assert!(out.dv_spent.unwrap() < out.dv_total);
    }

    /// Missiles.md standard profile: boost → dark coast → terminal burn.
    /// With all propellant spent before impact, v_terminal = v0 + full Δv.
    #[test]
    fn intercept_three_phase_profile() {
        let out = intercept(&asmodeus_intercept(
            1e11,
            20e3,
            vec![
                PhaseIn {
                    stage_id: "main".into(),
                    prop_frac: 0.35,
                    coast_to_range: Some(25e9),
                }, // terminal at 25 Gm
                PhaseIn {
                    stage_id: "main".into(),
                    prop_frac: 0.65,
                    coast_to_range: None,
                },
            ],
        ))
        .unwrap();
        assert!(out.hit, "{:?}", out.miss_reason);
        // burn, coast, burn, terminal coast
        let kinds: Vec<&str> = out.timeline.iter().map(|p| p.kind.as_str()).collect();
        assert_eq!(kinds, vec!["burn", "coast", "burn", "terminal coast"]);
        // Full load spent → terminal velocity is v0 + total Δv.
        assert!(close(out.v_terminal.unwrap(), 20e3 + out.dv_total, 1e-6));
        assert!(close(out.dv_spent.unwrap(), out.dv_total, 1e-6));
        // Boost Δv for 35% of propellant burned first (mass still high).
        assert!(close(
            out.timeline[0].dv,
            16700.0 * (2000.0f64 / 1335.0).ln(),
            1e-6
        ));
        // Terminal burn starts at 25 Gm to go.
        assert!(close(out.timeline[1].x1, 1e11 - 25e9, 1e-9));
    }

    #[test]
    fn intercept_stalls_in_coast() {
        let out = intercept(&asmodeus_intercept(
            1e10,
            -5e3,
            vec![
                PhaseIn {
                    stage_id: "main".into(),
                    prop_frac: 0.10,
                    coast_to_range: Some(1e9),
                },
                PhaseIn {
                    stage_id: "main".into(),
                    prop_frac: 0.90,
                    coast_to_range: None,
                },
            ],
        ))
        .unwrap();
        assert!(!out.hit);
        assert!(out.miss_reason.unwrap().contains("stalled"));
    }

    /// Lasers.md installed-systems table: BB main battery, 30 m lens, 10 GW,
    /// 200 nm, 0.01 s pulse, 1 cm of Ti-C hybrid → ~25 Mm kill range.
    #[test]
    fn laser_profiles_reproduce_canon_kill_table() {
        let out = laser_profiles(&LaserProfilesIn {
            p_beam: 1e10,
            aperture: 30.0,
            lambda: 2e-7,
            eta_drill: 0.5,
            open_fire_factor: 1.5,
            profiles: vec![LaserProfileIn {
                name: "missile kill".into(),
                rho: 4000.0,
                e_vap_mj: 35.0,
                t_pulse_s: 0.01,
                threshold_mm: 10.0,
            }],
            n: None,
        })
        .unwrap();
        let p = &out.profiles[0];
        assert!(
            close(p.r_kill, 25e6, 0.10),
            "BB main kill range {} m",
            p.r_kill
        );
        assert!(close(p.r_open, p.r_kill * 1.5, 1e-12));
        // Self-consistency: penetration per pulse at r_kill equals the threshold.
        let d = 1.22 * 2e-7 * p.r_kill / 30.0;
        let flux = 4.0 * 1e10 / (std::f64::consts::PI * d * d);
        let pen = 0.5 * flux / (4000.0 * 35e6) * 0.01 * 1000.0;
        assert!(close(pen, 10.0, 1e-9), "pen at r_kill {}", pen);
    }

    #[test]
    fn laser_profile_reports_spot_irradiance_and_fluence() {
        let out = laser_profiles(&LaserProfilesIn {
            p_beam: 2e9,
            aperture: 5.0,
            lambda: 2e-7,
            eta_drill: 0.5,
            open_fire_factor: 1.5,
            n: Some(20),
            profiles: vec![LaserProfileIn {
                name: "test".into(),
                rho: 4000.0,
                e_vap_mj: 35.0,
                t_pulse_s: 0.25,
                threshold_mm: 10.0,
            }],
        })
        .unwrap();
        assert_eq!(out.beam_power_w, 2e9);
        let a = 3usize;
        let b = 12usize;
        assert!(close(
            out.spot_diameter_m[b] / out.spot_diameter_m[a],
            out.range_m[b] / out.range_m[a],
            1e-12
        ));
        let rr = out.range_m[b] / out.range_m[a];
        assert!(close(
            out.irradiance_w_m2[a] / out.irradiance_w_m2[b],
            rr * rr,
            1e-12
        ));
        assert!(close(
            out.profiles[0].fluence_j_m2[a],
            out.irradiance_w_m2[a] * 0.25,
            1e-12
        ));
        assert_eq!(out.profiles[0].pulse_energy_j, 5e8);
    }

    #[test]
    fn default_fleet_uses_staged_schema_and_keeps_magazine_links_valid() {
        let doc: serde_json::Value =
            serde_json::from_str(include_str!("default_fleet.json")).unwrap();
        assert_eq!(doc["schema_version"].as_u64(), Some(2));
        let missiles = doc["missiles"].as_array().unwrap();
        let ids: std::collections::HashSet<&str> =
            missiles.iter().map(|m| m["id"].as_str().unwrap()).collect();
        assert!(missiles
            .iter()
            .all(|m| !m["stages"].as_array().unwrap().is_empty()));
        for d in doc["designs"].as_array().unwrap() {
            for c in d["components"].as_array().unwrap() {
                if c["kind"] == "magazine" {
                    assert!(ids.contains(c["missile_id"].as_str().unwrap()));
                }
            }
        }
    }

    #[test]
    fn designer_normalizes_legacy_defaults_without_mass_drift() {
        let fleet: crate::model::FleetDocument =
            serde_json::from_str(include_str!("default_fleet.json")).unwrap();
        let expected = [90_000.0, 55_000.0, 50_000.0, 5_000.0];
        for (design, expected_dry_t) in fleet.designs.iter().zip(expected) {
            let out = designer(&DesignerIn {
                settings: fleet.settings.clone(),
                missiles: fleet.missiles.clone(),
                design: design.clone(),
                action: None,
            })
            .unwrap();
            assert!(
                close(out.summary.dry_t, expected_dry_t, 1e-10),
                "{}: {} vs {}",
                design.id,
                out.summary.dry_t,
                expected_dry_t
            );
            assert_eq!(
                out.design
                    .components
                    .iter()
                    .filter(|component| component.kind == "structure")
                    .count(),
                1
            );
            assert!(out
                .design
                .components
                .iter()
                .all(|component| component.auto.is_none()));
            assert!(out.design.components.iter().all(|component| {
                component
                    .mass_t
                    .is_none_or(|mass| mass.is_finite() && mass >= 0.0)
            }));
        }
    }

    #[test]
    fn designer_executes_every_component_sizing_action() {
        let fleet: crate::model::FleetDocument =
            serde_json::from_str(include_str!("default_fleet.json")).unwrap();
        let mut design = designer(&DesignerIn {
            settings: fleet.settings.clone(),
            missiles: fleet.missiles.clone(),
            design: fleet.designs[0].clone(),
            action: None,
        })
        .unwrap()
        .design;
        let actions = [
            ("reactor", "reactor-min"),
            ("reactor", "reactor-max"),
            ("nozzle", "nozzle"),
            ("radiator_hot", "radiator-hot"),
            ("radiator_low", "radiator-low"),
            ("heat_sink", "heat-sink"),
            ("flywheel", "flywheel"),
            ("tank", "tank"),
            ("magazine", "magazine"),
            ("structure", "structure"),
        ];
        for (kind, mode) in actions {
            let component_id = design
                .components
                .iter()
                .find(|component| component.kind == kind)
                .unwrap()
                .id
                .clone();
            let out = designer(&DesignerIn {
                settings: fleet.settings.clone(),
                missiles: fleet.missiles.clone(),
                design,
                action: Some(DesignerActionIn {
                    component_id,
                    mode: mode.into(),
                }),
            })
            .unwrap();
            assert!(
                out.summary.dry_t.is_finite() && out.summary.dry_t > 0.0,
                "{mode}"
            );
            assert_eq!(out.action.as_ref().unwrap().mode, mode);
            design = out.design;
        }

        let crew: Component = serde_json::from_value(serde_json::json!({
            "id": "crew-test", "kind": "crew", "name": "Crew compartment",
            "mass_t": 0.0, "mass_override": false,
            "crew_count": 40, "tonnes_per_crew": 2.0
        }))
        .unwrap();
        design.components.push(crew);
        let radiator = design
            .components
            .iter_mut()
            .find(|component| component.kind == "radiator_hot")
            .unwrap();
        radiator.radiator_mode = Some("areal".into());
        radiator.area_m2 = Some(100.0);
        radiator.kg_per_m2 = Some(3.0);
        let out = designer(&DesignerIn {
            settings: fleet.settings.clone(),
            missiles: fleet.missiles.clone(),
            design,
            action: None,
        })
        .unwrap();
        let crew = out
            .design
            .components
            .iter()
            .find(|component| component.id == "crew-test")
            .unwrap();
        assert_eq!(crew.mass_t, Some(80.0));
        let radiator = out
            .design
            .components
            .iter()
            .find(|component| component.kind == "radiator_hot")
            .unwrap();
        assert_eq!(radiator.mass_t, Some(0.3));

        let mut manual = out.design;
        let crew = manual
            .components
            .iter_mut()
            .find(|component| component.id == "crew-test")
            .unwrap();
        crew.mass_override = Some(true);
        crew.mass_t = Some(12.0);
        let out = designer(&DesignerIn {
            settings: fleet.settings.clone(),
            missiles: fleet.missiles.clone(),
            design: manual,
            action: None,
        })
        .unwrap();
        assert_eq!(
            out.design
                .components
                .iter()
                .find(|component| component.id == "crew-test")
                .unwrap()
                .mass_t,
            Some(12.0)
        );
    }

    /// MR 8 with the research-doc scaling parameters caps out below 5 mg;
    /// MR 4 makes it feasible.
    #[test]
    fn autosize_reports_feasibility_honestly() {
        let base = |mr: f64| AutosizeIn {
            a_target: 0.049, // 5 mg
            mr,
            payload_t: 40000.0,
            p_fusion_pinned: None,
            auto_nozzle: true,
            auto_rad_hot: true,
            auto_rad_low: true,
            auto_tank: true,
            auto_sink: false,
            auto_flywheel: false,
            auto_structure: true,
            ve_max: 2.3e6,
            f_exh: 0.753,
            eta_noz: 0.85,
            sigma: 5.67e-8,
            reactor_t_per_tw: 750.0,
            nozzle_cap_factor: 10.0,
            nozzle_t_per_mn: 30.0,
            rad_load_frac: 0.10,
            hot_t_k: 2000.0,
            hot_eps: 0.9,
            hot_mw_per_kg: 0.326592,
            low_area_frac: 0.25,
            low_t_k: 500.0,
            low_eps: 0.9,
            low_mw_per_kg: 0.0015946875,
            tank_mass_per_prop: 0.04,
            structure_frac: 0.08,
            sink_endurance_s: 1800.0,
            sink_extra_mass_factor: 1.1,
            li_sink_mj_per_kg: 4.6,
            flywheel_fire_s: 30.0,
            flywheel_mj_per_t: 9000.0,
            lasers: vec![],
        };
        let mr8 = autosize(&base(8.0)).unwrap();
        assert!(!mr8.feasible);
        assert!(close(mr8.a_max, 0.0347, 0.02), "a_max {}", mr8.a_max);
        let mr4 = autosize(&base(4.0)).unwrap();
        assert!(mr4.feasible, "a_max at MR4 {}", mr4.a_max);
        // The delivered ship actually pulls the target accel: F/(MR·dry).
        let a = mr4.thrust_n / (mr4.wet_t * 1000.0);
        assert!(close(a, 0.049, 1e-9));
        // Bookkeeping closes: dry = payload + sized parts.
        let parts = mr4.reactor_t.unwrap()
            + mr4.nozzle_t.unwrap()
            + mr4.hot_t.unwrap()
            + mr4.low_t.unwrap()
            + mr4.tank_t.unwrap()
            + mr4.structure_t.unwrap();
        assert!(
            close(mr4.dry_t, 40000.0 + parts, 1e-6),
            "dry {} parts {}",
            mr4.dry_t,
            parts
        );
    }

    #[test]
    fn autosize_with_pinned_reactor_sizes_the_rest() {
        let out = autosize(&AutosizeIn {
            a_target: 0.0,
            mr: 8.0,
            payload_t: 52000.0, // includes the pinned 16 TW reactor's mass
            p_fusion_pinned: Some(1.6e13),
            auto_nozzle: true,
            auto_rad_hot: true,
            auto_rad_low: true,
            auto_tank: true,
            auto_sink: false,
            auto_flywheel: false,
            auto_structure: false,
            ve_max: 2.3e6,
            f_exh: 0.753,
            eta_noz: 0.85,
            sigma: 5.67e-8,
            reactor_t_per_tw: 750.0,
            nozzle_cap_factor: 10.0,
            nozzle_t_per_mn: 30.0,
            rad_load_frac: 0.10,
            hot_t_k: 2000.0,
            hot_eps: 0.9,
            hot_mw_per_kg: 0.326592,
            low_area_frac: 0.25,
            low_t_k: 500.0,
            low_eps: 0.9,
            low_mw_per_kg: 0.0015946875,
            tank_mass_per_prop: 0.04,
            structure_frac: 0.08,
            sink_endurance_s: 1800.0,
            sink_extra_mass_factor: 1.1,
            li_sink_mj_per_kg: 4.6,
            flywheel_fire_s: 30.0,
            flywheel_mj_per_t: 9000.0,
            lasers: vec![],
        })
        .unwrap();
        // 16 TW → 8.9 MN; hot area = 1.6 TW / 816 kW/m² ≈ 1.96e6 m².
        assert!(close(out.thrust_n, 8.9e6, 0.01));
        assert!(close(out.hot_area_m2.unwrap(), 1.96e6, 0.01));
        assert!(out.a_achieved > 0.0 && out.feasible);
    }

    #[test]
    fn timed_burn_matches_deltav_and_clamps() {
        let ve = 2.3e6;
        let thrust = 8.9e6;
        let mdot = thrust / ve;
        let out = timed_burn(&BurnIn {
            v0: 10e3,
            duration_s: 6.0 * 3600.0,
            thrust,
            mdot,
            m0: 7.2e8,
            m_floor: 9.0e7,
            direction: 1.0,
        })
        .unwrap();
        assert!(!out.clamped);
        let expect_dv = ve * (7.2e8f64 / out.m_end).ln();
        assert!(close(out.dv, expect_dv, 1e-9));
        assert!(close(out.v_end, 10e3 + expect_dv, 1e-9));
        // Retrograde mirrors it.
        let retro = timed_burn(&BurnIn {
            v0: 10e3,
            duration_s: 6.0 * 3600.0,
            thrust,
            mdot,
            m0: 7.2e8,
            m_floor: 9.0e7,
            direction: -1.0,
        })
        .unwrap();
        assert!(close(retro.v_end, 10e3 - expect_dv, 1e-9));
        // Absurd duration clamps at the floor.
        let clamped = timed_burn(&BurnIn {
            v0: 0.0,
            duration_s: 1e12,
            thrust,
            mdot,
            m0: 7.2e8,
            m_floor: 9.0e7,
            direction: 1.0,
        })
        .unwrap();
        assert!(clamped.clamped);
        assert!(close(clamped.m_end, 9.0e7, 1e-9));
    }

    #[test]
    fn sprint_burn_then_coast() {
        let ve = 2.3e6;
        let thrust = 8.9e6;
        let mdot = thrust / ve;
        let out = sprint(&SprintIn {
            distance: 1.5e11,
            v0: 0.0,
            thrust,
            mdot,
            m0: 7.2e8,
            m_floor: 7.1e8, // short burn (~30 d), then a long coast
        })
        .unwrap();
        assert!(out.hit);
        let v_b = ve * (7.2e8f64 / 7.1e8).ln();
        assert!(close(out.v_arrival.unwrap(), v_b, 1e-9));
        assert!(out.t_total.unwrap() > out.t_burn);
        // Sprint beats flip-and-burn on time over the same distance and budget
        // (compare with a propellant budget generous enough for both).
        let sp = sprint(&SprintIn {
            distance: 1.5e11,
            v0: 0.0,
            thrust,
            mdot,
            m0: 7.2e8,
            m_floor: 3.0e8,
        })
        .unwrap();
        let fb = travel(&TravelIn {
            distance: 1.5e11,
            ve,
            thrust,
            mdot,
            m0: 7.2e8,
            m_dry: 3.0e8,
            dv_reserve: 0.0,
        })
        .unwrap();
        assert!(fb.feasible);
        assert!(sp.t_total.unwrap() < fb.t_total);
    }

    // ---- system map ------------------------------------------------------

    const G: f64 = 6.674e-11;

    fn sol_earth() -> Vec<NavBody> {
        vec![
            NavBody {
                id: "sol".into(),
                mass_kg: 1.9885e30,
                radius_m: 6.957e8,
                a_m: 0.0,
                phase0_deg: 0.0,
                parent: None,
            },
            NavBody {
                id: "earth".into(),
                mass_kg: 5.97237e24,
                radius_m: 6.371e6,
                a_m: 1.49598e11,
                phase0_deg: 180.0, // far side, so it doesn't perturb the test ship
                parent: Some("sol".into()),
            },
        ]
    }

    #[test]
    fn nav_circular_orbit_closes_after_one_year() {
        let ov = orbit_v(&OrbitVIn {
            g_const: G,
            mass_kg: 1.9885e30,
            r_m: 1.49598e11,
        })
        .unwrap();
        assert!(close(ov.v_circ, 29784.0, 1e-2), "v_circ {}", ov.v_circ);
        let out = nav_tick(&NavTickIn {
            g_const: G,
            epoch_s: 0.0,
            dt_s: ov.period_s,
            substep_s: Some(600.0),
            bodies: sol_earth(),
            ships: vec![NavShipIn {
                id: "s1".into(),
                x: 1.49598e11,
                y: 0.0,
                vx: 0.0,
                vy: ov.v_circ,
                mass_kg: 7.2e8,
                m_floor_kg: 9.0e7,
                landed_on: None,
                burn: None,
            }],
            path_points: Some(50),
        })
        .unwrap();
        let s = &out.ships[0];
        let err = ((s.x - 1.49598e11).powi(2) + s.y.powi(2)).sqrt();
        assert!(err < 0.01 * 1.49598e11, "orbit drift {} m", err);
        assert!(s.path.len() >= 40);
    }

    #[test]
    fn nav_landed_ship_rides_its_body() {
        let dt = 10.0 * 86400.0;
        let out = nav_tick(&NavTickIn {
            g_const: G,
            epoch_s: 0.0,
            dt_s: dt,
            substep_s: Some(600.0),
            bodies: sol_earth(),
            ships: vec![NavShipIn {
                id: "s1".into(),
                // On Earth's surface (Earth starts at phase 180°: x = -a).
                x: -1.49598e11 + 6.371e6,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                mass_kg: 7.2e8,
                m_floor_kg: 9.0e7,
                landed_on: Some("earth".into()),
                burn: None,
            }],
            path_points: None,
        })
        .unwrap();
        let earth = &out.bodies[1];
        let s = &out.ships[0];
        let r = ((s.x - earth.x).powi(2) + (s.y - earth.y).powi(2)).sqrt();
        assert!(close(r, 6.371e6, 1e-6), "offset {}", r);
        assert!(close(s.vx, earth.vx, 1e-9) && close(s.vy, earth.vy, 1e-9));
    }

    #[test]
    fn nav_burn_matches_rocket_equation_without_gravity() {
        let out = nav_tick(&NavTickIn {
            g_const: G,
            epoch_s: 0.0,
            dt_s: 200.0,
            substep_s: Some(1.0),
            bodies: vec![],
            ships: vec![NavShipIn {
                id: "s1".into(),
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                mass_kg: 1e6,
                m_floor_kg: 1e5,
                landed_on: None,
                burn: Some(NavBurnIn {
                    thrust: 1e6,
                    mdot: 10.0,
                    t_remaining_s: 100.0,
                    mode: "angle".into(),
                    angle_deg: 0.0,
                    target_body: None,
                    t_start_s: 0.0,
                }),
            }],
            path_points: None,
        })
        .unwrap();
        let s = &out.ships[0];
        let expect = 1e5 * (1e6f64 / 999_000.0).ln(); // ve_eff·ln(m0/m1)
        assert!(close(s.vx, expect, 1e-3), "vx {} vs {}", s.vx, expect);
        assert!(close(s.dv_spent, expect, 1e-3));
        assert!(close(s.prop_used_kg, 1000.0, 1e-9));
        assert!(s.burn_t_remaining_s == 0.0);
        assert!(s.notes.iter().any(|n| n.contains("Burn complete")));
    }

    #[test]
    fn nav_delayed_burn_ignites_on_time() {
        // 100 s coast, then a 100 s burn: same Δv as an immediate burn, and the
        // ship must not accelerate before ignition.
        let mk = |t_start: f64, dt: f64| {
            nav_tick(&NavTickIn {
                g_const: G,
                epoch_s: 0.0,
                dt_s: dt,
                substep_s: Some(1.0),
                bodies: vec![],
                ships: vec![NavShipIn {
                    id: "s1".into(),
                    x: 0.0,
                    y: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    mass_kg: 1e6,
                    m_floor_kg: 1e5,
                    landed_on: None,
                    burn: Some(NavBurnIn {
                        thrust: 1e6,
                        mdot: 10.0,
                        t_remaining_s: 100.0,
                        mode: "angle".into(),
                        angle_deg: 0.0,
                        target_body: None,
                        t_start_s: t_start,
                    }),
                }],
                path_points: None,
            })
            .unwrap()
        };
        // Before ignition: pure coast, burn untouched.
        let early = mk(100.0, 50.0);
        assert!(early.ships[0].vx.abs() < 1e-12);
        assert!(close(early.ships[0].burn_t_start_remaining_s, 50.0, 1e-9));
        assert!(close(early.ships[0].burn_t_remaining_s, 100.0, 1e-9));
        // Through ignition and completion: full rocket-equation Δv.
        let done = mk(100.0, 300.0);
        let expect = 1e5 * (1e6f64 / 999_000.0).ln();
        assert!(
            close(done.ships[0].vx, expect, 1e-3),
            "vx {}",
            done.ships[0].vx
        );
        assert!(done.ships[0].notes.iter().any(|n| n == "Ignition"));
    }

    #[test]
    fn burn_for_dv_inverts_the_rocket_equation() {
        let out = burn_for_dv(&BurnForDvIn {
            thrust: 8.9e6,
            mdot: 8.9e6 / 2.3e6,
            m0: 7.2e8,
            m_floor: 9.0e7,
            dv: 10e3,
        })
        .unwrap();
        // Forward: Δv = ve·ln(m0/(m0 − mdot·t)) must give back 10 km/s.
        let ve = 2.3e6;
        let m1 = 7.2e8 - out.prop_kg;
        assert!(close(ve * (7.2e8f64 / m1).ln(), 10e3, 1e-9));
        assert!(!out.clamped);
        // Asking beyond the floor clamps.
        let big = burn_for_dv(&BurnForDvIn {
            thrust: 8.9e6,
            mdot: 8.9e6 / 2.3e6,
            m0: 7.2e8,
            m_floor: 9.0e7,
            dv: 1e7,
        })
        .unwrap();
        assert!(big.clamped);
        assert!(close(big.dv_delivered, big.dv_possible, 1e-12));
    }

    #[test]
    fn intercept_planner_finds_a_mars_transfer() {
        // Ship in Earth's solar orbit (no Earth body — clean departure), Mars on
        // rails ahead. Budget 15 km/s, generous capture bubble: must find a hit.
        let bodies = vec![
            NavBody {
                id: "sol".into(),
                mass_kg: 1.9885e30,
                radius_m: 6.957e8,
                a_m: 0.0,
                phase0_deg: 0.0,
                parent: None,
            },
            NavBody {
                id: "mars".into(),
                mass_kg: 6.4171e23,
                radius_m: 3.3895e6,
                a_m: 2.27939e11,
                phase0_deg: 44.0, // ~Hohmann-friendly lead angle
                parent: Some("sol".into()),
            },
        ];
        let v_earth = (G * 1.9885e30 / 1.49598e11).sqrt();
        let out = nav_intercept(&NavInterceptIn {
            g_const: G,
            epoch_s: 0.0,
            bodies,
            x: 1.49598e11,
            y: 0.0,
            vx: 0.0,
            vy: v_earth,
            mass_kg: 7.2e8,
            m_floor_kg: 9.0e7,
            thrust: 8.9e6,
            mdot: 8.9e6 / 2.3e6,
            target_body: Some("mars".into()),
            target_ship: None,
            dv_max: 15e3,
            depart_max_s: 200.0 * 86400.0,
            horizon_s: 700.0 * 86400.0,
            capture_radius_m: 3e9,
        })
        .unwrap();
        assert!(out.feasible, "closest {} m", out.closest_m);
        assert!(out.dv <= 15e3 * 1.0001);
        let transit_d = out.transit_s.unwrap() / 86400.0;
        assert!(
            transit_d > 50.0 && transit_d < 700.0,
            "transit {} d",
            transit_d
        );
        assert!(out.t_burn_s > 0.0 && out.prop_kg > 0.0);
        assert!(out.path.len() > 20, "path {} pts", out.path.len());
    }

    #[test]
    fn nav_ship_falls_onto_body_and_lands() {
        // Drop a ship from 3 Earth radii, dead stop, no sun: it must impact.
        let out = nav_tick(&NavTickIn {
            g_const: G,
            epoch_s: 0.0,
            dt_s: 3.0 * 86400.0,
            substep_s: Some(10.0),
            bodies: vec![NavBody {
                id: "earth".into(),
                mass_kg: 5.97237e24,
                radius_m: 6.371e6,
                a_m: 0.0,
                phase0_deg: 0.0,
                parent: None,
            }],
            ships: vec![NavShipIn {
                id: "s1".into(),
                x: 3.0 * 6.371e6,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                mass_kg: 1e6,
                m_floor_kg: 1e5,
                landed_on: None,
                burn: None,
            }],
            path_points: None,
        })
        .unwrap();
        let s = &out.ships[0];
        assert_eq!(s.landed_on.as_deref(), Some("earth"));
        let r = (s.x * s.x + s.y * s.y).sqrt();
        assert!(close(r, 6.371e6, 1e-6));
        assert!(
            s.notes.iter().any(|n| n.contains("IMPACT")),
            "{:?}",
            s.notes
        );
    }
}
