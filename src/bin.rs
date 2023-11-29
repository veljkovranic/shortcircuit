extern crate libsnarkrs;
use std::fs::File;
use std::io::Write;
use libsnarkrs::parser::compile;
use libsnarkrs::parser::ast::tokens::Token;
use libsnarkrs::parser::ast::Rule;

fn find_templates(subrules:&Vec<Token>) -> Vec<String> {
    let mut templates = Vec::new();

    for token in subrules {
        if let Token::NonTerminal(ntt) = token {
            // println!("Non terminal {:?}", ntt.rule);
            if ntt.rule == Rule::TemplateBlock {
                for subtoken in &ntt.subrules {
                    if let Token::Terminal(subtt) = subtoken {
                        if subtt.rule == Rule::TemplateName {
                            // println!("Template name {}", subtt.content);
                            templates.push(subtt.content.clone());
                            break;
                        }
                    }
                }
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
                    println!(" - {}", template);
                }
            }
            
        }
        break;
    }

    Ok(())
}

