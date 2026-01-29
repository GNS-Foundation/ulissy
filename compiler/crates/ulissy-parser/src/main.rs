// ulissy-parser/src/main.rs
// ULissy Parser CLI - Parse ULissy source files into AST

use std::env;
use std::fs;
use std::process;

use ulissy_parser::parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        run_demo();
        return;
    }

    let filename = &args[1];
    
    match fs::read_to_string(filename) {
        Ok(source) => {
            println!("╔════════════════════════════════════════════════════════════╗");
            println!("║              ULissy Parser v0.1.0                          ║");
            println!("╚════════════════════════════════════════════════════════════╝");
            println!();
            println!("Source file: {}", filename);
            println!("{}", "─".repeat(60));
            
            match parse(&source) {
                Ok(program) => {
                    println!("✓ Parsing successful!");
                    println!("  Total statements: {}", program.statements.len());
                    println!();
                    println!("Abstract Syntax Tree:");
                    println!("{}", "─".repeat(60));
                    println!("{}", program.pretty_print());
                }
                Err(e) => {
                    eprintln!("✗ Parse error: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    }
}

fn run_demo() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║              ULissy Parser v0.1.0 - DEMO                   ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    let demo_source = r#"// ULissy Demo Program
identity me = Keychain.primary

every 10.minutes when battery > 20 {
    let crumb = breadcrumb(
        cell: here.h3(10),
        context: sensors.digest,
        previous: me.trajectory.last
    )
    me.trajectory.append(crumb)
}

when me.trajectory.count >= 100 {
    print("Ready to claim @handle!")
}

send to @alice {
    message: "Meeting at 3pm"
}

dix@me.post("Hello GNS!", visibility: .public)
"#;

    println!("Demo Source Code:");
    println!("{}", "─".repeat(60));
    for (i, line) in demo_source.lines().enumerate() {
        println!("{:3} │ {}", i + 1, line);
    }
    println!("{}", "─".repeat(60));
    println!();
    
    match parse(demo_source) {
        Ok(program) => {
            println!("✓ Parsing successful!");
            println!("  Total statements: {}", program.statements.len());
            println!();
            
            // Summary of statement types
            let mut identity_count = 0;
            let mut let_count = 0;
            let mut every_count = 0;
            let mut when_count = 0;
            let mut send_count = 0;
            let mut expr_count = 0;
            
            for stmt in &program.statements {
                match stmt {
                    ulissy_parser::ast::Statement::IdentityDecl(_) => identity_count += 1,
                    ulissy_parser::ast::Statement::LetDecl(_) => let_count += 1,
                    ulissy_parser::ast::Statement::EveryBlock(_) => every_count += 1,
                    ulissy_parser::ast::Statement::WhenBlock(_) => when_count += 1,
                    ulissy_parser::ast::Statement::SendStatement(_) => send_count += 1,
                    ulissy_parser::ast::Statement::ExpressionStatement(_) => expr_count += 1,
                    _ => {}
                }
            }
            
            println!("Statement Summary:");
            println!("{}", "─".repeat(40));
            println!("  Identity declarations: {:4}", identity_count);
            println!("  Let declarations:      {:4}", let_count);
            println!("  Every blocks:          {:4}", every_count);
            println!("  When blocks:           {:4}", when_count);
            println!("  Send statements:       {:4}", send_count);
            println!("  Expression statements: {:4}", expr_count);
            println!("{}", "─".repeat(40));
            println!();
            
            println!("Abstract Syntax Tree:");
            println!("{}", "─".repeat(60));
            println!("{}", program.pretty_print());
            
            // Show detailed view of first few statements
            println!();
            println!("Detailed Statement Analysis:");
            println!("{}", "─".repeat(60));
            
            for (i, stmt) in program.statements.iter().take(5).enumerate() {
                println!();
                println!("Statement {}:", i + 1);
                match stmt {
                    ulissy_parser::ast::Statement::IdentityDecl(decl) => {
                        println!("  Type: IdentityDecl");
                        println!("  Name: {}", decl.name);
                        println!("  Init: {:?}", decl.initializer);
                    }
                    ulissy_parser::ast::Statement::EveryBlock(block) => {
                        println!("  Type: EveryBlock");
                        println!("  Interval: {:?}", block.interval);
                        println!("  Condition: {:?}", block.condition);
                        println!("  Body: {} statements", block.body.statements.len());
                    }
                    ulissy_parser::ast::Statement::WhenBlock(block) => {
                        println!("  Type: WhenBlock");
                        println!("  Condition: {:?}", block.condition);
                        println!("  Body: {} statements", block.body.statements.len());
                    }
                    ulissy_parser::ast::Statement::SendStatement(send) => {
                        println!("  Type: SendStatement");
                        println!("  Recipient: {:?}", send.recipient);
                        println!("  Fields: {} entries", send.fields.len());
                    }
                    ulissy_parser::ast::Statement::ExpressionStatement(expr) => {
                        println!("  Type: ExpressionStatement");
                        println!("  Expression: {:?}", expr);
                    }
                    _ => {
                        println!("  {:?}", stmt);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Parse error: {}", e);
            process::exit(1);
        }
    }
    
    println!();
    println!("{}", "─".repeat(60));
    println!("Usage: ulissy-parse <filename.ul>");
    println!();
    println!("Next step: Type checker will validate the AST");
}
