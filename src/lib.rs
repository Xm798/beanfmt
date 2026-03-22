pub mod align;
pub mod line;
pub mod normalize;
pub mod options;
pub mod recursive;
pub mod sort;

use options::Options;

pub fn format(input: &str, _options: &Options) -> String {
    input.to_string()
}
