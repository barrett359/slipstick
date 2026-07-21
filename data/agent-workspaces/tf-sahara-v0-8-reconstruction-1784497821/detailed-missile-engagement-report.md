# TF Sahara first missile engagement — native calculator report

Scenario: `TF Sahara first SICKLE salvo — severe contested baseline`

Calculator: `missile_engagement` (native Rust), using `lidar_pd` for every
range snapshot. Input: `testdata/tf_sahara_missile_engagement.json`. Full result:
`data/agent-workspaces/artifacts/missile-engagement-1784505329-b5e3e91221d8.json`.

## Result and interpretation

- Initial salvo: 96 SICKLE/Lancer-class buses, eight torplets each, 194 decoys,
  and nine LANTERN jammer sources.
- Initial threat equivalence: 768 torplets.
- Time from the 144,000 km checkpoint to 50 km standoff: 1,043.116 seconds.
- Standoff epoch: **04:36:23 UST**.
- The validated 200 kg MH counter-missile reaches a nominal merge at 27,126.753
  km, not the chapter's 78,000 km. Its configured 12,000 km merge window runs
  from 33,126.753 to 21,126.753 km.
- 212 counter-missiles destroy 42 buses and 106 decoys. The conditional model is
  50% real-target assignment and 40% kill probability after acquiring a real
  bus. Those are explicit scenario assumptions, not outputs of the intercept
  kinematics calculator.
- Eleven threatened buses release 88 torplets at 25,520 km. The 43 buses still
  alive at 20,000 km release another 344. Total terminal inventory is therefore
  432 torplets.
- Lasers destroy 378 torplets. **Fifty-four reach standoff.**
- Conservation closes exactly: 336 torplet-equivalents removed inside 42 killed
  buses + 378 torplets killed after release + 54 leakers = 768.

This result is close to, but does not reproduce, the chapter's 64 leakers. The
reason is visible rather than fitted away: the rounded 40% counter-missile hit
assumption kills 42 buses rather than the chapter's 41, leaving 432 terminal
torplets rather than the corrected 440.

## Critical fire-control caveat

The EWAR stress case gives every serviced target one perfectly aligned 100 W
LANTERN and a fresh centered 2 m chaff puff. CATHEDRAL's unquantified UV dazzle,
false associations, lost parallax, cross-platform scheduling, and confirmation
load are represented by a separate **10.84% association/scheduling efficiency**.

Under the local receiver model, every active laser interval is marked
`fire_control_usable = false`: the 1 m receivers are dropped at long range and
saturated later, while the 30 m receive array is saturated throughout. The
calculator still credits the explicitly configured 10.84% of service capacity
to tracks supplied by the cooperative network. The resulting 378 kills are
therefore a severe-contest scenario result, not a self-closing prediction from
one receiver. Removing that cooperative allowance reduces credited kills to
zero under this exact stress geometry.

The noise jammer and centered chaff do not create a mean centroid offset, so
centroid bias remains zero. They enlarge the random centroid distribution:

