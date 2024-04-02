extern crate libsnarkrs;
extern crate serde;
extern crate backtrace;

use std::fs;
use std::fs::File;
use std::io::Write;
use libsnarkrs::parser::compile;
use libsnarkrs::parser::ast::tokens::Token;
use libsnarkrs::parser::ast::Rule;
use libsnarkrs::parser::expression_parser;
use pest::pratt_parser::Op;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::process;
use backtrace::Backtrace;
use expression_parser::*;
use serde::{Serialize, Deserialize};
use warp::{http::Response, Filter};
use anyhow::Result;
use warp::http::header::{HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN};
use std::path::Path;
use std::env;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Template{
    name: String,
    params: Vec<String>,
    private_input_signals: Vec<Signal>,
    output_signals: Vec<Signal>,
    intermediate_signals: Vec<Signal>,
    components: Vec<Component>,
    constraints: Vec<Expression>,
    instructions: Vec<SingleCommand>,
    path: String,
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
    expression: Stmt,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Component {
    name: String,
    size_per_dimension: Vec<String>,
    template_to_use: String,
    arguments: Vec<u32>,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct DeclStatement {
    decl_type: DeclType,
    name: String,
    direction: SignalDirection,
    size_per_dimension: Vec<String>,
    template_to_use: String,
    expression: Stmt,
    arguments: Vec<u32>,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Expression {
    content: String,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct BoolExpression{
    lhs: String,
    rhs: String,
    operation: String,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct ForLoop {
    index: String,
    start_value: u32,
    condition: BoolExpression,
    step: i32,
    body: Vec<SingleCommand>
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct ExecutionVariable{
    id: String,
    indexing: Vec<String>,
    sub_variable: Option<Box<ExecutionVariable>>,
    is_constant: bool,
    numerical_value: u32,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Evaluation {
    variables: Vec<ExecutionVariable>,
    operation: Operation,
    params: Vec<ExecutionVariable>
}


#[derive(Eq, Hash, PartialEq, Debug, Clone)]
enum SingleCommand {
    ForLoop(ForLoop),
    Instruction(Instruction),
    DeclarationStatement(DeclStatement)
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
enum Operation {
    Multiply,
    ComponentInstance,
    Id
}
//Make this be enum
#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Instruction {
    statement: expression_parser::Stmt
}

fn get_value_for_evaluation(eval: Evaluation, heap: &mut Heap) -> u32 {
    for var in eval.variables {
        let (var_string, signal_string) = generate_string_from_variable(&var, &mut heap.variable_to_value_map);
        match heap.variable_to_value_map.get(&var_string) {
            Some(value) => { return value.clone();},
            None => { return var.numerical_value; } 
        }
    }
    // if eval.operation == Operation::Id {
    //     match mini_map.get(eval.variables[0]){
    //         Some(value) => { return value;},
    //         None
    //     }
    // }
    0
}

fn parse_expression(expression: &Token) -> Expression {
    let mut result = "".to_string();

    match expression {
        Token::NonTerminal(ntt) => {
            for token in &ntt.subrules {
                result = format!("{}{}", result, parse_expression(token).content);
            }
        }
        Token::Terminal(tt) => {
            result = format!("{}{}", result, tt.content.clone());
        }
    }
    Expression{
        content: result
    }
}

fn parse_array_declaration(array_decl_root: &Vec<Token>) -> String {
    if let Token::NonTerminal(ntt) = &array_decl_root[1] {
        if let Token::NonTerminal(subntt) = &ntt.subrules[0] {
            if let Token::NonTerminal(subnntt) = &subntt.subrules[0] {
                if let Token::Terminal(tt) = &subnntt.subrules[0] {
                    return tt.content.clone();
                }
            } else if let Token::Terminal(subtt) = &subntt.subrules[0] {
                return subtt.content.clone();
            }
        }
    }
    String::from("")
}

fn parse_list_of_vars_or_values(argument_decl_root: &Vec<Token>, index: usize) -> Vec<String> {
    let mut temp = vec![];
    
    if let Token::NonTerminal(ntt) = &argument_decl_root[index] {
        for token in &ntt.subrules {
            if let Token::NonTerminal(subntt) = &token {
                if let Token::Terminal(tt) = &subntt.subrules[0] {
                    temp.push(tt.content.clone());
                }
            }
        }
    }
    temp
}

fn parse_declaration_statement(declaration_statement_root: &Vec<Token>, path: &String, path_to_content_map: &HashMap<String, String>) -> DeclStatement {
    let mut name = "".to_string();
    let mut decl_type = DeclType::Variable;
    let mut direction = SignalDirection::Input;
    let mut size_per_dimension = vec![];
    let mut template_to_use = "".to_string();
    let mut expression = expression_parser::Stmt::Empty;
    let mut arguments = vec![];

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
                        let old_expression = parse_expression(&ntt.subrules[ntt.subrules.len() - 1]);
                        expression = extract_original_content_from_span(&path_to_content_map, subsubtt.span, path);
                        // println!("PARPRPAPRAR  {:?}", expression);
                    }
                } 
            }
        }
        if ntt.rule == Rule::ForStatement {
            println!("FOR {:?}", ntt.subrules);
        }
        if ntt.rule == Rule::ComponentDeclaration {
            // println!("{:?}", ntt.subrules);
            // println!("");

            decl_type = DeclType::Component;
            if let Token::Terminal(subtt) = &ntt.subrules[1] {
                name = subtt.content.clone();
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
                        // = TemplateName(A, B, C)
                        if subsubtt.rule == Rule::Expression {
                            if let Token::NonTerminal(exprntt) = &subsubtt.subrules[0] {
                                if let Token::NonTerminal(exprnntt) = &exprntt.subrules[0] {
                                    if let Token::Terminal(vartt) = &exprnntt.subrules[0] {
                                        //Template name is always first
                                        template_to_use = vartt.content.clone();
                                    }
                                }
                            }
                            if subsubtt.subrules.len() > 3 {
                                if let Token::NonTerminal(exprntt) = &subsubtt.subrules[2] {
                                    let argument_strings = parse_list_of_vars_or_values(&exprntt.subrules, 0);
                                    for arg_string in argument_strings {
                                        arguments.push(arg_string.parse::<u32>().unwrap());
                                    }
                                }
                            }
                        }
                        if subsubtt.rule == Rule::PublicSignalBlock {
                            // println!("Public signal block {:?}", subsubtt.subrules);
                            if let Token::NonTerminal(pub_block_ntt) = &subsubtt.subrules[0] {
                                if pub_block_ntt.rule == Rule::ArrayDeclaration {
                                    let public_signals = parse_list_of_vars_or_values(&pub_block_ntt.subrules, 1);
                                    // println!("These are public signals: {:?}", public_signals);
                                }
                            }
                        }
                    } 
                }
            }
        }
    }
    if let Token::Terminal(ntt) = &declaration_statement_root[0] {
        if ntt.rule == Rule::VariableDeclarationKW {
            if let Token::Terminal(subtt) = &declaration_statement_root[1] {
                name = subtt.content.clone();
            }
            // = is in play, or maybe array?
            if declaration_statement_root.len() > 2 {
                // println!("decl {:?}", declaration_statement_root);
                if let Token::NonTerminal(subntt) = &declaration_statement_root[2] {
                    let exp = extract_original_content_from_span(&path_to_content_map, subntt.span, path);
                }
            }

        }
    }
    let statement = DeclStatement{
        size_per_dimension: size_per_dimension,
        name: name,
        decl_type: decl_type,
        direction: direction,
        template_to_use: template_to_use,
        expression: expression,
        arguments: arguments,
    };
    statement
}

