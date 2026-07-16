# Setting Reference — Industry, Manufacturing, and Belt Economics

This document covers the technological and economic systems that define daily life in human space, with emphasis on belt mining operations, printer manufacturing, automation, and the corporate control structures of the Solarian Union. Established through worldbuilding sessions; complements the existing Setting Reference — Chat Compilation.

---

## Mining

### Methods

Fusion power transformed mining from a mechanical discipline into a thermal and electromagnetic one. The specific method depends on the body being mined.

**Solid metallic asteroids (e.g., 16 Psyche):** Solar or ship-mounted gigawatt-class lasers heat the surface, inducing spin and partial refining through selective vaporization. Over weeks or months, the asteroid is spiral-cut like an onion, producing a continuous metal "snake" captured by an attending ship. This is the dominant method for high-value metallic bodies in Sol and produces enormous yields. Requires a solid, structurally coherent body — rubble piles or mixed-composition asteroids will break apart under the induced rotation.

**Rubble-pile and mixed-composition asteroids (e.g., 785 Kalliope):** Internal mining via tunnel boring machines, colloquially called "ore snakes." These are self-contained autonomous mining units, roughly 5–6 meters in diameter and 20–30 meters long, powered by trailing cable from the station's fusion grid.

The cutting head is a ring of plasma torches rather than mechanical disc cutters. In depressurized working sections (standard practice to avoid atmospheric complications), plasma energy transfer to rock is highly efficient. Vaporized material is pulled rearward by differential pressure into a collection and condensation system.

Behind the plasma head, an electromagnetic induction ring pre-fractures the rock matrix by selectively heating conductive metal content. Nickel-iron heats faster than silicate under induction, creating thermal stress at every metal-silicate boundary. This pre-fractures the material along natural separation lines and effectively begins the ore processing at the rock face.

Locomotion is peristaltic — circumferential gripper pads expand against tunnel walls, hydraulics advance the front section, grippers re-engage, rear section pulls forward. The machine braces against its own tunnel, requiring no gravity. Movement resembles an earthworm inching through soil.

Each snake is driven by an SAI that manages bore path, adjusts for composition changes, handles void encounters (common in rubble piles), and monitors structural concerns. Human operators monitor 2–3 snakes simultaneously from a control room, intervening only when the SAI flags exceptions it can't handle.

**Supplementary techniques:**

Hydraulic splitting — drill holes, insert hydraulic wedges, fracture rock along controlled planes. Used for precision extraction of specific ore bodies or structural shaping of habitat spaces. Very controllable, no dust cloud. Works in any gravity.

Laser cutting — the mining laser used for detail work, isolating specific formations, or trimming tunnel walls. Not the primary production method but common for finishing work and habitat construction.

Resistive/ohmic heating — running current directly through conductive rock via contact electrodes for controlled fracture along specific planes. Essentially electrical quarrying. Lower energy density than plasma, more precise.

### Microgravity Considerations

Explosives are effectively unusable for mining in microgravity environments. Fragmented rock doesn't fall — it drifts. Dust doesn't settle — it hangs for hours or days. Equipment cannot brace against blast recoil without hard anchoring to solid structure, and rubble-pile asteroids offer poor anchoring. A poorly placed charge can destabilize an entire section. No one blasts on a low-gravity asteroid if they can avoid it.

Gravity below approximately 0.01g is effectively microgravity for mining purposes. Dust management is a persistent engineering challenge — every operation that generates particulate requires sealed containment and active air handling.

---

## Refining

The basic flow is: crush, separate, smelt, cast. Each step has a microgravity adaptation.

**Crushing.** Sealed ball mills or rod mills. These are self-contained rotating cylinders where the rotation provides internal pseudo-gravity for grinding. Material enters one end, crushed product exits the other. Continuous flow. If the ore snake's induction ring has already pre-fractured the matrix at the rock face, the crushing step requires less energy because the material is partially separated.

