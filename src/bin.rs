extern crate libsnarkrs;

use std::fs::File;
use std::io::Write;
use libsnarkrs::parser::compile;
use libsnarkrs::parser::ast::tokens::Token;
use libsnarkrs::parser::ast::Rule;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Template {
    name: String,
    params: Vec<String>,
    private_input_signals: Vec<Signal>,
    output_signals: Vec<Signal>,
    intermediate_signals: Vec<Signal>,
    components: Vec<Component>
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
enum DeclType {
    Signal,
    Variable,
    Component
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
enum SignalDirection{
    Input,
    Output,
    Intermediate
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Signal {
    name: String,
    direction: SignalDirection,
    size_per_dimension: Vec<String>
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Component {
    name: String,
    size_per_dimension: Vec<String>,
    template_to_use: String
}

#[derive(Eq, Hash, PartialEq, Debug)]
struct DeclStatement {
    decl_type: DeclType,
    name: String,
    direction: SignalDirection,
    size_per_dimension: Vec<String>,
    template_to_use: String
}

fn parse_array_declaration(array_decl_root: &Vec<Token>) -> String {
    if let Token::NonTerminal(ntt) = &array_decl_root[1] {
        if let Token::NonTerminal(subntt) = &ntt.subrules[0] {
            if let Token::Terminal(tt) = &subntt.subrules[0] {
                return tt.content.clone();
            }
        }
    }
    String::from("")
}

fn parse_declaration_statement(declaration_statement_root: &Vec<Token>) -> DeclStatement {
    let mut name = String::from("");
    let mut decl_type = DeclType::Variable;
    let mut direction = SignalDirection::Input;
    let mut size_per_dimension = vec![];
    let mut template_to_use = String::from("");

    if let Token::NonTerminal(ntt) = &declaration_statement_root[0] {
        if ntt.rule == Rule::SignalDeclarationKW {
            decl_type = DeclType::Signal;
            if let Token::Terminal(subtt) = &ntt.subrules[2] {
                if subtt.content.contains("output") {
                    direction = SignalDirection::Output;
                } else if subtt.content.contains("input") {
                    direction = SignalDirection::Input;
                } else {
                    direction = SignalDirection::Intermediate;
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
                            size_per_dimension.push(parse_array_declaration(&subsubtt.subrules));
                        }
                    } 
                }
            }
        }
        if ntt.rule == Rule::ForStatement {
            // Add logic
        }
    }
    if let Token::Terminal(ntt) = &declaration_statement_root[0] {
        if ntt.rule == Rule::ComponentDeclarationKW {
            decl_type = DeclType::Component;
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
                            size_per_dimension.push(parse_array_declaration(&subsubtt.subrules));
                        }
                        // = TemplateName(A, B, C)
                        if subsubtt.rule == Rule::Expression {
                            if let Token::NonTerminal(exprntt) = &subsubtt.subrules[0] {
                                if let Token::Terminal(vartt) = &exprntt.subrules[0] {
                                    template_to_use = vartt.content.clone();
                                }
                            }
                        }
                    } 
                }
            }
        }
        if ntt.rule == Rule::VariableDeclarationKW {
            if let Token::Terminal(subtt) = &declaration_statement_root[1] {
                name = subtt.content.clone();
            }
        }
    }
    let statement = DeclStatement{
        size_per_dimension: size_per_dimension,
        name: name,
        decl_type: decl_type,
        direction: direction,
        template_to_use: template_to_use,
    };
    statement
}

fn parse_template_from_ast(template_root_token: &Vec<Token>) -> Template {
    let mut temp_name = String::from("");
    let mut template_param_vec : Vec<String> = vec![];
    let mut output_signals : Vec<Signal> = vec![];
    let mut private_input_signals : Vec<Signal> = vec![];
    let mut intermediate_signals: Vec<Signal> = vec![];
    let mut components: Vec<Component> = vec![];

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
                                    name: statement.name.clone(),
                                    direction: statement.direction,
                                    size_per_dimension: statement.size_per_dimension.clone() 
                                });
                            } else if statement.direction == SignalDirection::Input {
                                private_input_signals.push(Signal{
                                    name: statement.name.clone(),
                                    direction: statement.direction,
                                    size_per_dimension: statement.size_per_dimension.clone() 
                                });
                            } else {
                                intermediate_signals.push(Signal{
                                    name: statement.name.clone(),
                                    direction: statement.direction,
                                    size_per_dimension: statement.size_per_dimension.clone() 
                                });
                            }
                        }
                        if statement.decl_type == DeclType::Component {
                            components.push(Component{
                                name: statement.name.clone(),
                                size_per_dimension: statement.size_per_dimension.clone(),
                                template_to_use: statement.template_to_use.clone()
                            });
                        }
                    }
                } else if let Token::Terminal(subsubtt) = subsubtoken {
                }
            }
        }
    }
    
    let temp =  Template{
        name: temp_name,
        params: template_param_vec,
        private_input_signals: private_input_signals,
        output_signals: output_signals,
        intermediate_signals: intermediate_signals,
        components: components
    };
    temp
}

