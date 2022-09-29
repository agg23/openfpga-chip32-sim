architecture chip32.vm

// Error vector (0x0)
nop

// Init vector (0x2)
ld r1,#0xCADEDEAD
{command} {targets}

data:
dd 0xDEADBEEF
