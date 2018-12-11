# Heli-X Scene3D Tool

A tool to interactively generate [Heli-X simulator](http://www.heli-x.info) Scene3D
projects from an input heightmap.

## TODO

Take the [C prototype](https://github.com/jonlamb-gh/heli-x-scene3d-tool) and make a usable tool
written in [2018](https://doc.rust-lang.org/edition-guide/rust-2018) Rust.

## Dependencies

- [xml-rs](https://netvl.github.io/xml-rs/xml/index.html) - XML library
- [kiss3d](http://kiss3d.org/doc/kiss3d/) - 3D graphics library
- [image](https://github.com/PistonDevelopers/image) - Image manipulation library

## Usage

```bash
heli-x-scene3d-tool /path/to/project/res/
```

Where `project` is the root directory of the scene:

```bash
/path/to/project/
└── res
    ├── alphamap.png
    └── heightmap.png
```