fn parse_compl_variable(container: &Token) -> ExecutionVariable {
    let mut compl_variable = ExecutionVariable{
        id: "".to_string(),
        indexing: vec![],
        is_constant: false,
        numerical_value: 0,
        sub_variable: Some(Box::new(ExecutionVariable{
            id: "".to_string(),
            indexing: vec![],
            sub_variable: None,
            is_constant: false,
            numerical_value: 0})),
    };
    if let Token::NonTerminal(x) = &container {
        if let Token::NonTerminal(subntt) = &x.subrules[0] {
            if let Token::Terminal(tt) = &subntt.subrules[0] {
                compl_variable.id = tt.content.clone();
            }
        
            let mut before_point_operator = true;

            for op_index in 1..subntt.subrules.len() {
                let mut inside = "".to_string();
                if before_point_operator {
                    if let Token::Terminal(tt) = &subntt.subrules[op_index] {
                        if tt.rule == Rule::E_19_MemberAccessOperator {
                            before_point_operator = false;
                        }
                    }
                } else {
                    if let Token::Terminal(tt) = &subntt.subrules[op_index] {
                        match compl_variable.sub_variable {
                            Some(ref mut var) => {var.id = tt.content.clone();},
                            None => {}
                        }
                    }
                }
                if let Token::NonTerminal(subnntt) = &subntt.subrules[op_index] {
                    if subnntt.rule == Rule::ArrayDeclaration {
                        if let Token::NonTerminal(exp) = &subnntt.subrules[1] {
                            if let Token::NonTerminal(value) = &exp.subrules[0] {
                                if let Token::NonTerminal(cv) = &value.subrules[0] {
                                    if let Token::Terminal(var) = &cv.subrules[0] {
                                        inside = var.content.clone();
                                    }
                                } else if let Token::Terminal(dec) = &value.subrules[0] {
                                    inside = dec.content.clone();
                                }
                            }
                        }
                        if before_point_operator {
                            compl_variable.indexing.push(inside);
                        } else {
                            match compl_variable.sub_variable {
                                Some(ref mut var) => {
                                    var.indexing.push(inside);  
                                },
                                None => {}
                            }
                        }
                        // println!("Argument contents {:?}", subntt.subrules);
                    }
                }
            }
        } else if let Token::Terminal(subtt) = &x.subrules[0] {
            compl_variable.is_constant = true;
            compl_variable.numerical_value = subtt.content.parse::<u32>().unwrap();
        }
    }
    compl_variable
}