|Range km|Receiver|State|SNR|J/S|Clean centroid r95|Interfered centroid r95|Inflation|Future aim r95|
|---:|---|---|---:|---:|---:|---:|---:|---:|
|144,000|Dedicated 1 m|dropped|0.000204|2.26e6|114.4 m|561.6 km|4,909x|561.6 km|
|144,000|Full 30 m receive|saturated|0.00611|2.26e6|0.435 m|655.3 m|1,506x|655.6 m|
|80,000|Dedicated 1 m|dropped|0.00118|7.01e5|63.56 m|53.70 km|845x|53.70 km|
|80,000|Full 30 m receive|saturated|0.0355|7.01e5|0.0821 m|68.90 m|840x|69.35 m|
|20,000|Dedicated 1 m|saturated|0.0669|4.97e4|1.002 m|238.3 m|238x|238.3 m|
|20,000|Full 30 m receive|saturated|2.01|4.97e4|2.80 mm|0.667 m|239x|1.231 m|
|10,000|Dedicated 1 m|saturated|0.312|2.12e4|0.126 m|25.72 m|204x|25.73 m|
|10,000|Full 30 m receive|saturated|9.37|2.12e4|0.656 mm|0.136 m|207x|0.410 m|
|3,000|Dedicated 1 m|saturated|0.413|5.36e4|3.79 mm|6.493 m|1,713x|6.493 m|
|3,000|Full 30 m receive|saturated|31.6|8.13e3|0.0579 mm|39.6 mm|684x|90.0 mm|
|1,000|Dedicated 1 m|saturated|3.69|1.80e4|0.399 mm|0.396 m|993x|0.397 m|
|1,000|Full 30 m receive|saturated|792|303|0.0112 mm|2.90 mm|260x|30.1 mm|
|500|Dedicated 1 m|saturated|7.37|1.80e4|0.176 mm|0.175 m|993x|0.177 m|
|500|Full 30 m receive|saturated|2,670|75.8|0.00558 mm|1.31 mm|234x|25.9 mm|
|300|Dedicated 1 m|saturated|12.3|1.80e4|0.102 mm|0.102 m|993x|0.105 m|
|300|Full 30 m receive|saturated|5,700|27.3|0.00335 mm|0.765 mm|229x|25.0 mm|
|50|Dedicated 1 m|saturated|366|682|0.0167 mm|3.34 mm|200x|24.7 mm|
|50|Full 30 m receive|saturated|42,500|0.758|0.000558 mm|0.126 mm|226x|24.5 mm|

### CATHEDRAL fire against the pickets

Quarrel and Tin Rose are input attrition events because the missile calculator
does not resolve ship internal damage. Their optical severity was checked with
the validated CATHEDRAL bracket rather than the draft's 3 GW / 62 m claim. A
4 GW, 60 m array at 152,000 km produces a 1.644 m central-lobe spot, 3.36 GW
central-lobe power while on, 1.68 GW duty-averaged power, and 0.791 GW/m²
duty-averaged mean flux. The clean 1 GJ/m² fluence time is 1.264 seconds; a
2.4-second dwell places about 1.90 GJ/m² on a held patch. That supports rapid
destruction of exposed spinal, sensor, radiator, or drive structure, but it does
not by itself prove that every 2.4-second hit must destroy an entire corvette.

## Complete range and inventory timeline

Checkpoints are every 10,000 km outside 20,000 km, plus exact event ranges.
From 20,000 km inward they are every 500 km, with additional 300 km and 50 km
weapon/standoff boundaries. `Bus K` and `Torp K` are kills credited during the
preceding interval.

