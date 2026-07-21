import json


def combat(role, exposure, vulnerability=1.0, group=None, degraded=0.7, disabled=0.25, destroyed=0.05):
    return {
        "role": role,
        "exposure": exposure,
        "vulnerability": vulnerability,
        "redundancy_group": group,
        "degraded_at": degraded,
        "disabled_at": disabled,
        "destroyed_at": destroyed,
    }


MISSILE_PROFILE = [{
    "id": "missile-kill",
    "name": "Torplet structural kill",
    "material": "Ti-C hybrid (missile hull)",
    "t_pulse_s": 0.1,
    "threshold_mm": 10.0,
}]
SHIP_PROFILE = [{
    "id": "ship-radiator-kill",
    "name": "Ship radiator ablation",
    "material": "Radiator panel (C-C)",
    "t_pulse_s": 0.5,
    "threshold_mm": 10.0,
}]


def component(cid, kind, name, mass_t=None, **fields):
    value = {"id": cid, "kind": kind, "name": name}
    if mass_t is not None:
        value["mass_t"] = mass_t
    value.update(fields)
    return value


def laser(cid, name, power, aperture, mass, role_group, pulse=0.1):
    return component(
        cid, "laser", name, mass,
        count=1,
        p_beam_w=power,
        aperture_m=aperture,
        lambda_m=532e-9,
        eta_wall=0.15,
        t_pulse_s=pulse,
        profiles=MISSILE_PROFILE + SHIP_PROFILE,
        combat=combat("weapon", 0.60, 1.0, role_group),
    )


def magazine(cid, name, missile_id, capacity, mass):
    return component(
        cid, "magazine", name, mass,
        missile_id=missile_id,
        capacity=capacity,
        combat=combat("magazine", 0.28, 0.9, "ordnance"),
    )


def standard_core(prefix, reactor_tw, reactor_mass, nozzle_mass, thrust_n, hot_area, low_area, sink_li, sink_mass, fly_mass, tank_mass):
    return [
        component(prefix+"-reactor", "reactor", "D-He3 fusion plant", reactor_mass,
                  p_fusion_w=reactor_tw*1e12, rad_load_frac=0.01,
                  combat=combat("drive", 0.24, 0.8, "drive")),
        component(prefix+"-nozzle", "nozzle", "Magnetic nozzle and gearing", nozzle_mass,
                  f_max_n=thrust_n, combat=combat("drive", 0.48, 1.0, "drive")),
        component(prefix+"-hot-rad", "radiator_hot", "Armored high-temperature radiator", None,
                  area_m2=hot_area, t_k=2000.0, eps=0.9, mw_per_kg=1.6,
                  combat=combat("radiator", 0.58, 0.75, "hot-loop")),
        component(prefix+"-low-rad", "radiator_low", "Retractable 700 K laser radiator", None,
                  area_m2=low_area, t_k=700.0, eps=0.9, mw_per_kg=0.0128,
                  combat=combat("radiator", 0.95, 1.0, "low-loop")),
        component(prefix+"-sink", "heat_sink", "Lithium phase-change sink", sink_mass,
                  li_t=sink_li, combat=combat("radiator", 0.22, 0.8, "low-loop")),
        component(prefix+"-fly", "flywheel", "CNT flywheel banks", fly_mass,
                  combat=combat("power", 0.28, 0.8, "weapon-power")),
        component(prefix+"-tank", "tank", "Metallic-hydrogen tankage", tank_mass,
                  combat=combat("drive", 0.50, 0.65, "tankage")),
    ]


