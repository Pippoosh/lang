use std::collections::HashMap;
use std::io::Write;
use rand::Rng;

mod compiler;
use compiler::Compiler;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Numbers and Identifiers
    Number(f64),
    Identifier(String),
    
    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    Equals,
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,
    NotEqual,
    
    // Brackets and Separators
    LParen,
    RParen,
    Comma,
    Semicolon,
    Colon,
    
    // Keywords
    Let,
    Print,
    Input,
    If,
    Then,
    Else,
    For,
    To,
    Step,
    Next,
    Goto,
    Gosub,
    Return,
    Rem,
    End,
    Stop,
    Dim,
    Read,
    Data,
    Restore,
    
    // Built-in Functions
    Abs,
    Rnd,
    Int,
    Sqr,
    Sin,
    Cos,
    Tan,
    Log,
    Exp,
    Len,
    Mid,
    Left,
    Right,
    
    // Special
    LineNumber(u32),
    String(String),
    EOL,
    EOF,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Expression {
    Number(f64),
    String(String),
    Variable(String),
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ForLoop {
    variable: String,
    start: Expression,
    end: Expression,
    step: Expression,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Statement {
    Let {
        variable: String,
        expression: Expression,
    },
    Print {
        expressions: Vec<Expression>,
        semicolon: bool,
    },
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Input {
        variable: String,
    },
    For {
        loop_data: ForLoop,
    },
    Next {
        variable: String,
    },
    End,
    Goto(u32),
    Rem(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Line {
    number: u32,
    statement: Statement,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Program {
    lines: Vec<Line>,
}

impl Program {
    fn new() -> Self {
        Program {
            lines: Vec::new(),
        }
    }
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\r' => {
                chars.next();
            }
            '\n' => {
                tokens.push(Token::EOL);
                chars.next();
            }
            '0'..='9' => {
                let mut number = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_digit(10) || c == '.' {
                        number.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Ok(n) = number.parse::<f64>() {
                    tokens.push(Token::Number(n));
                }
            }
            'A'..='Z' | 'a'..='z' | '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c.to_ascii_uppercase());
                        chars.next();
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "LET" => tokens.push(Token::Let),
                    "PRINT" => tokens.push(Token::Print),
                    "IF" => tokens.push(Token::If),
                    "THEN" => tokens.push(Token::Then),
                    "ELSE" => tokens.push(Token::Else),
                    "FOR" => tokens.push(Token::For),
                    "TO" => tokens.push(Token::To),
                    "STEP" => tokens.push(Token::Step),
                    "NEXT" => tokens.push(Token::Next),
                    "END" => tokens.push(Token::End),
                    "INPUT" => tokens.push(Token::Input),
                    _ => tokens.push(Token::Identifier(ident)),
                }
            }
            '"' => {
                chars.next();
                let mut string = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next();
                        break;
                    }
                    string.push(c);
                    chars.next();
                }
                tokens.push(Token::String(string));
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Multiply);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Divide);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Power);
                chars.next();
            }
            '=' => {
                tokens.push(Token::Equals);
                chars.next();
            }
            '<' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    tokens.push(Token::LessOrEqual);
                    chars.next();
                } else if let Some(&'>') = chars.peek() {
                    tokens.push(Token::NotEqual);
                    chars.next();
                } else {
                    tokens.push(Token::LessThan);
                }
            }
            '>' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    tokens.push(Token::GreaterOrEqual);
                    chars.next();
                } else {
                    tokens.push(Token::GreaterThan);
                }
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            ';' => {
                tokens.push(Token::Semicolon);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            _ => {
                chars.next();
            }
        }
    }

    // If the last token isn't EOL, add one
    if !tokens.is_empty() && !matches!(tokens.last(), Some(Token::EOL)) {
        tokens.push(Token::EOL);
    }
    tokens.push(Token::EOF);
    tokens
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1)
    }

    fn match_token(&mut self, expected: &[Token]) -> bool {
        if let Some(token) = self.peek() {
            if expected.contains(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn parse_program(&mut self) -> Program {
        let mut program = Program::new();
        let mut line_number = 0;

        while let Some(token) = self.peek() {
            match token {
                Token::EOL => {
                    self.advance();
                },
                Token::EOF => {
                    break;
                },
                _ => {
                    program.lines.push(Line {
                        number: line_number,
                        statement: self.parse_statement(),
                    });
                    line_number += 1;

                    // Consume any EOL after the statement
                    if let Some(Token::EOL) = self.peek() {
                        self.advance();
                    }
                }
            }
        }

        program
    }

    fn parse_statement(&mut self) -> Statement {
        let token = self.peek().cloned();
        match token {
            Some(Token::Let) => {
                self.advance();
                self.parse_let()
            },
            Some(Token::Print) => {
                self.advance();
                self.parse_print()
            },
            Some(Token::If) => {
                self.advance();
                self.parse_if()
            },
            Some(Token::For) => {
                self.advance();
                self.parse_for()
            },
            Some(Token::Input) => {
                self.advance();
                if let Some(Token::Identifier(name)) = self.advance().cloned() {
                    Statement::Input {
                        variable: name,
                    }
                } else {
                    panic!("Expected variable name after INPUT")
                }
            },
            Some(Token::Next) => {
                self.advance();
                if let Some(Token::Identifier(name)) = self.advance().cloned() {
                    Statement::Next {
                        variable: name,
                    }
                } else {
                    panic!("Expected variable name after NEXT")
                }
            },
            Some(Token::End) => {
                self.advance();
                Statement::End
            },
            Some(Token::Identifier(name)) => {
                self.advance();
                // Check for function call
                if let Some(Token::LParen) = self.peek() {
                    self.advance(); // consume (
                    let mut args = Vec::new();
                    loop {
                        if let Some(Token::RParen) = self.peek() {
                            self.advance();
                            break;
                        }
                        args.push(self.parse_expression());
                        if let Some(Token::Comma) = self.peek() {
                            self.advance();
                        } else if let Some(Token::RParen) = self.peek() {
                            self.advance();
                            break;
                        } else {
                            panic!("Expected ',' or ')' in function call");
                        }
                    }
                    Statement::Let {
                        variable: name.clone(),
                        expression: Expression::FunctionCall {
                            name,
                            arguments: args,
                        },
                    }
                } else if let Some(Token::Equals) = self.peek() {
                    self.advance();
                    Statement::Let {
                        variable: name,
                        expression: self.parse_expression(),
                    }
                } else {
                    panic!("Expected = after variable name")
                }
            },
            Some(token) => panic!("Unexpected token in statement: {:?}", token),
            None => panic!("Unexpected end of input"),
        }
    }

    fn parse_let(&mut self) -> Statement {
        let var_name = match self.advance() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => panic!("Expected variable name after LET"),
        };

        if !self.match_token(&[Token::Equals]) {
            panic!("Expected '=' after variable name in LET");
        }

        let expr = self.parse_expression();
        Statement::Let {
            variable: var_name,
            expression: expr,
        }
    }

    fn parse_print(&mut self) -> Statement {
        let mut expressions = Vec::new();
        let mut semicolon = false;

        while let Some(token) = self.peek() {
            match token {
                Token::Semicolon => {
                    semicolon = true;
                    self.advance();
                    break;
                }
                Token::EOL => break,
                _ => {
                    expressions.push(self.parse_expression());
                    if let Some(Token::Comma) = self.peek() {
                        self.advance();
                    }
                }
            }
        }

        Statement::Print {
            expressions,
            semicolon,
        }
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Expression {
        let mut expr = self.parse_additive();
        
        while let Some(token) = self.peek() {
            match token {
                Token::LessThan | Token::GreaterThan | Token::Equals | 
                Token::LessOrEqual | Token::GreaterOrEqual | Token::NotEqual => {
                    let operator = self.advance().unwrap().clone();
                    let right = self.parse_additive();
                    expr = Expression::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        
        expr
    }

    fn parse_additive(&mut self) -> Expression {
        let mut expr = self.parse_multiplicative();

        while let Some(token) = self.peek() {
            match token {
                Token::Plus | Token::Minus => {
                    let op = self.advance().unwrap().clone();
                    let right = self.parse_multiplicative();
                    expr = Expression::Binary {
                        left: Box::new(expr),
                        operator: op,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_multiplicative(&mut self) -> Expression {
        let mut expr = self.parse_power();

        while let Some(token) = self.peek() {
            match token {
                Token::Multiply | Token::Divide => {
                    let op = self.advance().unwrap().clone();
                    let right = self.parse_power();
                    expr = Expression::Binary {
                        left: Box::new(expr),
                        operator: op,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_power(&mut self) -> Expression {
        let mut expr = self.parse_primary();

        while let Some(token) = self.peek() {
            match token {
                Token::Power => {
                    let operator = self.advance().unwrap().clone();
                    let right = self.parse_primary();
                    expr = Expression::Binary {
                        left: Box::new(expr),
                        operator,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_primary(&mut self) -> Expression {
        match self.advance().cloned() {
            Some(Token::Number(n)) => Expression::Number(n),
            Some(Token::String(s)) => Expression::String(s),
            Some(Token::Identifier(name)) => {
                // Check for function call
                if let Some(Token::LParen) = self.peek() {
                    self.advance(); // consume (
                    let mut args = Vec::new();
                    loop {
                        if let Some(Token::RParen) = self.peek() {
                            self.advance();
                            break;
                        }
                        args.push(self.parse_expression());
                        if let Some(Token::Comma) = self.peek() {
                            self.advance();
                        } else if let Some(Token::RParen) = self.peek() {
                            self.advance();
                            break;
                        } else {
                            panic!("Expected ',' or ')' in function call");
                        }
                    }
                    Expression::FunctionCall {
                        name,
                        arguments: args,
                    }
                } else {
                    Expression::Variable(name)
                }
            },
            Some(Token::LParen) => {
                let expr = self.parse_expression();
                if !self.match_token(&[Token::RParen]) {
                    panic!("Expected closing parenthesis");
                }
                expr
            },
            Some(token) => panic!("Unexpected token in expression: {:?}", token),
            None => panic!("Unexpected end of input"),
        }
    }

    fn parse_if(&mut self) -> Statement {
        let condition = self.parse_expression();
        
        if !self.match_token(&[Token::Then]) {
            panic!("Expected THEN after IF condition");
        }

        let then_stmt = Box::new(self.parse_statement());
        let else_stmt = if self.match_token(&[Token::Else]) {
            Some(Box::new(self.parse_statement()))
        } else {
            None
        };

        Statement::If {
            condition,
            then_branch: then_stmt,
            else_branch: else_stmt,
        }
    }

    fn parse_for(&mut self) -> Statement {
        let var_name = match self.advance() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => panic!("Expected variable name after FOR"),
        };

        if !self.match_token(&[Token::Equals]) {
            panic!("Expected '=' after variable name in FOR statement");
        }

        let start = self.parse_expression();

        if !self.match_token(&[Token::To]) {
            panic!("Expected TO in FOR statement");
        }

        let end = self.parse_expression();

        let step = if self.match_token(&[Token::Step]) {
            self.parse_expression()
        } else {
            Expression::Number(1.0)
        };

        Statement::For {
            loop_data: ForLoop {
                variable: var_name,
                start,
                end,
                step,
            },
        }
    }
}

struct Interpreter {
    variables: HashMap<String, f64>,
    loops: Vec<ForLoop>,
    loop_stack: Vec<usize>,
    current_line: usize,
    running: bool,
    program: Program,
}

impl Interpreter {
    fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
            loops: Vec::new(),
            loop_stack: Vec::new(),
            current_line: 0,
            running: true,
            program: Program::new(),
        }
    }

    fn execute_program(&mut self, program: Program) -> Result<(), String> {
        self.program = program;
        self.current_line = 0;
        self.running = true;
        
        while self.running && self.current_line < self.program.lines.len() {
            let line = &self.program.lines[self.current_line].clone();
            match self.execute_statement(line.statement.clone()) {
                Ok(_) => {
                    self.current_line += 1;
                },
                Err(e) => return Err(format!("Error at line {}: {}", self.current_line, e)),
            }
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: Statement) -> Result<(), String> {
        match statement {
            Statement::Print { expressions, semicolon } => {
                for (i, expr) in expressions.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    match self.evaluate_expression(expr)? {
                        Value::Number(n) => print!("{}", n),
                        Value::String(s) => print!("{}", s),
                    }
                }
                if !semicolon {
                    println!();
                }
                std::io::stdout().flush().unwrap();
                Ok(())
            },
            Statement::Let { variable, expression } => {
                let value = self.evaluate_expression(&expression)?;
                match value {
                    Value::Number(n) => {
                        self.variables.insert(variable, n);
                        Ok(())
                    },
                    Value::String(_) => Err("Can only store numbers in variables".to_string()),
                }
            },
            Statement::If { condition, then_branch, else_branch } => {
                let value = self.evaluate_expression(&condition)?;
                match value {
                    Value::Number(n) => {
                        if n != 0.0 {
                            self.execute_statement(*then_branch)
                        } else if let Some(else_stmt) = else_branch {
                            self.execute_statement(*else_stmt)
                        } else {
                            Ok(())
                        }
                    },
                    Value::String(_) => Err("Condition must evaluate to a number".to_string()),
                }
            },
            Statement::Input { variable } => {
                print!("Enter {}: ", variable);
                std::io::stdout().flush().unwrap();
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        match input.trim().parse::<f64>() {
                            Ok(n) => {
                                self.variables.insert(variable, n);
                                Ok(())
                            },
                            Err(_) => Err("Invalid number input".to_string()),
                        }
                    },
                    Err(e) => Err(format!("Failed to read input: {}", e)),
                }
            },
            Statement::For { loop_data } => {
                let start = self.evaluate_expression(&loop_data.start)?;
                let end = self.evaluate_expression(&loop_data.end)?;
                let step = self.evaluate_expression(&loop_data.step)?;
                
                match (start, end, step) {
                    (Value::Number(start), Value::Number(end), Value::Number(step)) => {
                        self.variables.insert(loop_data.variable.clone(), start);
                        self.loops.push(loop_data);
                        self.loop_stack.push(self.current_line);
                        Ok(())
                    },
                    _ => Err("Loop bounds must be numbers".to_string()),
                }
            },
            Statement::Next { variable } => {
                if let Some(loop_data) = self.loops.last() {
                    if loop_data.variable != variable {
                        return Err(format!("NEXT {} doesn't match FOR {}", variable, loop_data.variable));
                    }
                    
                    let current = *self.variables.get(&variable).unwrap();
                    let step = match self.evaluate_expression(&loop_data.step)? {
                        Value::Number(n) => n,
                        _ => return Err("Step must be a number".to_string()),
                    };
                    let next_val = current + step;
                    
                    let end = match self.evaluate_expression(&loop_data.end)? {
                        Value::Number(n) => n,
                        _ => return Err("End must be a number".to_string()),
                    };
                    
                    if (step > 0.0 && next_val <= end) || (step < 0.0 && next_val >= end) {
                        self.variables.insert(variable.clone(), next_val);
                        if let Some(&loop_start) = self.loop_stack.last() {
                            self.current_line = loop_start;
                            Ok(())
                        } else {
                            Err("Loop start not found".to_string())
                        }
                    } else {
                        self.loops.pop();
                        self.loop_stack.pop();
                        Ok(())
                    }
                } else {
                    Err("NEXT without FOR".to_string())
                }
            },
            Statement::End => {
                self.running = false;
                Ok(())
            },
            _ => Err("Statement not implemented yet".to_string()),
        }
    }

    fn evaluate_expression(&self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Number(n) => Ok(Value::Number(*n)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Variable(name) => {
                self.variables.get(name)
                    .map(|&n| Value::Number(n))
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            },
            Expression::Binary { left, operator, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                
                match (left_val, operator, right_val) {
                    (Value::Number(l), Token::Plus, Value::Number(r)) => Ok(Value::Number(l + r)),
                    (Value::Number(l), Token::Minus, Value::Number(r)) => Ok(Value::Number(l - r)),
                    (Value::Number(l), Token::Multiply, Value::Number(r)) => Ok(Value::Number(l * r)),
                    (Value::Number(l), Token::Divide, Value::Number(r)) => {
                        if r == 0.0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(Value::Number(l / r))
                        }
                    },
                    (Value::Number(l), Token::Power, Value::Number(r)) => Ok(Value::Number(l.powf(r))),
                    (Value::Number(l), Token::LessThan, Value::Number(r)) => Ok(Value::Number(if l < r { 1.0 } else { 0.0 })),
                    (Value::Number(l), Token::GreaterThan, Value::Number(r)) => Ok(Value::Number(if l > r { 1.0 } else { 0.0 })),
                    (Value::Number(l), Token::Equals, Value::Number(r)) => Ok(Value::Number(if l == r { 1.0 } else { 0.0 })),
                    (Value::Number(l), Token::LessOrEqual, Value::Number(r)) => Ok(Value::Number(if l <= r { 1.0 } else { 0.0 })),
                    (Value::Number(l), Token::GreaterOrEqual, Value::Number(r)) => Ok(Value::Number(if l >= r { 1.0 } else { 0.0 })),
                    (Value::Number(l), Token::NotEqual, Value::Number(r)) => Ok(Value::Number(if l != r { 1.0 } else { 0.0 })),
                    _ => Err("Invalid operation or type mismatch".to_string()),
                }
            },
            Expression::FunctionCall { name, arguments } => {
                match name.as_str() {
                    "ABS" => {
                        let arg = self.evaluate_expression(&arguments[0])?;
                        match arg {
                            Value::Number(n) => Ok(Value::Number(n.abs())),
                            _ => Err("ABS requires a number argument".to_string()),
                        }
                    },
                    "SQR" => {
                        let arg = self.evaluate_expression(&arguments[0])?;
                        match arg {
                            Value::Number(n) => {
                                if n < 0.0 {
                                    Err("Cannot take square root of negative number".to_string())
                                } else {
                                    Ok(Value::Number(n.sqrt()))
                                }
                            },
                            _ => Err("SQR requires a number argument".to_string()),
                        }
                    },
                    "SIN" => {
                        let arg = self.evaluate_expression(&arguments[0])?;
                        match arg {
                            Value::Number(n) => Ok(Value::Number(n.sin())),
                            _ => Err("SIN requires a number argument".to_string()),
                        }
                    },
                    "COS" => {
                        let arg = self.evaluate_expression(&arguments[0])?;
                        match arg {
                            Value::Number(n) => Ok(Value::Number(n.cos())),
                            _ => Err("COS requires a number argument".to_string()),
                        }
                    },
                    "TAN" => {
                        let arg = self.evaluate_expression(&arguments[0])?;
                        match arg {
                            Value::Number(n) => Ok(Value::Number(n.tan())),
                            _ => Err("TAN requires a number argument".to_string()),
                        }
                    },
                    "RND" => Ok(Value::Number(rand::thread_rng().gen())),
                    "INT" => {
                        let arg = self.evaluate_expression(&arguments[0])?;
                        match arg {
                            Value::Number(n) => Ok(Value::Number(n.floor())),
                            _ => Err("INT requires a number argument".to_string()),
                        }
                    },
                    _ => Err(format!("Unknown function: {}", name)),
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
enum Value {
    Number(f64),
    String(String),
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let should_compile = args.len() > 1 && args[1] == "--compile";

    println!("Reading BASIC code from code.bs...");
    let contents = std::fs::read_to_string("code.bs")
        .map_err(|e| format!("Error reading file: {}", e))?;

    let mut tokens = tokenize(&contents);
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program();

    if should_compile {
        println!("Compiling to Rust code...");
        let mut compiler = Compiler::new();
        let rust_code = compiler.compile_program(&program);
        
        // Write Rust code to a temporary file
        std::fs::write("temp.rs", rust_code)
            .map_err(|e| format!("Error writing Rust code: {}", e))?;
        
        // Compile the Rust code
        println!("Compiling to executable...");
        let output = std::process::Command::new("rustc")
            .args(&["temp.rs", "-o", "code.exe"])
            .output()
            .map_err(|e| format!("Failed to run rustc: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Compilation failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        // Clean up temporary file
        std::fs::remove_file("temp.rs")
            .map_err(|e| format!("Error removing temporary file: {}", e))?;
        
        println!("Successfully compiled to code.exe!");
    } else {
        let mut interpreter = Interpreter::new();
        interpreter.execute_program(program)?;
        println!("\nProgram execution completed.");
    }
    
    Ok(())
}