# Simulator for Analogue's CHIP32 VM

This is a simulator for Analogue's CHIP32 virtual 32-bit CPU used as a data preprocessor for openFPGA. It allows you to step through instructions, view processor and memory state, and constantly log data (as opposed to CHIP32 only letting you print when exiting).

Some of the more obscure APF opcodes are not currently implemented, but their implementation will come soon.

![Main Screen](../assets/mainscreen.png)

## Usage

The TUI currently isn't as polished as I would like, and as such it doesn't display the valid commands. They are:

| Input          | Action                                                                                                  |
|----------------|---------------------------------------------------------------------------------------------------------|
| **s**          | Step through this instruction to the next                                                               |
| **m**          | Switch the display mode to/from memory. Arrow keys up/down will allow you to scroll memory when visible |
| **q**/**quit** | Quit the simulator                                                                                      |

## Example

Analogue provided an [example CHIP32 project](https://github.com/open-fpga/core-example-basicchip32). To run this project in the simulator, look at the `/example` directory in this repo. The only content that was modified from the Analogue example was the `data.json` file:

1. Add a `filename` to data slot 1 (we don't have a filepicker, so we need to give the sim the file to open)
2. Update all of the `filename`s to have a full path (may be improved in the future)
3. Run the project with:

```
cargo run -- --bin .\example\example_chip32.bin --data-json .\example\data.json --data-slot 1
```

This should allow you to simulate the entire program

## Docs

[The official Analogue CHIP32 docs can be found here](https://www.analogue.co/developer/docs/chip32-vm). Unfortunately, the opcode page hasn't been published for some reason, but hopefully it will be soon.