# Rust Land Generator for TIC-80

A tech demo of destructable land/terrian generator for TIC-80 written in Rust and compiled into WASM. The land can be edited or destructed by pixels, unlike the ordinary TIC-80 maps, which can be edited by only 8x8 cells.

This project is a part of my Rust practice and development of a Worms clone for TIC-80.

## What Is Done

- [x] Land mask with editable indiviual pixels
- [x] Land allocation in the map section of RAM
- [x] Land rendering with textures & chunk optimizations
- [x] Water at the bottom of the land
- [x] Land generation based on Perlin noise
- [x] Land smoothly goes under water at the borders
- [x] Demo land editor with map saving/loading
- [x] Native-styled in-game notifier (small bonus)

## What Is Not Done

- [ ] Better generation with various patterns
- [ ] Destructable static objects/landmarks, like in Worms
- [ ] Collision with dynamic objects
- [ ] More optimizations and less dependence from 3rd party crates
- [ ] Porting to Lua for usage in other TIC-80 games without WASM
- [ ] CRC check for saved lands to prevent loading of corrupted saves

## Demo Cartridge
Download the demo cartridge from the Release section or TIC-80 website. Alternatively, you can build it yourself with the `build.bat` or `build.sh` scripts.

Land is loaded or generated automatically on the cartridge run, depending on the content of its map memory. You can save or re-generate it using the belowed controls.

### Controls
- **Mouse:** Land editor cursor
  - **Left Button:** Destroy land
  - **Right Button:** Create land 
- **A Button:** Save the land to the map (see below)
- **B Button:** Delete the saved land data
- **X Button:** Generate another land with a different seed
- **Y Button:** Generate another land with the same seed (reset the land)

## Land and the Map Memory
**DO NOT MANUALLY EDIT THE CARTRIDGE MAP!!!**

The land utilizes the map memory to store its binary pixel mask and additional data. The mask is divided into chunks by 8x8 pixels, which are stored in 8 sequential map cells (1 byte or 8 pixels for each cell). The addtional data are stored in the first map cells and help to load the previously saved map with changes. It allows to save the land into the cartridge memory and free up some RAM memory for the game, but the total land size including the additional data cannot exceed the map memory size (32,640 bytes).

When you run the cartridge or press A, the code calls the system function `sync()` to save the land data from RAM into your cartridge. Theoretically, it allows you to save and share your land with the cartridge or by exporting it into `*.map` file. Remember that TIC-80 requires to manually save the changes done by `sync()` in your cartridge.