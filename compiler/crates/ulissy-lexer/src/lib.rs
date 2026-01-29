// ulissy-lexer/src/lib.rs
// ULissy Language Lexer - Tokenization Engine
// Version 0.1.0

use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

// ============================================================================
// TOKEN TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // === KEYWORDS ===
    Identity,       // identity
    Let,            // let
    Var,            // var
    Const,          // const
    Fn,             // fn
    Type,           // type
    Struct,         // struct
    Enum,           // enum
    Trait,          // trait
    Impl,           // impl
    If,             // if
    Else,           // else
    Match,          // match
    Case,           // case
    Guard,          // guard
    For,            // for
    While,          // while
    In,             // in
    Where,          // where
    When,           // when
    Every,          // every
    After,          // after
    Within,         // within
    Timeout,        // timeout
    Budget,         // budget
    Send,           // send
    To,             // to
    From,           // from
    As,             // as
    With,           // with
    Return,         // return
    Throw,          // throw
    Throws,         // throws
    Async,          // async
    Await,          // await
    Import,         // import
    Export,         // export
    Public,         // public
    Private,        // private
    Internal,       // internal
    True,           // true
    False,          // false
    Nil,            // nil
    SelfLower,      // self
    SelfUpper,      // Self
    Computed,       // computed
    Invariant,      // invariant

    // === LITERALS ===
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    
    // === IDENTIFIERS ===
    Identifier(String),
    
    // === HANDLE & FACETS ===
    Handle(String),              // @alice
    FacetAddress(String, String), // dix@alice (prefix, handle)
    FacetPath(String, String, String), // home@alice/lights
    
    // === UNITS ===
    UnitSuffix(String),          // .meters, .minutes, .percent
    
    // === OPERATORS ===
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    Equal,          // =
    EqualEqual,     // ==
    NotEqual,       // !=
    Less,           // <
    Greater,        // >
    LessEqual,      // <=
    GreaterEqual,   // >=
    And,            // &&
    Or,             // ||
    Not,            // !
    PlusEqual,      // +=
    MinusEqual,     // -=
    StarEqual,      // *=
    SlashEqual,     // /=
    Question,       // ?
    DoubleQuestion, // ??
    QuestionDot,    // ?.
    DotDot,         // ..
    DotDotLess,     // ..<
    Arrow,          // ->
    FatArrow,       // =>
    
    // === DELIMITERS ===
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Comma,          // ,
    Colon,          // :
    Semicolon,      // ;
    Dot,            // .
    At,             // @
    
    // === SPECIAL ===
    Newline,
    EOF,
    
    // === ERROR ===
    Error(String),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Identity => write!(f, "identity"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Var => write!(f, "var"),
            TokenKind::Identifier(s) => write!(f, "identifier({})", s),
            TokenKind::Handle(s) => write!(f, "@{}", s),
            TokenKind::FacetAddress(prefix, handle) => write!(f, "{}@{}", prefix, handle),
            TokenKind::IntLiteral(n) => write!(f, "{}", n),
            TokenKind::FloatLiteral(n) => write!(f, "{}", n),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

// ============================================================================
// TOKEN WITH POSITION
// ============================================================================

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Token { kind, lexeme, line, column }
    }
}

