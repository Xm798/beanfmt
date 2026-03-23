use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;

use crate::options::{Options, ThousandsSeparator};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn parse_thousands(s: &str) -> PyResult<ThousandsSeparator> {
    match s.to_ascii_lowercase().as_str() {
        "add" => Ok(ThousandsSeparator::Add),
        "remove" => Ok(ThousandsSeparator::Remove),
        "keep" => Ok(ThousandsSeparator::Keep),
        other => Err(PyValueError::new_err(format!(
            "invalid thousands_separator: {other:?}"
        ))),
    }
}

fn build_options(
    indent: Option<String>,
    currency_column: Option<usize>,
    cost_column: Option<usize>,
    thousands_separator: Option<String>,
    spaces_in_braces: Option<bool>,
    fixed_cjk_width: Option<bool>,
    sort: Option<bool>,
) -> PyResult<Options> {
    let defaults = Options::default();
    let ts = match thousands_separator {
        Some(s) => parse_thousands(&s)?,
        None => defaults.thousands_separator,
    };
    Ok(Options {
        indent: indent.unwrap_or(defaults.indent),
        currency_column: currency_column.unwrap_or(defaults.currency_column),
        cost_column: cost_column.unwrap_or(defaults.cost_column),
        thousands_separator: ts,
        spaces_in_braces: spaces_in_braces.unwrap_or(defaults.spaces_in_braces),
        fixed_cjk_width: fixed_cjk_width.unwrap_or(defaults.fixed_cjk_width),
        sort: sort.unwrap_or(defaults.sort),
    })
}

/// Reusable formatting configuration object.
#[pyclass(name = "Options")]
#[derive(Debug, Clone)]
struct PyOptions {
    inner: Options,
}

#[pymethods]
impl PyOptions {
    #[new]
    #[pyo3(signature = (
        indent = None,
        currency_column = None,
        cost_column = None,
        thousands_separator = None,
        spaces_in_braces = None,
        fixed_cjk_width = None,
        sort = None,
    ))]
    fn new(
        indent: Option<String>,
        currency_column: Option<usize>,
        cost_column: Option<usize>,
        thousands_separator: Option<String>,
        spaces_in_braces: Option<bool>,
        fixed_cjk_width: Option<bool>,
        sort: Option<bool>,
    ) -> PyResult<Self> {
        let inner = build_options(
            indent,
            currency_column,
            cost_column,
            thousands_separator,
            spaces_in_braces,
            fixed_cjk_width,
            sort,
        )?;
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        let o = &self.inner;
        let ts = match o.thousands_separator {
            ThousandsSeparator::Add => "add",
            ThousandsSeparator::Remove => "remove",
            ThousandsSeparator::Keep => "keep",
        };
        format!(
            "Options(indent='{}', currency_column={}, cost_column={}, \
             thousands_separator='{}', spaces_in_braces={}, fixed_cjk_width={}, sort={})",
            o.indent.replace('\\', "\\\\").replace('\'', "\\'"),
            o.currency_column,
            o.cost_column,
            ts,
            if o.spaces_in_braces { "True" } else { "False" },
            if o.fixed_cjk_width { "True" } else { "False" },
            if o.sort { "True" } else { "False" },
        )
    }
}

/// Resolve an `Options` value from an optional `PyOptions` object plus kwargs.
/// If both `options` and individual kwargs are provided, kwargs override.
fn resolve_options(
    options: Option<&PyOptions>,
    indent: Option<String>,
    currency_column: Option<usize>,
    cost_column: Option<usize>,
    thousands_separator: Option<String>,
    spaces_in_braces: Option<bool>,
    fixed_cjk_width: Option<bool>,
    sort: Option<bool>,
) -> PyResult<Options> {
    let base = match options {
        Some(o) => o.inner.clone(),
        None => Options::default(),
    };

    let ts = match thousands_separator {
        Some(s) => parse_thousands(&s)?,
        None => base.thousands_separator,
    };

    Ok(Options {
        indent: indent.unwrap_or(base.indent),
        currency_column: currency_column.unwrap_or(base.currency_column),
        cost_column: cost_column.unwrap_or(base.cost_column),
        thousands_separator: ts,
        spaces_in_braces: spaces_in_braces.unwrap_or(base.spaces_in_braces),
        fixed_cjk_width: fixed_cjk_width.unwrap_or(base.fixed_cjk_width),
        sort: sort.unwrap_or(base.sort),
    })
}

