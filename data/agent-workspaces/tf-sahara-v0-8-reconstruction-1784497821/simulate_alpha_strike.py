import copy
import json
import subprocess


with open("testdata/lidar_pd_baseline.json", encoding="utf-8") as handle:
    base = json.load(handle)

RANGE_M = 158_000_000.0
weapons = [
    ("Tiberius main", 1e9, 30.0, 1),
    ("Tiberius secondary", 2e8, 10.0, 5),
    ("Corvette spinal", 5e8, 15.0, 9),
]


def calculate(name, power, aperture):
    payload = copy.deepcopy(base)
    payload["scenario_name"] = name
    payload["jammers"] = []
    payload["chaff"] = []
    payload["target"].update({
        "position_m": [RANGE_M, 0.0, 0.0],
        "projected_area_m2": 2827.0,
        "characteristic_diameter_m": 60.0,
        "body_radius_m": 30.0,
        "vulnerable_patch_radius_m": 1.0,
        "uv_reflectivity": 0.1,
        "aspect_factor": 1.0,
        "closure_velocity_m_s": 1.0,
        "warhead_standoff_m": 0.0,
        "soft_kill_fluence_j_m2": 1e8,
        "structural_kill_fluence_j_m2": 1e9,
    })
    payload["fire_control"].update({
        "position_sigma_m": 0.01,
        "velocity_sigma_m_s": 0.01,
        "acceleration_sigma_m_s2": 0.0,
        "maneuver_persistence_s": 10.0,
        "reacquisition_time_s": 0.0,
    })
    payload["weapon"].update({
        "listed_optical_power_w": power,
        "aperture_m": aperture,
        "duty_cycle": 0.5,
    })
    completed = subprocess.run(
        ["target/debug/slipstick", "agent", "calculate", "lidar_pd", "--input", "-"],
        input=json.dumps(payload), text=True, capture_output=True, check=True,
    )
    return json.loads(completed.stdout)["data"]


rows = []
combined_flux = 0.0
for name, power, aperture, count in weapons:
    data = calculate(name, power, aperture)
    pd = data["point_defense"]
    combined_flux += pd["average_flux_w_m2"] * count
    rows.append({
        "weapon": name,
        "count": count,
        "spot_diameter_m": pd["weapon_spot_diameter_m"],
        "average_central_lobe_flux_w_m2_each": pd["average_flux_w_m2"],
        "soft_kill_time_s_each": pd["clean_soft_kill_time_s"],
        "structural_kill_time_s_each": pd["clean_structural_kill_time_s"],
    })

durations = {}
for duration in (3.0, 5.0, 10.0, 91.0):
    durations[str(duration)] = {
        "perfectly_overlapped_fluence_j_m2": combined_flux * duration,
        "soft_threshold_multiple": combined_flux * duration / 1e8,
        "structural_threshold_multiple": combined_flux * duration / 1e9,
    }

# Lasers v3 wall-plug waste, using average 50% duty for Tiberius main + five secondaries.
tiberius_listed_w = 1e9 + 5 * 2e8
tiberius_average_waste_w = tiberius_listed_w * 0.5 * (1.0 / 0.15 - 1.0)
tiberius_sink_capacity_j = 5000.0 * 1000.0 * 0.432e6

report = {
    "range_m": RANGE_M,
    "weapons": rows,
    "combined_perfect_overlap_flux_w_m2": combined_flux,
    "durations": durations,
    "thermal": {
        "tiberius_main_plus_five_secondaries_average_waste_w": tiberius_average_waste_w,
        "tiberius_reference_sink_capacity_j": tiberius_sink_capacity_j,
        "sink_fraction_for_91_s": tiberius_average_waste_w * 91.0 / tiberius_sink_capacity_j,
    },
    "limitations": [
        "Perfect overlap is an upper bound; a raster across CATHEDRAL's face distributes fluence.",
        "The pre-laid solution removes acquisition delay but not reciprocal line of sight after crest.",
        "This snapshot does not model return fire, chaff curtain burn-through, phase errors, or target attitude response.",
        "CATHEDRAL geometry and vulnerability are provisional because project knowledge contains no authoritative Chanduran ship specification.",
    ],
}

print(json.dumps(report, indent=2))