def sol_battleship():
    c = standard_core("sol-bb", 16, 12000, 6000, 1.0e9, 196000, 486000, 5000, 5500, 100, 18000)
    c += [laser("sol-bb-main", "30 m main phased array", 1e9, 30, 2500, "main-array")]
    for i in range(1, 7):
        c.append(laser(f"sol-bb-sec-{i}", f"10 m secondary {i}", 2e8, 10, 400, "secondaries"))
    for i in range(1, 13):
        c.append(laser(f"sol-bb-pdl-{i}", f"3 m PDL cluster {i}", 1.5e8, 3, 100, "pdl"))
    c += [
        component("sol-bb-sensors", "sensor_lidar", "Distributed radar, IR, 266 nm lidar and metrology", 80,
                  combat=combat("sensor", 0.72, 1.0, "sensors")),
        component("sol-bb-armor", "armor", "Whipple and local refractory armor", 4500,
                  combat=combat("armor", 1.0, 0.45, "structure")),
        component("sol-bb-hab", "habitat", "Crew, command, life support and stores", 750,
                  combat=combat("habitat", 0.25, 0.8, "habitat")),
        magazine("sol-bb-mag-jav", "Javelin offensive magazine", "javelin", 48, 180),
        magazine("sol-bb-mag-int", "Counter-missile magazine", "interceptor", 400, 240),
    ]
    return {"id":"sol-battleship","name":"Solarian 90 kt laser battleship","class":"battleship / lasestar","mr":6.0,
            "structure_t":35328.77657046875,"structure_auto":False,
            "note":"Lasers v3 armament. Dry-mass target 90,000 t. Module system mass is an explicit estimate because Lasers v3 leaves it open.","components":c}


def sol_missile_carrier():
    c = standard_core("sol-mc", 10, 7500, 3500, 6e8, 122500, 133000, 2000, 2200, 50, 11000)
    for i in range(1,5):
        c.append(laser(f"sol-mc-pdl-{i}", f"3 m PDL cluster {i}", 1.5e8, 3, 100, "pdl"))
    c += [
        component("sol-mc-ewar", "sensor_ewar", "Spinal EWAR and cooperative fire-control aperture", 500,
                  combat=combat("sensor", 0.78, 1.0, "sensors")),
        component("sol-mc-sensors", "sensor_lidar", "Radar, IR and 266 nm lidar suite", 80,
                  combat=combat("sensor", 0.68, 1.0, "sensors")),
        component("sol-mc-armor", "armor", "Whipple and local magazine armor", 2750,
                  combat=combat("armor", 1.0, 0.45, "structure")),
        component("sol-mc-hab", "habitat", "Crew and life support", 540,
                  combat=combat("habitat", 0.24, 0.8, "habitat")),
        component("sol-mc-handling", "ordnance_handling", "Cells, rails, handling and isolation", 1000,
                  combat=combat("magazine", 0.52, 0.8, "ordnance")),
        magazine("sol-mc-mag-jav", "Javelin offensive magazine", "javelin", 600, 800),
        magazine("sol-mc-mag-int", "Counter-missile magazine", "interceptor", 200, 160),
    ]
    return {"id":"sol-missile-carrier","name":"Solarian 55 kt missile carrier","class":"missile carrier / EWAR","mr":6.0,
            "structure_t":17090.179164140624,"structure_auto":False,"note":"600 Javelins matches the lower reference class band; full-magazine flush is a doctrine choice, not a structural assumption.","components":c}


def sol_drone_carrier():
    c = standard_core("sol-dc", 8, 6000, 3000, 5e8, 98000, 300000, 1000, 1100, 50, 10000)
    for i in range(1,5):
        c.append(laser(f"sol-dc-pdl-{i}", f"3 m PDL cluster {i}", 1.5e8, 3, 100, "pdl"))
    c += [
        component("sol-dc-bays", "drone_bay", "101-drone bays, workshops and deployment systems", 6000,
                  combat=combat("hangar", 0.65, 0.8, "drone-system")),
        component("sol-dc-c4i", "sensor_ewar", "Drone control, EWAR and cooperative fire-control", 1000,
                  combat=combat("sensor", 0.72, 1.0, "sensors")),
        component("sol-dc-armor", "armor", "Whipple and local hangar armor", 2500,
                  combat=combat("armor", 1.0, 0.45, "structure")),
        component("sol-dc-hab", "habitat", "Crew and life support", 600,
                  combat=combat("habitat", 0.24, 0.8, "habitat")),
        magazine("sol-dc-mag-jav", "Javelin reserve magazine", "javelin", 24, 120),
        magazine("sol-dc-mag-int", "Counter-missile magazine", "interceptor", 200, 180),
    ]
    return {"id":"sol-drone-carrier","name":"Solarian 50 kt drone control ship","class":"drone carrier / control ship","mr":6.0,
            "structure_t":18384.827248437497,"structure_auto":False,"note":"The 101 carried drones are included as bay/C4I mass; drone flight performance is outside fleet schema v1.","components":c}


