// ulissy-lexer/src/main.rs
// ULissy Lexer CLI - Tokenize ULissy source files

use std::env;
use std::fs;
use std::process;

use ulissy_lexer::{debug_tokens, tokenize, TokenKind};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // No file provided - run demo
        run_demo();
        return;
    }

    let filename = &args[1];

    match fs::read_to_string(filename) {
        Ok(source) => {
            println!("╔════════════════════════════════════════════════════════════╗");
            println!("║              ULissy Lexer v0.1.0                           ║");
            println!("╚════════════════════════════════════════════════════════════╝");
            println!();
            println!("Source file: {}", filename);
            println!("{}", "─".repeat(60));

            match tokenize(&source) {
                Ok(tokens) => {
                    println!("✓ Tokenization successful!");
                    println!("  Total tokens: {}", tokens.len());
                    println!();
                    println!("Tokens:");
                    println!("{}", "─".repeat(60));

                    for token in &tokens {
                        let kind_str = format!("{:?}", token.kind);
                        let kind_display = if kind_str.len() > 30 {
                            format!("{}...", &kind_str[..27])
                        } else {
                            kind_str
                        };

                        println!("  [{:3}:{:<3}] {}", token.line, token.column, kind_display);
                    }
                }
                Err(e) => {
                    eprintln!("✗ Lexer error: {}", e);
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
    println!("║              ULissy Lexer v0.1.0 - DEMO                    ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let demo_source = r#"// ULissy Demo Program
// Proof-of-Trajectory breadcrumb collection

identity me = Keychain.primary

// Collect breadcrumbs every 10 minutes
every 10.minutes when battery > 20% {
    let crumb = breadcrumb(
        cell: here.h3(10),
        context: sensors.digest,
        previous: me.trajectory.last
    ).signed(me)
    
    me.trajectory.append(crumb)
}

// Check if ready to claim handle
when me.trajectory.count >= 100 {
    print("Ready to claim @handle!")
}

// Send encrypted message
send to @alice {
    message: "Meeting at 3pm"
}

// Facet examples
dix@me.post("Hello GNS!", visibility: .public)
home@me/lights.set(brightness: 80%)
pay@merchant.request(50.USD)
"#;

    println!("Demo Source Code:");
    println!("{}", "─".repeat(60));
    for (i, line) in demo_source.lines().enumerate() {
        println!("{:3} │ {}", i + 1, line);
    }
    println!("{}", "─".repeat(60));
    println!();

    match tokenize(demo_source) {
        Ok(tokens) => {
            println!("✓ Tokenization successful!");
            println!("  Total tokens: {}", tokens.len());
            println!();

            // Group tokens by category for summary
            let mut keywords = 0;
            let mut identifiers = 0;
            let mut literals = 0;
            let mut handles = 0;
            let mut facets = 0;
            let mut operators = 0;
            let mut delimiters = 0;

            for token in &tokens {
                match &token.kind {
                    TokenKind::Identity
                    | TokenKind::Let
                    | TokenKind::Var
                    | TokenKind::Const
                    | TokenKind::Fn
                    | TokenKind::If
                    | TokenKind::Else
                    | TokenKind::When
                    | TokenKind::Every
                    | TokenKind::Send
                    | TokenKind::To
                    | TokenKind::Return => keywords += 1,

                    TokenKind::Identifier(_) => identifiers += 1,

                    TokenKind::IntLiteral(_)
                    | TokenKind::FloatLiteral(_)
                    | TokenKind::StringLiteral(_)
                    | TokenKind::True
                    | TokenKind::False => literals += 1,

                    TokenKind::Handle(_) => handles += 1,

                    TokenKind::FacetAddress(_, _) | TokenKind::FacetPath(_, _, _) => facets += 1,

                    TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Star
                    | TokenKind::Slash
                    | TokenKind::Equal
                    | TokenKind::EqualEqual
                    | TokenKind::NotEqual
                    | TokenKind::Less
                    | TokenKind::Greater
                    | TokenKind::LessEqual
                    | TokenKind::GreaterEqual
                    | TokenKind::And
                    | TokenKind::Or
                    | TokenKind::Dot => operators += 1,

                    TokenKind::LeftParen
                    | TokenKind::RightParen
                    | TokenKind::LeftBrace
                    | TokenKind::RightBrace
                    | TokenKind::LeftBracket
                    | TokenKind::RightBracket
                    | TokenKind::Comma
                    | TokenKind::Colon
                    | TokenKind::Semicolon => delimiters += 1,

                    _ => {}
                }
            }

            println!("Token Summary:");
            println!("{}", "─".repeat(40));
            println!("  Keywords:    {:4}", keywords);
            println!("  Identifiers: {:4}", identifiers);
            println!("  Literals:    {:4}", literals);
            println!("  Handles:     {:4}", handles);
            println!("  Facets:      {:4}", facets);
            println!("  Operators:   {:4}", operators);
            println!("  Delimiters:  {:4}", delimiters);
            println!("{}", "─".repeat(40));
            println!();

            println!("All Tokens:");
            println!("{}", "─".repeat(60));
            println!("{}", debug_tokens(&tokens));
            println!();

            // Highlight ULissy-specific tokens
            println!("ULissy-Specific Tokens (Handles & Facets):");
            println!("{}", "─".repeat(60));
            for token in &tokens {
                match &token.kind {
                    TokenKind::Handle(h) => {
                        println!("  [{:3}:{:<3}] Handle: @{}", token.line, token.column, h);
                    }
                    TokenKind::FacetAddress(prefix, handle) => {
                        println!(
                            "  [{:3}:{:<3}] Facet: {}@{}",
                            token.line, token.column, prefix, handle
                        );
                    }
                    TokenKind::FacetPath(prefix, handle, path) => {
                        println!(
                            "  [{:3}:{:<3}] Facet Path: {}@{}/{}",
                            token.line, token.column, prefix, handle, path
                        );
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Lexer error: {}", e);
            process::exit(1);
        }
    }

    println!();
    println!("{}", "─".repeat(60));
    println!("Usage: ulissy-lex <filename.ul>");
    println!();
    println!("Next step: Parser will convert these tokens into an AST");
}
