# Heli-X Scene3D Tool

A tool to interactively generate [Heli-X simulator](http://www.heli-x.info) Scene3D
projects from an input heightmap.

## TODO

Take the [C prototype](https://github.com/jonlamb-gh/heli-x-scene3d-tool) and make a usable tool
written in [2018](https://doc.rust-lang.org/edition-guide/rust-2018) Rust.

## Dependencies

- [png](https://crates.io/crates/png) - PNG decoding and encoding library
- [xml-rs](https://netvl.github.io/xml-rs/xml/index.html) - XML library
- [kiss3d](http://kiss3d.org/doc/kiss3d/) - 3D graphics engine

## Usage

Input image should be `PNG image data, 512 x 512, 8-bit grayscale` for now.
