// pub const SOLVERS: [&str; 1] = [
//     "BARON",
//     // "GUROBI",
//     // "HIGHS"
// ];


pub const BASE_URL: &str = "https://www.gams.com/latest/docs/";

#[macro_export]
macro_rules! supported_solvers {
    ($($name:ident => $url_name:expr);+ $(;)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq,)]
        pub enum SupportedSolver {
            $($name),+
        }

        impl SupportedSolver {
            pub const ALL: &'static [SupportedSolver] = &[
                $(SupportedSolver::$name),+
            ];

            pub fn url_name(&self) -> &'static str {
                match self {
                    $(SupportedSolver::$name => $url_name),+
                }
            }

        }
    };
}