|Range km|UST|Time to standoff|Buses alive|Torplets alive|Bus K|Torp K|Decoys alive|Counter-missiles remaining|1 m state|J/S|1 m centroid r95|30 m centroid r95|Event|
|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---:|---:|---:|---|
|144,000|04:19:00|1,043.12 s|96|0|0|0|194|212|dropped|2.26e6|561.6 km|655 m|Initial checkpoint|
|140,000|04:19:29|1,014.13 s|96|0|0|0|194|212|dropped|2.14e6|501.7 km|587 m|—|
|130,000|04:20:41|941.67 s|96|0|0|0|194|212|dropped|1.85e6|374.4 km|441 m|—|
|120,000|04:21:54|869.20 s|96|0|0|0|194|212|dropped|1.58e6|271.8 km|324 m|—|
|110,000|04:23:06|796.74 s|96|0|0|0|194|212|dropped|1.33e6|191.9 km|231 m|—|
|100,000|04:24:19|724.28 s|96|0|0|0|194|212|dropped|1.10e6|131.1 km|160 m|—|
|90,000|04:25:31|651.81 s|96|0|0|0|194|212|dropped|8.88e5|86.0 km|107 m|—|
|80,000|04:26:44|579.35 s|96|0|0|0|194|212|dropped|7.01e5|53.7 km|68.9 m|—|
|77,760|04:27:00|563.12 s|96|0|0|0|194|212|dropped|6.67e5|48.2 km|62.3 m|CATHEDRAL dazzle begins; Quarrel destroyed; one spinal, two PDLs, and one parallax bearing lost|
|72,240|04:27:40|523.12 s|96|0|0|0|194|212|dropped|5.75e5|35.9 km|47.4 m|Tin Rose destroyed; one spinal, two PDLs, and another bearing lost|
|70,000|04:27:56|506.88 s|96|0|0|0|194|212|dropped|5.40e5|31.7 km|42.2 m|—|
|60,000|04:29:09|434.42 s|96|0|0|0|194|212|saturated|3.97e5|17.1 km|24.0 m|—|
|50,000|04:30:21|361.96 s|96|0|0|0|194|212|saturated|2.78e5|8.32 km|12.6 m|—|
|40,000|04:31:34|289.49 s|96|0|0|0|194|212|saturated|1.80e5|3.45 km|5.85 m|—|
|33,126.753|04:32:23|239.69 s|96|0|0|0|194|212|saturated|1.25e5|1.64 km|3.13 m|Counter-missile merge window opens|
|30,000|04:32:46|217.03 s|85|0|11|0|167|157|saturated|1.04e5|1.13 km|2.29 m|55 counter-missiles expended: 11 buses and 27 decoys destroyed|
|27,126.753|04:33:07|196.21 s|75|0|10|0|141|106|saturated|8.53e4|752 m|1.65 m|Nominal merge; 51 more expended: 10 buses and 26 decoys destroyed|
|25,520|04:33:19|184.57 s|59|88|5|0|127|78|saturated|7.70e4|601 m|1.38 m|28 more expended: 5 buses and 14 decoys destroyed; 11 buses release 88 torplets early|
|21,126.753|04:33:50|152.73 s|43|88|16|0|88|0|saturated|5.40e4|289 m|0.773 m|Final 78 expended: 16 buses and 39 decoys destroyed; merge window closes|
|20,000|04:33:59|144.57 s|0|432|0|0|88|0|saturated|4.97e4|238 m|0.667 m|43 buses release 344 torplets|
|19,500|04:34:02|140.94 s|0|432|0|0|88|0|saturated|4.72e4|215 m|0.616 m|—|
|19,000|04:34:06|137.32 s|0|432|0|0|88|0|saturated|4.48e4|194 m|0.568 m|—|
|18,500|04:34:09|133.70 s|0|432|0|0|88|0|saturated|4.38e4|180 m|0.538 m|—|
|18,000|04:34:13|130.07 s|0|432|0|0|88|0|saturated|4.14e4|161 m|0.494 m|—|
|17,500|04:34:17|126.45 s|0|432|0|0|88|0|saturated|3.92e4|144 m|0.452 m|—|
|17,000|04:34:20|122.83 s|0|432|0|0|88|0|saturated|3.82e4|133 m|0.427 m|—|
|16,500|04:34:24|119.20 s|0|432|0|0|88|0|saturated|3.60e4|118 m|0.390 m|—|
|16,000|04:34:28|115.58 s|0|432|0|0|88|0|saturated|3.39e4|104 m|0.354 m|The chapter's stated 115-second terminal clock belongs here|
|15,500|04:34:31|111.96 s|0|432|0|0|88|0|saturated|3.30e4|95.3 m|0.334 m|—|
|15,000|04:34:35|108.33 s|0|432|0|0|88|0|saturated|3.09e4|83.6 m|0.302 m|—|
|14,500|04:34:38|104.71 s|0|432|0|0|88|0|saturated|3.01e4|76.2 m|0.284 m|—|
|14,000|04:34:42|101.09 s|0|432|0|0|88|0|saturated|2.81e4|66.2 m|0.255 m|—|
|13,500|04:34:46|97.46 s|0|432|0|0|88|0|saturated|2.74e4|60.1 m|0.239 m|—|
|13,000|04:34:49|93.84 s|0|432|0|0|88|0|saturated|2.54e4|51.7 m|0.213 m|—|
|12,500|04:34:53|90.22 s|0|432|0|0|88|0|saturated|2.47e4|46.6 m|0.199 m|—|
|12,000|04:34:57|86.59 s|0|432|0|0|88|0|saturated|2.28e4|39.6 m|0.176 m|—|
|11,500|04:35:00|82.97 s|0|432|0|0|88|0|saturated|2.22e4|35.5 m|0.165 m|—|
|11,000|04:35:04|79.35 s|0|432|0|0|88|0|saturated|2.18e4|31.8 m|0.154 m|—|
|10,500|04:35:07|75.72 s|0|432|0|0|88|0|saturated|2.14e4|28.6 m|0.144 m|—|
|10,000|04:35:11|72.10 s|0|432|0|0|88|0|saturated|2.12e4|25.7 m|0.136 m|V3 outer laser phase opens|
|9,500|04:35:15|68.48 s|0|432|0|0|88|0|saturated|2.12e4|23.2 m|0.129 m|Main outer capacity below one service|
|9,000|04:35:18|64.86 s|0|432|0|0|88|0|saturated|2.15e4|21.1 m|0.123 m|—|
|8,500|04:35:22|61.23 s|0|432|0|0|88|0|saturated|2.21e4|19.4 m|0.119 m|—|
|8,000|04:35:26|57.61 s|0|432|0|0|88|0|saturated|2.77e4|21.6 m|0.140 m|—|
|7,500|04:35:29|53.99 s|0|432|0|0|88|0|saturated|2.98e4|20.5 m|0.137 m|—|
|7,000|04:35:33|50.36 s|0|432|0|0|88|0|saturated|4.35e4|26.1 m|0.173 m|—|
|6,500|04:35:36|46.74 s|0|432|0|0|88|0|saturated|8.05e4|41.9 m|0.275 m|—|
|6,000|04:35:40|43.12 s|0|432|0|0|88|0|saturated|2.14e5|95.4 m|0.622 m|Worst modeled chaff/jammer interaction in this sweep|
|5,500|04:35:44|39.49 s|0|432|0|0|88|0|saturated|1.80e5|67.8 m|0.439 m|—|
|5,000|04:35:47|35.87 s|0|432|0|0|88|0|saturated|1.49e5|46.6 m|0.300 m|—|
|4,500|04:35:51|32.25 s|0|432|0|0|88|0|saturated|1.21e5|30.9 m|0.197 m|—|
|4,000|04:35:54|28.62 s|0|432|0|0|88|0|saturated|9.53e4|19.6 m|0.123 m|—|
|3,500|04:35:58|25.00 s|0|432|0|0|88|0|saturated|7.29e4|11.7 m|72.4 mm|—|
|3,000|04:36:02|21.38 s|0|432|0|0|88|0|saturated|5.36e4|6.49 m|39.6 mm|V3 transition to sequential main, secondaries, and corvette spinals|
|2,500|04:36:05|17.75 s|0|429|0|3|88|0|saturated|3.72e4|3.27 m|19.8 mm|Main kills 3|
|2,000|04:36:09|14.13 s|0|425|0|4|88|0|saturated|2.38e4|1.44 m|9.17 mm|Main kills 4|
|1,500|04:36:13|10.51 s|0|418|0|7|88|0|saturated|1.80e4|0.695 m|4.97 mm|Main 4; corvette spinals 3|
|1,000|04:36:16|6.88 s|0|404|0|14|88|0|saturated|1.80e4|0.396 m|2.90 mm|Main 4; secondaries 1; corvette spinals 9|
|500|04:36:20|3.26 s|0|367|0|37|88|0|saturated|1.80e4|0.175 m|1.31 mm|Main 3; secondaries 6; corvette spinals 28; distributed groups begin|
|300|04:36:21|1.81 s|0|303|0|64|88|0|saturated|1.80e4|0.102 m|0.765 mm|Main groups 13; secondary groups 8; corvette groups 43; fleet PDL opens|
|50|04:36:23|0.00 s|0|54|0|249|88|0|saturated|682|3.34 mm|0.126 mm|Main groups 19; secondary groups 37; corvette groups 119; PDL 74; 54 reach standoff|

