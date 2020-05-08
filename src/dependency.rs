use crate::color::ColorProperties;
use crate::data::{DataSource, DataSourceKind};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct DataDependency {
    pub require_rgb: bool,
    pub require_hsv: bool,
    pub require_point: bool,
}

impl DataDependency {
    pub fn hsv() -> Self {
        DataDependency {
            require_hsv: true,
            require_rgb: false,
            require_point: false,
        }
    }

    pub fn rgb() -> Self {
        DataDependency {
            require_hsv: false,
            require_rgb: true,
            require_point: false,
        }
    }

    pub fn point() -> Self {
        DataDependency {
            require_hsv: false,
            require_rgb: false,
            require_point: true,
        }
    }
}

pub struct DataDependencyGraph(HashMap<DataSource, DataDependency>);

impl DataDependencyGraph {
    pub fn new() -> Self {
        DataDependencyGraph(HashMap::new())
    }

    fn insert_point(&mut self, source: DataSource) {
        self.0
            .entry(source)
            .and_modify(|x| x.require_point = true)
            .or_insert(DataDependency::point());
    }

    fn insert_hsv(&mut self, source: DataSource) {
        self.0
            .entry(source)
            .and_modify(|x| x.require_point = true)
            .or_insert(DataDependency::hsv());
    }

    fn insert_rgb(&mut self, source: DataSource) {
        self.0
            .entry(source)
            .and_modify(|x| x.require_point = true)
            .or_insert(DataDependency::rgb());
    }

    pub fn require_color(&mut self, name: String) {
        self.insert_point(DataSource {
            name: name,
            kind: DataSourceKind::Color,
        })
    }

    pub fn require_color_channel(&mut self, name: String, property: ColorProperties) {
        use crate::color::ColorProperties::*;
        match property {
            Red | Blue | Green => self.insert_rgb(DataSource {
                name: name,
                kind: DataSourceKind::Color,
            }),
            Hue | Saturation | Value => self.insert_hsv(DataSource {
                name: name,
                kind: DataSourceKind::Color,
            }),
        }
    }

    pub fn require_image(&mut self, name: String) {
        self.insert_point(DataSource {
            name: name,
            kind: DataSourceKind::Image,
        })
    }

    pub fn require_image_channel(&mut self, name: String, property: ColorProperties) {
        use crate::color::ColorProperties::*;
        match property {
            Red | Blue | Green => self.insert_rgb(DataSource {
                name: name,
                kind: DataSourceKind::Image,
            }),
            Hue | Saturation | Value => self.insert_hsv(DataSource {
                name: name,
                kind: DataSourceKind::Image,
            }),
        }
    }

    pub fn require_float(&mut self, name: String) {
        self.0.insert(
            DataSource {
                name: name,
                kind: DataSourceKind::Float,
            },
            DataDependency::default(),
        );
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, DataSource, DataDependency> {
        self.0.iter()
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, DataSource, DataDependency> {
        self.0.keys()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<'_, DataSource, DataDependency> {
        self.0.values()
    }
}

impl IntoIterator for DataDependencyGraph {
    type Item = <HashMap<DataSource, DataDependency> as IntoIterator>::Item;
    type IntoIter = std::collections::hash_map::IntoIter<DataSource, DataDependency>;

    #[inline]
    fn into_iter(self) -> std::collections::hash_map::IntoIter<DataSource, DataDependency> {
        self.0.into_iter()
    }
}