// ============================================================================
// LEXER
// ============================================================================

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    current_pos: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            chars: source.chars().peekable(),
            current_pos: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
        }
    }

    /// Tokenize the entire source and return all tokens
    pub fn tokenize(mut self) -> Result<Vec<Token>, LexerError> {
        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            if !self.is_at_end() {
                self.scan_token()?;
            }
        }
        
        self.tokens.push(Token::new(
            TokenKind::EOF,
            String::new(),
            self.line,
            self.column,
        ));
        
        Ok(self.tokens)
    }

    fn is_at_end(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(ch) = c {
            self.current_pos += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        c
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn peek_next(&self) -> Option<char> {
        let mut iter = self.source[self.current_pos..].chars();
        iter.next();
        iter.next()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    // Optionally track newlines as tokens for statement separation
                }
                '/' => {
                    if self.peek_next() == Some('/') {
                        // Single-line comment
                        self.advance(); // consume first /
                        self.advance(); // consume second /
                        while let Some(ch) = self.peek() {
                            if ch == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else if self.peek_next() == Some('*') {
                        // Multi-line comment
                        self.advance(); // consume /
                        self.advance(); // consume *
                        let mut depth = 1;
                        while depth > 0 && !self.is_at_end() {
                            if self.peek() == Some('*') && self.peek_next() == Some('/') {
                                self.advance();
                                self.advance();
                                depth -= 1;
                            } else if self.peek() == Some('/') && self.peek_next() == Some('*') {
                                self.advance();
                                self.advance();
                                depth += 1;
                            } else {
                                self.advance();
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn scan_token(&mut self) -> Result<(), LexerError> {
        let start_line = self.line;
        let start_column = self.column;
        let c = self.advance().unwrap();

        let kind = match c {
            // Single-character tokens
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon,
            
            // Dot and ranges
            '.' => {
                if self.match_char('.') {
                    if self.match_char('<') {
                        TokenKind::DotDotLess
                    } else {
                        TokenKind::DotDot
                    }
                } else {
                    TokenKind::Dot
                }
            }

            // Operators with possible second character
            '+' => {
                if self.match_char('=') {
                    TokenKind::PlusEqual
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                if self.match_char('>') {
                    TokenKind::Arrow
                } else if self.match_char('=') {
                    TokenKind::MinusEqual
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                if self.match_char('=') {
                    TokenKind::StarEqual
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                if self.match_char('=') {
                    TokenKind::SlashEqual
                } else {
                    TokenKind::Slash
                }
            }
            '%' => TokenKind::Percent,
            
            '=' => {
                if self.match_char('=') {
                    TokenKind::EqualEqual
                } else if self.match_char('>') {
                    TokenKind::FatArrow
                } else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if self.match_char('=') {
                    TokenKind::NotEqual
                } else {
                    TokenKind::Not
                }
            }
            '<' => {
                if self.match_char('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if self.match_char('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '&' => {
                if self.match_char('&') {
                    TokenKind::And
                } else {
                    return Err(LexerError::new(
                        "Expected '&&' for logical AND",
                        start_line,
                        start_column,
                    ));
                }
            }
            '|' => {
                if self.match_char('|') {
                    TokenKind::Or
                } else {
                    return Err(LexerError::new(
                        "Expected '||' for logical OR",
                        start_line,
                        start_column,
                    ));
                }
            }
            '?' => {
                if self.match_char('?') {
                    TokenKind::DoubleQuestion
                } else if self.match_char('.') {
                    TokenKind::QuestionDot
                } else {
                    TokenKind::Question
                }
            }

            // Handle (@identifier)
            '@' => {
                if self.peek().map(|c| c.is_ascii_lowercase()).unwrap_or(false) {
                    let handle = self.scan_identifier_string();
                    TokenKind::Handle(handle)
                } else {
                    TokenKind::At
                }
            }

            // String literals
            '"' => self.scan_string()?,

            // Numbers
            c if c.is_ascii_digit() => self.scan_number(c)?,

            // Identifiers and keywords
            c if c.is_ascii_alphabetic() || c == '_' => self.scan_identifier_or_keyword(c)?,

            _ => {
                return Err(LexerError::new(
                    &format!("Unexpected character: '{}'", c),
                    start_line,
                    start_column,
                ));
            }
        };

        self.tokens.push(Token::new(
            kind.clone(),
            self.token_lexeme(&kind),
            start_line,
            start_column,
        ));

        Ok(())
    }

    fn scan_identifier_string(&mut self) -> String {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn scan_identifier_or_keyword(&mut self, first: char) -> Result<TokenKind, LexerError> {
        let mut ident = String::from(first);
        
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Check for facet address: identifier@identifier
        if self.peek() == Some('@') {
            let prefix = ident.clone();
            self.advance(); // consume @
            
            if self.peek().map(|c| c.is_ascii_lowercase()).unwrap_or(false) {
                let handle = self.scan_identifier_string();
                
                // Check for path: dix@alice/something
                if self.peek() == Some('/') {
                    self.advance(); // consume /
                    let path = self.scan_identifier_string();
                    return Ok(TokenKind::FacetPath(prefix, handle, path));
                }
                
                return Ok(TokenKind::FacetAddress(prefix, handle));
            }
        }

        // Check for unit suffix: number.unit
        // This is handled in number parsing, not here

        // Match keywords
        let kind = match ident.as_str() {
            "identity" => TokenKind::Identity,
            "let" => TokenKind::Let,
            "var" => TokenKind::Var,
            "const" => TokenKind::Const,
            "fn" => TokenKind::Fn,
            "type" => TokenKind::Type,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "trait" => TokenKind::Trait,
            "impl" => TokenKind::Impl,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "case" => TokenKind::Case,
            "guard" => TokenKind::Guard,
            "for" => TokenKind::For,
            "while" => TokenKind::While,
            "in" => TokenKind::In,
            "where" => TokenKind::Where,
            "when" => TokenKind::When,
            "every" => TokenKind::Every,
            "after" => TokenKind::After,
            "within" => TokenKind::Within,
            "timeout" => TokenKind::Timeout,
            "budget" => TokenKind::Budget,
            "send" => TokenKind::Send,
            "to" => TokenKind::To,
            "from" => TokenKind::From,
            "as" => TokenKind::As,
            "with" => TokenKind::With,
            "return" => TokenKind::Return,
            "throw" => TokenKind::Throw,
            "throws" => TokenKind::Throws,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "import" => TokenKind::Import,
            "export" => TokenKind::Export,
            "public" => TokenKind::Public,
            "private" => TokenKind::Private,
            "internal" => TokenKind::Internal,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "nil" => TokenKind::Nil,
            "self" => TokenKind::SelfLower,
            "Self" => TokenKind::SelfUpper,
            "computed" => TokenKind::Computed,
            "invariant" => TokenKind::Invariant,
            _ => TokenKind::Identifier(ident),
        };

        Ok(kind)
    }

    fn scan_number(&mut self, first: char) -> Result<TokenKind, LexerError> {
        let mut num_str = String::from(first);
        let mut is_float = false;

        // Handle hex: 0xFF
        if first == '0' && self.peek() == Some('x') {
            self.advance();
            num_str.push('x');
            while let Some(c) = self.peek() {
                if c.is_ascii_hexdigit() {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            let value = i64::from_str_radix(&num_str[2..], 16)
                .map_err(|_| LexerError::new("Invalid hex literal", self.line, self.column))?;
            return Ok(TokenKind::IntLiteral(value));
        }

        // Handle binary: 0b1010
        if first == '0' && self.peek() == Some('b') {
            self.advance();
            num_str.push('b');
            while let Some(c) = self.peek() {
                if c == '0' || c == '1' {
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            let value = i64::from_str_radix(&num_str[2..], 2)
                .map_err(|_| LexerError::new("Invalid binary literal", self.line, self.column))?;
            return Ok(TokenKind::IntLiteral(value));
        }

        // Regular number
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else if c == '.' && self.peek_next().map(|n| n.is_ascii_digit()).unwrap_or(false) {
                is_float = true;
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            let value: f64 = num_str.parse()
                .map_err(|_| LexerError::new("Invalid float literal", self.line, self.column))?;
            Ok(TokenKind::FloatLiteral(value))
        } else {
            let value: i64 = num_str.parse()
                .map_err(|_| LexerError::new("Invalid integer literal", self.line, self.column))?;
            Ok(TokenKind::IntLiteral(value))
        }
    }

    fn scan_string(&mut self) -> Result<TokenKind, LexerError> {
        let start_line = self.line;
        let start_column = self.column;
        let mut value = String::new();

        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance(); // consume closing quote
                return Ok(TokenKind::StringLiteral(value));
            } else if c == '\\' {
                self.advance(); // consume backslash
                match self.advance() {
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('r') => value.push('\r'),
                    Some('\\') => value.push('\\'),
                    Some('"') => value.push('"'),
                    Some('(') => {
                        // String interpolation: \(expr)
                        // For now, just capture as literal - parser will handle
                        value.push_str("\\(");
                    }
                    Some(c) => {
                        return Err(LexerError::new(
                            &format!("Invalid escape sequence: \\{}", c),
                            self.line,
                            self.column,
                        ));
                    }
                    None => {
                        return Err(LexerError::new(
                            "Unterminated string",
                            start_line,
                            start_column,
                        ));
                    }
                }
            } else if c == '\n' {
                return Err(LexerError::new(
                    "Unterminated string (newline in string)",
                    start_line,
                    start_column,
                ));
            } else {
                value.push(c);
                self.advance();
            }
        }

        Err(LexerError::new(
            "Unterminated string",
            start_line,
            start_column,
        ))
    }

    fn token_lexeme(&self, kind: &TokenKind) -> String {
        match kind {
            TokenKind::Identifier(s) => s.clone(),
            TokenKind::Handle(s) => format!("@{}", s),
            TokenKind::FacetAddress(p, h) => format!("{}@{}", p, h),
            TokenKind::FacetPath(p, h, path) => format!("{}@{}/{}", p, h, path),
            TokenKind::StringLiteral(s) => format!("\"{}\"", s),
            TokenKind::IntLiteral(n) => n.to_string(),
            TokenKind::FloatLiteral(n) => n.to_string(),
            _ => format!("{:?}", kind),
        }
    }
}

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Debug)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl LexerError {
    pub fn new(message: &str, line: usize, column: usize) -> Self {
        LexerError {
            message: message.to_string(),
            line,
            column,
        }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lexer error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for LexerError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let source = "identity let var fn if else when every";
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].kind, TokenKind::Identity));
        assert!(matches!(tokens[1].kind, TokenKind::Let));
        assert!(matches!(tokens[2].kind, TokenKind::Var));
        assert!(matches!(tokens[3].kind, TokenKind::Fn));
        assert!(matches!(tokens[4].kind, TokenKind::If));
        assert!(matches!(tokens[5].kind, TokenKind::Else));
        assert!(matches!(tokens[6].kind, TokenKind::When));
        assert!(matches!(tokens[7].kind, TokenKind::Every));
    }

    #[test]
    fn test_handle() {
        let source = "@alice @bob";
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(&tokens[0].kind, TokenKind::Handle(h) if h == "alice"));
        assert!(matches!(&tokens[1].kind, TokenKind::Handle(h) if h == "bob"));
    }

    #[test]
    fn test_facet_address() {
        let source = "dix@alice home@bob/lights pay@merchant";
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(&tokens[0].kind, TokenKind::FacetAddress(p, h) if p == "dix" && h == "alice"));
        assert!(matches!(&tokens[1].kind, TokenKind::FacetPath(p, h, path) if p == "home" && h == "bob" && path == "lights"));
        assert!(matches!(&tokens[2].kind, TokenKind::FacetAddress(p, h) if p == "pay" && h == "merchant"));
    }

    #[test]
    fn test_numbers() {
        let source = "42 3.14 0xFF 0b1010";
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].kind, TokenKind::IntLiteral(42)));
        assert!(matches!(tokens[1].kind, TokenKind::FloatLiteral(f) if (f - 3.14).abs() < 0.001));
        assert!(matches!(tokens[2].kind, TokenKind::IntLiteral(255)));
        assert!(matches!(tokens[3].kind, TokenKind::IntLiteral(10)));
    }

    #[test]
    fn test_strings() {
        let source = r#""Hello, world!" "With\nnewline""#;
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(&tokens[0].kind, TokenKind::StringLiteral(s) if s == "Hello, world!"));
        assert!(matches!(&tokens[1].kind, TokenKind::StringLiteral(s) if s == "With\nnewline"));
    }

    #[test]
    fn test_operators() {
        let source = "+ - * / == != <= >= && || -> =>";
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].kind, TokenKind::Plus));
        assert!(matches!(tokens[1].kind, TokenKind::Minus));
        assert!(matches!(tokens[2].kind, TokenKind::Star));
        assert!(matches!(tokens[3].kind, TokenKind::Slash));
        assert!(matches!(tokens[4].kind, TokenKind::EqualEqual));
        assert!(matches!(tokens[5].kind, TokenKind::NotEqual));
        assert!(matches!(tokens[6].kind, TokenKind::LessEqual));
        assert!(matches!(tokens[7].kind, TokenKind::GreaterEqual));
        assert!(matches!(tokens[8].kind, TokenKind::And));
        assert!(matches!(tokens[9].kind, TokenKind::Or));
        assert!(matches!(tokens[10].kind, TokenKind::Arrow));
        assert!(matches!(tokens[11].kind, TokenKind::FatArrow));
    }

    #[test]
    fn test_ulissy_program() {
        let source = r#"
            identity me = Keychain.primary
            
            every 10.minutes when battery > 20% {
                let crumb = breadcrumb(
                    cell: here.h3(10),
                    context: sensors.digest
                )
            }
        "#;
        
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        // Should tokenize without errors
        assert!(tokens.len() > 0);
        assert!(matches!(tokens[0].kind, TokenKind::Identity));
        assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "me"));
        assert!(matches!(tokens[2].kind, TokenKind::Equal));
    }

    #[test]
    fn test_comments() {
        let source = r#"
            // This is a comment
            identity me // inline comment
            /* Multi
               line
               comment */
            let x = 42
        "#;
        
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        // Comments should be skipped
        assert!(matches!(tokens[0].kind, TokenKind::Identity));
        assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "me"));
        assert!(matches!(tokens[2].kind, TokenKind::Let));
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Tokenize ULissy source code
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexerError> {
    Lexer::new(source).tokenize()
}

/// Pretty-print tokens for debugging
pub fn debug_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|t| format!("[{}:{}] {:?}", t.line, t.column, t.kind))
        .collect::<Vec<_>>()
        .join("\n")
}
