architecture chip32.vm

// Error vector (0x0)
nop

// Init vector (0x2)
ld r1,#{value}
ld r2,#1
{command} {targets}