def sol_corvette():
    c = standard_core("sol-cv", 1, 750, 250, 8e7, 12250, 111000, 250, 275, 20, 600)
    c += [laser("sol-cv-spinal", "15 m spinal phased array", 5e8, 15, 600, "spinal")]
    for i in range(1,3):
        c.append(laser(f"sol-cv-pdl-{i}", f"3 m PDL cluster {i}", 1.5e8, 3, 100, "pdl"))
    c += [
        component("sol-cv-sensors", "sensor_lidar", "Picket radar, IR and 266 nm lidar", 30,
                  combat=combat("sensor", 0.82, 1.0, "sensors")),
        component("sol-cv-armor", "armor", "Whipple and local armor", 250,
                  combat=combat("armor", 1.0, 0.5, "structure")),
        component("sol-cv-hab", "habitat", "25 crew, life support and stores", 75,
                  combat=combat("habitat", 0.28, 0.8, "habitat")),
        magazine("sol-cv-mag-jav", "Javelin cells", "javelin", 24, 75),
        magazine("sol-cv-mag-int", "Counter-missile cells", "interceptor", 40, 40),
    ]
    return {"id":"sol-corvette","name":"Solarian 5 kt screen corvette","class":"corvette / sensor picket","mr":4.0,
            "structure_t":1426.498384921875,"structure_auto":False,"note":"Lasers v3 spinal and PDL architecture; 24 offensive plus 40 defensive rounds is a role-balanced assumption.","components":c}


def chanduran_lasestar():
    c = standard_core("cha-ls", 24, 18000, 10000, 1.5e9, 294000, 1415000, 10000, 11000, 250, 28800)
    c += [laser("cha-ls-main", "60 m fine-segmented primary", 4e9, 60, 9000, "main-array")]
    for i in range(1,9):
        c.append(laser(f"cha-ls-sec-{i}", f"12 m secondary {i}", 3e8, 12, 600, "secondaries"))
    for i in range(1,17):
        c.append(laser(f"cha-ls-pdl-{i}", f"3 m PDL cluster {i}", 1.5e8, 3, 100, "pdl"))
    c += [
        component("cha-ls-sensors", "sensor_lidar", "Distributed high-coherence sensor and metrology lattice", 250,
                  combat=combat("sensor", 0.72, 0.85, "sensors")),
        component("cha-ls-ewar", "sensor_ewar", "Fleet UV dazzle and fire-control coordination array", 1200,
                  combat=combat("sensor", 0.82, 1.0, "sensors")),
        component("cha-ls-armor", "armor", "Whipple and local refractory armor", 9000,
                  combat=combat("armor", 1.0, 0.42, "structure")),
        component("cha-ls-hab", "habitat", "Minimal human accommodation and maintenance volume", 500,
                  combat=combat("habitat", 0.18, 0.7, "habitat")),
        magazine("cha-ls-mag-sickle", "SICKLE magazine", "sickle", 40, 250),
        magazine("cha-ls-mag-int", "Counter-missile magazine", "interceptor", 800, 400),
    ]
    return {"id":"cha-lasestar","name":"Chanduran CATHEDRAL bracket lasestar","class":"heavy lasestar / fleet fire control","mr":5.0,
            "structure_t":82205.51799179688,"structure_auto":False,
            "note":"PROVISIONAL: project knowledge has no authoritative Chanduran hull spec. 60 m / 4 GW brackets a same-technology aperture scaled from Lasers v3, not the chapter's unreliable 62 m / 3 GW claim.","components":c}


