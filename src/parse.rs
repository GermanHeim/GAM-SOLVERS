use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub enum DataType {
    Integer,
    Float,
    String,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub option: Option<String>,
    pub description: Option<String>,
    pub default: Option<String>,
    pub data_type: Option<DataType>,
}

fn cell_text(el: &scraper::ElementRef) -> String {
    el.text().collect::<String>().trim().to_string()
}

fn infer_type(v: &Option<String>) -> Option<DataType> {
    let val = v.as_ref()?;
    if val.parse::<i64>().is_ok() {
        Some(DataType::Integer)
    } else if val.parse::<f64>().is_ok() {
        Some(DataType::Float)
    } else {
        Some(DataType::String)
    }
}

fn opt(v: Option<&String>) -> Option<String> {
    v.filter(|s| !s.is_empty()).cloned()
}

pub fn parse_solver_options(html: &str) -> Vec<Data> {
    let document = Html::parse_document(html);

    let table_sel = Selector::parse(".markdownTable").unwrap();
    let th_sel = Selector::parse("th").unwrap();
    let td_sel = Selector::parse("td").unwrap();
    let tr_sel = Selector::parse("tr").unwrap();

    let mut results = Vec::new();

    for table in document.select(&table_sel) {
        let headers: Vec<String> = table
            .select(&th_sel)
            .map(|th| cell_text(&th))
            .filter(|s| !s.is_empty())
            .collect();

        let header_lower: Vec<String> = headers.iter().map(|h| h.to_lowercase()).collect();

        let has_option = header_lower.iter().any(|h| h == "option");
        let has_description = header_lower.iter().any(|h| h == "description");
        let has_default = header_lower.iter().any(|h| h == "default");

        if !(has_option && has_description && has_default) {
            continue;
        }

        let opt_idx = header_lower.iter().position(|h| h == "option").unwrap();
        let desc_idx = header_lower.iter().position(|h| h == "description").unwrap();
        let def_idx = header_lower.iter().position(|h| h == "default").unwrap();

        for row in table.select(&tr_sel) {
            if row.select(&th_sel).count() > 0 {
                continue;
            }

            let cells: Vec<String> = row.select(&td_sel).map(|td| cell_text(&td)).collect();

            let max_idx = opt_idx.max(desc_idx.max(def_idx));
            if cells.len() <= max_idx {
                continue;
            }

            let default = opt(cells.get(def_idx));

            results.push(Data {
                option: opt(cells.get(opt_idx)),
                description: opt(cells.get(desc_idx)),
                data_type: infer_type(&default),
                default,
            });
        }
    }

    results
}

fn to_snake_case(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();

    let mut result = String::with_capacity(sanitized.len() + 4);
    let chars: Vec<char> = sanitized.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];
        if c == '_' {
            result.push('_');
        } else if c.is_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                if prev.is_lowercase() || prev == '_' {
                    result.push('_');
                }
            }
            result.push(c.to_ascii_lowercase());
            if i > 0
                && i + 1 < chars.len()
                && chars[i + 1].is_lowercase()
                && chars[i - 1].is_uppercase()
            {
                result.push('_');
            }
        } else {
            result.push(c);
        }
    }

    let collapsed: String = result
        .chars()
        .fold(String::with_capacity(result.len()), |mut acc, c| {
            if c == '_' && acc.ends_with('_') {
                return acc;
            }
            acc.push(c);
            acc
        });

    collapsed.trim_matches('_').to_string()
}

fn escape_keyword(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn",
        "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
        "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
        "self", "Self", "static", "struct", "super", "trait", "true", "type",
        "union", "unsafe", "use", "where", "while",
        "abstract", "become", "box", "do", "final", "macro", "override",
        "priv", "try", "typeof", "unsized", "virtual", "yield",
    ];
    if KEYWORDS.contains(&name) {
        format!("{}_", name)
    } else {
        name.to_string()
    }
}