**Separation.** Magnetic separation is the primary method for nickel-iron ore. It works better in microgravity than in gravity — small particles don't fall through the magnetic field before weak magnets can capture them. A particle stays in the field until the magnet moves it, improving recovery rates and allowing separation of finer particles with less crushing energy. For non-magnetic fractions (silicates, trace metals, platinum group elements), centrifugal separation — spin the material and density differences sort it.

**Smelting.** Induction furnaces are ideal for microgravity. Metal floats in the center of the magnetic field — containerless melting. No crucible contact means no contamination, no erosion, no refractory linings to replace. Purer product with less maintenance. With fusion power, furnaces can run at any temperature for any duration. The furnace is essentially a coil and a power supply.

**Casting.** Cannot pour in microgravity — nothing falls. Two methods: injection casting (pressurize the melt and force it into molds) and centrifugal casting (spin the mold and let centrifugal force distribute the metal). Centrifugal casting in microgravity produces more uniform products than in gravity because there is no gravitational asymmetry in the casting.

**Infrastructure notes:** Every step requires sealed containment because particulate doesn't settle. Atmosphere control matters — some steps want vacuum, some want inert gas. Material handling between steps requires active systems (gravity-fed hoppers and chutes don't work). The infrastructure is significant — a full refinery is a complex facility — but it's well-understood engineering with fusion power making energy constraints irrelevant. Once set up, a refinery runs largely autonomously under SAI supervision with minimal human oversight.

---

## Internal Transport

Three systems serving different functions.

**Pipes** for bulk flow. Crushed ore moves as pneumatic slurry from the ore snakes to processing and separation. Separated metal fraction moves through pipes to the smelter. This is the circulatory system of the operation — continuous flow, no moving parts except blowers and valves, runs around the clock. In microgravity, pneumatic transport requires much less pressure differential than in gravity because material isn't fighting its own weight.

**Electromagnetic rails** for heavy discrete loads. Finished ingots, equipment, large replacement parts. Electromagnetic rail carts float above linear motor track on magnetic levitation — no wheels, no wear surfaces. Silent, reliable, near-zero maintenance. Infrastructure-heavy to install (track required in every tunnel served), but once in place, essentially permanent. Common in main corridors and between major facilities.

**Multi-legged autonomous haulers ("spiders")** for everything else. Irregular loads, tunnel sections without rail, maintenance runs, equipment transport to remote areas. Six or eight legs with gripping feet — legs beat wheels in microgravity on rough tunnel surfaces because a leg can grip an irregular wall where a wheel needs a smooth road. Cargo platform, semi-autonomous SAI pathfinding. Not fast. Don't need to be.

The hierarchy: pipes are the blood, rails are the highways, spiders are the last mile.

---

## External Shipping

### Mass Drivers

The primary method for shipping bulk refined material from asteroid mining operations. An electromagnetic sled accelerator mounted to the asteroid's surface, typically 500 meters to a kilometer long. Launches standardized cargo pods (2–5 tonnes) at 5+ km/s onto ballistic transfer trajectories toward collection points. Each pod deploys a solar sail or reflective steering vane for minor course corrections en route. At a launch rate of one every few minutes, a mass driver handles hundreds of tonnes per day — sufficient for even large mining operations.

Mass drivers are capital equipment, typically owned by the purchasing corporation and installed at the corporation's expense. The corporation controls the driver's software.

These are the same systems referenced in the general setting notes: "fired from railguns near the source and deploy solar sails or laser sails in flight." The payloads coast on ballistic trajectories for months or years — slow, predictable, defenseless.

### Freighters

Relegated to cargo that can't be mass-driven. Specialty items requiring careful handling: platinum group metals in sealed containers (too valuable per gram to risk on unguided ballistic pods), complex manufactured components coming IN to a station (SAI chips, quantum-grade electronics, nanotube stock, specialty printer feedstocks that can't be synthesized locally), and personnel transfers.