def chanduran_heavy():
    c = standard_core("cha-hv", 6, 4500, 2500, 4e8, 73500, 300000, 2000, 2200, 80, 5600)
    c += [laser("cha-hv-main", "30 m primary array", 1e9, 30, 2500, "main-array")]
    for i in range(1,5):
        c.append(laser(f"cha-hv-sec-{i}", f"10 m secondary {i}", 2e8, 10, 400, "secondaries"))
    c += [
        component("cha-hv-sensors", "sensor_lidar", "Chanduran cooperative sensor suite", 100,
                  combat=combat("sensor", 0.75, 0.9, "sensors")),
        component("cha-hv-ewar", "sensor_ewar", "Dazzle and jammer coordination", 350,
                  combat=combat("sensor", 0.78, 1.0, "sensors")),
        component("cha-hv-armor", "armor", "Whipple and local armor", 1750,
                  combat=combat("armor", 1.0, 0.45, "structure")),
        magazine("cha-hv-mag-sickle", "SICKLE magazine", "sickle", 32, 180),
        magazine("cha-hv-mag-int", "Counter-missile magazine", "interceptor", 240, 160),
    ]
    return {"id":"cha-heavy","name":"Chanduran heavy escort bracket","class":"heavy escort / cruiser","mr":5.0,
            "structure_t":12243.3295984375,"structure_auto":False,"note":"PROVISIONAL Chanduran estimate. Inventory helps close a 164-bus initial battle-group load.","components":c}


def chanduran_light():
    c = standard_core("cha-lt", 1.4, 1050, 450, 1.1e8, 17150, 125000, 400, 440, 30, 960)
    c += [laser("cha-lt-main", "15 m primary array", 5e8, 15, 650, "spinal")]
    for i in range(1,3):
        c.append(laser(f"cha-lt-pdl-{i}", f"3 m PDL cluster {i}", 1.5e8, 3, 100, "pdl"))
    c += [
        component("cha-lt-sensors", "sensor_lidar", "Cooperative picket sensor suite", 40,
                  combat=combat("sensor", 0.82, 0.9, "sensors")),
        component("cha-lt-ewar", "sensor_ewar", "Local dazzle and chaff control", 80,
                  combat=combat("sensor", 0.78, 1.0, "sensors")),
        component("cha-lt-armor", "armor", "Whipple and local armor", 400,
                  combat=combat("armor", 1.0, 0.5, "structure")),
        magazine("cha-lt-mag-sickle", "SICKLE cells", "sickle", 6, 50),
        magazine("cha-lt-mag-int", "Counter-missile cells", "interceptor", 60, 50),
    ]
    return {"id":"cha-light","name":"Chanduran light escort bracket","class":"light escort / picket","mr":4.0,
            "structure_t":3297.596958515625,"structure_auto":False,"note":"PROVISIONAL Chanduran estimate. Ten hulls plus two heavies and CATHEDRAL close the 13-contact roster.","components":c}


missiles = [
    {
        "id":"javelin","name":"Javelin 12 t six-torplet bus","payload_kg":904.0,
        "note":"missiles_v2 Javelin: six 25 kg-dry torplets at 30 km/s; Casaba effect is a transparent combat calibration proxy.",
        "stages":[{"id":"javelin-bus","name":"Fusion-heated MH bus","dry_mass_kg":1839.0,"propellant_kg":9257.0,"propulsion":"custom","ve_m_s":92000.0,"a0_g":1.0,"jettison":False}],
        "default_phases":[{"stage_id":"javelin-bus","prop_frac":1.0,"coast_to_range_m":20000000.0}],
        "terminal_effect":{"effect_energy_j":6e12,"accuracy_sigma_m":50.0,"component_bias":{"drive":1.2,"sensor":1.5,"weapon":1.5,"radiator":1.2,"magazine":1.0,"structure":1.0}},
    },
    {
        "id":"sickle","name":"SICKLE 27 t eight-torplet Lancer bracket","payload_kg":3454.0,
        "note":"Physical design uses missiles_v2 Lancer: eight 50 kg-dry torplets at 36 km/s. SICKLE label/inventory are scenario assumptions.",
        "stages":[{"id":"sickle-bus","name":"Fusion-heated MH bus","dry_mass_kg":6184.0,"propellant_kg":17362.0,"propulsion":"custom","ve_m_s":137000.0,"a0_g":1.25,"jettison":False}],
        "default_phases":[{"stage_id":"sickle-bus","prop_frac":1.0,"coast_to_range_m":20000000.0}],
        "terminal_effect":{"effect_energy_j":8e12,"accuracy_sigma_m":35.0,"component_bias":{"drive":1.2,"sensor":1.4,"weapon":1.4,"radiator":1.2,"magazine":1.0,"structure":1.0}},
    },
    {
        "id":"interceptor","name":"200 kg MH kinetic counter-missile","payload_kg":0.0,
        "note":"Underconstrained defensive round retained as a conservative calculator baseline; the combat v1 resolver does not launch interceptor magazines explicitly.",
        "stages":[{"id":"interceptor-stage","name":"MH sprint stage","dry_mass_kg":25.0,"propellant_kg":175.0,"propulsion":"mh","a0_g":15.0,"jettison":False}],
        "default_phases":[{"stage_id":"interceptor-stage","prop_frac":1.0,"coast_to_range_m":None}],
        "terminal_effect":{"effect_energy_j":0.0,"accuracy_sigma_m":5.0,"component_bias":{}},
    },
]