// AST root level function
fn find_templates(subrules:&Vec<Token>) -> (Option<Component>, Vec<Template>) {
    let mut templates : Vec<Template> = Vec::new();
    let mut main_component = None;

    for token in subrules {
        if let Token::NonTerminal(ntt) = token {
            if ntt.rule == Rule::TemplateBlock {
                let temp = parse_template_from_ast(&ntt.subrules); 
                templates.push(temp);
            }
            // this is probably main component definition
            if ntt.rule == Rule::DeclarationStatement {
                let declStatement = parse_declaration_statement(&ntt.subrules);
                if declStatement.decl_type == DeclType::Component{
                    if declStatement.name.eq(&String::from("main")) {
                        main_component = Some(Component{
                            name: declStatement.name,
                            template_to_use: declStatement.template_to_use,
                            size_per_dimension: declStatement.size_per_dimension
                        });
                    }
                    else {
                        println!("Found non-main component instance in root level!");
                    }
                }
            }
        } else if let Token::Terminal(tt) = token {
        }
    }
    (main_component, templates)
}

fn get_all_templates(template_map: &HashMap<String, Template>, start_node: Component) -> HashSet<Template> {
    let mut queue: VecDeque<Component> = VecDeque::new();
    let mut set = HashSet::<Template>::new();

    queue.push_back(start_node);
    while queue.len() > 0 {
        println!("Iterating over queue: {}", queue.len());
        match queue.pop_front() {
            Some(curr_component) => {
                println!("Some {:?}", curr_component);
                let current_template = curr_component.template_to_use;
                if template_map.contains_key(&current_template) {
                    set.insert(template_map[&current_template].clone());
                    println!("Found first template: {:?}", template_map[&current_template].clone());

                    for component in &template_map[&current_template].components {
                        queue.push_back(component.clone());
                    }
                }
            },
            None => {
                println!("Empty");
            },
        }
    }

    set
}

fn main() -> std::io::Result<()> {
    let mut template_map: HashMap<String, Template> = HashMap::new();
    let mut main_component: Component = Component{
        name: String::from(""),
        template_to_use: String::from(""),
        size_per_dimension: vec![]
    };

    let path = std::fs::canonicalize("./src/lib/parser/sample_circuits/multiplier4.circom").expect("Invalid Path");

    let ctx = compile::build_context(&path);
    // println!("{}", ctx.include_stack.len());

    let mut output_file = File::create("output.txt")?;
    

    for (path, source_file) in ctx.files {
        println!("{:?}", path);

        // if !path.clone().into_os_string().into_string().unwrap().contains("multi") {
        //     continue;
        // }
        // println!("{:?}", path);
        if let libsnarkrs::parser::compile::LoadAttempt::Loaded(file) = source_file {
            // println!("{:?}", file.root.ast.len());
            write!(output_file, "{:?}", file.root.ast);
            if let libsnarkrs::parser::ast::tokens::Token::NonTerminal(token) = &file.root.ast[0] {
                // println!("{:?}", token.rule);
                let (main_component_tmp, templates) = find_templates(&token.subrules);
                match main_component_tmp {
                    Some(component) => {
                        main_component = component;
                        println!("This is main component {:?}", main_component.clone());
                    },
                    None => {}
                }
                println!("There are {} templates. ", templates.len());
                for template in templates {
                    let tmp_name = template.name.clone();
                    template_map.insert(tmp_name, template);
                    // println!(" - {}", template.name);
                    // for param in template.params {
                    //     // println!(" Params: {}", param);
                    // }
                    // // println!("Private input length {}", template.private_input_signals.len());
                    // for pi_signal in template.private_input_signals {
                    //     // println!(" Private input signal: {}", pi_signal.name);
                    //     // println!(" Private signal dimension: {}", pi_signal.size_per_dimension.len());
                    // }
                    // for o_signal in template.output_signals {
                    //     // println!(" Output signal: {}", o_signal.name);
                    //     // println!(" Output signal dimension: {}", o_signal.size_per_dimension.len());
                    // }
                    // for i_signal in template.intermediate_signals {
                    //     // println!(" Intermediate signal: {}", i_signal.name);
                    //     // println!(" Intermediate signal dimension: {}", i_signal.size_per_dimension.len());
                    // }
                    // for components in template.components {
                    //     // println!(" Components: {}", components.name);
                    //     // println!(" Component dimension: {}", components.size_per_dimension.len());
                    //     // println!(" Component is of type: {}", components.template_to_use);
                    // }
                }
                
            }
            
        }
    }
    let set_of_used_templates = get_all_templates(&template_map, main_component);
    println!("Set of used templates {:?}", set_of_used_templates);
    Ok(())
}