## Laser service ledger

Power is useful central-lobe power per channel. Flux is the mean, duty-averaged
central-lobe flux at the interval midpoint. Service time includes effective
structural dwell, reacquisition, and the configured 50 ms non-dwell term. Gross
capacity is before the 10.84% association allowance; effective capacity is
after it. Fractional effective capacity carries forward and becomes whole kills.

|Interval end km|Weapon mode|Channels|Midpoint km|Spot|Power/channel|Average flux|Service time|Gross capacity|Effective capacity|Kills|
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|9,500|Main uncertainty-matched|1|9,750|0.211 m|840 MW|12.0 GW/m²|954.88 s|0.004|0.000|0|
|9,000|Main uncertainty-matched|1|9,250|0.200 m|840 MW|13.4 GW/m²|757.89 s|0.005|0.001|0|
|8,500|Main uncertainty-matched|1|8,750|0.189 m|840 MW|14.9 GW/m²|808.82 s|0.004|0.000|0|
|8,000|Main uncertainty-matched|1|8,250|0.179 m|840 MW|16.8 GW/m²|687.52 s|0.005|0.001|0|
|7,500|Main uncertainty-matched|1|7,750|0.168 m|840 MW|19.0 GW/m²|591.97 s|0.006|0.001|0|
|7,000|Main uncertainty-matched|1|7,250|0.157 m|840 MW|21.7 GW/m²|823.41 s|0.004|0.000|0|
|6,500|Main uncertainty-matched|1|6,750|0.146 m|840 MW|25.1 GW/m²|1,592.66 s|0.002|0.000|0|
|6,000|Main uncertainty-matched|1|6,250|0.135 m|840 MW|29.2 GW/m²|18,675.28 s|0.0002|0.0000|0|
|5,500|Main uncertainty-matched|1|5,750|0.124 m|840 MW|34.6 GW/m²|9,310.90 s|0.0004|0.0000|0|
|5,000|Main uncertainty-matched|1|5,250|0.114 m|840 MW|41.5 GW/m²|4,353.12 s|0.0008|0.0001|0|
|4,500|Main uncertainty-matched|1|4,750|0.103 m|840 MW|50.6 GW/m²|1,884.37 s|0.0019|0.0002|0|
|4,000|Main uncertainty-matched|1|4,250|0.0920 m|840 MW|63.3 GW/m²|742.65 s|0.0049|0.0005|0|
|3,500|Main uncertainty-matched|1|3,750|0.0811 m|840 MW|81.2 GW/m²|260.46 s|0.0139|0.0015|0|
|3,000|Main uncertainty-matched|1|3,250|0.0703 m|840 MW|108 GW/m²|78.75 s|0.0460|0.0050|0|
|2,500|Main sequential|1|2,750|0.0595 m|840 MW|151 GW/m²|0.1088 s|33.29|3.61|3|
|2,500|Six secondaries|6|2,750|0.179 m|168 MW|3.36 GW/m²|182.36 s|0.12|0.01|0|
|2,500|Thirteen corvette spinals|13|2,750|0.119 m|420 MW|18.9 GW/m²|61.27 s|0.77|0.08|0|
|2,000|Main sequential|1|2,250|0.0487 m|840 MW|226 GW/m²|0.1052 s|34.45|3.73|4|
|2,000|Six secondaries|6|2,250|0.146 m|168 MW|5.02 GW/m²|37.97 s|0.57|0.06|0|
|2,000|Thirteen corvette spinals|13|2,250|0.0974 m|420 MW|28.2 GW/m²|12.43 s|3.79|0.41|0|
|1,500|Main sequential|1|1,750|0.0379 m|840 MW|373 GW/m²|0.1029 s|35.21|3.82|4|
|1,500|Six secondaries|6|1,750|0.114 m|168 MW|8.29 GW/m²|5.59 s|3.89|0.42|0|
|1,500|Thirteen corvette spinals|13|1,750|0.0757 m|420 MW|46.6 GW/m²|1.812 s|25.99|2.82|3|
|1,000|Main sequential|1|1,250|0.0270 m|840 MW|731 GW/m²|0.1014 s|35.72|3.87|4|
|1,000|Six secondaries|6|1,250|0.0811 m|168 MW|16.2 GW/m²|1.738 s|12.51|1.36|1|
|1,000|Thirteen corvette spinals|13|1,250|0.0541 m|420 MW|91.4 GW/m²|0.579 s|81.37|8.82|9|
|500|Main sequential|1|750|0.0162 m|840 MW|2,030 GW/m²|0.1005 s|36.05|3.91|3|
|500|Six secondaries|6|750|0.0487 m|168 MW|45.1 GW/m²|0.403 s|53.98|5.85|6|
|500|Thirteen corvette spinals|13|750|0.0325 m|420 MW|254 GW/m²|0.180 s|262.23|28.43|28|
|300|Ten main groups|10|400|0.0273 m|84 MW|71.6 GW/m²|0.1140 s|127.09|13.78|13|
|300|Twenty-four secondary groups|24|400|0.0519 m|42 MW|9.92 GW/m²|0.4635 s|75.05|8.14|8|
|300|Sixty-five corvette groups|65|400|0.0388 m|84 MW|35.6 GW/m²|0.2323 s|405.44|43.95|43|
|50|Ten main groups|10|175|0.0120 m|84 MW|374 GW/m²|0.1028 s|176.27|19.11|19|
|50|Twenty-four secondary groups|24|175|0.0227 m|42 MW|51.8 GW/m²|0.1262 s|344.63|37.36|37|
|50|Sixty-five corvette groups|65|175|0.0170 m|84 MW|186 GW/m²|0.1080 s|1,089.90|118.15|119|
|50|Forty-six fleet PDL clusters|46|175|0.0379 m|126 MW|56.0 GW/m²|0.1205 s|691.72|74.98|74|

