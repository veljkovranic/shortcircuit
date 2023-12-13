use pest::Parser;
use std::process;
use std::collections::HashMap;


#[derive(Parser)]
#[grammar = "lib/parser/expression_grammar.pest"]
pub struct ExpressionParser;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum Operator {
    BinOp(BinOp),
    LogicalOp(LogicalOp),
    BitwiseOp(BitwiseOp),
    UnOp(UnOp),
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponent,
    Quotient,
    Modulo
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum BitwiseOp {
    And,
    Or,
    ShiftRight,
    ShiftLeft,
    Xor
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum LogicalOp{
    And,
    Or,
    NotEqual,
    Equal,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum UnOp {
    Negate, // For -x
    Not, // For 
    BitwiseNot, // For ~x
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Variable{
    id: String,
    indexing: Vec<Expr>,
    sub_variable: Option<Box<Variable>>
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum Expr {
    Number(u32),
    ComplexVariable(Variable),
    BinaryOperation {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    LogicalOperation {
        left: Box<Expr>,
        op: LogicalOp,
        right: Box<Expr>,
    },
    BitwiseOperation {
        left: Box<Expr>,
        op: BitwiseOp,
        right: Box<Expr>,
    },
    UnaryOperation {
        op: UnOp,
        expr: Box<Expr>,
    },
    Conditional {
        condition: Box<Expr>,
        true_value: Box<Expr>,
        false_value: Box<Expr>,
    },
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum Stmt {
    Assign {
        target: Expr,
        value: Expr,
    },
    ConditionalAssign {
        condition: Expr,
        true_value: Expr,
        false_value: Expr,
    },
    RegularExpr(Expr),
}

pub fn evaluate(exp: Expr, heap: &HashMap<String, u32>) -> u32 {
    match exp {
        Expr::Number(value) => {
            return value.clone();
        },
        Expr::ComplexVariable(var_name) => {
            match heap.get(&var_name.id) {
                Some(value) => {
                    return value.clone();
                },
                None => {
                    println!("Variable {:?} not initialized", var_name);
                }
            }
        },
        _ => {

        }
    }
    0 as u32
}

pub fn parse_operation(pairs: pest::iterators::Pair<Rule>) -> Operator {
    let rule: Rule = pairs.as_rule();
    let span: pest::Span = pairs.as_span();
    let inner_pairs: Vec<pest::iterators::Pair<Rule>> = pairs.into_inner().into_iter().collect();

    // println!("{:?}", rule);
    match rule {
        Rule::ternary_expr | Rule::logical_expr | Rule::arith_expr | Rule::term | Rule::exponent | Rule::bitwise_expr | Rule::shift_expr | Rule::unary_expr | Rule::factor | Rule::complex_variable | Rule::identifier | Rule::number=> {
            return parse_operation(inner_pairs[0].clone());
        },
        Rule::logical_op => {
            match span.as_str() {
                "!=" => { return Operator::LogicalOp(LogicalOp::NotEqual);},
                "&&" => { return Operator::LogicalOp(LogicalOp::And);},
                "||" => { return Operator::LogicalOp(LogicalOp::Or);},
                "==" => { return Operator::LogicalOp(LogicalOp::Equal);},
                "<" => { return Operator::LogicalOp(LogicalOp::LessThan);},
                "<=" => { return Operator::LogicalOp(LogicalOp::LessThanOrEqual);},
                ">" => { return Operator::LogicalOp(LogicalOp::GreaterThan);},
                ">=" => { return Operator::LogicalOp(LogicalOp::GreaterThanOrEqual);},
                _ => {}
            }
        },
        Rule::add_op | Rule::multiply_op | Rule::exponent_op => {
            match span.as_str() {
                "+" => { return Operator::BinOp(BinOp::Add);},
                "-" => { return Operator::BinOp(BinOp::Subtract);},
                "*" => { return Operator::BinOp(BinOp::Multiply);},
                "/" => { return Operator::BinOp(BinOp::Divide);},
                "**" => { return Operator::BinOp(BinOp::Exponent);},
                "\\" => { return Operator::BinOp(BinOp::Quotient);},
                "%" => { return Operator::BinOp(BinOp::Modulo);},
                _ => {}
            }
        },
        Rule::shift_op | Rule::bitwise_op => {
            match span.as_str() {
                ">>" => { return Operator::BitwiseOp(BitwiseOp::ShiftRight);},
                "<<" => { return Operator::BitwiseOp(BitwiseOp::ShiftLeft);},
                "|" => { return Operator::BitwiseOp(BitwiseOp::Or);},
                "&" => { return Operator::BitwiseOp(BitwiseOp::And);},
                "^" => { return Operator::BitwiseOp(BitwiseOp::Xor);},
                _ => {}
            }
        },
        Rule::unary_op => {
            match span.as_str() {
                "!" => { return Operator::UnOp(UnOp::Not);},
                "-" => { return Operator::UnOp(UnOp::Negate);},
                "~" => { return Operator::UnOp(UnOp::BitwiseNot);},
                _ => {}
            }
        },
        _ => {
            println!("something else");
        }
    };             
    Operator::BinOp(BinOp::Add)
}

pub fn parse_expression(pairs: pest::iterators::Pair<Rule>) -> Expr {
    let mut heap = HashMap::<String, u32>::new();
    heap.insert("in".to_string(), 1 as u32);

    let rule: Rule = pairs.as_rule();
    let span: pest::Span = pairs.as_span();
    let inner_pairs: Vec<pest::iterators::Pair<Rule>> = pairs.into_inner().into_iter().collect();
    // println!("{:?}", rule);
    match rule {
        Rule::ternary_expr => {
            if inner_pairs.len() != 3 {
                // println!("not real ternary");
                return parse_expression(inner_pairs[0].clone());
            } else {
                return Expr::Conditional{
                    condition: Box::new(parse_expression(inner_pairs[0].clone())),
                    true_value: Box::new(parse_expression(inner_pairs[1].clone())),
                    false_value: Box::new(parse_expression(inner_pairs[2].clone()))
                };
            }
        },
        Rule::logical_expr => {
            if inner_pairs.len() != 3 {
                // println!("not real logical");
                return parse_expression(inner_pairs[0].clone());
            } else {
                let op_raw = parse_operation(inner_pairs[1].clone());
                let mut op = LogicalOp::And;
                match op_raw {
                    Operator::LogicalOp(lop) => {op = lop;},
                    _ => {}
                };
                return  Expr::LogicalOperation{
                    left: Box::new(parse_expression(inner_pairs[0].clone())),
                    op: op,
                    right: Box::new(parse_expression(inner_pairs[2].clone()))
                };
            }
        },
        Rule::arith_expr => {
            if inner_pairs.len() == 1 {
                return parse_expression(inner_pairs[0].clone());
            } else {
                let mut op_index = 1;
                let mut res : Expr = Expr::Number(0);
                while op_index < inner_pairs.len() - 1 {
                    let op_raw = parse_operation(inner_pairs[op_index].clone());
                    let mut op: BinOp = BinOp::Add;
                    match op_raw {
                        Operator::BinOp(lop) => {op = lop;},
                        _ => {}
                    };
                    if (op_index == 1) {
                        res = parse_expression(inner_pairs[op_index-1].clone())
                    }
                    res = Expr::BinaryOperation{
                        left: Box::new(res),
                        op: op,
                        right: Box::new(parse_expression(inner_pairs[op_index+1].clone()))
                    };
                    op_index = op_index + 2;
                }
                return res;
            }
        },
        Rule::term => {
            if inner_pairs.len() != 3 {
                // println!("not real term");
                return parse_expression(inner_pairs[0].clone());
            } else {
                let op_raw = parse_operation(inner_pairs[1].clone());
                let mut op = BinOp::Multiply;
                match op_raw {
                    Operator::BinOp(lop) => {op = lop;},
                    _ => {}
                };
                return Expr::BinaryOperation{
                    left: Box::new(parse_expression(inner_pairs[0].clone())),
                    op: op,
                    right: Box::new(parse_expression(inner_pairs[2].clone()))
                };
            }
        },
        Rule::exponent => {
            if inner_pairs.len() != 3 {
                // println!("not real exponent");
                return parse_expression(inner_pairs[0].clone());
            } else {
                let op_raw = parse_operation(inner_pairs[1].clone());
                let mut op = BinOp::Exponent;
                match op_raw {
                    Operator::BinOp(lop) => {op = lop;},
                    _ => {}
                };
                return Expr::BinaryOperation{
                    left: Box::new(parse_expression(inner_pairs[0].clone())),
                    op: op,
                    right: Box::new(parse_expression(inner_pairs[2].clone()))
                };
            }
        },
        Rule::bitwise_expr | Rule::shift_expr => {
            if inner_pairs.len() != 3 {
                // println!("not real bitwise");
                return parse_expression(inner_pairs[0].clone());
            } else {
                let op_raw = parse_operation(inner_pairs[1].clone());
                let mut op = BitwiseOp::And;
                match op_raw {
                    Operator::BitwiseOp(lop) => {op = lop;},
                    _ => {}
                };
                return  Expr::BitwiseOperation{
                    left: Box::new(parse_expression(inner_pairs[0].clone())),
                    op: op,
                    right: Box::new(parse_expression(inner_pairs[2].clone()))
                };
            }
        },
        Rule::unary_expr => {
            if inner_pairs.len() != 2 {
                // println!("not real unary");
                return parse_expression(inner_pairs[0].clone());
            } else {
                let op_raw = parse_operation(inner_pairs[1].clone());
                let mut op = UnOp::Negate;
                match op_raw {
                    Operator::UnOp(lop) => {op = lop;},
                    _ => {}
                };
                return Expr::UnaryOperation{
                    op: op,
                    expr: Box::new(parse_expression(inner_pairs[1].clone()))
                };
            }
        },
        Rule::factor => {
            if inner_pairs.len() > 2 {
                // println!("not real factor");
                return parse_expression(inner_pairs[1].clone());
            } else {
                return parse_expression(inner_pairs[0].clone());
            }
        },
        Rule::complex_variable => {
            let mut compl_var = parse_expression(inner_pairs[0].clone());
            let mut cvar : Variable;
            match compl_var {
                Expr::ComplexVariable(mut cvar) => {
                    let mut before_second_identifier = true;
                    for index in 1..inner_pairs.len() {
                        if inner_pairs[index].as_rule() == Rule::identifier {
                            cvar.sub_variable = Some(Box::new(Variable{
                                id: inner_pairs[index].as_str().to_string(),
                                indexing: vec![],
                                sub_variable: None
                            }));
                            before_second_identifier = false;
                            continue;
                        }
                        if before_second_identifier {
                            cvar.indexing.push(parse_expression(inner_pairs[index].clone()));
                        } else {
                            match cvar.sub_variable {
                                Some(ref mut sub_var) => { (*sub_var).indexing.push(parse_expression(inner_pairs[index].clone()));},
                                None => {}
                            }
                        }
                    }
                    return Expr::ComplexVariable(cvar);
                },
                _ => {}
            };
            return Expr::ComplexVariable(Variable{
                id: "".to_string(),
                indexing: vec![],
                sub_variable: None,
            });
        },
        Rule::array_declaration => {
            return parse_expression(inner_pairs[0].clone())
        }
        Rule::identifier => {
            println!("real identifier {}", span.as_str());
            return Expr::ComplexVariable(Variable{
                id:span.as_str().to_string(),
                indexing: vec![],
                sub_variable: None
            });
        },
        Rule::number => {
            println!("real number {}", span.as_str());
            return Expr::Number(span.as_str().parse::<u32>().unwrap());
        },
        _ => {
            println!("{:?}", rule);
            println!("something else");
        }
    };                
    Expr::Number(0)
}

pub fn parse_statement(line: &str) -> Stmt {
    match ExpressionParser::parse(Rule::statement, line) {
        Ok(statement) => {
            for token in statement {
                let rule: Rule = token.as_rule();
                let span: pest::Span = token.as_span();
                let inner_pairs: Vec<pest::iterators::Pair<Rule>> = token.into_inner().into_iter().collect();
                // println!("Token rule {:?}", rule);

                match rule {
                    Rule::ternary_expr => {
                        if inner_pairs.len() != 3 {
                            return Stmt::RegularExpr(parse_expression(inner_pairs[0].clone()));
                        } else {
                            return Stmt::ConditionalAssign{
                                condition: parse_expression(inner_pairs[0].clone()),
                                true_value: parse_expression(inner_pairs[1].clone()),
                                false_value: parse_expression(inner_pairs[2].clone())
                            };
                        }
                    },
                    _ => {
                        println!("something else");
                    }
                };                
            }
            },
        _ => {}
    };
    Stmt::Assign {
        target: Expr::ComplexVariable(Variable{
            id: "example".to_string(),
            indexing: vec![],
            sub_variable: None,
        }),
        value: Expr::Number(0),
    };
    process::exit(1);
}
