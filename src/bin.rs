extern crate libsnarkrs;
extern crate serde;

use std::fs;
use std::fs::File;
use std::io::Write;
use libsnarkrs::parser::compile;
use libsnarkrs::parser::ast::tokens::Token;
use libsnarkrs::parser::ast::Rule;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use serde::Serialize;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Template {
    name: String,
    params: Vec<String>,
    private_input_signals: Vec<Signal>,
    output_signals: Vec<Signal>,
    intermediate_signals: Vec<Signal>,
    components: Vec<Component>,
    constraints: Vec<Expression>
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
    size_per_dimension: Vec<String>,
    expression: Expression
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
    template_to_use: String,
    expression: Expression
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Expression {
    content: String,
    value: String,
    variables: Vec<String>,
    contains_constraints: bool,
    is_left_constraint: bool
}

fn parse_expression(expression: &Token) -> Expression {
    let mut result = "".to_string();
    let mut value = "".to_string();
    let mut deps : Vec<String> = vec![];
    let mut contains_constraints = false;
    let mut is_left_constraint = false;

    match expression {
        Token::NonTerminal(ntt) => {
            for token in &ntt.subrules {
                let mut tmp_exp = parse_expression(token);
                result = format!("{}{}", result, tmp_exp.content);
                deps.append(&mut tmp_exp.variables);
                contains_constraints = contains_constraints || tmp_exp.contains_constraints;
                is_left_constraint = is_left_constraint || tmp_exp.is_left_constraint;
            }
        }
        Token::Terminal(tt) => {
            if tt.rule == Rule::E_VariableName {
                deps.push(tt.content.clone());
            }
            
            contains_constraints = tt.rule == Rule::E_2_SignalLeftHandOperator || tt.rule == Rule::E_3_SignalRightHandOperator;
            is_left_constraint = tt.rule == Rule::E_2_SignalLeftHandOperator;
            
            result = format!("{}{}", result, tt.content.clone());
        }
    }
    Expression{
        value: value,
        content: result,
        variables: deps,
        contains_constraints: contains_constraints,
        is_left_constraint: is_left_constraint
    }
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
    let mut name = "".to_string();
    let mut decl_type = DeclType::Variable;
    let mut direction = SignalDirection::Input;
    let mut size_per_dimension = vec![];
    let mut template_to_use = "".to_string();
    let mut expression = Expression{
        value: "".to_string(),
        content: "".to_string(),
        variables: vec![],
        contains_constraints: false,
        is_left_constraint: false
    };

    if let Token::NonTerminal(ntt) = &declaration_statement_root[0] {

        if ntt.rule == Rule::SignalDeclarationWithConstraint {
            decl_type = DeclType::Signal;
            if let Token::NonTerminal(wrapper) = &ntt.subrules[0] {
                if let Token::Terminal(subtt) = &wrapper.subrules[2] {
                    if subtt.content.contains("output") {
                        direction = SignalDirection::Output;
                    } else if subtt.content.contains("input") {
                        direction = SignalDirection::Input;
                    } else {
                        direction = SignalDirection::Intermediate;
                    }
                }
                if let Token::Terminal(subtt) = &ntt.subrules[1] {
                    name = subtt.content.clone();
                }
            }

            if ntt.subrules.len() > 2 {
                for st_index in 2..ntt.subrules.len() {
                    if let Token::Terminal(subsubtt) = &ntt.subrules[st_index] {
                        if subsubtt.rule == Rule::END_OF_LINE {
                            break;
                        }
                    }
                    if let Token::NonTerminal(subsubtt) = &ntt.subrules[st_index] {
                        if subsubtt.rule == Rule::ArrayDeclaration {
                            size_per_dimension.push(parse_array_declaration(&subsubtt.subrules));
                        }
                    } 
                }
                if let Token::NonTerminal(subsubtt) = &ntt.subrules[ntt.subrules.len() - 1] {
                    if subsubtt.rule == Rule::Expression {
                        // println!("{:?}", subsubtt.subrules);
                        expression = parse_expression(&ntt.subrules[ntt.subrules.len() - 1]);
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
        expression: expression
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
    let mut constraints: Vec<Expression> = vec![];

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
            for subsubtoken in &subtt.subrules {
                if let Token::NonTerminal(subsubtt) = subsubtoken {
                    if subsubtt.rule == Rule::DeclarationStatement {
                        let statement = parse_declaration_statement(&subsubtt.subrules);
                        if statement.decl_type == DeclType::Signal {
                            if statement.direction == SignalDirection::Output {
                                output_signals.push(Signal{
                                    name: statement.name.clone(),
                                    direction: statement.direction,
                                    size_per_dimension: statement.size_per_dimension.clone(),
                                    expression: statement.expression.clone(),
                                });
                            } else if statement.direction == SignalDirection::Input {
                                private_input_signals.push(Signal{
                                    name: statement.name.clone(),
                                    direction: statement.direction,
                                    size_per_dimension: statement.size_per_dimension.clone(),
                                    expression: statement.expression.clone(),

                                });
                            } else {
                                intermediate_signals.push(Signal{
                                    name: statement.name.clone(),
                                    direction: statement.direction,
                                    size_per_dimension: statement.size_per_dimension.clone(),
                                    expression: statement.expression.clone(),
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
                    if subsubtt.rule == Rule::Expression {
                        // println!("Found {:?}", subsubtt.subrules);
                        let mut expression = parse_expression(subsubtoken);
                        if expression.contains_constraints {
                            if expression.is_left_constraint { 
                                expression.value = expression.variables[0].clone();
                             } else {
                                expression.value = expression.variables[expression.variables.len() - 1].clone();
                             }
                        }

                        let index = expression.variables.iter().position(|x| *x == expression.value).unwrap();
                        expression.variables.remove(index);
                        let index = match expression.variables.iter().position(|x| *x == "in".to_string()) { Some(index) => { expression.variables.remove(index);}, None => {}};
                        let index = match expression.variables.iter().position(|x| *x == "out".to_string()) { Some(index) => { expression.variables.remove(index);}, None => {}};
                        constraints.push(expression);
                        // println!("Exp {:?}", expression);
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
        components: components,
        constraints: constraints,
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
                let decl_statement = parse_declaration_statement(&ntt.subrules);
                if decl_statement.decl_type == DeclType::Component{
                    if decl_statement.name.eq(&String::from("main")) {
                        main_component = Some(Component{
                            name: decl_statement.name,
                            template_to_use: decl_statement.template_to_use,
                            size_per_dimension: decl_statement.size_per_dimension
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

fn get_used_templates(template_map: &HashMap<String, Template>, start_node: Component) -> HashSet<Template> {
    let mut queue: VecDeque<Component> = VecDeque::new();
    let mut set = HashSet::<Template>::new();

    queue.push_back(start_node);
    while queue.len() > 0 {
        match queue.pop_front() {
            Some(curr_component) => {
                let current_template = curr_component.template_to_use;
                if template_map.contains_key(&current_template) {
                    set.insert(template_map[&current_template].clone());

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

        if !path.clone().into_os_string().into_string().unwrap().contains("multi") {
            continue;
        }
        // println!("{:?}", path);
        if let libsnarkrs::parser::compile::LoadAttempt::Loaded(file) = source_file {
            // write!(output_file, "{:?}", file.root.ast);
            if let libsnarkrs::parser::ast::tokens::Token::NonTerminal(token) = &file.root.ast[0] {
                // println!("{:?}", token.rule);
                let (main_component_tmp, templates) = find_templates(&token.subrules);
                match main_component_tmp {
                    Some(component) => {
                        main_component = component;
                        // println!("This is main component {:?}", main_component.clone());
                    },
                    None => {}
                }
                println!("There are {} templates. ", templates.len());
                for template in templates {
                    let tmp_name = template.name.clone();
                    template_map.insert(tmp_name, template.clone());
                    // println!(" - {}", template.name);
                    // for param in template.params {
                    //     // println!(" Params: {}", param);
                    // }
                    // // println!("Private input length {}", template.private_input_signals.len());
                    // for pi_signal in template.private_input_signals {
                    //     println!(" Private input signal: {}", pi_signal.name);
                    //     println!(" Private signal dimension: {}", pi_signal.size_per_dimension.len());
                    // }
                    // for o_signal in template.output_signals {
                    //     println!(" Output signal: {}", o_signal.name);
                    //     println!(" Output signal dimension: {}", o_signal.size_per_dimension.len());
                    // }
                    // for i_signal in &template.intermediate_signals {
                    //     println!(" Intermediate signal: {}", i_signal.name.clone());
                    //     println!(" Intermediate signal dimension: {}", i_signal.size_per_dimension.len());
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
    let set_of_used_templates = get_used_templates(&template_map, main_component.clone());
    // println!("Set of used templates {:?}", set_of_used_templates);

    #[derive(Serialize)]
    struct OutputFormat {
        category: String,
        key: String,
        loc: String,
        isGroup: bool,
        group: String,
    }

    #[derive(Serialize)]
    struct Link{
        from: String,
        fromPort: String,
        to: String,
        toPort: String
    }
    let mut components : Vec<OutputFormat> = vec![];
    let mut links : Vec<Link> = vec![];

    let mut curr_x = 0;
    let mut curr_y = 0;

    match template_map.get(&main_component.template_to_use) {
        Some(main_template) => {
            components.push(OutputFormat{
                key: main_template.name.clone(),
                isGroup: true,
                category: "none".to_string(),
                loc: format!("{}  {}", 200, 0),
                group: "".to_string()
            });
            components.push(OutputFormat{
                key: "Private inputs".to_string(),
                isGroup: true,
                group: "".to_string(),
                category: "none".to_string(),
                loc: format!("{}  {}", 0, 0),
            });

            for input_signal in &main_template.private_input_signals {
                components.push(OutputFormat{
                    category: "privateInput".to_string(),
                    key: input_signal.name.clone(),
                    loc: format!("{}  {}", curr_x, curr_y),
                    isGroup: false,
                    group: "Private inputs".to_string()
                });
                curr_y += 50;
            }

            curr_x += 50;
            for i_signal in &main_template.intermediate_signals {
                components.push(OutputFormat{
                    category: "temp".to_string(),
                    key: i_signal.name.clone(),
                    loc: format!("{}  {}", curr_x, curr_y),
                    isGroup: false,
                    group: main_template.name.clone()
                });
                curr_y += 50;
                components.push(OutputFormat{
                    category: "temp".to_string(),
                    key: i_signal.expression.content.clone(),
                    loc: format!("{}  {}", curr_x, curr_y),
                    isGroup: false,
                    group: main_template.name.clone()
                });

                for dependency in &i_signal.expression.variables {
                    links.push(Link{
                        from: dependency.clone(),
                        fromPort: "out".to_string(),
                        to: i_signal.expression.content.clone(),
                        toPort: "in1".to_string()
                    });
                }
                links.push(Link{
                    from: i_signal.expression.content.clone(),
                    fromPort: "out".to_string(),
                    to: i_signal.name.clone(),
                    toPort: "in1".to_string()
                });
                curr_y += 20;
            }

            components.push(OutputFormat{
                category: "outputgroup".to_string(),
                key: "Output".to_string(),
                loc: format!("{}  {}", 400, 0),
                isGroup: true,
                group: "".to_string()
            });

            curr_x += 50;
            for constraint in &main_template.constraints {
                components.push(OutputFormat{
                    category: "temp".to_string(),
                    key: constraint.content.clone(),
                    loc: format!("{}  {}", curr_x, curr_y),
                    isGroup: false,
                    group: main_template.name.clone()
                });
                curr_y += 20;

                for dependency in &constraint.variables {
                    links.push(Link{
                        from: dependency.clone(),
                        fromPort: "out".to_string(),
                        to: constraint.content.clone(),
                        toPort: "in".to_string()
                    });
                }
                links.push(Link{
                    from: constraint.content.clone(),
                    fromPort: "out".to_string(),
                    to: constraint.value.clone(),
                    toPort: "in".to_string()
                });
                curr_y += 50;
                
            }
            curr_x += 50;
            for o_signal in &main_template.output_signals {
                components.push(OutputFormat{
                    category: "output".to_string(),
                    key: o_signal.name.clone(),
                    loc: format!("{}  {}", curr_x, curr_y),
                    isGroup: false,
                    group: "Output".to_string()
                });
                curr_y +=20;
            }

            println!("Components {:?}", &main_template.components);
            for component in &main_template.components {
                curr_x = curr_x + 40;
                // curr_y = curr_y + 20; 
                println!("Component category {:?}", component.template_to_use.clone());
                println!("Component key {:?}", component.name.clone());

                components.push(OutputFormat{
                    category: "template".to_string(),
                    key: component.name.clone(),
                    loc: format!("{} {}", curr_x, curr_y),
                    isGroup: false,
                    group: main_template.name.clone()
                });
            }
        },
        None => {
            println!("main initialisation is wrong");
        }
    }

    let mut prev_temp_name = main_component.template_to_use.clone();
    
    
    let serialized_nodes = serde_json::to_string(&components).unwrap();
    let serialized_links = serde_json::to_string(&links).unwrap();

    let json_data = format!(r#"{{
        "class": "go.GraphLinksModel",
        "linkFromPortIdProperty": "fromPort",
        "linkToPortIdProperty": "toPort",
        "nodeDataArray": {}, 
        "linkDataArray": {}
    }}"#, serialized_nodes, serialized_links);

    // Create a file
    let file_path = "src/visual/logicCircuit.html";

    // Read the content of the HTML file
    let content = fs::read_to_string(file_path)?;

    // The string to replace
    let to_replace = "###replaceme###";

    let updated_content = content.replace(to_replace, &json_data);

    // Write the updated content back to the file
    let mut file = fs::File::create(file_path)?;
    file.write_all(updated_content.as_bytes())?;

    Ok(())
}