## Weapon and heat totals

|Weapon layer|Buses killed|Torplets killed|Decoys killed|Heat added|
|---|---:|---:|---:|---:|
|200 kg MH counter-missiles|42|0|106|not assigned to ship sinks|
|Tiberius main, uncertainty-matched outer phase|0|0|0|143.72 GJ|
|Tiberius main, sequential mid phase|0|18|0|51.33 GJ|
|Six battleship secondaries, sequential|0|7|0|61.59 GJ|
|Corvette spinals, sequential|0|40|0|333.64 GJ|
|Ten Tiberius main terminal groups|0|32|0|9.24 GJ|
|Twenty-four secondary terminal groups|0|45|0|11.09 GJ|
|Sixty-five corvette terminal groups|0|162|0|60.05 GJ|
|Forty-six fleet PDL clusters|0|74|0|35.42 GJ|

Tiberius's modeled main/secondary sink load ends at 276.97 GJ, 12.82% of
its 2.16 TJ capacity. The thirteen surviving corvettes' spinal load is 393.69
GJ, 28.04% of their collective 1.404 TJ capacity. Fleet PDL heat is reported
separately because those 46 clusters draw from Tiberius, Luna's Dream,
Bonaparte, and thirteen independent corvette sinks rather than one honest pool.

## What this changes in the chapter

1. The first counter-missile merge cannot occur at 78,000 km with the validated
   MH interceptor. The native intercept solution puts it near 27,000 km.
2. The 115-second clock starts near 15,920 km, not at the chapter's 04:30 header.
3. Lasers do not produce meaningful credited kills at 10,000–3,000 km under the
   severe aligned-jammer/fresh-chaff model. The main begins killing around
   2,750 km midpoint; secondaries and corvette spinals become important inside
   roughly 1,500 km; distributed groups dominate inside 500 km.
4. The actual terminal climax is extremely compressed: 367 torplets remain at
   500 km with 3.26 seconds to standoff; 303 remain at 300 km with 1.81 seconds;
   249 are destroyed in the final 250 km interval.
5. The chapter's broad outcome remains plausible only if the cooperative sensor
   network can turn roughly a tenth of the enormous clean scheduling capacity
   into valid target services despite every local receiver being saturated or
   dropped in the stress case. That is the next model requiring refinement.
