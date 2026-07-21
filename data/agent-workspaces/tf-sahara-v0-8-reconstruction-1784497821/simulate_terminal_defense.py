import copy
import json
import subprocess


with open("testdata/lidar_pd_baseline.json", encoding="utf-8") as handle:
    BASE = json.load(handle)

CLOSURE = 138_000.0
STANDOFF = 50_000.0

# name, start range, end range, listed optical W, aperture m, parallel channels,
# number installed in the undamaged initial TF Sahara roster, receive aperture.
LAYERS = [
    ("BB main concentrated", 10_000_000.0, 500_000.0, 1e9, 30.0, 1, 1, 30.0),
    ("BB main terminal groups", 500_000.0, STANDOFF, 1e8, 9.5, 10, 1, 9.5),
    ("BB secondaries concentrated", 3_000_000.0, 500_000.0, 2e8, 10.0, 1, 6, 10.0),
    ("BB secondary terminal groups", 500_000.0, STANDOFF, 5e7, 5.0, 4, 6, 5.0),
    ("Corvette spinals concentrated", 3_000_000.0, 500_000.0, 5e8, 15.0, 1, 15, 15.0),
    ("Corvette spinal terminal groups", 500_000.0, STANDOFF, 1e8, 6.7, 5, 15, 6.7),
    ("Standard PDL clusters", 300_000.0, STANDOFF, 1.5e8, 3.0, 1, 50, 3.0),
]

CASES = [
    ("clean_dedicated_1m", 0.0, False, False),
    ("aligned_100W_dedicated_1m", 100.0, False, False),
    ("aligned_1kW_dedicated_1m", 1000.0, False, False),
    ("aligned_100W_fresh_chaff", 100.0, False, True),
    ("aligned_100W_weapon_receive", 100.0, True, False),
]


def calculate(payload):
    completed = subprocess.run(
        ["target/debug/slipstick", "agent", "calculate", "lidar_pd", "--input", "-"],
        input=json.dumps(payload), text=True, capture_output=True, check=True,
    )
    return json.loads(completed.stdout)["data"]


def integrate_layer(layer, case):
    name, r_start, r_end, power, aperture, channels, installed, receive_aperture = layer
    case_name, jammer_w, use_weapon_receive, use_chaff = case
    bins = 48
    dr = (r_start - r_end) / bins
    kills = 0.0
    usable_seconds = 0.0
    first_feasible = None
    worst_j_s = 0.0
    for index in range(bins):
        range_m = r_start - (index + 0.5) * dr
        payload = copy.deepcopy(BASE)
        payload["scenario_name"] = f"{case_name}: {name}"
        payload["target"]["position_m"] = [range_m, 0.0, 0.0]
        payload["target"]["closure_velocity_m_s"] = CLOSURE
        payload["target"]["warhead_standoff_m"] = STANDOFF
        payload["target"].update({
            "projected_area_m2": 0.25,
            "characteristic_diameter_m": 0.5,
            "body_radius_m": 0.25,
            "vulnerable_patch_radius_m": 0.02,
            "uv_reflectivity": 0.08,
        })
        payload["weapon"]["listed_optical_power_w"] = power
        payload["weapon"]["aperture_m"] = aperture
        payload["detector"]["receiver_aperture_m"] = receive_aperture if use_weapon_receive else 1.0
        if jammer_w == 0.0:
            payload["jammers"] = []
        else:
            payload["jammers"][0]["position_m"] = [range_m * 1.2, 0.0, 0.0]
            payload["jammers"][0]["optical_power_w"] = jammer_w
        if use_chaff:
            payload["chaff"] = [{
                "id": "fresh-puff", "enabled": True,
                "position_m": [range_m - 0.1, 0.0, 0.0],
                "width_m": 2.0, "height_m": 2.0, "depth_m": 0.1,
                "age_s": 0.0, "expansion_speed_m_s": 1.0,
                "optical_depth": 2.0, "single_scatter_albedo": 0.8,
                "backscatter_fraction": 0.1, "range_response": 1.0,
                "clearance_fluence_j_m2": 1e7,
            }]
        data = calculate(payload)
        summary = data["summary"]
        worst_j_s = max(worst_j_s, summary["jammer_to_signal"])
        dt = dr / CLOSURE
        service_s = summary["structural_kill_time_s"] + payload["weapon"]["slew_time_s"]
        if summary["structural_kill_feasible"] and service_s > 0:
            first_feasible = range_m if first_feasible is None else max(first_feasible, range_m)
            usable_seconds += dt
            kills += dt / service_s
    return {
        "layer": name,
        "capacity": kills * channels * installed,
        "single_channel_capacity": kills,
        "channels": channels,
        "installed": installed,
        "first_feasible_range_m": first_feasible,
        "integrated_seconds": usable_seconds,
        "max_jammer_to_signal": worst_j_s,
        "service_includes_fixed_slew_s": BASE["weapon"]["slew_time_s"],
    }


report = {
    "model": "Lasers_v3 range-gated integral of constant-range Lidar/PD snapshots",
    "closure_m_s": CLOSURE,
    "standoff_m": STANDOFF,
    "important_limitations": [
        "Capacities are scheduling bounds, not canonical kills.",
        "Each layer assumes an unlimited queue of valid target associations.",
        "One aligned jammer is applied independently to every serviced target.",
        "Chaff, decoys, false associations, shared target queues, heat depletion, battle damage, interceptors, and kill confirmation beyond the configured 50 ms reacquisition are not coupled.",
        "Summing layers assumes perfect fleet scheduling with no duplicated target service.",
    ],
    "cases": [],
}

for case in CASES:
    layers = [integrate_layer(layer, case) for layer in LAYERS]
    report["cases"].append({
        "case": case[0],
        "total_capacity_upper_bound": sum(layer["capacity"] for layer in layers),
        "layers": layers,
    })

print(json.dumps(report, indent=2))
