use crate::color::ColorProperties;
use crate::dependency::DataDependencyGraph;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
pub enum DataSourceKind {
    Color,
    Float,
    Image,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct DataSource {
    pub name: String,
    pub kind: DataSourceKind,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum FloatData {
    Constant(f32),
    FloatRef(String),
    ColorChannel {
        color_source: Box<ColorData>,
        channel: ColorProperties,
    },
}

impl FloatData {
    pub fn reference_string(&self) -> String {
        match self {
            &Self::Constant(ref x) => format!("{}", x),
            &Self::FloatRef(ref source) => format!("float_{}", source),
            &Self::ColorChannel {
                ref color_source,
                ref channel,
            } => color_source.channel_reference_string(*channel),
        }
    }

    pub fn get_required_sources(&self, graph: &mut DataDependencyGraph) {
        match self {
            &Self::ColorChannel {
                ref color_source,
                ref channel,
            } => color_source.get_required_channel_sources(graph, *channel),
            &Self::Constant(_) => {}
            &Self::FloatRef(ref name) => graph.require_float(name.clone()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ColorData {
    HSVColor {
        hue: FloatData,
        saturation: FloatData,
        value: FloatData,
    },
    RGBColor {
        red: FloatData,
        green: FloatData,
        blue: FloatData,
    },
    ColorRef {
        color_source: String,
    },
    ImageRef {
        image_source: String,
    },
}

impl ColorData {
    pub fn rgb_vec(&self) -> String {
        use self::ColorData::*;
        match self {
            &HSVColor {
                ref hue,
                ref saturation,
                ref value,
            } => format!(
                "hsv2rgb(vec3({}, {}, {}))",
                hue.reference_string(),
                saturation.reference_string(),
                value.reference_string()
            ),
            &RGBColor {
                ref red,
                ref green,
                ref blue,
            } => format!(
                "vec3({}, {}, {})",
                red.reference_string(),
                green.reference_string(),
                blue.reference_string()
            ),
            &ColorRef { ref color_source } => format!("col_{}_rgb", color_source),
            &ImageRef { ref image_source } => format!("img_{}_rgb", image_source),
        }
    }

    pub fn hsv_vec(&self) -> String {
        use self::ColorData::*;
        match self {
            &HSVColor {
                ref hue,
                ref saturation,
                ref value,
            } => format!(
                "vec3({}, {}, {})",
                hue.reference_string(),
                saturation.reference_string(),
                value.reference_string()
            ),
            &RGBColor {
                ref red,
                ref green,
                ref blue,
            } => format!(
                "rgb2hsv(vec3({}, {}, {}))",
                red.reference_string(),
                green.reference_string(),
                blue.reference_string()
            ),
            &ColorRef { ref color_source } => format!("col_{}_hsv", color_source),
            &ImageRef { ref image_source } => format!("img_{}_hsv", image_source),
        }
    }

    pub fn channel_reference_string(&self, channel: ColorProperties) -> String {
        use self::ColorData::*;
        use crate::color::ColorProperties::*;
        match (self, channel) {
            (&HSVColor { ref hue, .. }, Hue) => hue.reference_string(),
            (&HSVColor { ref saturation, .. }, Saturation) => saturation.reference_string(),
            (&HSVColor { ref value, .. }, Value) => value.reference_string(),
            (&RGBColor { ref red, .. }, Red) => red.reference_string(),
            (&RGBColor { ref green, .. }, Green) => green.reference_string(),
            (&RGBColor { ref blue, .. }, Blue) => blue.reference_string(),
            (x, Hue) => format!("{}.x", x.hsv_vec()),
            (x, Saturation) => format!("{}.y", x.hsv_vec()),
            (x, Value) => format!("{}.z", x.hsv_vec()),
            (x, Red) => format!("{}.x", x.rgb_vec()),
            (x, Green) => format!("{}.y", x.rgb_vec()),
            (x, Blue) => format!("{}.z", x.rgb_vec()),
        }
    }

    pub fn hsv_spherical_vec(&self) -> String {
        use self::ColorData::*;
        match self {
            &HSVColor {
                ref hue,
                ref saturation,
                ref value,
            } => format!(
                "hsv2half_spherical(vec3({}, {}, {}))",
                hue.reference_string(),
                saturation.reference_string(),
                value.reference_string()
            ),
            &RGBColor {
                ref red,
                ref green,
                ref blue,
            } => format!(
                "hsv2half_spherical(rgb2hsv(vec3({}, {}, {})))",
                red.reference_string(),
                green.reference_string(),
                blue.reference_string()
            ),
            &ColorRef { ref color_source } => format!("col_{}", color_source),
            &ImageRef { ref image_source } => format!("img_{}", image_source),
        }
    }

    pub fn get_required_channel_sources(
        &self,
        graph: &mut DataDependencyGraph,
        channel: ColorProperties,
    ) {
        use self::ColorData::*;
        use crate::color::ColorProperties::*;
        match self {
            &HSVColor {
                ref hue,
                ref saturation,
                ref value,
            } => match channel {
                Hue => hue.get_required_sources(graph),
                Saturation => saturation.get_required_sources(graph),
                Value => value.get_required_sources(graph),
                _ => {
                    hue.get_required_sources(graph);
                    saturation.get_required_sources(graph);
                    value.get_required_sources(graph);
                }
            },
            &RGBColor {
                ref red,
                ref green,
                ref blue,
            } => match channel {
                Red => red.get_required_sources(graph),
                Green => green.get_required_sources(graph),
                Blue => blue.get_required_sources(graph),
                _ => {
                    red.get_required_sources(graph);
                    green.get_required_sources(graph);
                    blue.get_required_sources(graph);
                }
            },
            &ColorRef { ref color_source } => {
                graph.require_color_channel(color_source.clone(), channel)
            }
            &ImageRef { ref image_source } => {
                graph.require_image_channel(image_source.clone(), channel)
            }
        }
    }

    pub fn get_required_sources(&self, graph: &mut DataDependencyGraph) {
        use self::ColorData::*;
        match self {
            &HSVColor {
                ref hue,
                ref saturation,
                ref value,
            } => {
                hue.get_required_sources(graph);
                saturation.get_required_sources(graph);
                value.get_required_sources(graph);
            }
            &RGBColor {
                ref red,
                ref green,
                ref blue,
            } => {
                red.get_required_sources(graph);
                green.get_required_sources(graph);
                blue.get_required_sources(graph);
            }
            &ColorRef { ref color_source } => graph.require_color(color_source.clone()),
            &ImageRef { ref image_source } => graph.require_image(image_source.clone()),
        }
    }
}
