import copy
import json
import subprocess


with open("testdata/lidar_pd_baseline.json", encoding="utf-8") as handle:
    baseline = json.load(handle)

cases = [
    ("clean", 1.0, 0.0),
    ("jam100", 1.0, 100.0),
    ("jam1000", 1.0, 1000.0),
    ("full30_jam100", 30.0, 100.0),
]

print("case\trange_m\tdetector\tfc_usable\tJ_over_S\tbody_P\tpatch_P\tkill_s\tavailable_s\tfeasible")
for range_m in (10_000_000.0, 3_000_000.0, 1_000_000.0, 500_000.0):
    for case_name, receiver_m, jammer_w in cases:
        payload = copy.deepcopy(baseline)
        payload["scenario_name"] = case_name
        payload["target"]["position_m"] = [range_m, 0.0, 0.0]
        payload["target"]["warhead_standoff_m"] = 50_000.0
        payload["detector"]["receiver_aperture_m"] = receiver_m
        if jammer_w == 0.0:
            payload["jammers"] = []
        else:
            payload["jammers"][0]["position_m"] = [range_m * 1.2, 0.0, 0.0]
            payload["jammers"][0]["optical_power_w"] = jammer_w
        completed = subprocess.run(
            ["target/debug/slipstick", "agent", "calculate", "lidar_pd", "--input", "-"],
            input=json.dumps(payload), text=True, capture_output=True, check=True,
        )
        envelope = json.loads(completed.stdout)
        data = envelope["data"]
        summary = data["summary"]
        row = [
            case_name,
            data["geometry"]["target_range_m"],
            summary["detector_state"],
            summary["fire_control_usable"],
            summary["jammer_to_signal"],
            summary["body_capture_probability"],
            summary["patch_capture_probability"],
            summary["structural_kill_time_s"],
            summary["time_to_standoff_s"],
            summary["structural_kill_feasible"],
        ]
        print("\t".join(str(value) for value in row))
