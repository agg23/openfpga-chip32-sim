architecture chip32.vm

// Error vector (0x0)
nop

// Init vector (0x2)
ld r1,#{slot}
open r1,r2
ld r1,#{seek}
seek r1
ld r1,#{output}
ld r2,#{length}
read r1,r2
