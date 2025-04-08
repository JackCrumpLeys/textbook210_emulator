- Auto breakpoint at HALT (So it is easy to see registers before OS messes with them)
- Fix control out of pane
- figure out how to make egui not rerender each frame but still update emulator
- Make IO pane actually good (it sucks)
- Update help
- Easy mode (that makes all panes super easy and remove the OS layer, so you can just focus on the program)
- Add support for terminal control and ASCII escapes
- Make light mode less ass
- I think breakpoints are broken
  - Move breakpoints to memory viewer
- Make memory viewer less ass
  - right now we decompile every memory address every frame, not good for CPU!
  - make offsets use labels when relevant
- Make a proper parser
  - most of the bugs with the current parser come from the fact it is a bunch of string matching. We don't need a tree for assembly but tokenization would be good
  - Add tests against the lc3tools compiler
- Add some other devices
  - pixel display
  - file system (higher level than just writing storage I think)
  - ETC
- Cool tools that aren't just emulator
  - single instruction compiler/decompiler
  - IDFK
- A more feature full OS
  - OS building tool where users can edit just one trap (to implement OS activity without having to worry much about internals)
  - utility traps for math (mult, mod, div)
    - OS managed 16 bit float types?
- A table for user to define shorthand mappings like puts etc
- JSON format for themes
- Themes creation tool
- Store program state
  - except for emulator (only the panels and stuff)
- Emulator snapshot and restore
- Add info about current version
- Add credit and buy me a coffee
- memory viewer highlights on value get/set
- Memory region highlighting
- Memory protection visualization
- Memory access history tracking
- HISTORY FOR EVERY VALUE AND ROllBACK debugging AT ANY TIME
  - Historical value tracking with time graphs
  - pane to sort and filter changes
  - Register change highlighting
- Custom memory region labels
- Watchpoints on memory addresses

- editor stuff (HARD)
  - Syntax error highlighting
  - Auto-completion for opcodes and labels
  - Code folding for sections
  - Multiple file tabs
  - Template insertion system
  - Line execution frequency heatmap
  - file saving and loading both in browser and on disk
  - Add breakpoint in the editor (when compiling will be exportted to the memory view)

- Skip to next/previous function call
- Conditional breakpoints
- Run-until-value-change option
- Execution path recording/playback
- Analyse function calls pane

## pane ideas:

### **Memory Timeline**
  - Historical memory state tracking
  - Memory access patterns
  - Time-travel debugging
  - Change frequency heatmap
  - Allocation/deallocation tracking

### **Smart Breakpoint Manager**
  - Conditional breakpoints
  - Hit count breakpoints
  - Value change breakpoints
  - Call pattern breakpoints
  - Temporary breakpoints


### **Binary Converter**
    - Numeric base conversion
    - 16 bit floating point viewer (110 syle)
    - Custom number formatting
    - Bitwise operation calculator
    - ASCII/Unicode converter


###  **Program Loader**
  - Multiple file format support
  - Drag-and-drop loading
  - Web URL import


### **Theme Designer**
  - Custom color schemes
  - Font selection
  - UI density options
  - Light/dark mode toggles
  - Institutional branding options
  - Save/Load themes

### **Layout Manager**
  - Custom workspace layouts
  - Layout presets for different tasks
  - Multi-monitor support (WHEN AVAILABLE)
  - Workspace saving/sharing
