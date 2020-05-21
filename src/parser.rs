use std::collections::HashSet;
use std::str::Chars;

// macro_rules! whitespace {
//     () => {
//         ' ' | '\t' | '\n' | '\r'
//     };
// }

fn s(content: &str) -> String {
    content.to_string()
}

const TAG_ATTR_ILLEGAL_CHARS: &'static str = "!?@#$%^&*+,.~/|\\\"'`()[]{}";
fn is_illegal(_c: &char) -> bool {
    false
}
 
#[derive(Debug, PartialEq)]
enum Token {
    Prolog {
        encoding: String,
        version: String,
        standalone: bool,
    },
    DocType {
        name: String,
        content: Option<String>,
        url: Option<String>,
    },
    Start {
        ns: Option<String>,
        name: String,
    },
    ProcessingInstructions {
        name: String,
        content: String,
    },
    Text(String),
    Attr {
        ns: Option<String>,
        name: String,
        value: Option<String>,
    },
    End {
        ns: Option<String>,
        name: String,
    },
    Comment(String),
}

pub fn parse(xml_contents: &str) {
    let tokens = tokenizer(xml_contents);
    if is_valid(&tokens) {
        // create elements
    }
}

struct XmlChars<'a> {
    _contents: Chars<'a>,
    _current: Option<char>,
}

impl<'a> XmlChars<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            _contents: content.chars(),
            _current: None,
        }
    }

    pub fn next(&mut self) -> Option<char> {
        self._current = self._contents.next();
        self._current
    }

    pub fn current(&self) -> Option<char> {
        self._current
    }
}

fn tokenizer(xml_contents: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut iter = XmlChars::new(xml_contents);

    iter.next();
    while let Some(c) = iter.current() {
        match c {
            '<' => consume_entity(&mut iter, &mut tokens),
            _ => break,
        }
    }

    tokens
}

fn is_valid(_tokens: &Vec<Token>) -> bool {
    false
}

fn consume_entity(mut iter: &mut XmlChars, mut tokens: &mut Vec<Token>) {
    iter.next();
    while let Some(c) = iter.current() {
        match c {
            '>' => break,                                                  // close
            '?' => consume_processing_instruction(&mut iter, &mut tokens), // prolog or processing instruction
            '!' => consume_comment(&mut iter, &mut tokens),                // comment or dtd
            '/' => break,
            _ => break,
        }
    }
}

fn consume_comment(mut iter: &mut XmlChars, mut tokens: &mut Vec<Token>) {
    if Some('-') != iter.next() {
        consume_dtd(&mut iter, &mut tokens);
        return;
    }
    if Some('-') != iter.next() {
        panic!("Malformed comment");
    }

    let mut comment = Vec::new();
    let mut current = iter.next();
    let mut end = 0;

    while let Some(c) = current {
        match c {
            '>' if end != 2 => panic!("Malformed comment end"),
            '>' if end == 2 => {
                tokens.push(Token::Comment(comment.iter().collect::<String>()));
                return;
            }
            '-' => end += 1,
            _ => {
                end = 0;
                comment.push(c)
            }
        }
        current = iter.next();
    }
}

fn consume_white_spaces(iter: &mut XmlChars) {
    while let Some(c) = iter.current() {
        match c {
            ' ' | '\t' | '\n' | '\r' => {
                iter.next();
                continue;
            }
            _ => break,
        }
    }
}

fn consume_name(iter: &mut XmlChars) -> (Option<String>, String) {
    let mut namespace: Option<String> = None;
    let mut name = "".to_string();
    let mut has_namespace = false;
    let mut attr = Vec::new();
    while let Some(c) = iter.current() {
        match c {
            ' ' | '\t' | '\n' | '\r' | '=' => {
                if attr.is_empty() {
                    panic!("Malformed attribute name")
                }
                name = attr.iter().collect::<String>();
                break;
            }
            ':' if !has_namespace => {
                has_namespace = true;
                namespace = Some(attr.iter().collect::<String>());
                attr.clear();
            }
            ':' if has_namespace => panic!("Malformed attribute name"),
            _ if is_illegal(&c) => panic!("Malformed attribute name"),
            _ => attr.push(c),
        }

        iter.next();
    }

    return (namespace, name);
}

fn consume_value(iter: &mut XmlChars) -> String {
    iter.next();

    let mut value = Vec::new();
    while let Some(c) = iter.current() {
        match c {
            '"' => {
                return value.iter().collect::<String>();
            }
            '\\' => {
                value.push(c);
                if let Some(c) = iter.next() {
                    value.push(c);
                } else {
                    panic!("Unexpected end of stream");
                }
            }
            _ => value.push(c),
        }

        iter.next();
    }
    panic!("Unexpected end of stream: consume value");
}

fn consume_tag_attribute(mut iter: &mut XmlChars, tokens: &mut Vec<Token>) {
    consume_white_spaces(&mut iter);
    let (ns, name) = consume_name(&mut iter);
    consume_white_spaces(&mut iter);
    if Some('=') == iter.current() {
        iter.next();
        consume_white_spaces(&mut iter);
        if Some('"') == iter.current() {
            let value = consume_value(&mut iter);
            if Some('"') == iter.current() {
                tokens.push(Token::Attr {
                    ns,
                    name,
                    value: Some(value),
                });
            } else {
                panic!("Malformed attribute: missing closing \"");
            }
        } else {
            panic!("Malformed attribute");
        }
    } else {
        tokens.push(Token::Attr {
            ns,
            name,
            value: None,
        });
    }
}

fn consume_processing_instruction(mut iter: &mut XmlChars, mut tokens: &mut Vec<Token>) {
    iter.next(); // remove ?
    iter.next(); // take next
    let (ns, name) = consume_name(&mut iter);
    match (ns, &name[..]) {
        (None, "xml") => {
            let (version, encoding, standalone) = consume_prolog(&mut iter);
            tokens.push(Token::Prolog {
                version,
                encoding,
                standalone, 
            });
        }
        (None, _) => {
            let content = consume_pi(&mut iter);
            tokens.push(Token::ProcessingInstructions { name, content });
        }
        (Some(_), _) => panic!("Processing instruction does not have namespace"),
    }

    if Some('>') != iter.next() {
        panic!("Malformed prolog end");
    }
}

fn consume_prolog(mut iter: &mut XmlChars) -> (String, String, bool) {
    let mut version = s("1.0");
    let mut encoding = s("UTF-8");
    let mut standalone = true;

    while Some('?') != iter.current() {
        consume_white_spaces(&mut iter);
        let mut tokens = Vec::new();
        consume_tag_attribute(&mut iter, &mut tokens);
        if let Some(token) = tokens.first() {
            match token {
                // TODO: make it more clear
                Token::Attr { ns: _, name, value } => match &name[..] {
                    "version" => version = value.as_ref().unwrap_or(&version).clone(),
                    "encoding" => encoding = value.as_ref().unwrap_or(&encoding).clone(),

                    // TODO "yes" || "true" 
                    // TODO are there there others
                    "standalone" if Some(&s("true")) != value.as_ref() => standalone = false,
                    _ => {}
                },
                _ => {}
            }
        }
    }

    (version, encoding, standalone)
}

fn consume_pi(iter: &mut XmlChars) -> String {
    let mut content = Vec::new();
    while Some('?') != iter.current() {
        match iter.current() {
            Some(c) => content.push(c),
            None => panic!("Unexpected end of stream"),
        }
        iter.next();
    }
    content.iter().collect::<String>()
}

fn consume_dtd(mut iter: &mut XmlChars, mut tokens: &mut Vec<Token>) {
    if (None, s("DOCTYPE")) == consume_name(&mut iter) {
        consume_white_spaces(&mut iter);
        let (ns, element) = consume_name(&mut iter);
        
        consume_white_spaces(&mut iter);
        if Some('"') == iter.current() {
            // url DTD identifer
        }
        consume_white_spaces(&mut iter);
        if Some('[') == iter.current() {
            while let Some(c) = iter.next() {
                match c {
                    ']' => break,
                    _ => continue,
                }
            }
        }
    } else {
        panic!("Malformed XML");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    #[should_panic(expected = "Malformed attribute name")]
    fn test_consume_pi_fail_no_name() {
        let mut contents = XmlChars::new(
            r#"<? target instruction instruction   ?>
        "#,
        );
        contents.next();
        let mut tokens = vec![];

        consume_processing_instruction(&mut contents, &mut tokens);

    }

    #[test] 
    fn test_consume_pi_1() {
        let mut contents = XmlChars::new(
            r#"<?target instruction instruction   ?>
        "#,
        );
        contents.next();
        let mut tokens = vec![];

        consume_processing_instruction(&mut contents, &mut tokens);

        assert_eq!(
            vec![Token::ProcessingInstructions {
                name: "target".to_string(),
                content: " instruction instruction   ".to_string(),
            }],
            tokens
        );
    }

    #[test] 
    fn test_consume_prolog_1() {
        let mut contents = XmlChars::new(
            r#"<?xml 
             version="2.0" 

             standalone="test"

             attr1
attr2="with value"
              ?>
        "#,
        );
        contents.next();
        let mut tokens = vec![];

        consume_processing_instruction(&mut contents, &mut tokens);

        assert_eq!(
            vec![Token::Prolog {
                encoding: "UTF-8".to_string(),
                version: "2.0".to_string(),
                standalone: false,
            }],
            tokens
        );
    }

    #[test]
    fn test_consume_prolog_2() {
        let mut contents = XmlChars::new(
            r#"<?xml 
             version="1.0"             encoding="UTF-8" ?>
        "#,
        );
        contents.next();
        let mut tokens = vec![];

        consume_processing_instruction(&mut contents, &mut tokens);

        assert_eq!(
            vec![Token::Prolog {
                encoding: "UTF-8".to_string(),
                version: "1.0".to_string(),
                standalone: true,
            }],
            tokens
        );
    }

    #[test]
    fn test_consume_tag_attribute_1() {
        let mut contents = XmlChars::new(r#"a-namespace:b-attribute="c-va\"lue" "#);
        contents.next();
        let mut tokens = vec![];

        consume_tag_attribute(&mut contents, &mut tokens);

        assert_eq!(
            vec![Token::Attr {
                ns: Some("a-namespace".to_string()),
                name: "b-attribute".to_string(),
                value: Some("c-va\\\"lue".to_string())
            }],
            tokens
        );
    }

    #[test]
    fn test_consume_value_1() {
        let mut contents = XmlChars::new("\"b-value\"");
        contents.next();

        let value = consume_value(&mut contents);

        assert_eq!("b-value".to_string(), value);
    }

    #[test]
    fn test_consume_value_2() {
        let mut contents = XmlChars::new(r#""b\"test""#);
        contents.next();

        let value = consume_value(&mut contents);

        assert_eq!("b\\\"test".to_string(), value);
    }

    #[test]
    fn test_consume_name_1() {
        let mut contents = XmlChars::new(r#"a-attribute="b-value"#);
        contents.next();

        let (ns, name) = consume_name(&mut contents);

        assert_eq!(None, ns);
        assert_eq!("a-attribute".to_string(), name);
    }

    #[test]
    fn test_consume_name_2() {
        let mut contents = XmlChars::new(r#"a-namespace:b-attribute="c-value"#);
        contents.next();

        let (ns, name) = consume_name(&mut contents);

        assert_eq!(Some("a-namespace".to_string()), ns);
        assert_eq!("b-attribute".to_string(), name);
    }

    #[test]
    fn test_consume_whites_paces() {
        let mut contents = XmlChars::new(" a");
        contents.next();

        consume_white_spaces(&mut contents);

        assert_eq!(Some('a'), contents.current());
    }

    #[test]
    fn test_consume_tag_comment_1() {
        let mut contents = XmlChars::new("<!--xxx xxx-->");
        contents.next();
        contents.next();
        let mut tokens = vec![];

        consume_comment(&mut contents, &mut tokens);

        assert_eq!(vec![Token::Comment("xxx xxx".to_string())], tokens);
    }

    #[test]
    #[should_panic(expected = "Malformed comment")]
    fn test_consume_tag_comment_bad_format_1() {
        let mut contents = XmlChars::new("<! -- -->");
        contents.next();
        contents.next();
        let mut tokens = vec![];

        consume_comment(&mut contents, &mut tokens);
    }

    #[test]
    #[should_panic(expected = "Malformed comment end")]
    fn test_consume_tag_comment_bad_format_2() {
        let mut contents = XmlChars::new("<!-- - ->");
        contents.next();
        contents.next();
        let mut tokens = vec![];

        consume_comment(&mut contents, &mut tokens);
    }
}

//     let mut tokens = Vec::new();
//     let mut iter = xml_contents.chars();

//     while let Some(c) = iter.next() {
//         match c {
//             ' ' => continue,
//             '<' => token_tag(&mut iter, &mut tokens),
//             _ => continue,
//         }
//     }

//     tokens
// }

// fn token_tag(mut iter: &mut Chars, tokens: &mut Vec<Token>) {
//     let mut name = "".to_string();
//     while let Some( c) = iter.next() {
//         match c {
//             '!' => { /* comment */},
//             '?' => {
//                 token_prolog(&mut iter, tokens);
//                 return;
//             },
//             ' ' => {
//                 token_attribute(&mut iter, tokens);
//             },
//             '/' => {
//                 match iter.next() {
//                     Some('>') => {
//                         if name.is_empty() {
//                             panic!("Tag without name");
//                         }
//                         tokens.push(Token::End(name.clone()));
//                         return;
//                     },
//                     Some(a) => {
//                         name = token_name(&mut iter, vec![a]);
//                         tokens.push(Token::End(name.clone()));
//                     }
//                     None => return,
//                 }
//                 return;
//             },
//             _ => {
//                 name = token_name(&mut iter, vec![c]);
//                 tokens.push(Token::Start(name.clone()))
//             }
//         }
//     }
// }

// use std::collections::HashSet;
// fn token_name(mut iter: &mut Chars, mut name: Vec<char>) -> String {
//     let NAME_ILLEGAL_CHARS: HashSet<char> = "!@#$%^&*+,.~`\"'()[]{}".chars().collect();
//     while let Some(c) = iter.next() {
//         match c {
//             ' ' | '/' | '>' => break,
//             _ if NAME_ILLEGAL_CHARS.contains(&c) => panic!("Illegal character found!"),
//             _ => name.push(c),
//         }
//     }
//     name.iter().collect::<String>()
// }

// fn token_prolog(mut iter: &mut Chars, mut tokens: &mut Vec<Token>) {

// }

// fn token_attribute(mut iter: &mut Chars, mut tokens: &mut Vec<Token>) {

// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_token_name() {
//         assert_eq!("test", token_name(&mut "est ".chars(), vec!['t']));
//     }

//     #[test]
//     fn test_parse_simplest() {

//         assert_eq!(vec![Token::Start("test".to_string())], vec![Token::Start("test".to_string())]);
//         assert!(vec![Token::Start("bakbakbab".to_string())] != vec![Token::Start("test".to_string())]);

//         let expected = vec![Token::Start("test".to_string()), Token::End("test".to_string())];
//         let result = tokenzier("<test/>");

//         assert_eq!(expected, result);

//         let _xml = r#"<?xml version="1.0" encoding="UTF-8"?><test/>"#;

//     }

// }
