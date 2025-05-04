#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    // Values
    Num(i64),
    Id(String),
    Char(u8),
    Str(String),   // String literal

    // Keywords
    Int,
    CharType, // Keyword for char type
    Return,
    If,
    Else,
    While,
    Break,
    Continue,
    Enum,
    Sizeof,

    // System calls
    Open,
    Read,
    Close,
    Printf,
    Malloc,
    Free,
    Memset,
    Memcmp,
    Exit,
	Colon,

    // Types
    Void,

    // Special identifiers
    Main,

    // Delimiters
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semi,
    Comma,

    // Operators
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Assign,  // =
    Eq,      // ==
    Ne,      // !=
    Lt,      // <
    Gt,      // >
    Le,      // <=
    Ge,      // >=
    And,     // &
    Or,      // |
    Xor,     // ^
    Lan,     // &&
    Lor,     // ||
    Shl,     // <<
    Shr,     // >>
    Inc,     // ++
    Dec,     // --
    Cond,    // ?
    Brak,    // [

    // Special
    Eof,
    Unknown(u8),
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Lexer<'a> {
    pub src: &'a [u8],                // reference to the source code
    pub pos: usize,                   // current position in the source code
    pub line: usize,                  // current line number
    pub current_token: Option<Token>, // current token
    pub ival: i64,                    // current integer value
}

impl<'a> Lexer<'a> {
    pub fn peek_token(&self) -> Option<Token> {
        self.current_token.clone()
    }
    pub fn new(src: &'a [u8]) -> Self {
        Self {
            src,
            pos: 0,
            line: 1,
            current_token: None,
            ival: 0,
        }
    }

    pub fn peek(&self) -> Option<u8> {
        // peek at the next character without advancing the position
        self.src.get(self.pos).copied() // return the next character without advancing the position
    }

    fn advance(&mut self) -> Option<u8> {
        // advance the position and return the current character
        let ch = self.src.get(self.pos).copied();
        if ch.is_some() {
            // if there is a character at the current position
            self.pos += 1;
        }
        ch
    }

