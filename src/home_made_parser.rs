#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "expression_grammar.pest"]
struct MyParser;

enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    BitwiseAnd,
    BitwiseOr,
    BitwiseShiftRight,
    Equal,
}

enum UnOp {
    Negate, // For -x
    BitwiseNot, // For ~x
}

enum Expr {
    Number(i32),
    Variable(String),
    BinaryOperation {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOperation {
        op: UnOp,
        expr: Box<Expr>,
    },
    Conditional {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}

enum Stmt {
    Assign {
        target: Expr,
        value: Expr,
    },
    ConditionalAssign {
        condition: Expr,
        true_value: Expr,
        false_value: Expr,
    },
}


fn parse_line_to_ast(line: &str) -> Stmt {
    // Simplified example
    Stmt::Assign {
        target: Expr::Variable("example".to_string()),
        value: Expr::Number(0),
    }
}

fn main() {
    let successful_parse = MyParser::parse(Rule::statement, "lin = in * e2");
    match successful_parse {
        Ok(pairs) => {
            for pair in pairs {
                println!("Rule:    {:?}", pair.as_rule());
                println!("Span:    {:?}", pair.clone().into_span());
                println!("Text:    {}", pair.clone().into_span().as_str());
            }
        }
        Err(e) => println!("{}", e),
    }
}
