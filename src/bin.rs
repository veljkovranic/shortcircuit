extern crate libsnarkrs;
use std::fs::File;
use std::io::Write;
use libsnarkrs::parser::compile;
use libsnarkrs::parser::ast::tokens::Token;
use libsnarkrs::parser::ast::Rule;

struct Template {
    name: String,
    params: Vec<String>,
    private_input_signals: Vec<Signal>,
    output_signals: Vec<Signal>
}

#[derive(PartialEq)]
enum DeclType {
    Signal,
    Variable,
    Component
}

#[derive(PartialEq)]
enum SignalDirection{
    Input,
    Output
}

struct Signal {
    name: String,
    direction: SignalDirection,
    // size_per_dimension: Vec<i32>,
    dimensions: i32
}

struct DeclStatement {
    decl_type: DeclType,
    name: String,
    direction: SignalDirection,
    dimensions: i32,
}

fn parse_declaration_statement(declaration_statement_root: &Vec<Token>) -> DeclStatement {
    let mut name = String::from("");
    let mut decl_type = DeclType::Variable;
    let mut direction = SignalDirection::Input;
    let mut dimensions = 0;
    if let Token::NonTerminal(ntt) = &declaration_statement_root[0] {
        // println!("Non terminal {:?}", ntt.rule);
        if ntt.rule == Rule::SignalDeclarationKW {
            decl_type = DeclType::Signal;
            if let Token::Terminal(subtt) = &ntt.subrules[2] {
                if subtt.content.contains("output") {
                    direction = SignalDirection::Output;
                }
            }
            if let Token::Terminal(subtt) = &declaration_statement_root[1] {
                name = subtt.content.clone();
            }
            if declaration_statement_root.len() > 2 {
                for st_index in 2..declaration_statement_root.len() {
                    if let Token::Terminal(subsubtt) = &declaration_statement_root[st_index] {
                        if subsubtt.rule == Rule::END_OF_LINE {
                            break;
                        }
                    }
                    if let Token::NonTerminal(subsubtt) = &declaration_statement_root[st_index] {
                        if subsubtt.rule == Rule::ArrayDeclaration {
                            dimensions += 1;
                        }
                    } 
                }
            }
        }
    }
    let statement = DeclStatement{
        dimensions: dimensions,
        name: name,
        decl_type: decl_type,
        direction: direction,
    };
    statement
}

fn parse_template_from_ast(template_root_token: &Vec<Token>) -> Template {
    let mut temp_name = String::from("");
    let mut template_param_vec : Vec<String> = vec![];
    let mut output_signals : Vec<Signal> = vec![];
    let mut private_input_signals : Vec<Signal> = vec![];

    if let Token::Terminal(subtt) = &template_root_token[1] {
        if subtt.rule == Rule::TemplateName {
            temp_name=subtt.content.clone();
        }
    }
    if let Token::NonTerminal(subtt) = &template_root_token[2] {
        if subtt.rule == Rule::Parameters {
            for subsubtoken in &subtt.subrules {
                if let Token::Terminal(subsubtt) = subsubtoken {
                    template_param_vec.push(subsubtt.content.clone());
                }
            }
        }
    }
    if let Token::NonTerminal(subtt) = &template_root_token[3] {
        if subtt.rule == Rule::Body {
            //TODO: parse_signals_from_body(), parse_components_from_body();
            //iterating over lines in template body
            for subsubtoken in &subtt.subrules {
                if let Token::NonTerminal(subsubtt) = subsubtoken {
                    if subsubtt.rule == Rule::DeclarationStatement {
                        let statement = parse_declaration_statement(&subsubtt.subrules);
                        if statement.decl_type == DeclType::Signal {
                            if statement.direction == SignalDirection::Output {
                                output_signals.push(Signal{
                                    name: statement.name,
                                    direction: statement.direction,
                                    dimensions: statement.dimensions 
                                });
                            } else {
                                private_input_signals.push(Signal{
                                    name: statement.name,
                                    direction: statement.direction,
                                    dimensions: statement.dimensions 
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    let temp =  Template{
        name: temp_name,
        params: template_param_vec,
        private_input_signals: private_input_signals,
        output_signals: output_signals
    };
    temp
}

fn find_templates(subrules:&Vec<Token>) -> Vec<Template> {
    let mut templates : Vec<Template> = Vec::new();

    for token in subrules {
        if let Token::NonTerminal(ntt) = token {
            // println!("Non terminal {:?}", ntt.rule);
            if ntt.rule == Rule::TemplateBlock {
                let temp = parse_template_from_ast(&ntt.subrules); 
                templates.push(temp);
            }
        } else if let Token::Terminal(tt) = token {
            // println!("Terminal {:?}", tt.rule);
        }
    }
    templates
}

fn main() -> std::io::Result<()> {
    let path = std::fs::canonicalize("./src/lib/parser/sample_circuits/warships_raw.circom").expect("Invalid Path");

    let ctx = compile::build_context(&path);
    // println!("{}", ctx.include_stack.len());

    let mut output_file = File::create("output.txt")?;
    


    for (path, source_file) in ctx.files {
        if !path.clone().into_os_string().into_string().unwrap().contains("warships") {
            continue;
        }
        // println!("{:?}", path);
        if let libsnarkrs::parser::compile::LoadAttempt::Loaded(file) = source_file {
            // println!("{:?}", file.root.ast.len());
            write!(output_file, "{:?}", file.root.ast);
            if let libsnarkrs::parser::ast::tokens::Token::NonTerminal(token) = &file.root.ast[0] {
                // println!("{:?}", token.rule);
                let templates = find_templates(&token.subrules);
                println!("There are {} templates. ", templates.len());
                for template in templates {
                    println!(" - {}", template.name);
                    for param in template.params {
                        println!(" Params: {}", param);
                    }
                    println!("Private input length {}", template.private_input_signals.len());
                    for pi_signal in template.private_input_signals {
                        println!(" Private input signal: {}", pi_signal.name);
                        println!(" Private signal dimension: {}", pi_signal.dimensions);
                    }
                    for o_signal in template.output_signals {
                        println!(" Output signal: {}", o_signal.name);
                        println!(" Output signal dimension: {}", o_signal.dimensions);
                    }
                }
            }
            
        }
        break;
    }

    Ok(())
}