designs = [sol_battleship(), sol_missile_carrier(), sol_drone_carrier(), sol_corvette(), chanduran_lasestar(), chanduran_heavy(), chanduran_light()]


def state(sid, name, design_id, propellant_t, sink_capacity, flywheel, mags):
    return {"id":sid,"name":name,"design_id":design_id,"propellant_t":propellant_t,"velocity_kms":0.0,
            "sink_mj":0.0,"sink_capacity_mj":sink_capacity,"flywheel_mj":flywheel,
            "radiator_hot_pct":100.0,"radiator_low_pct":100.0,"magazines":mags}


states = [
    state("tiberius","BB-013 Tiberius","sol-battleship",225000,2160000,900000,{"sol-bb-mag-jav":48,"sol-bb-mag-int":400}),
    state("lunas-dream","Luna's Dream","sol-missile-carrier",137500,864000,450000,{"sol-mc-mag-jav":600,"sol-mc-mag-int":200}),
    state("bonaparte","Bonaparte","sol-drone-carrier",125000,432000,450000,{"sol-dc-mag-jav":24,"sol-dc-mag-int":200}),
]

corvette_names = ["Shinobi's Pride","Cheyenne","Quarrel","Tin Rose","Castellan","Marat","Pilar","Sable","Long Odds","Dry Run","Vagrant","Kestrel","Halfpenny","Sahara-14","Sahara-15"]
for i, name in enumerate(corvette_names, 1):
    states.append(state(f"sahara-cv-{i:02d}",name,"sol-corvette",7500,108000,180000,{"sol-cv-mag-jav":24,"sol-cv-mag-int":40}))

states += [
    state("cathedral","CATHEDRAL","cha-lasestar",360000,4320000,2250000,{"cha-ls-mag-sickle":40,"cha-ls-mag-int":800}),
    state("epsilon-heavy-1","Bogey Epsilon heavy 1","cha-heavy",70000,864000,720000,{"cha-hv-mag-sickle":32,"cha-hv-mag-int":240}),
    state("epsilon-heavy-2","Bogey Epsilon heavy 2","cha-heavy",70000,864000,720000,{"cha-hv-mag-sickle":32,"cha-hv-mag-int":240}),
]
for i in range(1, 11):
    states.append(state(f"epsilon-light-{i:02d}",f"Bogey Epsilon light {i}","cha-light",12000,172800,270000,{"cha-lt-mag-sickle":6,"cha-lt-mag-int":60}))

patch = [
    {"op":"replace","path":"/settings/laser_eta_wall","value":0.15},
    {"op":"replace","path":"/settings/li_sink_mj_per_kg","value":0.432},
    {"op":"replace","path":"/settings/as_rad_load_frac","value":0.01},
    {"op":"replace","path":"/settings/as_low_t_k","value":700.0},
    {"op":"replace","path":"/settings/as_low_eps","value":0.9},
    {"op":"replace","path":"/settings/as_low_mw_per_kg","value":0.0128},
    {"op":"replace","path":"/settings/pulse_missile_s","value":0.1},
    {"op":"replace","path":"/missiles","value":missiles},
    {"op":"replace","path":"/designs","value":designs},
    {"op":"replace","path":"/states","value":states},
    {"op":"replace","path":"/events","value":[]},
    {"op":"replace","path":"/system/nav","value":{}},
]

print(json.dumps(patch, separators=(",", ":")))