/// Format a beancount string and return the formatted output.
#[pyfunction]
#[pyo3(signature = (
    input,
    options = None,
    indent = None,
    currency_column = None,
    cost_column = None,
    thousands_separator = None,
    spaces_in_braces = None,
    fixed_cjk_width = None,
    sort = None,
))]
#[allow(clippy::too_many_arguments)]
fn format(
    input: &str,
    options: Option<&PyOptions>,
    indent: Option<String>,
    currency_column: Option<usize>,
    cost_column: Option<usize>,
    thousands_separator: Option<String>,
    spaces_in_braces: Option<bool>,
    fixed_cjk_width: Option<bool>,
    sort: Option<bool>,
) -> PyResult<String> {
    let opts = resolve_options(
        options,
        indent,
        currency_column,
        cost_column,
        thousands_separator,
        spaces_in_braces,
        fixed_cjk_width,
        sort,
    )?;
    Ok(crate::format(input, &opts))
}

/// Format a single beancount file by path and return the formatted content.
#[pyfunction]
#[pyo3(signature = (
    path,
    options = None,
    indent = None,
    currency_column = None,
    cost_column = None,
    thousands_separator = None,
    spaces_in_braces = None,
    fixed_cjk_width = None,
    sort = None,
))]
#[allow(clippy::too_many_arguments)]
fn format_file(
    path: &str,
    options: Option<&PyOptions>,
    indent: Option<String>,
    currency_column: Option<usize>,
    cost_column: Option<usize>,
    thousands_separator: Option<String>,
    spaces_in_braces: Option<bool>,
    fixed_cjk_width: Option<bool>,
    sort: Option<bool>,
) -> PyResult<String> {
    let opts = resolve_options(
        options,
        indent,
        currency_column,
        cost_column,
        thousands_separator,
        spaces_in_braces,
        fixed_cjk_width,
        sort,
    )?;
    let content = std::fs::read_to_string(path).map_err(|e| {
        PyOSError::new_err(format!("{path}: {e}"))
    })?;
    Ok(crate::format(&content, &opts))
}

/// Recursively format a beancount file and all its includes.
/// Returns a list of (absolute_path, formatted_content) tuples.
#[pyfunction]
#[pyo3(signature = (
    path,
    options = None,
    indent = None,
    currency_column = None,
    cost_column = None,
    thousands_separator = None,
    spaces_in_braces = None,
    fixed_cjk_width = None,
    sort = None,
))]
#[allow(clippy::too_many_arguments)]
fn format_recursive(
    path: &str,
    options: Option<&PyOptions>,
    indent: Option<String>,
    currency_column: Option<usize>,
    cost_column: Option<usize>,
    thousands_separator: Option<String>,
    spaces_in_braces: Option<bool>,
    fixed_cjk_width: Option<bool>,
    sort: Option<bool>,
) -> PyResult<Vec<(String, String)>> {
    use std::path::Path;

    let p = Path::new(path);
    if !p.exists() {
        return Err(PyOSError::new_err(format!(
            "{path}: No such file or directory"
        )));
    }

    let opts = resolve_options(
        options,
        indent,
        currency_column,
        cost_column,
        thousands_separator,
        spaces_in_braces,
        fixed_cjk_width,
        sort,
    )?;

    let results = crate::recursive::format_recursive(p, &opts);
    Ok(results
        .into_iter()
        .map(|f| (f.path.to_string_lossy().into_owned(), f.content))
        .collect())
}

/// A fast beancount file formatter with CJK support.
#[pymodule]
fn husk(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", VERSION)?;
    m.add_class::<PyOptions>()?;
    m.add_function(wrap_pyfunction!(format, m)?)?;
    m.add_function(wrap_pyfunction!(format_file, m)?)?;
    m.add_function(wrap_pyfunction!(format_recursive, m)?)?;
    Ok(())
}
