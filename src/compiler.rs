use std::collections::HashMap;

pub struct Compiler {
    temp_vars: usize,
    indent_level: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            temp_vars: 0,
            indent_level: 0,
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn next_temp(&mut self) -> String {
        self.temp_vars += 1;
        format!("temp_{}", self.temp_vars)
    }

    pub fn compile_program(&mut self, program: &crate::Program) -> String {
        let mut output = String::new();
        
        // Add necessary imports and main function
        output.push_str("use std::io::{self, Write};\n\n");
        output.push_str("fn main() {\n");
        self.indent_level += 1;
        
        // Add variables hashmap
        output.push_str(&self.indent());
        output.push_str("let mut variables: HashMap<String, f64> = HashMap::new();\n");
        
        // Compile each statement
        for line in &program.lines {
            output.push_str(&self.compile_statement(&line.statement));
        }
        
        self.indent_level -= 1;
        output.push_str("}\n");
        
        format!(
            r#"use std::collections::HashMap;
{}
"#,
            output
        )
    }

    fn compile_statement(&mut self, statement: &crate::Statement) -> String {
        let mut output = String::new();
        match statement {
            crate::Statement::Print { expressions, semicolon } => {
                for (i, expr) in expressions.iter().enumerate() {
                    if i > 0 {
                        output.push_str(&self.indent());
                        output.push_str("print!(\" \");\n");
                    }
                    output.push_str(&self.indent());
                    output.push_str(&format!("print!(\"{{}}\", {});\n", self.compile_expression(expr)));
                }
                if !semicolon {
                    output.push_str(&self.indent());
                    output.push_str("println!();\n");
                }
            },
            crate::Statement::Let { variable, expression } => {
                output.push_str(&self.indent());
                output.push_str(&format!(
                    "variables.insert(\"{}\".to_string(), {});\n",
                    variable,
                    self.compile_expression(expression)
                ));
            },
            crate::Statement::Input { variable } => {
                output.push_str(&self.indent());
                output.push_str(&format!("print!(\"Enter {}: \");\n", variable));
                output.push_str(&self.indent());
                output.push_str("io::stdout().flush().unwrap();\n");
                output.push_str(&self.indent());
                output.push_str("let mut input = String::new();\n");
                output.push_str(&self.indent());
                output.push_str("io::stdin().read_line(&mut input).unwrap();\n");
                output.push_str(&self.indent());
                output.push_str(&format!(
                    "variables.insert(\"{0}\".to_string(), input.trim().parse::<f64>().unwrap());\n",
                    variable
                ));
            },
            crate::Statement::For { loop_data } => {
                let start = self.compile_expression(&loop_data.start);
                let end = self.compile_expression(&loop_data.end);
                let step = self.compile_expression(&loop_data.step);
                let var = &loop_data.variable;
                
                output.push_str(&self.indent());
                output.push_str(&format!(
                    "let mut {} = {};\n",
                    var, start
                ));
                output.push_str(&self.indent());
                output.push_str(&format!(
                    "while {} <= {} {{\n",
                    var, end
                ));
                
                self.indent_level += 1;
                output.push_str(&self.indent());
                output.push_str(&format!(
                    "variables.insert(\"{}\".to_string(), {});\n",
                    var, var
                ));
            },
            crate::Statement::Next { variable } => {
                output.push_str(&self.indent());
                output.push_str(&format!(
                    "{} += 1.0;\n",
                    variable
                ));
                self.indent_level -= 1;
                output.push_str(&self.indent());
                output.push_str("}\n");
            },
            crate::Statement::End => {
                output.push_str(&self.indent());
                output.push_str("return;\n");
            },
            _ => panic!("Statement not implemented for compilation"),
        }
        output
    }

    fn compile_expression(&mut self, expr: &crate::Expression) -> String {
        match expr {
            crate::Expression::Number(n) => format!("{:.1}", n),
            crate::Expression::String(s) => format!("\"{}\"", s),
            crate::Expression::Variable(name) => {
                format!("*variables.get(\"{}\").unwrap()", name)
            },
            crate::Expression::Binary { left, operator, right } => {
                let left = self.compile_expression(left);
                let right = self.compile_expression(right);
                match operator {
                    crate::Token::Plus => format!("({} + {})", left, right),
                    crate::Token::Minus => format!("({} - {})", left, right),
                    crate::Token::Multiply => format!("({} * {})", left, right),
                    crate::Token::Divide => format!("({} / {})", left, right),
                    crate::Token::Power => format!("({}).powf({})", left, right),
                    crate::Token::LessThan => format!("if {} < {} {{ 1.0 }} else {{ 0.0 }}", left, right),
                    crate::Token::GreaterThan => format!("if {} > {} {{ 1.0 }} else {{ 0.0 }}", left, right),
                    crate::Token::Equals => format!("if {} == {} {{ 1.0 }} else {{ 0.0 }}", left, right),
                    crate::Token::LessOrEqual => format!("if {} <= {} {{ 1.0 }} else {{ 0.0 }}", left, right),
                    crate::Token::GreaterOrEqual => format!("if {} >= {} {{ 1.0 }} else {{ 0.0 }}", left, right),
                    crate::Token::NotEqual => format!("if {} != {} {{ 1.0 }} else {{ 0.0 }}", left, right),
                    _ => panic!("Operator not implemented for compilation"),
                }
            },
            crate::Expression::FunctionCall { name, arguments } => {
                let args: Vec<String> = arguments.iter()
                    .map(|arg| self.compile_expression(arg))
                    .collect();
                match name.as_str() {
                    "ABS" => format!("({}).abs()", args[0]),
                    "SQR" => format!("({}).sqrt()", args[0]),
                    "SIN" => format!("({}).sin()", args[0]),
                    "COS" => format!("({}).cos()", args[0]),
                    "TAN" => format!("({}).tan()", args[0]),
                    "INT" => format!("({}).floor()", args[0]),
                    "RND" => "rand::random::<f64>()".to_string(),
                    _ => panic!("Function not implemented for compilation"),
                }
            },
        }
    }
}