fn parse_body_nested(elements: &Vec<Token>, path: &String, path_to_content_map: &HashMap<String, String>) -> Vec<SingleCommand> {
    let mut lines : Vec<SingleCommand> = vec![];
    for element in elements {
        if let Token::NonTerminal(ntt) = element {
            if (ntt.rule == Rule::ForStatement) {
                lines.push(parse_for_loop(&ntt.subrules, path, path_to_content_map));
            }
            if (ntt.rule == Rule::Expression) {
                let statement = extract_original_content_from_span(&path_to_content_map, ntt.span, path);
                // println!("Parsed instruction {:?}", statement);
                lines.push(SingleCommand::Instruction(Instruction{statement:statement}));
            }
        }
    }
    lines
}

fn parse_for_loop(elements: &Vec<Token>, path: &String, path_to_content_map: &HashMap<String, String>) -> SingleCommand {       
    let mut for_loop = ForLoop {
        index: "".to_string(),
        start_value: 0,
        condition: BoolExpression{
            lhs: "".to_string(),
            rhs: "".to_string(),
            operation: "".to_string(),
        },
        step: 0,
        body: vec![]
    };
    // for (___;  ; ) 
    if let Token::NonTerminal(ntt) = &elements[1] {
        if ntt.rule == Rule::DeclarationStatement {
            let statement = parse_declaration_statement(&ntt.subrules, path, path_to_content_map);
            for_loop.index = statement.name.clone();
            for_loop.start_value = 0;
        }
    } 
    // for (;___; ) 
    if let Token::NonTerminal(ntt) = &elements[2] {
        let mut lhs = "".to_string();
        let mut operation = "".to_string();
        let mut rhs = "".to_string();
        if ntt.rule == Rule::Expression {
            if let Token::NonTerminal(subntt) = &ntt.subrules[0] {
                if subntt.rule == Rule::E_Value {
                    if let Token::NonTerminal(subnntt) = &subntt.subrules[0] {
                        if let Token::Terminal(subtt) = &subnntt.subrules[0] {
                            lhs = subtt.content.clone();
                        }
                    }
                }
            }
            if let Token::Terminal(subtt) = &ntt.subrules[1] {
                if subtt.rule == Rule::E_12_RelationalOrderingOperator {
                    operation = subtt.content.clone();
                }
            }
            if let Token::NonTerminal(subntt) = &ntt.subrules[2] {
                if subntt.rule == Rule::E_Value {
                    if let Token::NonTerminal(subnntt) = &subntt.subrules[0] {
                        if let Token::Terminal(subtt) = &subnntt.subrules[0] {
                            rhs = subtt.content.clone();
                        }
                    }
                }
            }
            for_loop.condition = BoolExpression{
                lhs: lhs,
                rhs: rhs,
                operation: operation
            }
        }
    }
    // for (;;___) 
    if let Token::NonTerminal(ntt) = &elements[3] {
        let mut step_value = 0;
        let mut operator = "".to_string();
        if ntt.rule == Rule::Expression {
            if let Token::Terminal(subntt) = &ntt.subrules[1] {
                if subntt.rule == Rule::E_18_PostfixOperator {
                    operator = subntt.content.clone();
                    if operator.contains("++") {
                        step_value = 1;
                    } else {
                        step_value = -1;
                    }
                }
            }
            for_loop.step = step_value;
        }
    }
    if let Token::NonTerminal(ntt) = &elements[4] {
        if ntt.rule == Rule::Body {
            let parsed_body = parse_body_nested(&ntt.subrules, path, path_to_content_map);
            for_loop.body = parsed_body;
        }
    }
    // println!("Parsed body of for loop: {:?}", for_loop.body);
    SingleCommand::ForLoop(for_loop)
}

fn parse_template_from_ast(template_root_token: &Vec<Token>, path: &String, path_to_content_map: &HashMap<String, String>) -> Template {
    let mut temp_name = String::from("");
    let mut template_param_vec : Vec<String> = vec![];
    let mut output_signals : Vec<Signal> = vec![];
    let mut private_input_signals : Vec<Signal> = vec![];
    let mut intermediate_signals: Vec<Signal> = vec![];
    let mut components: Vec<Component> = vec![];
    let mut constraints: Vec<Expression> = vec![];
    let mut commands: Vec<SingleCommand> = vec![];

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
                        let statement = parse_declaration_statement(&subsubtt.subrules, path, path_to_content_map);
                        commands.push(SingleCommand::DeclarationStatement(statement.clone()));
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
                                template_to_use: statement.template_to_use.clone(),
                                arguments: statement.arguments.clone(),
                            });
                        }
                    }
                    if subsubtt.rule == Rule::ForStatement {
                        let for_loop = parse_for_loop(&subsubtt.subrules, path, path_to_content_map);
                        commands.push(for_loop);
                    }
                    if subsubtt.rule == Rule::Expression {
                        let mut statement = extract_original_content_from_span(&path_to_content_map, subsubtt.span, path);
                        commands.push(SingleCommand::Instruction(Instruction{
                            statement: statement.clone()
                        }));
                    }
                } else if let Token::Terminal(subsubtt) = subsubtoken {
                }
            }
        }
    }
    
    Template{
        name: temp_name,
        params: template_param_vec,
        private_input_signals: private_input_signals,
        output_signals: output_signals,
        intermediate_signals: intermediate_signals,
        components: components,
        constraints: constraints,
        instructions: commands,
        path: path.clone(),
        }
}

