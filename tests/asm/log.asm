architecture chip32.vm

// Error vector (0x0)
nop

// Init vector (0x2)
ld r1,#{value}
{command} r1

data:
db "{string}",0