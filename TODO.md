# Completed Features

- Image loading complete
- Manual Implementation of single image palettes
  - Color to color rotations
  - Supports two types of scaling

# TODO List

- Filter File Format and Parsing
  - Number of images and their locations
  - Number of mappings and their definitions
- Support two+ image filtering
- More filter types
  - Circle to circle mapping
  - Avoid a color mapping
  - Line to line mapping
  - Using second image to interpolate between mappings
- Rendering to output file (add to file format)
  - Feature is hard, requires library change
  - Maybe updating to the new gfx-rs would work and using compute shaders...
- Color smoothing techniques (fixes artifacts in jpgs and the like)
- UI (So that we don't have to specify a file with little meaning forever)