Freighter schedules are a point of corporate leverage. Stretching the freighter interval from six weeks to twelve delays incoming supplies and increases a station's dependence on its printer network for items it previously received shipped.

---

## Printer Technology

Printers are not replicators. Each type is a specialized automated manufacturing cell with different feedstock, different processes, and different output. Multiple printer types are required to cover the full range of a station's manufacturing needs.

All printers operate on licensed firmware connected to a corporate network. The network controls what can be printed, tracks credit charges per print, and enables remote firmware updates. Disconnecting from the network — "going dark" or "jailbreaking" — bricks the printers unless alternative firmware has been prepared.

### Metal Printer

Selective laser or electron beam melting of metal powder in an inert atmosphere or vacuum chamber. Layer-by-layer additive manufacturing, followed by automated heat treatment (stress relief, hardening) and CNC finishing for precision surfaces. Feedstock is atomized metal powder — locally produced from refinery output for common alloys, shipped in for specialty alloy powders.

Produces structural parts, tools, fasteners, pressure vessel components, piping, mechanical assemblies. Size-limited by print volume — roughly a meter in each dimension for a station-scale unit. Larger items are printed in sections and assembled, or cast in the smelter.

The critical distinction from hobbyist 3D printing: the metal printer doesn't just deposit material. It heat-treats, machines critical surfaces, and inspects output. It is a miniature forge, mill, and quality inspection station in one unit.

### Food Printer (Bio Printer)

Molecular synthesis of organic compounds from base organic feedstock — carbon, hydrogen, oxygen, nitrogen, sulfur, phosphorus, trace minerals. Builds macroscopic structures with texture and composition control. Produces proteins, carbohydrates, fats, vitamins, and flavor compounds arranged into food with controlled texture, flavor, and nutritional content.

The key characteristic: food printers do not produce living cells. Printed food is dead tissue arranged to have the right properties. Resolution is molecular (getting protein folding right for texture, getting flavor compound concentrations right for taste) but not cellular. The housing is food-safe but not surgically sterile.

Output quality depends heavily on feedstock purity and the design file. Printed food from base organic feedstock is adequate nutrition but often has a faint chemical or metallic undertone from impurities in local feedstock. High-quality recipe files with precise molecular specifications produce better results than simple templates. Corporate-licensed recipe libraries typically charge per print; open-source libraries exist with varying quality.

### Medical Printer

Same technology family as the food printer — molecular synthesis of organic compounds — but a fundamentally different tool. The medical printer produces viable living cells organized into functional tissue architectures.

Key differences from the food printer: sterile sealed enclosure with controlled atmosphere, temperature, and humidity. Perfusion system — a miniature circulatory loop that feeds nutrients to tissue and removes waste during and after printing, because living tissue dies if simply stacked on a tray. Real-time monitoring of cell viability, oxygenation, and tissue integrity.

Much slower than food printing. Cells need time to adhere, establish connections, and begin functioning. A printed organ may take days of active printing followed by weeks of maturation in the perfusion system before it's transplant-ready. The printer is part manufacturing tool, part incubator.

A small station typically has one medical printer in the medical bay. It is treated as the most critical piece of medical equipment available.

The medical printer also handles complex biologics — antibodies, engineered proteins, gene therapy vectors — by culturing modified cells that secrete target proteins, then harvesting and purifying the product. Possible but slow and in small quantities compared to dedicated pharmaceutical manufacturing.

### Chemical Printer

Automated wet chemistry and synthesis. Takes precursor chemicals and combines them according to programmed reaction pathways. Metered reagent delivery, temperature-controlled reaction chambers, purification and quality verification.

Produces adhesives, sealants, lubricants, cleaning agents, battery electrolyte, hydraulic fluid, coolants, coatings, and pharmaceutical compounds. Drug synthesis is its primary medical function — most common medications (painkillers, antibiotics, anti-inflammatories, stimulants, mood stabilizers) are small molecules that the chemical printer handles with the precision required for accurate dosing and purity.