// AST root level function
fn find_templates(subrules:&Vec<Token>, path: &String, path_to_content_map: &HashMap<String, String>) -> (Option<Component>, Vec<Template>) {
    let mut templates : Vec<Template> = Vec::new();
    let mut main_component = None;

    for token in subrules {
        if let Token::NonTerminal(ntt) = token {
            if ntt.rule == Rule::TemplateBlock {
                let temp = parse_template_from_ast(&ntt.subrules, path, path_to_content_map); 
                templates.push(temp);
            }
            // this is probably main component definition
            if ntt.rule == Rule::DeclarationStatement {
                let decl_statement = parse_declaration_statement(&ntt.subrules, path, path_to_content_map);
                // println!("Parsed declaration statement {:?}", decl_statement);
                if decl_statement.decl_type == DeclType::Component{
                    if decl_statement.name.eq(&String::from("main")) {
                        main_component = Some(Component{
                            name: decl_statement.name,
                            template_to_use: decl_statement.template_to_use,
                            size_per_dimension: decl_statement.size_per_dimension,
                            arguments: decl_statement.arguments
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

fn get_used_templates(template_map: &HashMap<String, Template>, start_node: Component) -> HashMap<String, Template> {
    let mut queue: VecDeque<Component> = VecDeque::new();
    let mut set = HashMap::<String, Template>::new();

    queue.push_back(start_node);
    while queue.len() > 0 {
        match queue.pop_front() {
            Some(curr_component) => {
                let current_template = curr_component.template_to_use;
                if template_map.contains_key(&current_template) {
                    set.insert(current_template.clone(), template_map[&current_template].clone());

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

#[derive(Serialize, Debug, Clone)]
struct NodePosition{
    x: usize,
    y: usize,
}
#[derive(Serialize, Debug, Clone)]
struct NodeData {
    label: String,
    inputHandles: u32
}


#[derive(Serialize, Debug, Clone)]
struct Style{
    backgroundColor: String,
    width: u32,
    height: u32
}

#[derive(Serialize, Debug, Clone)]
struct Node {
    id: String,
    r#type: String,
    data: NodeData,
    position: NodePosition,
    className: String,
    style: Option<Style>,
    draggable: Option<bool>,
    selectable: Option<bool>
}

#[derive(Serialize, Debug, Clone)]
struct Edge{
    id: String,
    source: String,
    target: String,
    targetHandle: Option<String>
}

fn serialize_for_visual(template_map: &HashMap<String, Template>, main_component: Component, heap: Heap) -> Result<(Vec<Node>, Vec<Edge>)> {
    let mut nodes : Vec<Node> = vec![];
    let mut edges : Vec<Edge> = vec![];
    let mut curr_x = 20;
    let mut curr_y = 20;

    let main_template = template_map.get(&main_component.template_to_use).expect("main initialization is wrong");

    nodes.push(Node{
        id: "title".to_string(),
        r#type: "default".to_string(),
        className: "annotation".to_string(),
        data: NodeData{
            label: main_template.name.clone(),
            inputHandles: 1
        },
        draggable: Some(false),
        selectable: Some(false),
        style: None,
        position:  NodePosition{x:0, y:0}});

    nodes.push(Node{
        id: main_template.name.clone(),
        r#type: "group".to_string(),
        data: NodeData{label:main_template.name.clone(), inputHandles: 1},
        position: NodePosition{x:0, y:100},
        className: "light".to_string(),
        style: Some(Style{ backgroundColor: String::from("rgba(255, 0, 0, 0.2)"), width: 900, height: 600}),
        draggable: Some(true),
        selectable: Some(true)
    });

    curr_x = 20;
    curr_y = 100;
    for input_signal in &main_template.private_input_signals {
        println!("{} {:?}", input_signal.name, &heap.variable_to_value_map);
        nodes.push(Node{
            id: input_signal.name.clone(),
            r#type: "input".to_string(),
            data: NodeData{label:format!("{} - value {}", input_signal.name.clone(), &heap.variable_to_value_map[&input_signal.name]),inputHandles: 1},
            position: NodePosition{x:curr_x, y:curr_y},
            className: "light".to_string(),
            style: None,
            draggable: Some(true),
            selectable: Some(true)
        });
        curr_x = curr_x + 150;
    }

    curr_y = curr_y + 50;
    curr_x = 20;
    for interm_signal in &main_template.intermediate_signals {
        nodes.push(Node{
            id: interm_signal.name.clone(),
            r#type: "default".to_string(),
            data: NodeData{label:format!("{} value - {}", interm_signal.name.clone(), heap.variable_to_value_map[&interm_signal.name]), inputHandles: 1},
            position: NodePosition{x:curr_x, y:curr_y},
            className: "light".to_string(),
            style: None,
            draggable: Some(true),
            selectable: Some(true)
        });
        curr_x = curr_x + 150;
        for neighbour in heap.var_depending_on[&interm_signal.name].iter() {        
            edges.push(Edge{
                id: format!("e{}-{}", neighbour, interm_signal.name),
                source: neighbour.clone(),
                target: interm_signal.name.clone(),
                targetHandle: None,
            });
        }
    }

    curr_y = curr_y + 50;
    curr_x = 20;
    for (comp_name, sub_comp) in heap.variable_to_component_map.into_iter() {
        nodes.push(Node{
            id: comp_name.clone(),
            r#type: "customNode".to_string(),
            data: NodeData{label:comp_name.clone(), inputHandles: 2},
            position: NodePosition{x:curr_x, y:curr_y},
            className: "light".to_string(),
            style: Some(Style{ backgroundColor: String::from("rgba(255, 0, 0, 0.2)"), width: 100, height:120}),
            draggable: Some(true),
            selectable: Some(true)
        });
        curr_x = curr_x + 200;
        for (tmp_item, values) in heap.var_depending_on.iter() {
            if !tmp_item.starts_with(&comp_name) {
                continue;
            }
            for v in values.iter(){
                edges.push(Edge{
                    id: format!("e{}-{}", v, comp_name),
                    source: v.clone(),
                    target: comp_name.clone(),
                    targetHandle: Some(tmp_item.clone())
                });
            }
        }
    }

    curr_y = curr_y + 200;
    curr_x = 20;
    for signal in &main_template.output_signals {
        nodes.push(Node{
            id: signal.name.clone(),
            r#type: "output".to_string(),
            data: NodeData{label:format!("{} value - {}", signal.name.clone(), heap.variable_to_value_map[&signal.name]), inputHandles: 1},
            position: NodePosition{x:curr_x, y:curr_y},
            className: "light".to_string(),
            style: None,
            draggable: Some(true),
            selectable: Some(true)
        });
        curr_x = curr_x + 200;
        for neighbour in heap.var_depending_on[&signal.name].iter() {
            println!("- neighbour {}", neighbour);
        
            edges.push(Edge{
                id: format!("e{}-{}", neighbour, signal.name),
                source: neighbour.clone(),
                target: signal.name.clone(),
                targetHandle: None
            });
        }
    }
    println!("{:?}", nodes);
    println!("{:?}", edges);
    Ok((nodes, edges))
}

fn produce_signals(base_name: String, limit_per_dimension: &[u32]) -> Vec<String> {
    let mut names = vec![];

    if limit_per_dimension.len() == 0 {
        names.push(base_name);
        return names;
    } else {
        for x in 0..limit_per_dimension[0] as usize {
            let temp_name = format!("{}[{}]", base_name, x.to_string());
            for tmp_name in produce_signals(temp_name, &limit_per_dimension[1..]) {
                names.push(tmp_name);
            }
        }
    } 
    names
}

fn get_actual_value_for_signals_components(csize_per_dimension: &Vec<String>, cname: &String, variable_to_value_map: &HashMap<String, u32>) -> Vec<String> {
    let mut actual_value_vector = vec![];

    if csize_per_dimension.len() == 0 {
        actual_value_vector.push(cname.clone());
        return actual_value_vector;
    }
    let mut total_count = 1;
    let mut limit_per_dimension = vec![];
    for dimension in csize_per_dimension {
        match variable_to_value_map.get(dimension) {
            Some(size) => {
                limit_per_dimension.push(size.clone());
                total_count = total_count * size;
            },
            None => {
                match dimension.parse::<u32>() {
                    Ok(size) => {
                        limit_per_dimension.push(size.clone());
                        total_count = total_count * size;
                    },
                    Err(e) => {
                        println!("PANIC!!! Argument {} not initialized!", dimension);
                        break;
                    }
                }

            }
        }
    }
    actual_value_vector.append(&mut produce_signals(cname.clone(), &limit_per_dimension[0..]));
    actual_value_vector
}

fn evaluate(bool_exp: BoolExpression, variable_to_value_map:&mut HashMap<String, u32>) -> bool{
    let mut lhs_value :u32 = 0;
    let mut rhs_value :u32 = 0;
    match variable_to_value_map.get(&bool_exp.lhs) {
        Some(lhs_v) => {lhs_value=lhs_v.clone()},
        None => {println!("PANIC!! 1072");}
    }
    match variable_to_value_map.get(&bool_exp.rhs) {
        Some(rhs_v) => {rhs_value=rhs_v.clone()},
        None => {println!("PANIC!! 1076");}
    }
    let operation = bool_exp.operation.trim();
    if operation.eq("<") {
        return lhs_value < rhs_value;
    }
    if operation.eq("<=") {
        return lhs_value <= rhs_value;
    }
    if operation.eq(">") {
        return lhs_value > rhs_value;
    }
    if operation.eq(">=") {
        return lhs_value >= rhs_value;
    }
    false
}

fn generate_string_from_variable(var: &ExecutionVariable, variable_to_value_map: &mut HashMap<String, u32>) -> (String, String) {
    let mut result = format!("{}", var.id).to_string();
    for x in &var.indexing {
        match variable_to_value_map.get(x) {
            Some(value) => {result = format!("{}[{}]", result, value).to_string();},
            None => {result = format!("{}[{}]", result, x).to_string();}
        }
    }
    let mut signal = "".to_string();
    match &var.sub_variable {
        Some(sub_var) => {
            if sub_var.id.len() > 0 {
                result = format!("{}.{}", result, signal);
                signal = format!("{}", sub_var.id).to_string();
                for x in &sub_var.indexing {
                    match variable_to_value_map.get(x) {
                        Some(value) => {signal = format!("{}[{}]", signal, value).to_string();},
                        None => {signal = format!("{}[{}]", signal, x).to_string();}
                    }
                }
            }
        },
        None => {println!("PANIC!! 1116");}
    };
    (result, signal.to_string())
}

fn execute(single_command:&SingleCommand, heap: &mut Heap, template_map: &HashMap<String, Template>) {
    // println!("~ {:?}", single_command);
    match single_command {
        SingleCommand::ForLoop(for_loop) => {
            // println!("For loop {}",for_loop.index);
            // println!("Heap {:?}", heap);
            let mut curr_value = for_loop.start_value;
            heap.variable_to_value_map.insert(for_loop.index.clone(), for_loop.start_value.clone());
            let mut condition = evaluate(for_loop.condition.clone(), &mut heap.variable_to_value_map);
            while condition {
                for command in for_loop.body.clone() {
                    execute(&command, heap, template_map);
                }
                curr_value = curr_value + for_loop.step as u32;
                heap.variable_to_value_map.insert(for_loop.index.clone(), curr_value);
                condition = evaluate(for_loop.condition.clone(), &mut heap.variable_to_value_map);
                if !condition {
                    heap.variable_to_value_map.remove(&for_loop.index);
                }
            }
        },
        SingleCommand::Instruction(instruction) => {
           match &instruction.statement {
                Stmt::Assign(assign) => {
                    match assign.assign_op {
                        Operator::Assignment => {
                            let evaluated_target = expression_parser::evaluate(&Expr::ComplexVariable(assign.target.clone()), &mut heap.variable_to_value_map);
                            let evaluated_value = expression_parser::evaluate(&assign.value, &mut heap.variable_to_value_map);
                            // println!("Executing evaluated_target {:?}", evaluated_target);
                            // println!("Executing evaluated_value {:?}", evaluated_value);
                            match evaluated_target {
                                (EvaluationResult::Identifier(iden), vec) => {
                                    match evaluated_value {
                                        (EvaluationResult::ComponentInstance(component_instance), vec) => {
                                            let tmp_comp = Component{
                                                name: iden.clone(),
                                                template_to_use: component_instance.name.clone(),
                                                size_per_dimension: vec![],
                                                arguments: component_instance.parameter_list.clone(),
                                            };
                                            heap.variable_to_component_map.insert(iden.clone(), tmp_comp);
                                            heap.var_depending_on.insert(iden, Box::new(vec));
                                        },
                                        (EvaluationResult::Value(num), vec) => {
                                            heap.variable_to_value_map.insert(iden, num);
                                        },
                                        _ => {}
                                    }
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                
                }, 
                Stmt::Constraint(constraint) => {
                    let const_trgt = constraint.target.serialize(&mut heap.variable_to_value_map);
                    let evaluated_target = expression_parser::evaluate(&Expr::ComplexVariable(constraint.target.clone()), &mut heap.variable_to_value_map);
                    let (evaluated_value, vec2) = expression_parser::evaluate(&constraint.value, &mut heap.variable_to_value_map);
                    // println!("Executing evaluated_target {:?}", evaluated_target);
                    // println!("Executing evaluated_value {:?}", evaluated_value);
                    heap.var_depending_on.insert(const_trgt, Box::new(vec2));
                    match evaluated_target {
                        (EvaluationResult::Identifier(iden), vec1) => {
                            match evaluated_value {
                                EvaluationResult::Value(num) => {
                                    heap.variable_to_value_map.insert(iden, num);
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },
                // Stmt::ConditionalAssign(cond_assign) => {
                //     println!("Executing cond_assign {:?}", cond_assign);
                // }, 
                // Stmt::Constraint(constraint) => {
                //     println!("Executing constraint {:?}", constraint);
                // },
                Stmt::RegularExpr(expr) => {
                    expression_parser::evaluate(expr, &mut heap.variable_to_value_map);
                    println!("Executing expr {:?}", expr);
                }, 
                // Stmt::Assert(assert) => {
                //     println!("Executing assert {:?}", assert);
                // }, 
                _ => {

                }
            }
        },
        SingleCommand::DeclarationStatement(decl_statement) => {
            if decl_statement.decl_type == DeclType::Signal || decl_statement.decl_type == DeclType::Variable {
                let signal_vector = get_actual_value_for_signals_components(&decl_statement.size_per_dimension, &decl_statement.name, &mut heap.variable_to_value_map);
                for signal in signal_vector{
                    heap.variable_set.insert(signal.clone());
                    match &decl_statement.expression {
                        Stmt::RegularExpr(expr) => {
                            let val = expression_parser::evaluate(&expr, &mut heap.variable_to_value_map);
                            // println!("Valuelue {:?}", val);
                            match val {
                                (EvaluationResult::Value(number), vec) => {
                                    heap.variable_to_value_map.insert(signal.clone(), number.clone());
                                    heap.var_depending_on.insert(signal.clone(), Box::new(vec));
                                    // println!("heap new {:?}", &heap.var_depending_on);
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                    if decl_statement.decl_type == DeclType::Variable {
                        heap.variable_to_value_map.insert(signal.clone(), 0);
                    }
                }

            } else {
                let component_vector = get_actual_value_for_signals_components(&decl_statement.size_per_dimension, &decl_statement.name, &mut heap.variable_to_value_map);
                for component in &component_vector{
                    heap.component_set.insert(component.clone());
                }
                if component_vector.len() == 1 {
                    // println!("decl_statement {:?}", decl_statement);
                    let tmp_component = Component{
                        name: component_vector[0].clone(),
                        template_to_use: decl_statement.template_to_use.clone(),
                        size_per_dimension: vec![],
                        arguments: vec![],
                    };
                    heap.variable_to_component_map.insert(component_vector[0].clone(), tmp_component.clone());
                    heap.variable_to_heap_map.insert(component_vector[0].clone(), Heap{
                        current_component: tmp_component,
                        variable_set: HashSet::new(),
                        component_set: HashSet::new(),
                        variable_to_value_map: HashMap::new(),
                        variable_to_component_map: HashMap::new(),
                        variable_to_heap_map : HashMap::new(),
                        var_depending_on: HashMap::new(),
                    });
                }
            }
        },
    }

}

fn execute_component(component: &Component, heap: &mut Heap, template_map: &HashMap<String, Template>) {
    match template_map.get(&component.template_to_use) {
        Some(template) => {
            // println!("Executing component {:?} with {} commands", component.name, template.instructions.len());
            for command in &template.instructions {
                // println!("Executing command {:?}", command);
                execute(&command, heap, &template_map);
            }
        },
        None => {}
    }
    // println!("Finished component {:?}, heap: {:?}", component, heap);
    // process::exit(1);
}

#[derive(Eq, PartialEq, Debug, Clone)]
struct Heap{
    current_component: Component,
    variable_set: HashSet<String>,
    component_set: HashSet<String>,
    variable_to_value_map: HashMap<String, u32>,
    variable_to_component_map: HashMap<String, Component>,
    variable_to_heap_map: HashMap<String, Heap>,
    var_depending_on: HashMap<String, Box<Vec<String>>>
}

fn extract_original_content_from_span(path_to_content_map: &HashMap<String, String>, span: (usize, usize), file_path: &String) -> expression_parser::Stmt {
    let mut result = "";
    match path_to_content_map.get(file_path) {
        Some(content) => {
            result = &content[span.0..span.1];
        }
        None => {}
    }
    // println!("Try to parse {}", result);

    expression_parser::parse_statement(result)
 }

 fn extract_values() -> Result<(Vec<Node>, Vec<Edge>)> {
    let mut template_map: HashMap<String, Template> = HashMap::new();
    let mut main_component: Component = Component{
        name: String::from(""),
        template_to_use: String::from(""),
        size_per_dimension: vec![],
        arguments: vec![],
    };
    let mut path_to_content_map = HashMap::<String, String>::new();
    let path = std::fs::canonicalize("./src/lib/parser/sample_circuits/multiplier4.circom").expect("Invalid Path");

    let ctx = compile::build_context(&path);

    let mut output_file = File::create("output.txt")?;
    

    for (path, source_file) in ctx.files {
        let path_as_string = path.clone().into_os_string().into_string().unwrap();
        if !path_as_string.contains("multi") {
            continue;
        }
        let content = fs::read_to_string(&path_as_string).expect("Failed to read file");
        
        path_to_content_map.insert(path_as_string.clone(), content);
        // println!("{:?}", path_as_string);
        if let libsnarkrs::parser::compile::LoadAttempt::Loaded(file) = source_file {
            // write!(output_file, "{:?}", file.root.ast);
            if let libsnarkrs::parser::ast::tokens::Token::NonTerminal(token) = &file.root.ast[0] {
                // println!("{:?}", token.rule);
                let (main_component_tmp, templates) = find_templates(&token.subrules, &path_as_string, &path_to_content_map);
                match main_component_tmp {
                    Some(component) => {
                        main_component = component;
                    },
                    None => {}
                }
                // println!("These are {:?} templates. ", templates);
                for template in templates {
                    let tmp_name = template.name.clone();
                    template_map.insert(tmp_name, template.clone());
                    
                    // println!(" - {}", template.name);
                    // for param in template.params {
                    //     // println!(" Params: {}", param);
                    // }
                    // println!("Private input length {}", template.private_input_signals.len());
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
                    //     println!(" Components: {}", components.name);
                    //     println!(" Component dimension: {}", components.size_per_dimension.len());
                    //     println!(" Component is of type: {}", components.template_to_use);
                    // }
                }
            }

        }
    }

    //Compilation is done. Proceed with execution
    let set_of_used_templates = get_used_templates(&template_map, main_component.clone());

    println!("This is what I know about main compoment: {:?}", main_component);

    let mut heap = Heap{
        variable_set: HashSet::new(),
        component_set: HashSet::new(),
        current_component: main_component.clone(),
        variable_to_value_map: HashMap::new(),
        variable_to_component_map: HashMap::new(),
        variable_to_heap_map: HashMap::new(),
        var_depending_on: HashMap::new()
    };
    let mut running = true;

    let mut test_variable_to_value_map = HashMap::from([
        ("in1".to_string(), 3),
        ("in2".to_string(), 1),
        ("in3".to_string(), 7),
        ("in4".to_string(), 1),
        ("mult".to_string(), 21)
    ]);
    // let mut test_variable_to_value_map = HashMap::from([
    //     ("board[1][0]".to_string(), 0 as u32),
    //     ("board[0][2]".to_string(), 0 as u32),
    //     ("board[2][1]".to_string(), 1 as u32),
    //     ("board[0][1]".to_string(), 0 as u32),
    //     ("board[1][1]".to_string(), 1 as u32),
    //     ("board[1][2]".to_string(), 1 as u32),
    //     ("board[2][0]".to_string(), 1 as u32),
    //     ("board[2][2]".to_string(), 0 as u32),
    //     ("board[0][0]".to_string(), 0 as u32),
    //     ("ii".to_string(), 1 as u32),
    //     ("jj".to_string(), 1 as u32)
    // ]);
    heap.variable_to_value_map = test_variable_to_value_map;
    let mut actual_input_signal_vector : Vec<String> = vec![];
    let mut actual_output_signal_vector : Vec<String> = vec![];
    let mut actual_component_vector : Vec<String> = vec![];
    let mut actual_intermediate_signal_vector : Vec<String> = vec![];
    // struct Var {
    //     name: String,
    //     type: String
    // }
    // let mut heap : HashSet<Var>;
    let mut current_component = main_component.clone();
    while running {
        if current_component.template_to_use.len() == 0 {
            break;
        }
        match template_map.get(&current_component.template_to_use) {
            Some(template) => {
                for param_index in 0..template.params.len() {
                    heap.variable_to_value_map.insert(template.params[param_index].clone(), current_component.arguments[param_index]);
                }
                println!("{:?}", heap.variable_to_value_map);
                // for signal in &template.private_input_signals {
                //     actual_input_signal_vector.append(&mut get_actual_value_for_signals_components(&signal.size_per_dimension, &signal.name, &heap.variable_to_value_map));
                // }
                // for signal in &template.output_signals {
                //     actual_output_signal_vector.append(&mut get_actual_value_for_signals_components(&signal.size_per_dimension, &signal.name, &variable_to_value_map));
                // }
                // for signal in &template.intermediate_signals {
                //     actual_intermediate_signal_vector.append(&mut get_actual_value_for_signals_components(&signal.size_per_dimension, &signal.name, &heap.variable_to_value_map));
                // }
                // for component in &template.components {
                //     actual_component_vector.append(&mut get_actual_value_for_signals_components(&component.size_per_dimension, &component.name, &heap.variable_to_value_map));
                // }
                
                // declarations done
                running = false;
                for command in &template.instructions {
                    execute(&command, &mut heap, &template_map);
                }
                println!("HEAP DUMP =========");
                println!("{:?}", heap.var_depending_on);
                println!("{:?}", heap.variable_to_value_map);
            },
            None => {
                running = false;
            }
        }
    }

    println!("{:?}", actual_input_signal_vector);
    println!("{:?}", actual_output_signal_vector);
    println!("{:?}", actual_intermediate_signal_vector);
    println!("{:?}", actual_component_vector);

    //Drawing
    println!("{:?}", heap);
    serialize_for_visual(&template_map, main_component.clone(), heap)
 }

 #[derive(Deserialize, Serialize)]
 struct GraphData {
     data: String,
 }

 async fn fetch_graph_data(component: String) -> Result<impl warp::Reply, warp::Rejection> {
    let (mut nodes, mut edges) = (vec![Node{id: "1".to_string(), r#type:"default".to_string(), data:NodeData { label: "()".to_string(), inputHandles: 0 }, position:NodePosition{x:100, y:100}, className:"abc".to_string(), style:None, draggable:None, selectable:None}], vec![]);
    if component.contains("main") {
        (nodes, edges) = extract_values().unwrap();
    }
    let graph_data = serde_json::json!({ "initialNodes": nodes, "initialEdges": edges });
    
    Ok(Response::builder().header("Content-Type", "application/json").body(serde_json::to_string(&graph_data).unwrap()))
}


 #[tokio::main]
 async fn main() {
    let cors = warp::cors()
    .allow_any_origin()
    .allow_headers(vec!["*"])
    .allow_methods(vec!["GET", "POST", "DELETE", "PUT", "OPTIONS"]);

    let route1 = warp::path!("graph-data" / String)
    .and(warp::get())
    .and_then(fetch_graph_data).with(cors.clone());

    println!("serving on 0.0.0.0:3030");

    let file_path = "/Users/veljko/Documents/GitHub/shortcircuit/src/lib/parser/sample_circuits/multiplier4.circom";

    // Create a route that serves the file
    let route2 = warp::path("get_file").and(warp::get()).map(move || {
        match fs::read(file_path) {
            Ok(content) => {
                return Response::builder().header("Content-Type", "application/text").body(content).unwrap();
            }
            Err(_) => return Response::builder().status(404).body(Vec::new()).unwrap(),
        };
    }).with(cors).with(warp::log("warp_server"));

    let routes = route1.or(route2);
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

}

