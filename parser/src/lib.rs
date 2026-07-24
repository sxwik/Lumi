use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token at line {line}, col {col}: {msg}")]
    SyntaxError {
        line: usize,
        col: usize,
        msg: String,
    },
    #[error("Unexpected end of input")]
    UnexpectedEof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementType {
    Page,
    Title,
    Heading,
    Paragraph,
    Button,
    Text,
    List,
    Item,
    Input,
    Form,
    Image,
    Container,
    Row,
    Column,
    Divider,
    CodeBlock,
    Badge,
    Custom(String),
}

impl ElementType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "page" => ElementType::Page,
            "title" => ElementType::Title,
            "heading" => ElementType::Heading,
            "paragraph" => ElementType::Paragraph,
            "button" => ElementType::Button,
            "text" => ElementType::Text,
            "list" => ElementType::List,
            "item" => ElementType::Item,
            "input" => ElementType::Input,
            "form" => ElementType::Form,
            "image" => ElementType::Image,
            "container" => ElementType::Container,
            "row" => ElementType::Row,
            "column" => ElementType::Column,
            "divider" => ElementType::Divider,
            "codeblock" => ElementType::CodeBlock,
            "badge" => ElementType::Badge,
            other => ElementType::Custom(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LumiNode {
    pub element_type: ElementType,
    pub attributes: HashMap<String, String>,
    pub value: Option<String>,
    pub children: Vec<LumiNode>,
}

impl LumiNode {
    pub fn new(element_type: ElementType) -> Self {
        Self {
            element_type,
            attributes: HashMap::new(),
            value: None,
            children: Vec::new(),
        }
    }

    pub fn get_attr(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(|s| s.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Identifier(String),
    StringLiteral(String),
    BraceOpen,
    BraceClose,
}

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn parse_page(&mut self) -> Result<LumiNode, ParseError> {
        self.skip_whitespace_and_comments();
        if let Some(token) = self.next_token()? {
            match token {
                Token::Identifier(ref name) if name == "page" => {
                    let mut page_node = LumiNode::new(ElementType::Page);
                    self.skip_whitespace_and_comments();

                    if let Some(Token::BraceOpen) = self.next_token()? {
                        while let Some(tok) = self.peek_token()? {
                            if tok == Token::BraceClose {
                                self.next_token()?; // consume '}'
                                break;
                            }
                            let child = self.parse_element()?;
                            page_node.children.push(child);
                        }
                        Ok(page_node)
                    } else {
                        Err(ParseError::SyntaxError {
                            line: self.line,
                            col: self.col,
                            msg: "Expected '{' after 'page'".to_string(),
                        })
                    }
                }
                _ => Err(ParseError::SyntaxError {
                    line: self.line,
                    col: self.col,
                    msg: "Document must start with 'page'".to_string(),
                }),
            }
        } else {
            Err(ParseError::UnexpectedEof)
        }
    }

    fn parse_element(&mut self) -> Result<LumiNode, ParseError> {
        self.skip_whitespace_and_comments();
        let token = self.next_token()?.ok_or(ParseError::UnexpectedEof)?;

        let tag_name = match token {
            Token::Identifier(name) => name,
            _ => {
                return Err(ParseError::SyntaxError {
                    line: self.line,
                    col: self.col,
                    msg: format!("Expected element identifier, found {:?}", token),
                })
            }
        };

        let mut node = LumiNode::new(ElementType::from_str(&tag_name));

        self.skip_whitespace_and_comments();
        if let Some(Token::StringLiteral(val)) = self.peek_token()? {
            self.next_token()?;
            node.value = Some(val);
            return Ok(node);
        }

        if let Some(Token::BraceOpen) = self.peek_token()? {
            self.next_token()?; // consume '{'
            while let Some(tok) = self.peek_token()? {
                if tok == Token::BraceClose {
                    self.next_token()?; // consume '}'
                    break;
                }
                let child = self.parse_element()?;
                node.children.push(child);
            }
            return Ok(node);
        }

        loop {
            self.skip_whitespace_and_comments();
            let next_tok = match self.peek_token()? {
                Some(tok) => tok,
                None => break,
            };

            match next_tok {
                Token::Identifier(ref name) => {
                    let mut clone = Self {
                        input: self.input,
                        pos: self.pos,
                        line: self.line,
                        col: self.col,
                    };
                    clone.skip_whitespace_and_comments();
                    let _ = clone.next_token();
                    clone.skip_whitespace_and_comments();

                    if let Ok(Some(Token::StringLiteral(_))) = clone.peek_token() {
                        let attr_name = name.clone();
                        self.next_token()?;
                        self.skip_whitespace_and_comments();
                        if let Some(Token::StringLiteral(attr_val)) = self.next_token()? {
                            node.attributes.insert(attr_name, attr_val);
                        }
                    } else {
                        break;
                    }
                }
                Token::BraceClose => break,
                _ => break,
            }
        }

        Ok(node)
    }

    fn skip_whitespace_and_comments(&mut self) {
        let chars: Vec<char> = self.input[self.pos..].chars().collect();
        let mut idx = 0;
        while idx < chars.len() {
            let c = chars[idx];
            if c == ' ' || c == '\t' || c == '\r' {
                if c == '\n' {
                    self.line += 1;
                    self.col = 1;
                } else {
                    self.col += 1;
                }
                self.pos += c.len_utf8();
                idx += 1;
            } else if c == '\n' {
                self.line += 1;
                self.col = 1;
                self.pos += c.len_utf8();
                idx += 1;
            } else if c == '/' && idx + 1 < chars.len() && chars[idx + 1] == '/' {
                // single line comment
                while idx < chars.len() && chars[idx] != '\n' {
                    self.pos += chars[idx].len_utf8();
                    idx += 1;
                }
            } else {
                break;
            }
        }
    }

    fn peek_token(&self) -> Result<Option<Token>, ParseError> {
        let mut clone = Self {
            input: self.input,
            pos: self.pos,
            line: self.line,
            col: self.col,
        };
        clone.skip_whitespace_and_comments();
        clone.next_token()
    }

    fn next_token(&mut self) -> Result<Option<Token>, ParseError> {
        self.skip_whitespace_and_comments();
        if self.pos >= self.input.len() {
            return Ok(None);
        }

        let ch = self.input[self.pos..].chars().next().unwrap();
        match ch {
            '{' => {
                self.pos += 1;
                self.col += 1;
                Ok(Some(Token::BraceOpen))
            }
            '}' => {
                self.pos += 1;
                self.col += 1;
                Ok(Some(Token::BraceClose))
            }
            '"' => {
                self.pos += 1;
                self.col += 1;
                let mut result = String::new();
                let mut escaped = false;

                while self.pos < self.input.len() {
                    let c = self.input[self.pos..].chars().next().unwrap();
                    self.pos += c.len_utf8();
                    self.col += 1;

                    if escaped {
                        result.push(c);
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        return Ok(Some(Token::StringLiteral(result)));
                    } else {
                        result.push(c);
                    }
                }
                Err(ParseError::SyntaxError {
                    line: self.line,
                    col: self.col,
                    msg: "Unterminated string literal".to_string(),
                })
            }
            _ if ch.is_alphanumeric() || ch == '_' || ch == '-' => {
                let start = self.pos;
                while self.pos < self.input.len() {
                    let c = self.input[self.pos..].chars().next().unwrap();
                    if c.is_alphanumeric() || c == '_' || c == '-' {
                        self.pos += c.len_utf8();
                        self.col += 1;
                    } else {
                        break;
                    }
                }
                let s = &self.input[start..self.pos];
                Ok(Some(Token::Identifier(s.to_string())))
            }
            _ => Err(ParseError::SyntaxError {
                line: self.line,
                col: self.col,
                msg: format!("Unexpected character '{}'", ch),
            }),
        }
    }
}

pub fn parse(input: &str) -> Result<LumiNode, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse_page()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_page() {
        let code = r#"
        page {
            title "Hello"

            heading {
                text "Welcome to Lumi"
            }

            paragraph {
                text "Privacy-first web."
            }

            button {
                text "Explore"
                goto "lumi://docs.home"
            }
        }
        "#;

        let ast = parse(code).unwrap();
        assert_eq!(ast.element_type, ElementType::Page);
        assert_eq!(ast.children.len(), 4);
        assert_eq!(ast.children[0].element_type, ElementType::Title);
        assert_eq!(
            ast.children[3].children[1].get_attr("goto"),
            Some("lumi://docs.home")
        );
    }

    #[test]
    fn test_element_type_mappings() {
        assert_eq!(ElementType::from_str("page"), ElementType::Page);
        assert_eq!(ElementType::from_str("button"), ElementType::Button);
        assert_eq!(
            ElementType::from_str("custom-widget"),
            ElementType::Custom("custom-widget".to_string())
        );
    }

    #[test]
    fn test_parse_nested_containers_and_codeblock() {
        let code = r#"
        page {
            container {
                row {
                    column {
                        codeblock "fn main() {}"
                        badge "Experimental"
                    }
                }
            }
        }
        "#;

        let ast = parse(code).unwrap();
        assert_eq!(ast.element_type, ElementType::Page);
        let container = &ast.children[0];
        assert_eq!(container.element_type, ElementType::Container);
        let row = &container.children[0];
        assert_eq!(row.element_type, ElementType::Row);
        let col = &row.children[0];
        assert_eq!(col.element_type, ElementType::Column);
        assert_eq!(col.children[0].element_type, ElementType::CodeBlock);
        assert_eq!(col.children[0].value, Some("fn main() {}".to_string()));
        assert_eq!(col.children[1].element_type, ElementType::Badge);
    }

    #[test]
    fn test_parse_malformed_syntax_errors() {
        // Missing opening brace
        let bad_code_1 = "page title \"Test\"";
        assert!(matches!(
            parse(bad_code_1),
            Err(ParseError::SyntaxError { .. })
        ));

        // Unterminated string literal
        let bad_code_2 = "page { title \"Unclosed string }";
        assert!(matches!(
            parse(bad_code_2),
            Err(ParseError::SyntaxError { .. })
        ));

        // Unexpected characters
        let bad_code_3 = "page { @invalid }";
        assert!(matches!(
            parse(bad_code_3),
            Err(ParseError::SyntaxError { .. })
        ));

        // Empty input EOF
        assert!(matches!(parse(""), Err(ParseError::UnexpectedEof)));
    }
}
