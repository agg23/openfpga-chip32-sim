architecture chip32.vm

// Error vector (0x0)
nop

// Init vector (0x2)
ld r1,#{r1value}
ld r2,#{r2value}
{command} {targets}
