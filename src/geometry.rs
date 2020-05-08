//! # Geometry
//! This module contains "Geometry" objects which are used to combine multiple image objects
//! together to generate points used in the final filtering steps.
//!
//! # TODO
//!
//! - Implement Nearest Point Evaluation
//! - Implement Circle Geom1D
//! - Implement Geom2D
//!   - Plane
//!   - Disc
//!   - Square
//! - Implement Geom3D
//!   - Cubes
//!   - Hemispheres
//!   - Spheres
//!

use data::{ColorData, FloatData};
use dependency::DataDependencyGraph;

/// Geom0D represent objects which are points. Geom0D may be specific points on higher dimensional
/// objects such as lines, circles, planes, etc. which are obtained deterministically by some
/// evaluation technique.
#[derive(Clone, Serialize, Deserialize)]
pub enum Geom0D {
    Point(ColorData),
    Evaluation1D(Box<Geom1D>, GeomEvalTechnique1D),
}

/// Geom1D represents a set of 1-Dimensional objects sitting in the color space. This objects are
/// all defined by a continuous function which is evaluatable at certain "times" to get out a point.
#[derive(Clone, Serialize, Deserialize)]
pub enum Geom1D {
    /// Line represents the line which passes through start and end with respective times
    /// for interpolating based on value. This line is infinite some times less than start_time
    /// or greater than end_time will still be calculated and used.
    Line {
        start: Geom0D,
        end: Geom0D,
        start_time: f32,
        end_time: f32,
    },
    /// LineSegment represents the line segment starting at start and ending at end with respective
    /// times for interpolating based on value. This line any times input less than start_time or
    /// greater than end time will return null values.
    LineSegment {
        start: Geom0D,
        end: Geom0D,
        start_time: f32,
        end_time: f32,
    },
    /// Arc represents an arc centered at the origin between start and end obtained by rotation
    /// and scaling only. It takes the smallest scaling and then the shortest rotation to generate
    /// this arc. Input values outside of [start_time, end_time] will extrapolate the arc.
    Arc {
        start: Geom0D,
        end: Geom0D,
        start_time: f32,
        end_time: f32,
    },
    /// ArcSegment represents an arc centered at the origin between start and end obtained by
    /// rotation and scaling only. It takes the smallest scaling and then the shortest rotation to
    /// generate this arc. Input values outside of [start_time, end_time] will return null.
    ArcSegment {
        start: Geom0D,
        end: Geom0D,
        start_time: f32,
        end_time: f32,
    },
}

/// GeomEvalTechnique1D represent ways to get points from Geom1D objects
#[derive(Clone, Serialize, Deserialize)]
pub enum GeomEvalTechnique1D {
    /// ColorProp represents using the underlying function of a Geom1D object to find the point. It
    /// uses a particular color property of a given source image to compute this.
    Value(FloatData),
    // /// NearestPoint returns the closest point on a Geom1D object to the color in the source image
    // /// TODO: Implement this
    // NearestPoint { source : String }
}

// TODO: Implement 2D and 3D Geometry Objects
// pub enum Geom2D
// {
//
// }
//
// pub enum Geom3D
// {
//
// }

impl Geom0D {
    /// Shader generation code for Geom0D objects
    pub fn get_shader(&self) -> String {
        use geometry::Geom0D::*;
        match self {
            &Point(ref c) => format!("{}", c.hsv_spherical_vec()),
            &Evaluation1D(ref geom, ref evaluation_technique) => {
                evaluation_technique.get_shader(geom)
            }
        }
    }

    pub fn get_required_sources(&self, graph: &mut DataDependencyGraph) {
        use geometry::Geom0D::*;

        match self {
            &Evaluation1D(ref geom, ref evaluation_technique) => {
                use geometry::GeomEvalTechnique1D::*;
                match evaluation_technique {
                    &Value(ref data) => data.get_required_sources(graph),
                }
                geom.get_required_sources(graph)
            }
            &Point(ref data) => data.get_required_sources(graph),
        }
    }
}

impl GeomEvalTechnique1D {
    /// Shader code for accessing color property
    pub fn get_shader(&self, geom: &Box<Geom1D>) -> String {
        use geometry::GeomEvalTechnique1D::*;

        match self {
            &Value(ref data) => (*geom).get_shader(data.reference_string()),
        }
    }
}

impl Geom1D {
    /// Shader code for getting points off of a 1D geometry based on a parameter.
    pub fn get_shader(&self, param: String) -> String {
        use geometry::Geom1D::*;
        match self {
            &Line {
                ref start,
                ref end,
                ref start_time,
                ref end_time,
            } => format!(
                "lin_interp({}, {}, ({} - {})/({} - {}))",
                start.get_shader(),
                end.get_shader(),
                param,
                start_time,
                end_time,
                start_time
            ),
            &LineSegment {
                ref start,
                ref end,
                ref start_time,
                ref end_time,
            } => format!(
                "lin_interp_bounded({}, {}, ({} - {})/({} - {}))",
                start.get_shader(),
                end.get_shader(),
                param,
                start_time,
                end_time,
                start_time
            ),
            &Arc {
                ref start,
                ref end,
                ref start_time,
                ref end_time,
            } => format!(
                "rot_interp({}, {}, ({} - {})/({} - {}))",
                start.get_shader(),
                end.get_shader(),
                param,
                start_time,
                end_time,
                start_time
            ),
            &ArcSegment {
                ref start,
                ref end,
                ref start_time,
                ref end_time,
            } => format!(
                "rot_interp_bounded({}, {}, ({} - {})/({} - {}))",
                start.get_shader(),
                end.get_shader(),
                param,
                start_time,
                end_time,
                start_time
            ),
        }
    }

    pub fn get_required_sources(&self, graph: &mut DataDependencyGraph) {
        use geometry::Geom1D::*;

        match self {
            &Line {
                ref start, ref end, ..
            } => {
                start.get_required_sources(graph);
                end.get_required_sources(graph);
            }
            &LineSegment {
                ref start, ref end, ..
            } => {
                start.get_required_sources(graph);
                end.get_required_sources(graph);
            }
            &Arc {
                ref start, ref end, ..
            } => {
                start.get_required_sources(graph);
                end.get_required_sources(graph);
            }
            &ArcSegment {
                ref start, ref end, ..
            } => {
                start.get_required_sources(graph);
                end.get_required_sources(graph);
            }
        }
    }
}