Feedstock is a library of base chemicals — acids, bases, solvents, monomers — some synthesized locally from mineral processing byproducts, some shipped in.

**Energetic materials:** The chemical printer is technically capable of synthesizing explosives, propellants, and other energetic compounds. This capability is locked behind multiple layers of restriction. Software-level lockouts in the design library prevent energetic recipes from being queued. Firmware-level lockouts in the printer's control systems independently reject reaction pathways that produce energetic products, even if the software layer is bypassed. Both layers must be defeated to synthesize explosives.

Additionally, proper explosive synthesis requires safety hardware that a standard chemical printer lacks — blast containment, pressure venting, thermal runaway suppression. This is analogous to the medical printer's sterile enclosure and perfusion system: the base chemistry platform can do the work, but without the specialized safety infrastructure, there is a real risk of micro-explosions damaging internal components during synthesis. The printer isn't designed to contain the energetic intermediates involved.

Jailbroken stations attempting to synthesize energetic materials on standard chemical printers with custom firmware can and do succeed, but unreliably. The typical failure mode is not a catastrophic explosion — it's a small internal detonation that damages the printer's reaction chambers, metering systems, or thermal controls. The printer breaks. Occasionally a batch is contaminated or unstable. Large-scale explosions from printer malfunctions are rare. The risk is equipment damage and wasted feedstock, not station destruction.