fn type_name_str(dt: &Option<DataType>) -> &'static str {
    match dt {
        Some(DataType::Integer) => "i64",
        Some(DataType::Float) => "f64",
        Some(DataType::String) | None => "String",
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_snake_basic_pascal() {
        assert_eq!(to_snake_case("CUTnrcuts"), "cut_nrcuts");
        assert_eq!(to_snake_case("ECPmaster"), "ecp_master");
        assert_eq!(to_snake_case("MIPsolver"), "mip_solver");
        assert_eq!(to_snake_case("TOLepsf"), "tol_epsf");
        assert_eq!(to_snake_case("NLPcall"), "nlp_call");
    }

    #[test]
    fn test_snake_all_lowercase() {
        assert_eq!(to_snake_case("reslim"), "reslim");
        assert_eq!(to_snake_case("solvelink"), "solvelink");
        assert_eq!(to_snake_case("solvetrace"), "solvetrace");
    }

    #[test]
    fn test_snake_trailing_acronym() {
        assert_eq!(to_snake_case("MIPoptcr"), "mip_optcr");
        assert_eq!(to_snake_case("TOLoptcr"), "tol_optcr");
        assert_eq!(to_snake_case("MIPoptimaliter"), "mip_optimaliter");
    }

    #[test]
    fn test_snake_with_dots() {
        assert_eq!(to_snake_case("output.debug.path"), "output_debug_path");
        assert_eq!(to_snake_case("subsolver.cplex.work_directory"), "subsolver_cplex_work_directory");
        assert_eq!(to_snake_case("primal.fixed_integer.call_strategy"), "primal_fixed_integer_call_strategy");
    }

    #[test]
    fn test_snake_with_leading_dot() {
        assert_eq!(to_snake_case(".equ_class"), "equ_class");
        assert_eq!(to_snake_case(".feaspref"), "feaspref");
        assert_eq!(to_snake_case(".lazy"), "lazy");
        assert_eq!(to_snake_case(".partition"), "partition");
    }

    #[test]
    fn test_snake_with_spaces() {
        assert_eq!(to_snake_case("central difference interval"), "central_difference_interval");
        assert_eq!(to_snake_case("feasibility tolerance"), "feasibility_tolerance");
        assert_eq!(to_snake_case("crash option"), "crash_option");
    }

    #[test]
    fn test_snake_mixed_case_with_dots() {
        assert_eq!(to_snake_case("dual.mip.solver"), "dual_mip_solver");
        assert_eq!(to_snake_case("dual.esh.interior_point.cutting_plane.time_limit"), "dual_esh_interior_point_cutting_plane_time_limit");
    }

    #[test]
    fn test_snake_consecutive_special_chars() {
        assert_eq!(to_snake_case("a..b"), "a_b");
        assert_eq!(to_snake_case("a  b"), "a_b");
        assert_eq!(to_snake_case("a._.b"), "a_b");
    }

    #[test]
    fn test_snake_already_snake_case() {
        assert_eq!(to_snake_case("cut_generation_epsilon"), "cut_generation_epsilon");
        assert_eq!(to_snake_case("max_number_nodes"), "max_number_nodes");
    }

    #[test]
    fn test_snake_leading_trailing_special() {
        assert_eq!(to_snake_case(".leading"), "leading");
        assert_eq!(to_snake_case("trailing."), "trailing");
        assert_eq!(to_snake_case(".both."), "both");
    }

    #[test]
    fn test_infer_integer() {
        assert!(matches!(infer_type(&Some("0".into())), Some(DataType::Integer)));
        assert!(matches!(infer_type(&Some("50".into())), Some(DataType::Integer)));
        assert!(matches!(infer_type(&Some("-1".into())), Some(DataType::Integer)));
        assert!(matches!(infer_type(&Some("200".into())), Some(DataType::Integer)));
    }

    #[test]
    fn test_infer_float() {
        assert!(matches!(infer_type(&Some("1.3".into())), Some(DataType::Float)));
        assert!(matches!(infer_type(&Some("2.0".into())), Some(DataType::Float)));
        assert!(matches!(infer_type(&Some("1e-3".into())), Some(DataType::Float)));
        assert!(matches!(infer_type(&Some("1e10".into())), Some(DataType::Float)));
        assert!(matches!(infer_type(&Some("1e-6".into())), Some(DataType::Float)));
    }

    #[test]
    fn test_infer_string() {
        assert!(matches!(infer_type(&Some("GAMS MIP solver".into())), Some(DataType::String)));
        assert!(matches!(infer_type(&Some("GAMS optCR".into())), Some(DataType::String)));
        assert!(matches!(infer_type(&Some("Filename".into())), Some(DataType::String)));
    }

    #[test]
    fn test_infer_none() {
        assert!(infer_type(&None).is_none());
    }


    const BASIC_HTML: &str = r#"<table class="markdownTable">
<tr class="markdownTableHead">
<th>Option</th><th>Description</th><th>Default</th>
</tr>
<tr class="markdownTableRowOdd">
<td>CUTnrcuts</td><td>Cut generation pace</td><td><code>0</code></td>
</tr>
<tr class="markdownTableRowEven">
<td>ECPbeta</td><td>Updating multiplier</td><td><code>1.3</code></td>
</tr>
</table>"#;

    const DOTS_HTML: &str = r#"<table class="markdownTable">
<tr class="markdownTableHead">
<th>Option</th><th>Description</th><th>Default</th>
</tr>
<tr class="markdownTableRowOdd">
<td>output.debug.path</td><td>Debug output path</td><td><code>tmp</code></td>
</tr>
<tr class="markdownTableRowEven">
<td>subsolver.cplex.workdir</td><td>CPLEX work dir</td><td><code>.</code></td>
</tr>
</table>"#;

    const LEADING_DOT_HTML: &str = r#"<table class="markdownTable">
<tr class="markdownTableHead">
<th>Option</th><th>Description</th><th>Default</th>
</tr>
<tr class="markdownTableRowOdd">
<td>.lazy</td><td>Lazy constraints</td><td><code>0</code></td>
</tr>
<tr class="markdownTableRowEven">
<td>.feaspref</td><td>Feasibility preference</td><td><code>1</code></td>
</tr>
</table>"#;

    const SPACES_HTML: &str = r#"<table class="markdownTable">
<tr class="markdownTableHead">
<th>Option</th><th>Description</th><th>Default</th>
</tr>
<tr class="markdownTableRowOdd">
<td>central difference interval</td><td>Interval for central differences</td><td><code>1e-8</code></td>
</tr>
<tr class="markdownTableRowEven">
<td>new superbasics limit</td><td>Limit on new superbasics</td><td><code>100</code></td>
</tr>
</table>"#;

    const SKIP_HTML: &str = r#"<table class="markdownTable">
<tr class="markdownTableHead">
<th>value</th><th>meaning</th>
</tr>
<tr class="markdownTableRowOdd">
<td>0</td><td>Off</td>
</tr>
</table>"#;

    const MIXED_HTML: &str = r#"<table class="markdownTable">
<tr class="markdownTableHead">
<th>Option</th><th>Description</th><th>Default</th>
</tr>
<tr class="markdownTableRowOdd">
<td>primal.tolerance.integer</td><td>Integer tolerance</td><td><code>1e-6</code></td>
</tr>
<tr class="markdownTableRowEven">
<td>.dofuncpieceerror</td><td>Do function piece error</td><td><code>1.0</code></td>
</tr>
<tr class="markdownTableRowOdd">
<td>warm start</td><td>Use warm start</td><td><code>GAMS default</code></td>
</tr>
<tr class="markdownTableRowEven">
<td>MIPoptcr</td><td>Relative MIP gap</td><td><code>1.0</code></td>
</tr>
</table>"#;


    #[test]
    fn test_parse_basic() {
        let data = parse_solver_options(BASIC_HTML);
        assert_eq!(data.len(), 2);

        assert_eq!(data[0].option.as_deref(), Some("CUTnrcuts"));
        assert_eq!(data[0].description.as_deref(), Some("Cut generation pace"));
        assert_eq!(data[0].default.as_deref(), Some("0"));
        assert!(matches!(data[0].data_type, Some(DataType::Integer)));

        assert_eq!(data[1].option.as_deref(), Some("ECPbeta"));
        assert_eq!(data[1].default.as_deref(), Some("1.3"));
        assert!(matches!(data[1].data_type, Some(DataType::Float)));
    }

    #[test]
    fn test_parse_dots() {
        let data = parse_solver_options(DOTS_HTML);
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].option.as_deref(), Some("output.debug.path"));
        assert_eq!(data[1].option.as_deref(), Some("subsolver.cplex.workdir"));
    }

    #[test]
    fn test_parse_leading_dot() {
        let data = parse_solver_options(LEADING_DOT_HTML);
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].option.as_deref(), Some(".lazy"));
        assert!(matches!(data[0].data_type, Some(DataType::Integer)));
        assert_eq!(data[1].option.as_deref(), Some(".feaspref"));
        assert!(matches!(data[1].data_type, Some(DataType::Integer)));
    }

    #[test]
    fn test_parse_spaces() {
        let data = parse_solver_options(SPACES_HTML);
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].option.as_deref(), Some("central difference interval"));
        assert!(matches!(data[0].data_type, Some(DataType::Float)));
        assert_eq!(data[1].option.as_deref(), Some("new superbasics limit"));
        assert!(matches!(data[1].data_type, Some(DataType::Integer)));
    }

    #[test]
    fn test_parse_skips_bad_table() {
        let data = parse_solver_options(SKIP_HTML);
        assert!(data.is_empty());
    }

    #[test]
    fn test_parse_mixed() {
        let data = parse_solver_options(MIXED_HTML);
        assert_eq!(data.len(), 4);

        assert_eq!(data[0].option.as_deref(), Some("primal.tolerance.integer"));
        assert!(matches!(data[0].data_type, Some(DataType::Float)));

        assert_eq!(data[1].option.as_deref(), Some(".dofuncpieceerror"));
        assert!(matches!(data[1].data_type, Some(DataType::Float)));

        assert_eq!(data[2].option.as_deref(), Some("warm start"));
        assert!(matches!(data[2].data_type, Some(DataType::String)));

        assert_eq!(data[3].option.as_deref(), Some("MIPoptcr"));
        assert!(matches!(data[3].data_type, Some(DataType::Float)));
    }

    #[test]
    fn test_parse_skip_among_valid() {
        let combined = format!("{}{}{}", BASIC_HTML, SKIP_HTML, DOTS_HTML);
        let data = parse_solver_options(&combined);
        assert_eq!(data.len(), 4);
        assert_eq!(data[0].option.as_deref(), Some("CUTnrcuts"));
        assert_eq!(data[2].option.as_deref(), Some("output.debug.path"));
        assert_eq!(data[3].option.as_deref(), Some("subsolver.cplex.workdir"));
    }

    #[test]
    fn test_parse_default_none() {
        let html = r#"<table class="markdownTable">
<tr class="markdownTableHead">
<th>Option</th><th>Description</th><th>Default</th>
</tr>
<tr class="markdownTableRowOdd">
<td>solvetrace</td><td>Trace file</td><td></td>
</tr>
</table>"#;
        let data = parse_solver_options(html);
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].option.as_deref(), Some("solvetrace"));
        assert!(data[0].default.is_none());
        assert!(data[0].data_type.is_none());
    }

    #[test]
    fn test_escape_keyword() {
        assert_eq!(escape_keyword("continue"), "continue_");
        assert_eq!(escape_keyword("return"), "return_");
        assert_eq!(escape_keyword("match"), "match_");
        assert_eq!(escape_keyword("while"), "while_");
        assert_eq!(escape_keyword("for"), "for_");
        assert_eq!(escape_keyword("fn"), "fn_");
        assert_eq!(escape_keyword("loop"), "loop_");
        assert_eq!(escape_keyword("let"), "let_");
    }

    #[test]
    fn test_escape_non_keyword() {
        assert_eq!(escape_keyword("cut_nrcuts"), "cut_nrcuts");
        assert_eq!(escape_keyword("ecp_beta"), "ecp_beta");
        assert_eq!(escape_keyword("output_debug_path"), "output_debug_path");
        assert_eq!(escape_keyword(""), "");
    }

    #[test]
    fn test_generate_keyword_escaped() {
        let html = r#"<table class="markdownTable">
<tr class="markdownTableHead">
<th>Option</th><th>Description</th><th>Default</th>
</tr>
<tr class="markdownTableRowOdd">
<td>continue</td><td>Continue option</td><td><code>0</code></td>
</tr>
<tr class="markdownTableRowEven">
<td>return</td><td>Return value</td><td><code>1</code></td>
</tr>
<tr class="markdownTableRowOdd">
<td>for</td><td>For option</td><td><code>2</code></td>
</tr>
</table>"#;
        let data = parse_solver_options(html);
        let solvers = [("Test", data)];
        let generated = generate_all_rs(&solvers);

        assert!(generated.contains("pub continue_: Option<i64>"), "continue should be escaped: {}", generated);
        assert!(generated.contains("pub return_: Option<i64>"));
        assert!(generated.contains("pub for_: Option<i64>"));
        // raw keywords should NOT appear
        assert!(!generated.contains("pub continue: "));
        assert!(!generated.contains("pub return: "));
        assert!(!generated.contains("pub for: "));
    }

    #[test]
    fn test_parse_continue_keyword() {
        let html = r#"<table class="markdownTable">
<tr class="markdownTableHead">
<th>Option</th><th>Description</th><th>Default</th>
</tr>
<tr class="markdownTableRowOdd">
<td>continue</td><td>A continuation option</td><td><code>0</code></td>
</tr>
</table>"#;
        let data = parse_solver_options(html);
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].option.as_deref(), Some("continue"));
        assert_eq!(data[0].default.as_deref(), Some("0"));
        assert!(matches!(data[0].data_type, Some(DataType::Integer)));
    }

    #[test]
    fn test_generate_nested_all_valid() {
        let data = parse_solver_options(MIXED_HTML);
        let solvers = [("SHOT", data)];
        let generated = generate_all_rs(&solvers);

        assert!(generated.contains("pub struct SHOT"));
        assert!(generated.contains("pub primal_tolerance_integer: Option<f64>"));
        assert!(generated.contains("pub dofuncpieceerror: Option<f64>"));
        assert!(generated.contains("pub warm_start: Option<String>"));
        assert!(generated.contains("pub mip_optcr: Option<f64>"));

        // no invalid identifiers
        assert!(!generated.contains("pub ."));
        assert!(!generated.contains("pub  "));
    }
}

pub fn generate_all_rs(solvers: &[(&str, Vec<Data>)]) -> String {
    let mut out = String::new();
    out.push_str("// Auto-generated by gans-scraper\n");
    out.push_str("#![allow(non_camel_case_types, dead_code)]\n\n");

    for (solver_name, options) in solvers {
        if options.is_empty() {
            continue;
        }
        out.push_str(&format!("pub struct {} {{\n", solver_name));
        for data in options {
            let field_name = data
                .option
                .as_deref()
                .map(to_snake_case)
                .map(|s| escape_keyword(&s))
                .unwrap_or_else(|| "unknown".to_string());
            let type_name = type_name_str(&data.data_type);
            out.push_str(&format!("    pub {}: Option<{}>,\n", field_name, type_name));
        }
        out.push_str("}\n\n");
    }

    out
}
