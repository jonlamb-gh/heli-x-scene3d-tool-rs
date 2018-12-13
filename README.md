# Heli-X Scene3D Tool

A hacked-together prototype tool to interactively view and generate
[Heli-X simulator](http://www.heli-x.info) Scene3D projects from an input heightmap image.

## Dependencies

- [xml-rs](https://netvl.github.io/xml-rs/xml/index.html) - XML library
- [kiss3d](http://kiss3d.org/doc/kiss3d/) - 3D graphics library
- [image](https://github.com/PistonDevelopers/image) - Image manipulation library

## Usage

TODO - change this after cli/opts

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
