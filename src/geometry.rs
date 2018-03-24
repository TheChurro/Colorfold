//! # Geometry
//! This module contains "Geometry" objects which are used to combine multiple image objects
//! together to generate points used in the final filtering steps.

use color::{Color, ColorProperties};

/// Geom0D represent objects which are points. Geom0D may be specific points on higher dimensional
/// objects such as lines, circles, planes, etc. which are obtained deterministically by some
/// evaluation technique.
pub enum Geom0D
{
    Point(Color),
    Evaluation1D(Box<Geom1D>, GeomEvalTechnique1D)
}

/// Geom1D represents a set of 1-Dimensional objects sitting in the color space. This objects are
/// all defined by a continuous function which is evaluatable at certain "times" to get out a point.
pub enum Geom1D
{
    /// Line represents the line which passes through start and end with respective times
    /// for interpolating based on value. This line is infinite some times less than start_time
    /// or greater than end_time will still be calculated and used.
    Line { start : Geom0D, end : Geom0D, start_time : f32, end_time : f32 } ,
    /// LineSegment represents the line segment starting at start and ending at end with respective
    /// times for interpolating based on value. This line any times input less than start_time or
    /// greater than end time will return null values.
    LineSegment { start : Geom0D, end : Geom0D, start_time : f32, end_time : f32 },
    /// Arc represents an arc centered at the origin between start and end obtained by rotation
    /// and scaling only. It takes the smallest scaling and then the shortest rotation to generate
    /// this arc. Input values outside of [start_time, end_time] will extrapolate the arc.
    Arc { start : Geom0D, end : Geom0D, start_time : f32, end_time : f32 } ,
    /// ArcSegment represents an arc centered at the origin between start and end obtained by
    /// rotation and scaling only. It takes the smallest scaling and then the shortest rotation to
    /// generate this arc. Input values outside of [start_time, end_time] will return null.
    ArcSegment { start : Geom0D, end : Geom0D, start_time : f32, end_time : f32 }
}

/// GeomEvalTechnique1D represent ways to get points from Geom1D objects
pub enum GeomEvalTechnique1D
{
    /// ColorProp represents using the underlying function of a Geom1D object to find the point. It
    /// uses a particular color property of a given source image to compute this.
    ColorProp { source : String, property : ColorProperties },
    /// NearestPoint returns the closest point on a Geom1D object to the color in the source image
    NearestPoint { source : String }
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
