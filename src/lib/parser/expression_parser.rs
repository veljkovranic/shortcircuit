use pest::Parser;
use std::process;
use std::collections::HashMap;
use std::fmt;

#[derive(Parser)]
#[grammar = "lib/parser/expression_grammar.pest"]
pub struct ExpressionParser;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum Operator {
    BinOp(BinOp),
    LogicalOp(LogicalOp),
    BitwiseOp(BitwiseOp),
    UnOp(UnOp),
    LeftConstraint,
    LeftSignalAssign,
    RightConstraint,
    RightSignalAssign,
    Assignment,
    PlusAssignment,
    MinusAssignment,
    TimesAssignment,
    ExponentAssignment,
    DivideAssignment,
    QuotientAssignment,
    ModuloAssignment,
    SymmetricConstraintOp,
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
pub struct Variable{
    pub id: String,
    pub indexing: Vec<Expr>,
    pub sub_variable: Option<Box<Variable>>
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct ComponentInstance {
    pub name: Variable,
    pub parameter_list: Vec<Expr>
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct EvaluatedComponentInstance {
    pub name: String,
    pub parameter_list: Vec<u32>
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct BinaryOperation {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum Expr {
    Empty, 
    Number(u32),
    ComplexVariable(Variable),
    BinaryOperation(BinaryOperation),
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
    ComponentInstance(ComponentInstance)
}

// impl Expr {
//     fn extract_dependent_values(&self) -> Vec<String> {
//         let mut dep_vars : Vec<String> = vec![];
//         match self {
//             Expr::Number(n) => {
//                 dep_vars.push(n.to_string());
//             },
//             Expr::ComplexVariable(variable) => {
//                 dep_vars.push(variable.to_string());
//                 println!("The Quit variant has no data to act on.");
//             },
//             _ => {

//             },
//         }
//         dep_vars
//     }
// }
#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Constraint {
    pub target: Variable,
    pub value: Expr,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Assign {
    pub target: Variable,
    pub value: Expr,
    pub assign_op: Operator,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct ConditionalAssign {
    pub condition: Expr,
    pub true_value: Expr,
    pub false_value: Expr,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Assert {
    pub value: Expr
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct SymmetricConstraint {
    pub left: Expr,
    pub right: Expr
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum Stmt {
    Constraint(Constraint),
    Assign(Assign),
    ConditionalAssign(ConditionalAssign),
    RegularExpr(Expr),
    Assert(Assert),
    SymmetricConstraint(SymmetricConstraint),
    Empty,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum EvaluationResult {
    Empty,
    Value(u32),
    Identifier(String),
    ComponentInstance(EvaluatedComponentInstance)
}
pub trait Serializable {
    fn serialize(&self, heap: &mut HashMap<String, u32>) -> String;
}

impl Serializable for Variable {
    fn serialize(&self, heap: &mut HashMap<String, u32>) -> String {
        let mut res = "".to_string();
        res = format!("{}{}", res, self.id);
        let var_name = self;
        if var_name.indexing.len() > 0 {
            for index in 0..var_name.indexing.len() {
                let index_evaluated = evaluate(&var_name.indexing[index], heap);
                match index_evaluated {
                    (EvaluationResult::Value(num), vec) => {
                        res = format!("{}[{}]", res, num.clone());
                    },
                    (EvaluationResult::Identifier(num), vec) => {
                        match heap.get(&num) {
                            Some(value) => {
                                res = format!("{}[{}]", res, value.clone());
                            },
                            None => {
                                res = format!("{}[{}]", res, num.clone());
                            }
                        }
                    },
                    _ => {}
                }
            }
            match &var_name.sub_variable {
                Some(var) => {
                    res = format!("{}.{}", res, var.id);
                    if var.indexing.len() > 0 {
                        for index in 0..var.indexing.len() {
                            let index_evaluated = evaluate(&var.indexing[index], heap);
                            match index_evaluated {
                                (EvaluationResult::Value(num), vec) => {
                                    res = format!("{}[{}]", res, num.clone());
                                },
                                (EvaluationResult::Identifier(num), vec) => {
                                    match heap.get(&num) {
                                        Some(value) => {
                                            res = format!("{}[{}]", res, value.clone());
                                        },
                                        None => {
                                            res = format!("{}[{}]", res, num.clone());
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                },
                None => {}
                }
        } else {
            match &var_name.sub_variable {
                Some(var) => {
                    res = format!("{}.{}", res, var.id);
                    if var.indexing.len() > 0 {
                        for index in 0..var.indexing.len() {
                            let index_evaluated = evaluate(&var.indexing[index], heap);
                            match index_evaluated {
                                (EvaluationResult::Value(num), vec) => {
                                    res = format!("{}[{}]", res, num.clone());
                                },
                                (EvaluationResult::Identifier(num), vec) => {
                                    match heap.get(&num) {
                                        Some(value) => {
                                            res = format!("{}[{}]", res, value.clone());
                                        },
                                        None => {
                                            res = format!("{}[{}]", res, num.clone());
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                },
                None => {}
                }
        }
        res
    }
}
pub fn get_string_from_variable(var_name: &Variable, heap: &mut HashMap<String, u32>) -> String {
    let mut res = "".to_string();
    res = format!("{}{}", res, var_name.id);
    if var_name.indexing.len() > 0 {
        for index in 0..var_name.indexing.len() {
            let index_evaluated = evaluate(&var_name.indexing[index], heap);
            match index_evaluated {
                (EvaluationResult::Value(num), vec) => {
                    res = format!("{}[{}]", res, num.clone());
                },
                (EvaluationResult::Identifier(num), vec) => {
                    match heap.get(&num) {
                        Some(value) => {
                            res = format!("{}[{}]", res, value.clone());
                        },
                        None => {
                            res = format!("{}[{}]", res, num.clone());
                        }
                    }
                },
                _ => {}
            }
        }
        match &var_name.sub_variable {
            Some(var) => {
                res = format!("{}.{}", res, var.id);
                if var.indexing.len() > 0 {
                    for index in 0..var.indexing.len() {
                        let index_evaluated = evaluate(&var.indexing[index], heap);
                        match index_evaluated {
                            (EvaluationResult::Value(num), vec) => {
                                res = format!("{}[{}]", res, num.clone());
                            },
                            (EvaluationResult::Identifier(num), vec) => {
                                match heap.get(&num) {
                                    Some(value) => {
                                        res = format!("{}[{}]", res, value.clone());
                                    },
                                    None => {
                                        res = format!("{}[{}]", res, num.clone());
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                }
            },
            None => {}
            }
    }
    res
}

pub fn evaluate(exp: &Expr, heap: &mut HashMap<String, u32>) -> (EvaluationResult, Vec<String>) {
    match exp {
        Expr::Number(value) => {
            return (EvaluationResult::Value(value.clone()), vec![]);
        },
        Expr::ComplexVariable(var_name) => {
            let res = get_string_from_variable(&var_name, heap);

            match heap.get(&res) {
                Some(value) => {
                    return (EvaluationResult::Value(value.clone()), vec![res]);
                },
                None => {
                    return (EvaluationResult::Identifier(res.clone()), vec![res]);
                }
            }
        },
        Expr::ComponentInstance(component) => {
            let res = get_string_from_variable(&component.name, heap);
            let mut params = vec![];
            for param in &component.parameter_list {
                match evaluate(param, heap) {
                    (EvaluationResult::Value(number), vec) => {
                        params.push(number);
                    },
                    _ => {}
                }
            }
            return (EvaluationResult::ComponentInstance(EvaluatedComponentInstance{
                name: res,
                parameter_list: params
            }), vec![]);
        },
        Expr::BinaryOperation(bin_op) => {
            // println!("Binary operation {:?}", *bin_op.left);
            match evaluate(&*bin_op.left, heap) {
                (EvaluationResult::Value(number_l), mut vec_l) => {
                    match evaluate(&*bin_op.right, heap) {
                        (EvaluationResult::Value(number_r), mut vec_r) => {
                            vec_l.append(vec_r.as_mut());
                            match bin_op.op {
                                BinOp::Multiply => {
                                    return (EvaluationResult::Value(number_l*number_r), vec_l);
                                },
                                BinOp::Add => {
                                    return (EvaluationResult::Value(number_l+number_r), vec_l);
                                },
                                BinOp::Subtract => {
                                    return (EvaluationResult::Value(number_l-number_r), vec_l);
                                },
                                BinOp::Divide => {
                                    return (EvaluationResult::Value(number_l / number_r), vec_l);
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },
                _ => { println!("Missing valuesluelue ");}
            }
        }
        _ => {

        }
    }
    (EvaluationResult::Empty, vec![])
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
        Rule::symmetric_constraint_op => {
            match span.as_str() {
                "===" => { return Operator::SymmetricConstraintOp;},
                _ => {}
            }
        },
        Rule::left_constraint_op | Rule::right_constraint_op => {
            match span.as_str() {
                "<==" => { return Operator::LeftConstraint;},
                "<--" => { return Operator::LeftSignalAssign;},
                "==>" => { return Operator::RightConstraint;},
                "-->" => { return Operator::RightSignalAssign;},
                _ => {}
            }
        },
        Rule::assignment_op => {
            match span.as_str() {
                "=" => { return Operator::Assignment;},
                "+=" => { return Operator::PlusAssignment;},
                "-=" => { return Operator::MinusAssignment;},
                "*=" => { return Operator::TimesAssignment;},
                "**=" => { return Operator::ExponentAssignment;},
                "/=" => { return Operator::DivideAssignment;},
                "\\=" => { return Operator::QuotientAssignment;},
                "%=" => { return Operator::ModuloAssignment;},
                _ => {}
            };
        },
        _ => {
            println!("something else");
        }
    };             
    Operator::BinOp(BinOp::Add)
}

pub fn parse_expression(pairs: pest::iterators::Pair<Rule>) -> Expr {
    let rule: Rule = pairs.as_rule();
    // println!("{:?}", rule);

    let span: pest::Span = pairs.as_span();
    let inner_pairs: Vec<pest::iterators::Pair<Rule>> = pairs.into_inner().into_iter().collect();
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
                    res = Expr::BinaryOperation(BinaryOperation{
                        left: Box::new(res),
                        op: op,
                        right: Box::new(parse_expression(inner_pairs[op_index+1].clone()))
                    });
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
                return Expr::BinaryOperation(BinaryOperation{
                    left: Box::new(parse_expression(inner_pairs[0].clone())),
                    op: op,
                    right: Box::new(parse_expression(inner_pairs[2].clone()))
                });
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
                return Expr::BinaryOperation(BinaryOperation{
                    left: Box::new(parse_expression(inner_pairs[0].clone())),
                    op: op,
                    right: Box::new(parse_expression(inner_pairs[2].clone()))
                });
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
        Rule::component_instance => {
            let name_raw = parse_expression(inner_pairs[0].clone());
            let mut name : Variable = Variable{
                id : "".to_string(),
                indexing: vec![],
                sub_variable: None,
            };
            let mut params : Vec<Expr> = vec![];
            if inner_pairs.len() > 1 {
                for index in 1..inner_pairs.len() {
                    params.push(parse_expression(inner_pairs[index].clone()));
                }
            }
            match name_raw {
                Expr::ComplexVariable(var) => {
                    name = var;
                }, 
                _ => {}
            };
            return Expr::ComponentInstance(ComponentInstance{
                name: name,
                parameter_list: params
            }); 
        }
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
            // println!("real identifier {}", span.as_str());
            return Expr::ComplexVariable(Variable{
                id:span.as_str().to_string(),
                indexing: vec![],
                sub_variable: None
            });
        },
        Rule::number => {
            // println!("real number {}", span.as_str());
            return Expr::Number(span.as_str().parse::<u32>().unwrap());
        },
        _ => {
            println!("{:?}", rule);
            println!("something else");
        }
    };                
    Expr::Empty
}

pub fn parse_statement(line: &str) -> Stmt {
    // println!("LINE TO PARSE: {}", line);
    match ExpressionParser::parse(Rule::statement, line) {
        Ok(statement) => {
            // println!("{:?}", statement);

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
                            return Stmt::ConditionalAssign(ConditionalAssign{
                                condition: parse_expression(inner_pairs[0].clone()),
                                true_value: parse_expression(inner_pairs[1].clone()),
                                false_value: parse_expression(inner_pairs[2].clone())
                            });
                        }
                    },
                    Rule::constraint => {
                        if inner_pairs.len() != 3 {
                            println!("Error while parsing!");
                        } else {
                            let left = parse_expression(inner_pairs[0].clone());
                            let op_raw = parse_operation(inner_pairs[1].clone());
                            let right = parse_expression(inner_pairs[2].clone());

                            match op_raw {
                                Operator::LeftConstraint => {
                                    match left {
                                        Expr::ComplexVariable(var) => {
                                            return Stmt::Constraint(Constraint{
                                                target: var.clone(),
                                                value: right.clone(),
                                            });
                                        },
                                        _ => {}
                                    }
                                },
                                Operator::RightConstraint => {
                                    match right {
                                        Expr::ComplexVariable(var) => {
                                            return Stmt::Constraint(Constraint{
                                                target: var.clone(),
                                                value: left.clone()
                                            });
                                        }, _ => {}
                                    }
                                },
                                Operator::LeftSignalAssign => {
                                    match left {
                                        Expr::ComplexVariable(var) => {
                                            return Stmt::Assign(Assign{
                                                target: var.clone(),
                                                value: right.clone(),
                                                assign_op: Operator::LeftSignalAssign
                                            });
                                        }, _ => {}
                                    }
                                },
                                Operator::RightSignalAssign => {
                                    match right {
                                        Expr::ComplexVariable(var) => {
                                            return Stmt::Assign(Assign{
                                                target: var.clone(),
                                                value: left.clone(),
                                                assign_op: Operator::RightSignalAssign
                                            });
                                        }, _ => {}
                                    }
                                },
                                _ => {
                                    println!("Error while parsing!");
                                }
                            }
                        }
                    },
                    Rule::assignment => {
                        if inner_pairs.len() != 3 {
                            println!("Error while parsing!");
                        } else {
                            let left = parse_expression(inner_pairs[0].clone());
                            let op_raw = parse_operation(inner_pairs[1].clone());
                            let right = parse_expression(inner_pairs[2].clone());
                            match left {
                                Expr::ComplexVariable(var) => {
                                    return Stmt::Assign(Assign{
                                        target: var,
                                        value: right.clone(),
                                        assign_op: op_raw
                                    });
                                }, _ => {}
                            }

                        }
                    },
                    Rule::assertion => {
                        if inner_pairs.len() == 3 {
                            return Stmt::SymmetricConstraint(SymmetricConstraint{
                                left: parse_expression(inner_pairs[0].clone()),
                                right: parse_expression(inner_pairs[2].clone())
                            });
                        } else {
                            return Stmt::Assert(Assert{
                                value: parse_expression(inner_pairs[0].clone())
                            });
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
    Stmt::Empty
}