    pub fn next_token(&mut self) {
        // advance the position and return the current token
        loop {
            let ch = match self.advance() {
                // advance the position and return the current character
                Some(c) => c,
                None => {
                    self.current_token = Some(Token::Eof); // set the current token to EOF if there is no more input
                    return;
                }
            };

            match ch {

                b'\'' | b'"' => {
                    let mut value = String::new();
                    let quote = ch;

                    // Process characters until closing quote
                    while let Some(c) = self.peek() {
                        if c == quote {
                            break;
                        } else if c == b'\\' {
                            if let Some(esc) = self.peek() {
                                self.advance(); // Consume the escape character
                                match esc {
                                    b'n' => value.push('\n'),
                                    b't' => value.push('\t'),
                                    b'r' => value.push('\r'),
                                    b'\'' => value.push('\''),
                                    b'"' => value.push('"'),
                                    b'\\' => value.push('\\'),
                                    _ => value.push(esc as char),
                                }
                                self.advance(); // Consume the escape character
                            }
                        } else {
                            value.push(c as char);
                            self.advance(); // Consume the character
                        }
                    }

                    self.advance(); // Consume the closing quote

                    if quote == b'"' {
                        self.current_token = Some(Token::Str(value.clone()));
					println!("DEBUG: String literal: {}", value.escape_default());
                    } else {
                        self.ival = value.chars().next().unwrap_or('0') as i64;
                        self.current_token = Some(Token::Char(self.ival as u8));
                    }
                    return;
                }
                b':' => {
                    self.current_token = Some(Token::Colon);
                    return;
                }
                b';' => {
					self.current_token = Some(Token::Semi);
                    return;
                }
                b'}' => {
                    self.current_token = Some(Token::CloseBrace);
                    return;
                }
                b'{' => {
                    self.current_token = Some(Token::OpenBrace);
                    return;
                }
                b'(' => {
                    self.current_token = Some(Token::OpenParen);
                    return;
                }
                b')' => {
                    self.current_token = Some(Token::CloseParen);
                    return;
                }
                // skip whitespace
                b' ' | b'\t' | b'\r' => {
                    continue;
                }
                // match the current character
                b'\n' => {
                    self.line += 1;
                    continue;
                }
                b'#' => {
                    // Skip preprocessor directives
                    while let Some(c) = self.advance() {
                        if c == b'\n' {
                            break;
                        }
                    }
                    continue;
                }
                b'0'..=b'9' => {
                    let mut val = 0i64;

                    if ch == b'0' {
                        match self.peek() {
                            Some(b'x') | Some(b'X') => {
                                // Hexadecimal
                                self.advance(); // consume 'x' or 'X'
                                while let Some(c) = self.peek() {
                                    self.advance();
                                    val = match c {
                                        b'0'..=b'9' => val * 16 + (c - b'0') as i64,
                                        b'a'..=b'f' => val * 16 + (c - b'a' + 10) as i64,
                                        b'A'..=b'F' => val * 16 + (c - b'A' + 10) as i64,
                                        _ => break,
                                    };
                                }
                            }
                            Some(b'0'..=b'7') => {
                                // Octal
                                while let Some(c @ b'0'..=b'7') = self.peek() {
                                    self.advance();
                                    val = val * 8 + (c - b'0') as i64;
                                }
                            }
                            _ => {
                                // It's just 0
                                val = 0;
                            }
                        }
                    } else {
                        // Decimal
                        val = (ch - b'0') as i64;
                        while let Some(c @ b'0'..=b'9') = self.peek() {
                            self.advance();
                            val = val * 10 + (c - b'0') as i64;
                        }
                    }

                    self.ival = val;
                    self.current_token = Some(Token::Num(self.ival));
                    return;
                }

                b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                    // Scan the identifier
                    let start = self.pos - 1;
                    while let Some(_c @ (b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')) = self.peek() {
                        self.advance();
                    }
                    let end = self.pos;
                    let ident = &self.src[start..end];
                    // Check for keywords
                    self.current_token = match ident {
                        b"char" => Some(Token::CharType), // Char keyword
                        b"else" => Some(Token::Else),
                        b"enum" => Some(Token::Enum),
                        b"if" => Some(Token::If),
                        b"int" => Some(Token::Int),
                        b"return" => Some(Token::Return),
                        b"sizeof" => Some(Token::Sizeof),
                        b"while" => Some(Token::While),
                        b"open" => Some(Token::Open),
                        b"read" => Some(Token::Read),
                        b"close" => Some(Token::Close),
                        b"printf" => Some(Token::Printf),
                        b"malloc" => Some(Token::Malloc),
                        b"free" => Some(Token::Free),
                        b"memset" => Some(Token::Memset),
                        b"memcmp" => Some(Token::Memcmp),
                        b"exit" => Some(Token::Exit),
                        b"void" => Some(Token::Void),
                        _ => Some(Token::Id(String::from_utf8_lossy(ident).to_string())),
                    };
                    return;
                }
                b'+' => {
                    if self.peek() == Some(b'+') {
                        self.advance();
                        self.current_token = Some(Token::Inc);
                    } else {
                        self.current_token = Some(Token::Add);
                    }
                    return;
                }
                b'-' => {
                    if self.peek() == Some(b'-') {
                        self.advance();
                        self.current_token = Some(Token::Dec);
                    } else {
                        self.current_token = Some(Token::Sub);
                    }
                    return;
                }
                b'=' => {
                    if self.peek() == Some(b'=') {
                        self.advance();
                        self.current_token = Some(Token::Eq);
                    } else {
                        self.current_token = Some(Token::Assign);
                    }
                    return;
                }
                b'/' => {
                    if self.peek() == Some(b'/') {
                        // skip single-line comment
                        self.advance();
                        while let Some(c) = self.peek() {
                            if c == b'\n' {
                                break;
                            }
                            self.advance();
                        }
                        continue;
                    } else {
                        self.current_token = Some(Token::Div);
                        return;
                    }
                }
                // bitwise-not (~) we skip
                b'~' => {
                    continue;
                }
                b',' => {
                    self.current_token = Some(Token::Comma);
                    return;
                }
                // closing bracket
                b']' => {
                    self.current_token = Some(Token::Brak);
                    return;
                }
                b'!' => {
                    if self.peek() == Some(b'=') {
                        self.advance();
                        self.current_token = Some(Token::Ne);
                    }
                    return;
                }
                b'<' => {
                    if self.peek() == Some(b'=') {
                        self.advance(); // consume '='
                        self.current_token = Some(Token::Le);
                    } else if self.peek() == Some(b'<') {
                        self.advance(); // consume '<'
                        self.current_token = Some(Token::Shl);
                    } else {
                        self.current_token = Some(Token::Lt);
                    }
                    return;
                }

                b'>' => {
                    if self.peek() == Some(b'=') {
                        self.advance(); // consume '='
                        self.current_token = Some(Token::Ge);
                    } else if self.peek() == Some(b'>') {
                        self.advance(); // consume '<'
                        self.current_token = Some(Token::Shr);
                    } else {
                        self.current_token = Some(Token::Gt);
                    }
                    return;
                }
                b'|' => {
                    if self.peek() == Some(b'|') {
                        self.advance();
                        self.current_token = Some(Token::Lor); // Logical OR
                    } else {
                        self.current_token = Some(Token::Or); // Bitwise OR
                    }
                    return;
                }
                b'&' => {
                    if self.peek() == Some(b'&') {
                        self.advance();
                        self.current_token = Some(Token::Lan); // Logical AND
                    } else {
                        self.current_token = Some(Token::And); // Bitwise AND
                    }
                    return;
                }
                b'^' => {
                    self.current_token = Some(Token::Xor); // Bitwise XOR
                    return;
                }
                b'%' => {
                    self.current_token = Some(Token::Mod); // Modulo
                    return;
                }
                b'*' => {
                    self.current_token = Some(Token::Mul); // Multiplication
                    return;
                }
                b'[' => {
                    self.current_token = Some(Token::Brak); // Bracket [
                    return;
                }
                b'?' => {
                    self.current_token = Some(Token::Cond); // Conditional ?
                    return;
                }
                _ => {
                    self.current_token = Some(Token::Unknown(ch));
                    return;
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paren_tokens() {
        let src = "(2 + 3) * 4";
        let tokens = lex_all(src);
        println!("tokens: {:?}", tokens);
        assert!(tokens.contains(&Token::OpenParen));
        assert!(tokens.contains(&Token::CloseParen));
    }
    use super::*;

    fn lex_all(src: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(src.as_bytes());
        let mut tokens = Vec::new();
        loop {
            lexer.next_token();
            if let Some(ref t) = lexer.current_token {
                if let Token::Eof = t {
                    tokens.push(Token::Eof);
                    break;
                } else {
                    tokens.push(t.clone());
                }
            }
        }
        tokens
    }

    #[test]
    fn test_colon_token() {
        let src = ":";
        let mut lexer = Lexer::new(src.as_bytes());
        lexer.next_token();
        println!("DEBUG: token after colon: {:?}", lexer.current_token);
        assert_eq!(lexer.current_token, Some(Token::Colon));
        lexer.next_token();
        assert_eq!(lexer.current_token, Some(Token::Eof));
    }

    #[test]
    fn test_simple_tokens() {
        let src = "int main() { return 42; }";
        let tokens = lex_all(src);
        assert!(tokens.contains(&Token::Int));
        assert!(tokens.contains(&Token::Id("main".to_string())));
        assert!(tokens.contains(&Token::Return));
        assert!(tokens.iter().any(|t| matches!(t, Token::Num(_))));
    }

    #[test]
    fn test_add_and_inc() {
        let src = "a + b++";
        let tokens = lex_all(src);
        assert_eq!(tokens, vec![Token::Id(String::from("a")), Token::Add, Token::Id(String::from("b")), Token::Inc, Token::Eof]);
    }

    #[test]
    fn test_hex_and_oct() {
        let src = "0x10 077";
        let mut lexer = Lexer::new(src.as_bytes());
        lexer.next_token();
        assert_eq!(lexer.current_token, Some(Token::Num(16)));
        lexer.next_token();
        assert_eq!(lexer.current_token, Some(Token::Num(63)));
    }

    #[test]
    fn test_logical_operators() {
        let src = "a && b || c";
        let tokens = lex_all(src);
        assert_eq!(tokens, vec![Token::Id(String::from("a")), Token::Lan, Token::Id(String::from("b")), Token::Lor, Token::Id(String::from("c")), Token::Eof]);
    }

    #[test]
    fn test_keywords() {
        let src = "char else enum if int return sizeof while open read close printf malloc free memset memcmp exit void main";
        let tokens = lex_all(src);
        let expected = vec![
            Token::CharType,
            Token::Else,
            Token::Enum,
            Token::If,
            Token::Int,
            Token::Return,
            Token::Sizeof,
            Token::While,
            Token::Open,
            Token::Read,
            Token::Close,
            Token::Printf,
            Token::Malloc,
            Token::Free,
            Token::Memset,
            Token::Memcmp,
            Token::Exit,
            Token::Void,
            Token::Id("main".to_string()),
            Token::Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    // fn test_braces_and_semicolons() {
    //     let src = "{ x = 1; y = 2; z = x + y; }";
    //     let tokens = lex_all(src);
    //     println!("tokens: {:?}", tokens);
    //     // Check for braces and semicolons in the correct order
    //     let expected = vec![
    //         Token::OpenBrace,
    //         Token::Id("x".to_string()), Token::Assign, Token::Num(1), Token::Semi,
    //         Token::Id("y".to_string()), Token::Assign, Token::Num(2), Token::Semi,
    //         Token::Id("z".to_string()), Token::Assign, Token::Id("x".to_string()), Token::Add, Token::Id("y".to_string()), Token::Semi,
    //         Token::CloseBrace,
    //         Token::Eof
    //     ];
    //     assert_eq!(tokens, expected);
    // }

    #[test]
    fn test_statement_keywords() {
        let src = "if (x > 0) { while (y < 10) { return z; } }";
        let tokens = lex_all(src);
        println!("tokens: {:?}", tokens);
        assert!(tokens.contains(&Token::If));
        assert!(tokens.contains(&Token::While));
        assert!(tokens.contains(&Token::Return));
        assert!(tokens.contains(&Token::OpenBrace));
        assert!(tokens.contains(&Token::CloseBrace));
        assert!(tokens.contains(&Token::OpenParen));
        assert!(tokens.contains(&Token::CloseParen));
        assert!(tokens.contains(&Token::Gt));
        assert!(tokens.contains(&Token::Lt));
    }

    #[test]
    fn test_complex_c_code() {
        let src = r#"
			#include <stdio.h>

			int main()
			{
			printf("hello, world\n");
			return 0;
			}
"#;
        let tokens = lex_all(src);
        // Spot-check for presence of key tokens
        println!("tokens: {:?}", tokens);
        assert!(tokens.contains(&Token::Int));
        assert!(tokens.contains(&Token::Id("main".to_string())));
        assert!(tokens.contains(&Token::Printf));
        assert!(tokens.contains(&Token::Return));
        assert!(tokens.contains(&Token::Eof));
    }
}