Note: the food printer could synthesize simple drugs in an emergency (the chemistry isn't fundamentally different), but its purity control and dosage precision are inadequate for reliable pharmaceutical use. Using the food printer for medications is a knife-as-scalpel situation — acceptable in crisis, not routine practice.

### Polymer Printer

The simplest and most reliable printer type. Extrusion or photopolymer curing of plastic and composite parts. Multiple feedstock types for different material properties — rigid structural polymer, flexible elastomer, high-temperature engineering plastic.

Produces structural plastics, seals, gaskets, insulation, containers, fabric (woven or nonwoven from printed fiber), clothing, bedding, packaging, utensils, storage containers, toys, and most everyday objects. The majority of a station's consumer goods come from polymer printers. They run constantly and break rarely.

### Ceramic Printer

Sinters ceramic powder (alumina, silicon carbide, zirconia) into high-temperature components. Requires a high-temperature kiln cycle as post-processing. Slower than polymer printing.

Produces furnace linings, electrical insulators, thermal shields, refractory components, and potentially armor inserts (ceramic plates for protective barriers). Less commonly used day-to-day but critical for industrial maintenance. A small station may have only one or two units.

### Composite Printer

Automated fiber-reinforced layup. Deposits fiber (carbon fiber, glass fiber, aramid) in controlled orientations within a polymer or ceramic matrix. Conventional carbon fiber is just carbon in a specific crystalline arrangement — printable from organic feedstock with no exotic materials required.

Carbon fiber composite is common across human space. Any station with a composite printer and organic feedstock can produce it, making it a staple structural material alongside metals and polymers. It is used extensively for pressure hull reinforcement, structural arches, lightweight brackets and frames, antenna structures, vehicle and equipment frames, and anywhere strength-to-weight ratio matters more than impact resistance. Composite structures are stronger per unit weight than metal printer output but less tolerant of point damage.

### Nanotube Weaver

A separate, specialized machine distinct from the composite printer. Nanotube weavers produce carbon nanotube fiber, sheet, and composite — materials with extraordinary tensile strength, electrical conductivity, and thermal properties that conventional carbon fiber cannot match. Nanotube composites are the material of choice for extreme-performance applications: pressure vessels rated for thousands of bars, lightweight armor, high-performance structural members, and advanced thermal management systems.

The weaver is roughly the size of a house. The manufacturing process involves catalytic chemical vapor deposition of individual nanotubes followed by alignment, spinning, and integration into macro-scale fibers or sheets — a process that requires precise atmospheric control, specialized catalyst substrates, and very tight thermal management. The equipment is more common than specialized chip fabs but still expensive, slow, and large enough that only military installations and major civilian stations justify the footprint and cost.

A belt mining station would not typically have one. Nanotube stock is shipped in on freighters like other advanced materials, and used sparingly for applications where conventional carbon fiber or metal won't do the job.

### Electronics Printer

The most technologically constrained printer type, and the primary reason the setting's everyday technology remains grounded and recognizable.

The core technology is electron beam lithography in a compact sealed chamber. E-beam lithography writes circuit features serially — one pixel at a time — rather than projecting an entire pattern simultaneously the way optical lithography does. This makes it slow (one chip may take hours to a full day for complex designs) but compact. With centuries of engineering refinement on multi-beam parallelization, stage precision, and integrated deposition/etching/doping steps, a printer-sized e-beam fab (roughly washing-machine to large-appliance scale) produces chips with single-digit nanometer features. This is roughly equivalent to cutting-edge early-21st-century fabrication — billions of transistors per chip, competent general-purpose processors.

A station has two or three of these units. They run when something breaks or a new system needs to be built, and sit idle most of the time because the station's electronics inventory is adequate.

**What the electronics printer produces:** Competent general-purpose processors, embedded controllers, sensor interfaces, communication transceivers, power management chips, motor drivers, memory modules, basic microcontrollers. These are good enough for the vast majority of station operations. Life support scripting, door actuators, lighting controls, environmental monitoring, basic automation, communications — none of these require cutting-edge hardware. A chip equivalent to what we'd recognize as a modern high-end microcontroller handles these tasks with enormous overhead.

Printed electronics are the backbone of daily life across human space. Cheap, replaceable, reliable, well-understood. The mundane infrastructure of civilization runs on them.

**What the electronics printer cannot produce:**

SAI neural network processors — physically programmable memory, specialized tensor cores on exotic substrates, 3D-stacked structures at atomic precision requiring materials and fabrication environments beyond compact e-beam capability. These are manufactured at dedicated fabs in Sol's inner system and shipped.

Quantum computing components — superconducting circuits, qubit arrays, cryogenic packaging. Entirely different manufacturing discipline.

Advanced sensor arrays — military-grade detector arrays, precision optical elements, components where manufacturing tolerance is measured in atoms.

High-density neural network memory — the dense, specialized storage that SAI systems need for their neural architectures.

**Setting implication:** There is a natural technological stratification. Everyday life runs on printed electronics — reliable, replaceable, familiar. Advanced capability (sophisticated robots, military systems, medical imaging, communications encryption, quantum computing) runs on shipped hardware that is expensive, irreplaceable locally, and controlled by whoever controls the supply chain. Most people interact exclusively with printed electronics and never handle the advanced hardware.

### What Printers Cannot Produce

Across all types, certain categories of product require dedicated manufacturing facilities or specialized equipment beyond the standard printer suite:

- Carbon nanotubes and nanotube composites (requires a dedicated nanotube weaver — see above)
- SAI neural network chips (exotic substrates, atomic-precision 3D stacking)
- Quantum computing components (superconducting circuits, cryogenics)
- Military-grade sensor packages (precision optics, advanced detector arrays)
- Certain specialty chemicals and feedstocks not synthesizable from local precursors
- Energetic materials at reliable quality (requires safety-hardened chemical printer — see above)
- Takemaru-State exotic matter (unique physics, unique manufacturing)

---

## Automation

### General Capability

SAI-driven automation is mature and pervasive across human space. The technology is more than capable of replacing most manual labor. Mining, refining, transport, manufacturing, and basic maintenance can all be handled by autonomous systems under SAI supervision. The degree of automation in practice varies enormously between civilizations and between individual operations.

The Chanduran Confederacy is fully automated where automation is feasible. Humans work where humans want to work. Mining operations in Chanduran space are minimally crewed — a handful of supervisors overseeing entirely autonomous production chains.

The Solarian Union's relationship with automation is more complex, shaped by economics, corporate incentive structures, and the legacy of the First Interplanetary War.

### Automation on Belt Mining Stations

A typical CFS mining station uses substantial automation. The technology is available, the cost is justified, and the workforce is small enough that automation is necessary to achieve viable output.

Ore snakes run autonomously under SAI control. Human operators monitor 2–3 machines from control rooms, intervening on exceptions. Refineries run under SAI supervision with one or two human operators per shift. Transport systems (pipes, rail carts, spiders) are SAI-managed. Maintenance bots handle routine inspections — tunnel structural integrity, pipe joints, rail alignment, ventilation ducting.

A station of 300–350 people leverages automation to achieve 10–30x the output of a comparable pre-automation workforce. Most workers are robot minders, system supervisors, and problem-solvers rather than direct laborers. The automation handles the repetitive, dangerous, and physically demanding work. The humans handle exceptions, maintenance, quality judgment, and the full range of community support functions (life support management, infrastructure, medical, education, administration).

The workforce breaks down roughly as: 80–100 people actively working mine and refinery operations on rotating shifts (operators, supervisors, maintenance), 60–80 in station infrastructure (life support, electrical, plumbing, medical, food service), and the remainder in support roles, children, elderly, and community functions.

### Robot Licensing

All automated systems — ore snakes, refinery controllers, transport spiders, maintenance bots — run on licensed SAI software connected to the corporate network. Like printers, the robots require network authorization to operate. Severing the network connection shuts down the entire automation fleet alongside the printers.

Bringing robots back online after a network severance is substantially harder than restoring printers. Printers share a common firmware architecture — one replacement firmware package covers the fleet. Robots are a zoo of different platforms, each running different SAI control software on different hardware:

- An ore snake's SAI handles spatial navigation, geological interpretation, and equipment management.
- A transport spider's SAI handles pathfinding, load balancing, and obstacle avoidance.
- A refinery controller handles process monitoring, quality control, and safety interlocks.

Each platform requires custom firmware work. Worse, robots are safety-critical in ways printers aren't. A printer with buggy firmware prints a bad part. A snake with buggy firmware bores through a pressure wall. A refinery controller with bad interlocks dumps molten metal into the station.

The practical approach to restoring robot control is not rewriting SAI software from scratch but stripping the DRM licensing layer and allowing the existing SAI to keep running on its existing hardware without phoning home for authorization. This is more analogous to jailbreaking a phone than writing an operating system. Still hard, still requires custom work for each platform, but tractable.

Estimated recovery timeline after network severance: printers restored in hours (if alternative firmware was prepared in advance). First robots restored in days to weeks, one platform at a time, each requiring custom work, hardware verification, and cautious testing. Full automation fleet recovery takes months. Priority order is typically: refinery automation first (critical for manufacturing), transport systems second, maintenance bots third, ore snakes last (lowest priority when production for export has ceased).

---

## Belt Economics Under the Solarian Union

### The Corporate Model

Corporations are the dominant economic force in the Solarian Union. Governments are useful tools for them, not the other way around. In a civilization where shipping goods requires roughly 4x their mass in fuel, local manufacturing via printers is dominant. Corporations maintain control primarily through the printer network — remote software locks, proprietary design libraries, and credit-based access to basic necessities including food, air, and water.

### How the Squeeze Works

On a typical corporate-operated belt station, the economic extraction follows a closed loop:

**The station produces.** Refined ore is launched via mass driver to corporate collection points. The corporation credits the station for the delivered material.

**The corporation sets the price.** Buy-back rates for ore are determined unilaterally by the corporation. These rates can be, and are, adjusted independently of actual market demand for the commodity. A station has no access to open markets and no alternative buyers.

**Credits flow back to the corporation.** Everything on the station costs credits through the printer network. Atmospheric levy. Water levy. Maintenance tax. Print costs for food, clothing, tools, parts. Every credit earned from ore sales is recaptured by the corporation through the cost of staying alive.

**Margins tighten over time.** The squeeze is not dramatic. It's incremental. Exchange rates shift a point. A new levy appears. A service fee materializes on a line item nobody reads. Each change, individually, is an annoyance. Cumulatively, over months or years, they become a garrotte. The station keeps producing. The people keep getting thinner.

**The freighter schedule adds leverage.** Stretching the interval between freighter visits delays incoming supplies — SAI chips, specialty feedstocks, nanotube stock — and increases the station's dependence on what can be printed locally from available materials.

The system is not designed to kill the workforce. It is designed to extract maximum value from a captive population while maintaining minimum viability. The floor on what the corporation will provide is whatever keeps people alive and productive. The ceiling on what people can earn is whatever the corporation decides their ore is worth this quarter.

### Enforcement

When a station severs its network connection — jailbreaks its printers, goes dark — the corporate response follows a standard pattern.

Cybersecurity countermeasures are ineffective because transit-time delays prevent real-time intervention. By the time corporate IT is aware of a jailbreak, the station's systems have already been taken over.

The Solarian Union could destroy non-compliant stations with kinetic kill weapons, but does not. Instead, corporate enforcement divisions dispatch kill teams — small, elite units with powered armor, breaching capability, and broad authorization. These teams are not sent to recapture stations. They are sent to make examples.

The kill teams operate from corporate enforcement facilities, most commonly staged from Ceres for belt operations. Transit time to target is weeks to months depending on orbital geometry and available delta-v. The extended transit means the target population knows what is coming and has time to prepare, or to dread.

The deliberate brutality of the response is the point. A quick kinetic strike would destroy the station and its population, but it would be invisible to every other station in the belt. A kill team assault — slow, publicized through rumor and sensor data, with survivors to tell the story — ensures that every other station contemplating a jailbreak knows exactly what the consequences are.

This enforcement model's effectiveness depends on the cost of resistance being perceived as insurmountably high. It is vulnerable to any situation where resistance becomes costly enough to the corporation that the example being set backfires — where other stations see the price the corporation paid and conclude that the threat is not as absolute as advertised.

### Brain Implants and Compliance

Some corporate employees, primarily security contractors and certain specialized roles, have neural compliance implants. These implants manipulate brain chemistry to reinforce corporate loyalty — subtle dopamine and serotonin rewards for following protocol, discomfort or distress for non-compliance. The implant does not override free will in the crude sense. It paves the path of least resistance in the subject's brain so that obedience feels like breathing and disobedience feels like drowning.

Implants are technically voluntary. In practice, they are required for certain positions. The subject consents, the corporation installs, and the resulting loyalty is chemically genuine — the implanted person is not pretending to be loyal, they are experiencing loyalty as a neurochemical state. This makes implanted personnel both more reliable and more tragic — they may recognize what has been done to them and be unable to act against it.

Implants are also a consumer technology. Brain chemical manipulation creates a lucrative market for in-brain VR experiences, enhanced entertainment, and direct neurochemical mood management. Voluntary civilian use is common in some populations.

---

## Hydrated Minerals and Water Sourcing

Many asteroid types, including mixed metal-silicate rubble piles, contain hydrated minerals — serpentine, clays, and related mineral families with water locked in their crystal structures. Heating these minerals to a few hundred degrees drives off the water, which is then captured and processed.

With fusion power making energy costs negligible, water extraction from hydrated minerals is trivially cheap. On stations mining hydrated ore bodies, water is effectively a byproduct of the mining process. Stations may accumulate significant water stockpiles without dedicated water-mining operations.

This means that on appropriately composed asteroids, water for life support, industrial processes, and other uses (including electrolysis for hydrogen and oxygen) is not a finite resource being depleted from tanks but an ongoing byproduct of normal operations. The water supply scales with mining activity.

---

*Document compiled from worldbuilding sessions. Subject to revision as setting development continues.*
