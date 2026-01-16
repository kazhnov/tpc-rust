use std::{collections::HashMap, fs, rc::Rc, env, io::BufWriter, io::Write, process::Command};

#[derive(Debug, PartialEq, Eq)]
#[allow(unused)]
enum Word {
    Name(String),
    Semicolon,
    OpeningTab(usize),
    OpenParenthesis,
    CloseParenthesis,
    Signed,
    Unsigned,
    Period,
    Comma,
    Colon,
    Number(String),
    OpenCurly,
    CloseCurly,
    StringLiteral(String),
    Equals,
    DoubleEquals,
    Unequals,
    LessThan,
    GreaterThan,
    Plus,
    Minus,
    Star,
    ForwardSlash,
    BackSlash,
    A,
    Ala,
    Alasa,
    Ale,
    Anpa,
    Ante,
    Anu,
    Asen,
    Awen,
    E,
    En,
    Ijo,
    Ilo,
    Insa,
    Jo,
    Kama,
    Ken,
    Kepeken,
    Kulupu,
    La,
    Li,
    Lili,
    Linja,
    Lipu,
    Lon,
    Luka,
    Lukin,
    Mute,
    Nanpa,
    Nasin,
    Ni,
    Nimi,
    O,
    Open,
    Pakala,
    Pali,
    Pana,
    Pi,
    Pini,
    Poki,
    Pona,
    Sama,
    Seme,
    Sin,
    Sitelen,
    Suli,
    Tan,
    Tawa,
    Telo,
    Tenpo,
    Tomo,
    Tu,
    Wan,
    Wile,
    Weka,
}

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Undefined = 0,
    Comparing,
    Linear,
    Scaling,
    Unary,
    TheBiggest,
}

impl Precedence {
    fn next(&self) -> Precedence {
        match self {
            Self::Undefined => Self::Comparing,
            Self::Comparing => Self::Linear,
            Self::Linear => Self::Scaling,
            Self::Scaling => Self::Unary,
            Self::Unary => Self::TheBiggest,
            Self::TheBiggest => Self::TheBiggest,
        }
    }
}

fn get_precedence(binary_expression_type: BinaryExpressionType) -> Precedence {
    return match binary_expression_type {
        BinaryExpressionType::Add | BinaryExpressionType::Subtract => Precedence::Linear,
        BinaryExpressionType::Multiply | BinaryExpressionType::Divide => Precedence::Scaling,
        BinaryExpressionType::GreaterThan
        | BinaryExpressionType::Equals
        | BinaryExpressionType::LessThan => Precedence::Comparing,
    };
}

struct Lexer {
    debug_mode: bool,
    buffer: String,
    current_position: usize,
}

impl Lexer {
    fn consume(&mut self) {
        self.current_position += 1;
    }

    fn peek(&self) -> Option<char> {
        self.buffer.chars().nth(self.current_position)
    }

    fn lex(&mut self) -> Vec<Word> {
        let mut words: Vec<Word> = Vec::new();
        let mut c;

        let mut line_number = 0usize;
        let mut is_line_start = true;

        while self.current_position < self.buffer.len() {
            c = match self.peek() {
                None => break,
                Some(ch) => ch,
            };

            if c.is_alphabetic() || c == '_' {
                is_line_start = false;
                let firstchar = self.current_position;
                loop {
                    match self.peek() {
                        None => break,
                        Some(ch) => c = ch,
                    }
                    if !(c.is_alphanumeric() || c == '_') {
                        break;
                    }

                    self.consume();
                }
                let name = self.buffer[firstchar..self.current_position].to_string();
                let mut prefix = "keyword";

                let token = match name.as_str() {
                    "ale" => Word::Ale,
                    "sin" => Word::Sin,
		    "a" => Word::A,
                    "o" => Word::O,
                    "e" => Word::E,
                    "en" => Word::En,
                    "kama" => Word::Kama,
                    "li" => Word::Li,
                    "nanpa" => Word::Nanpa,
                    "sitelen" => Word::Sitelen,
                    "suli" => Word::Suli,
                    "lili" => Word::Lili,
                    "telo" => Word::Telo,
                    "signed" => Word::Signed,
                    "unsigned" => Word::Unsigned,
                    "la" => Word::La,
                    "ante" => Word::Ante,
                    "tenpo" => Word::Tenpo,
                    "pini" => Word::Pini,
                    "awen" => Word::Awen,
                    "linja" => Word::Linja,
                    "asen" => Word::Asen,
                    "sama" => Word::Sama,
                    "pali" => Word::Pali,
                    "kepeken" => Word::Kepeken,
                    "ni" => Word::Ni,
                    "weka" => Word::Weka,
                    "tu" => Word::Tu,
                    "wan" => Word::Wan,
                    "luka" => Word::Luka,
                    "pana" => Word::Pana,
                    "pi" => Word::Pi,
                    _ => {
                        prefix = "name";
                        Word::Name(name.clone())
                    }
                };

                words.push(token);

                if self.debug_mode {
                    println!("{prefix}: {name}");
                }
            } else if c.is_numeric() {
                is_line_start = false;
                let firstchar = self.current_position;
                loop {
                    c = match self.peek() {
                        None => break,
                        Some(ch) => ch,
                    };

                    if !c.is_alphanumeric() {
                        break;
                    }

                    self.consume();
                }

                let value = self.buffer[firstchar..self.current_position].to_string();

                words.push(Word::Number(value.clone()));

                if self.debug_mode {
                    println!("number: {value}");
                }
            } else if c == '"' {
                is_line_start = false;
                let firstchar = self.current_position;
                loop {
                    match self.peek() {
                        None => break,
                        Some(ch) => c = ch,
                    }
                    if c == '"' {
                        break;
                    }

                    self.consume();
                }
                let string = self.buffer[firstchar..self.current_position].to_string();
		words.push(Word::StringLiteral(string));
            } else if c == '\n' {
                line_number += 1;
                is_line_start = true;
                self.consume();
            } else if c == '\t' && is_line_start {
                let mut tab_count = 0usize;

                loop {
                    c = match self.peek() {
                        None => break,
                        Some(c) => c,
                    };

                    if c == '\t' {
                        tab_count += 1;
                        self.consume();
                    } else {
                        break;
                    }
                }

                if self.debug_mode {
                    println!("found {} opening tabs", tab_count);
                }

                words.push(Word::OpeningTab(tab_count));
            } else if c.is_whitespace() {
                is_line_start = false;
                self.consume();
            } else {
                is_line_start = false;
                let token = match c {
                    ';' => Word::Semicolon,
                    '(' => Word::OpenParenthesis,
                    ')' => Word::CloseParenthesis,
                    ',' => Word::Comma,
                    ':' => Word::Colon,
                    '=' => Word::Equals,
                    '<' => Word::LessThan,
                    '>' => Word::GreaterThan,
                    '+' => Word::Plus,
                    '-' => Word::Minus,
                    '*' => Word::Star,
                    '.' => Word::Period,
                    '/' => Word::ForwardSlash,
                    _ => {
                        panic!(
                            "Unexpected character {c} at position {}",
                            self.current_position
                        );
                    }
                };
                self.consume();
                if self.debug_mode {
                    println!("symbol: {:#?}", token);
                }
                words.push(token);
            }
        }
        words
    }
}

#[derive(Debug)]
enum Token {
    // Composite tokens
    OTawa,
    OSin,
    LiKamaSama,
    Tenpo,
    Asen,
    LiKepeken,
    OPali,
    LiPaliENi,
    OWeka,
    LiPanaE,
    OPini,

    // Execute a function
    O,
    E,
    Pali,
    En,
    Kepeken,
    A,
    
    // Punctuation
    Period,
    Comma,
    OpeningTab(usize),

    //
    Name(String),
    Number(String),
    StringLiteral(String),
    Colon,
    // Clauses
    TenpoPi,
    TenpoAlePi,
    La,
    Ante,
    Li,

    // Types
    Nanpa,
    Sitelen,
    Suli,
    Lili,
    Telo,
    Telotu,
    Linja,
    Awen,
    // Parens
    OpenParenthesis,
    CloseParenthesis,
    // Comparison
    Equals,
    DoubleEquals,
    Unequals,
    LessThan,
    GreaterThan,
    // Arithmetics
    Plus,
    Minus,
    Star,
    ForwardSlash,
    BackSlash,
}

struct Abstracter {
    words: Vec<Word>,
    current_word: usize,
    tokens: Vec<Token>,
    debug_mode: bool,
}

impl Abstracter {
    fn peek(&self) -> Option<&Word> {
        self.words.get(self.current_word)
    }

    fn consume(&mut self) {
        self.current_word += 1;
    }

    fn push(&mut self, token: Token) {
        if self.debug_mode {
            println!("token: {:#?}", token);
        }
        self.tokens.push(token);
    }

    fn unconsume(&mut self) {
        self.current_word -= 1;
    }

    fn is_number(word: &Word) -> bool {
        match word {
            Word::Wan | Word::Tu | Word::Luka => true,
            _ => false,
        }
    }

    fn expect(&self, expected: Word) -> bool {
        match self.peek() {
            None => return false,
            Some(word) => {
                if std::mem::discriminant(word) == std::mem::discriminant(&expected) {
                    return true;
                } else {
                    return false;
                }
            }
        }
    }

    fn tokenize_o(&mut self) {
        if !self.expect(Word::O) {
            unreachable!();
        }

        self.consume();

        // o tawa
        if self.expect(Word::Tawa) {
            self.consume();
            self.push(Token::OTawa);
            return;
        }

        // o 'name of function'
        if self.expect(Word::Name("".to_string())) {
            self.push(Token::O);
            return;
        }

        if self.expect(Word::Weka) {
            self.consume();
            self.push(Token::OWeka);
            return;
        }

        // o sin e
        if self.expect(Word::Sin) {
            self.consume();
            if self.expect(Word::E) {
                self.consume();
                self.push(Token::OSin);
                return;
            }
            panic!("No 'e' in 'o sin' statement");
        }

        if self.expect(Word::Pini) {
            self.consume();
            self.push(Token::OPini);
            return;
        }

        panic!("not a valid o statement");
    }

    fn tokenize_nanpas(&mut self) {
        let mut result = 0;

        loop {
            result += match self.peek() {
                None => break,
                Some(word) => match word {
                    Word::Wan => 1,
                    Word::Tu => 2,
                    Word::Luka => 5,
                    _ => break,
                },
            };
            self.consume();
        }
        self.push(Token::Number(result.to_string()));
    }

    fn tokenize_name(&mut self) {
        if !self.expect(Word::Name("".to_string())) {
            return;
        }

        let name = match self.peek().unwrap() {
            Word::Name(name) => name,
            _ => return,
        };

        self.push(Token::Name(name.clone()));

        self.consume();
    }

    fn tokenize_number(&mut self) {
        if !self.expect(Word::Number("".to_string())) {
            return;
        }

        let number = match self.peek().unwrap() {
            Word::Number(number) => number,
            _ => return,
        };

        self.push(Token::Number(number.clone()));

        self.consume();
    }

    fn tokenize_tenpo(&mut self) {
        if !self.expect(Word::Tenpo) {
            return;
        }
        self.consume();

        // tenpo ale pi
        if self.expect(Word::Ale) {
            self.consume();

            if !self.expect(Word::Pi) {
                panic!("no 'pi' in 'tenpo ale pi'");
            }
            self.consume();

            self.push(Token::TenpoAlePi);
            return;
        }

        // tenpo pi
        if self.expect(Word::Pi) {
            self.consume();
            self.push(Token::TenpoPi);
            return;
        }

        panic!("not a valid tenpo statement");
    }

    fn tokenize_li(&mut self) {
        if !self.expect(Word::Li) {
            return;
        }

        self.consume();

        // li kama sama
        if self.expect(Word::Kama) {
            self.consume();

            if !self.expect(Word::Sama) {
                panic!("no 'sama' in 'kama sama' statement");
            }

            self.consume();

            self.push(Token::LiKamaSama);

            return;
        }

        // li kepeken
        if self.expect(Word::Kepeken) {
            self.consume();

            self.push(Token::LiKepeken);

            return;
        }

        // li pali e ni:
        if self.expect(Word::Pali) {
            self.consume();
            if !self.expect(Word::E) {
                panic!("no 'e' in 'li pali e ni' token");
            }
            self.consume();

            if !self.expect(Word::Ni) {
                panic!("no 'ni' in 'li pali e ni' token");
            }
            self.consume();
            self.push(Token::LiPaliENi);
        }

        if self.expect(Word::Pana) {
            self.consume();
            if !self.expect(Word::E) {
                panic!("no 'e' in 'li pana e' token");
            }
            self.consume();
            self.push(Token::LiPanaE);
        }
    }

    fn tokenize_arithmetics(&mut self) {
        let word = match self.peek() {
            None => unreachable!(),
            Some(word) => word,
        };

        let token = match word {
            Word::Plus => Token::Plus,
            Word::Minus => Token::Minus,
            Word::Star => Token::Star,
            Word::ForwardSlash => Token::ForwardSlash,
            Word::Equals => Token::Equals,
            _ => todo!(),
        };

        self.consume();

        self.push(token);
    }

    fn tokenize(&mut self) {
        while self.current_word < self.words.len() {
            let word = match self.peek() {
                None => return,
                Some(word) => word,
            };

            match word {
                Word::Name(_) => self.tokenize_name(),
                Word::Number(_) => self.tokenize_number(),
		Word::StringLiteral(string) => {
		    self.push(Token::StringLiteral(string.to_string()));
		    self.consume();
		}
                Word::O => self.tokenize_o(),
                Word::Tenpo => self.tokenize_tenpo(),
                Word::Plus | Word::Minus | Word::ForwardSlash | Word::Star | Word::Equals => {
                    self.tokenize_arithmetics()
                }
                Word::Nanpa => {
                    self.push(Token::Nanpa);
                    self.consume();
                }
                Word::Period => {
                    self.push(Token::Period);
                    self.consume();
                }
                Word::Li => self.tokenize_li(),
                Word::Kepeken => {
                    self.push(Token::Kepeken);
                    self.consume();
                }
                Word::En => {
                    self.push(Token::En);
                    self.consume();
                }
                Word::E => {
                    self.push(Token::E);
                    self.consume();
                }
		Word::A => {
		    self.push(Token::A);
		    self.consume();
		}
                Word::OpeningTab(tabs) => {
                    self.push(Token::OpeningTab(*tabs));
                    self.consume();
                }
                Word::Pali => {
                    self.push(Token::Pali);
                    self.consume();
                }
                Word::OpenParenthesis => {
                    self.push(Token::OpenParenthesis);
                    self.consume();
                }
                Word::CloseParenthesis => {
                    self.push(Token::CloseParenthesis);
                    self.consume();
                }
                Word::La => {
                    self.push(Token::La);
                    self.consume();
                }
                Word::Wan | Word::Tu | Word::Luka => self.tokenize_nanpas(),
                _ => todo!("{:#?}", word),
            }
        }
    }
}

#[derive(Debug)]
enum Expression {
    Unary(Box<UnaryExpression>),
    Binary(Box<BinaryExpression>),
}

impl Expression {
    fn get_type_name(&self, scope: &Scope) -> String {
        match self {
            Self::Unary(unary) => (*unary).get_type_name(scope).unwrap(),
            Self::Binary(binary) => (*binary).get_type_name(scope),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BinaryExpressionType {
    Add,
    Multiply,
    Subtract,
    Divide,
    GreaterThan,
    Equals,
    LessThan,
}

#[derive(Debug)]
struct BinaryExpression {
    lhs: Box<Expression>,
    rhs: Box<Expression>,
    kind: BinaryExpressionType,
}

impl BinaryExpression {
    fn get_type_name(&self, scope: &Scope) -> String {
        let lhstype = match &*self.lhs {
            Expression::Binary(binary) => binary.get_type_name(scope),
            Expression::Unary(unary) => unary
                .get_type_name(scope)
                .expect("binary expression hand has no type"),
        };

        let rhstype = match &*self.rhs {
            Expression::Binary(binary) => binary.get_type_name(scope),
            Expression::Unary(unary) => unary
                .get_type_name(scope)
                .expect("binary expression hand has no type"),
        };

        if lhstype == rhstype {
            return rhstype;
        } else {
            panic!(
                "Expressions have incompatible types: {:#?} and {:#?}",
                lhstype, rhstype
            );
        }
    }
}

// Returning Expressions

#[derive(Debug)]
struct NimiExpression {
    value: String,
}

#[derive(Debug)]
struct LinjaExpression {
    value: String,
}

#[derive(Debug)]
struct NanpaExpression {
    value: isize,
}

#[derive(Debug)]
enum UnaryExpression {
    Nanpa(Box<NanpaExpression>),
    Nimi(Box<NimiExpression>),
    O(Box<OExpression>),
    Linja(Box<LinjaExpression>),
}

impl UnaryExpression {
    fn get_type_name(&self, scope: &Scope) -> Option<String> {
        match self {
            Self::Nanpa(_) => Some("nanpa".to_string()),
            Self::Linja(_) => Some("linja".to_string()),
            Self::Nimi(nimi) => Some(scope.get_variable(&nimi.value).unwrap().0.type_name.clone()),
            Self::O(o) => Some(scope.get_function(&o.nimi.value).unwrap().return_type.clone()?),
        }
    }
}

#[derive(Debug)]
// Non-Returning
struct AsenpeliStatement {
    value: String,
}
#[derive(Debug)]
struct Parenthesis {
    nodes: Vec<Node>,
}

#[derive(Debug)]
struct OtawaStatement {
    expr: Box<Expression>,
}

#[derive(Debug)]
struct KepekenStatement {
    nimi: NimiExpression
}

#[derive(Debug)]
struct OWekaStatement {
    expr: Option<Box<Expression>>,
}

#[derive(Debug)]
struct OSinStatement {
    var_type: String,
    name: NimiExpression,
    expr: Option<Box<Expression>>,
}

#[derive(Debug)]
struct LiKamaSamaStatement {
    nimi: NimiExpression,
    expression: Box<Expression>,
}

#[derive(Debug)]
struct PaliStatement {
    nimi: NimiExpression,
    params: Vec<(String, NimiExpression)>,
    nodes: Vec<Node>,
    retval: Option<String>,
}

#[derive(Debug)]
struct PaliDeclaration {
    nimi: NimiExpression,
    params: Vec<(String, NimiExpression)>,
    retval: Option<String>
}

#[derive(Debug)]
struct OExpression {
    nimi: NimiExpression,
    params: Vec<Expression>,
}

#[derive(Debug)]
struct TenpoStatement {
    expr: Box<Expression>,
    nodes: Vec<Node>,
}

#[derive(Debug)]
struct TenpoAleStatement {
    expr: Expression,
    nodes: Vec<Node>,
}

#[derive(Debug)]
enum Node {
    Expression(Box<Expression>),
    Asenpeli(Box<AsenpeliStatement>),
    LiKamaSama(Box<LiKamaSamaStatement>),
    Tenpo(Box<TenpoStatement>),
    TenpoAle(Box<TenpoAleStatement>),
    Otawa(Box<OtawaStatement>),
    OSin(Box<OSinStatement>),
    Pali(Box<PaliStatement>),
    PaliDeclaration(Box<PaliDeclaration>),
    O(Box<OExpression>),
    OpeningTab(usize),
    OWeka(Box<OWekaStatement>),
    Parenthesis(Box<Parenthesis>),
    Kepeken(Box<KepekenStatement>)
}

struct Parser {
    tokens: Vec<Token>,
    current_token: usize,
    nodes: Vec<Node>,
    debug_mode: bool,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current_token)
    }

    fn consume(&mut self) {
        self.current_token += 1;
    }

    fn expect(&self, expected: Token) -> bool {
        match self.peek() {
            None => false,
            Some(token) => {
                if std::mem::discriminant(token) == std::mem::discriminant(&expected) {
                    true
                } else {
                    false
                }
            }
        }
    }

    fn parse_nanpa_expression(&mut self) -> Result<NanpaExpression, String> {
        let token = match self.peek() {
            None => panic!(),
            Some(token) => token,
        };

        match token {
            Token::Number(number) => {
                let result = Ok(NanpaExpression {
                    value: number.parse().expect("not a valid number"),
                });
                self.consume();

                result
            }
            _ => Err("not a number".to_string()),
        }
    }

    fn parse_nimi_expression(&mut self) -> Result<NimiExpression, String> {
        let token = match self.peek() {
            None => panic!(),
            Some(token) => token,
        };

        match token {
            Token::Name(name) => {
                let result = Ok(NimiExpression {
                    value: name.clone(),
                });
                self.consume();

                result
            }
            _ => Err("not a name".to_string()),
        }
    }

    fn parse_kepeken(&mut self) -> Result<KepekenStatement, String> {
	if !self.expect(Token::StringLiteral("".to_string())) {
	    return Err("not a kepeken statement".to_string());
	}

	let nimi = self.parse_nimi_expression()?;

	return Ok(KepekenStatement{
	    nimi
	})
    }
    
    fn parse_unary_expression(&mut self) -> Result<UnaryExpression, String> {
        let token = match self.peek() {
            None => panic!(),
            Some(token) => token,
        };

        if matches!(token, Token::Number(_)) {
            return Ok(UnaryExpression::Nanpa(Box::new(
                self.parse_nanpa_expression()?,
            )));
        } else if matches!(token, Token::Name(_)) {
            return Ok(UnaryExpression::Nimi(Box::new(
                self.parse_nimi_expression()?,
            )));
        } else if matches!(token, Token::O) {
            return Ok(UnaryExpression::O(Box::new(self.parse_o()?)));
        } else if matches!(token, Token::StringLiteral(_)) {
            todo!("parse linja expression");
        }

        Err("Not an unary expression".to_string())
    }

    fn parse_expression(&mut self, min_precedence: Precedence) -> Result<Expression, String> {
        let lhs_unary = match self.parse_unary_expression() {
            Err(err) => return Err(err),
            Ok(expr) => expr,
        };

        let mut lhs_expr = Expression::Unary(Box::new(lhs_unary));

        loop {
            let token = match self.peek() {
                None => break,
                Some(token) => token,
            };

            let binary_type: BinaryExpressionType = match token {
                Token::Plus => BinaryExpressionType::Add,
                Token::Minus => BinaryExpressionType::Subtract,
                Token::Star => BinaryExpressionType::Multiply,
                Token::ForwardSlash => BinaryExpressionType::Divide,
                Token::GreaterThan => BinaryExpressionType::GreaterThan,
                Token::LessThan => BinaryExpressionType::LessThan,
                Token::Equals => BinaryExpressionType::Equals,
                _ => break,
            };

            let current_precedence = get_precedence(binary_type);

            if current_precedence < min_precedence {
                break;
            }

            self.consume();

            let rhs_expr = match self.parse_expression(current_precedence.next()) {
                Err(err) => return Err(err),
                Ok(expr) => expr,
            };

            let lhs_expr2 = lhs_expr;

            // get variable type
            let vartype: String;
            let binary_expression = BinaryExpression {
                kind: binary_type,
                lhs: Box::new(lhs_expr2),
                rhs: Box::new(rhs_expr),
            };

            lhs_expr = Expression::Binary(Box::new(binary_expression));
        }

        Ok(lhs_expr)
    }

    fn parse_otawa(&mut self) -> Result<OtawaStatement, String> {
        let token = match self.peek() {
            None => panic!(),
            Some(token) => token,
        };
        if !matches!(token, Token::OTawa) {
            return Err(String::from("Not an otawa statement"));
        };

        self.consume();

        let expr: Expression = match self.parse_expression(Precedence::Undefined) {
            Err(err) => return Err(err),
            Ok(expr) => expr,
        };

        Ok(OtawaStatement {
            expr: Box::new(expr),
        })
    }

    // tenpo pi 'expr' la
    fn parse_tenpo(&mut self) -> Result<TenpoStatement, String> {
        if !self.expect(Token::TenpoPi) {
            return Err("not a tenpo statement".to_string());
        }
        self.consume();

        let expr = self.parse_expression(Precedence::Undefined)?;

        if !self.expect(Token::La) {
            return Err("no la in tenpo statement".to_string());
        }
        self.consume();

        let mut nodes: Vec<Node> = Vec::new();

        loop {
            nodes.push(self.parse_statement());
            if self.expect(Token::OPini) {
                self.consume();
                break;
            }
        }
        Ok(TenpoStatement {
            expr: Box::new(expr),
            nodes,
        })
    }

    // o 'name' e 'name' e 'name'.
    fn parse_o(&mut self) -> Result<OExpression, String> {
        if !self.expect(Token::O) {
            return Err("not an o expression".to_string());
        };
        self.consume();

        let nimi: NimiExpression = self.parse_nimi_expression()?;

        // params
        let mut params: Vec<Expression> = Vec::new();

        // e
        if self.expect(Token::E) {
            self.consume();

            loop {
                params.push(self.parse_expression(Precedence::Undefined)?);

                if !self.expect(Token::E) {
                    break;
                }
                self.consume();
            }
        }

	// a!
	if !self.expect(Token::A) {
	    return Err("not happy enough!".to_string());
	}
	self.consume();

        Ok(OExpression { nimi, params })
    }

    fn parse_o_weka(&mut self) -> Result<OWekaStatement, String> {
        if !self.expect(Token::OWeka) {
            return Err("not an o weka statement".to_string());
        };
        self.consume();
        let mut expr = None;
        if self.expect(Token::E) {
            self.consume();
            expr = Some(Box::new(self.parse_expression(Precedence::Undefined)?));
        };

        Ok(OWekaStatement { expr })
    }

    // pali 'name' li kepeken 'args' li pali e ni:
    fn parse_pali(&mut self) -> Result<(Option<PaliStatement>, Option<PaliDeclaration>), String> {
        if !self.expect(Token::Pali) {
            return Err("not a pali statement".to_string());
        }
        self.consume();

        let mut nimi = self.parse_nimi_expression()?;

        let mut params: Vec<(String, NimiExpression)> = Vec::new();
        let mut has_params = false;
        let mut retval: Option<String> = None;
        let mut has_type = false;
        loop {
            if self.expect(Token::LiPanaE) && !has_type {
                self.consume();
                retval = Some(self.parse_type()?);
                has_type = true;
            } else if self.expect(Token::LiKepeken) && !has_params {
                self.consume();

                loop {
                    params.push((self.parse_type()?, self.parse_nimi_expression()?));

                    if !self.expect(Token::En) {
                        break;
                    }
                    self.consume();
                }
                has_params = true;
            } else {
                break;
            }
        }

        if !self.expect(Token::LiPaliENi) {
	    return Ok(
		(None,
		 Some(PaliDeclaration{
		     nimi,
		     params,
		     retval,
		 })
		)
	    );
        }
        self.consume();

        let mut nodes: Vec<Node> = Vec::new();

        loop {
            if self.expect(Token::OPini) {
                self.consume();
                break;
            }

            let node: Node = self.parse_statement();
            nodes.push(node);
        }

        let oweka = Node::OWeka(Box::new(OWekaStatement { expr: None }));
        if !nodes.iter().any(|node| {
            return std::mem::discriminant(node) == std::mem::discriminant(&oweka);
        }) {
            nodes.push(oweka);
        }

        Ok((Some(PaliStatement {
            nimi,
            params,
            nodes,
            retval,
        }), None))
    }

    fn parse_type(&mut self) -> Result<String, String> {
        let token = match self.peek() {
            None => return Err("Unexpected end of file".to_string()),
            Some(token) => token,
        };

        let vartype = match token {
            Token::Nanpa => "nanpa".to_string(),
            Token::Linja => "linja".to_string(),
            _ => return Err("Not a type".to_string()),
        };
        self.consume();

        let token = match self.peek() {
            None => return Err("Unexpected end of file".to_string()),
            Some(token) => token,
        };

        let is_const = matches!(token, Token::Awen);

        if is_const {
            self.consume();
            todo!("implement constants");
        }

        Ok(vartype)
    }

    fn parse_o_sin(&mut self) -> Result<OSinStatement, String> {
        // consume "o sin e"
        self.consume();

        // get type
        let variable_type = self.parse_type()?;

        // get name
        let name = self.parse_nimi_expression()?;

        Ok(OSinStatement {
            expr: None,
            name: name,
            var_type: "nanpa".to_string(),
        })
    }

    fn parse_li_kama_sama(&mut self) -> Result<LiKamaSamaStatement, String> {
        let nimi = self.parse_nimi_expression()?;

        if !self.expect(Token::LiKamaSama) {
            return Err("No 'li kama sama' in kama sama statement".to_string());
        }

        self.consume();

        let expression = self.parse_expression(Precedence::Undefined)?;

        Ok(LiKamaSamaStatement {
            nimi,
            expression: Box::new(expression),
        })
    }

    fn parse_parenthesis(&mut self) -> Result<Parenthesis, String> {
        if !self.expect(Token::OpenParenthesis) {
            return Err("not an opening parenthesis".to_string());
        };
        self.consume();

        let mut nodes: Vec<Node> = Vec::new();
        loop {
            if self.expect(Token::CloseParenthesis) {
                self.consume();
                break;
            }
            nodes.push(self.parse_statement());
        }

        Ok(Parenthesis { nodes })
    }

    fn parse_statement(&mut self) -> Node {
        let token = match self.peek() {
            None => panic!("Out of tokens!"),
            Some(token) => token,
        };

        if self.debug_mode {
            println!("parsing statement starting from token: {:#?}", token);
        }

        let node = match token {
            Token::OTawa => Node::Otawa(Box::new(self.parse_otawa().unwrap())),
            Token::Asen => todo!("Asenpeli"),
            Token::Tenpo => todo!("Tenpo"),
            Token::Name(_) => Node::LiKamaSama(Box::new(self.parse_li_kama_sama().unwrap())),
            Token::OSin => Node::OSin(Box::new(self.parse_o_sin().unwrap())),
	    Token::Pali => {
		let pali = self.parse_pali().unwrap();
		if pali.1.is_some() {
		    Node::PaliDeclaration(Box::new(pali.1.unwrap()))
		} else {
		    Node::Pali(Box::new(pali.0.unwrap()))
		}
	    },
            Token::OpeningTab(tabs) => {
                let tabs = *tabs;
                self.consume();
                Node::OpeningTab(tabs)
            }
            Token::OWeka => Node::OWeka(Box::new(self.parse_o_weka().unwrap())),
            Token::O => Node::O(Box::new(self.parse_o().unwrap())),
            Token::OpenParenthesis => {
                Node::Parenthesis(Box::new(self.parse_parenthesis().unwrap()))
            }
            Token::TenpoPi => Node::Tenpo(Box::new(self.parse_tenpo().unwrap())),
	    Token::Kepeken => Node::Kepeken(Box::new(self.parse_kepeken().unwrap())),
            _ => todo!("{:#?}", token),
        };
        node
    }

    fn parse(&mut self) {
        loop {
            match self.peek() {
                None => break,
                Some(_) => {}
            };

            let node = self.parse_statement();
            self.nodes.push(node);
        }
    }
}

#[derive(Debug)]
struct Variable {
    type_name: String,
    stack_pos: usize,
}

#[derive(Debug)]
struct Function {
    return_type: Option<String>,
    parameter_types: Vec<String>,
}

#[derive(Debug)]
enum EnvironmentName {
    Variable(Variable),
}

#[derive(Debug)]
struct Environment {
    names: HashMap<String, EnvironmentName>,
    stack_pointer: usize,
    tab_depth: usize,
}

#[derive(Debug)]
struct Type {
    size: usize,
    name: String
}

#[derive(Debug)]
struct Scope {
    functions: HashMap<String, Function>,
    types: HashMap<String, Rc<Type>>,
    envs: Vec<Environment>,
    label_counter: usize,
}

impl Scope {
    fn get_environment_mut(&mut self) -> &mut Environment {
        self.envs.last_mut().unwrap()
    }

    fn get_environment(&self) -> &Environment {
        self.envs.last().unwrap()
    }

    fn get_type(&self, name: &str) -> Option<Rc<Type>> {
	self.types.get(name).cloned()
    }
    
    fn add_function(&mut self, pali: &PaliStatement) {
        self.functions.insert(
            pali.nimi.value.to_owned(),
            Function {
                return_type: pali.retval.clone(),
                parameter_types: pali.params.iter().map(|val| val.0.clone()).collect(),
            },
        );
    }

    fn declare_function(&mut self, pali: &PaliDeclaration) {
	self.functions.insert(
            pali.nimi.value.to_owned(),
            Function {
                return_type: pali.retval.clone(),
                parameter_types: pali.params.iter().map(|val| val.0.clone()).collect(),
            },
        );
    }

    fn add_variable(&mut self, name: &str, variable_type: &str, reg: Option<&str>, writer: &mut BufWriter<fs::File>) -> &EnvironmentName {
	let size = self.get_type(variable_type).unwrap().size;

	if reg.is_some() { 
	    Generator::push_reg(reg.unwrap(), size, self, writer);
	} else {
	    println!("    sub rsp, {size}");
	}
	
        self.get_environment_mut().add_name(name, size, variable_type)
    }

    fn get_variable(&self, name: &str) -> Result<(&Variable, isize), String> {
        let mut found_env_index = 0;
        let mut variable: Option<&Variable> = None;
        for env in (&self.envs).into_iter().enumerate().rev() {
            let result = env.1.get_variable(name);
            match result {
                Err(err) => {
                    if env.0 == 0 {
                        println!("{:#?}", self);
                        return Err(err);
                    }
                }
                Ok(ok) => {
                    found_env_index = env.0;
                    variable = Some(ok);
                    break;
                }
            }
        }

        if variable.is_none() {
            unreachable!();
        }

        let mut start_offset = 0usize;
        for (index, env) in (&self.envs).iter().enumerate() {
            if index != found_env_index {
                start_offset += env.stack_pointer;
            } else {
                start_offset += variable.unwrap().stack_pos;
                break;
            }
        }

	let mut base_offset  = 0usize;
        for (index, env) in (&self.envs).iter().enumerate() {
	    if self.envs.len() - 1 == index {
		break;
	    }
            base_offset += env.stack_pointer;
        }

	Ok((variable.unwrap(), start_offset as isize - base_offset as isize))
        
    }

    fn get_function(&self, name: &str) -> Result<&Function, String> {
        match self.functions.get(name) {
            Some(func) => Ok(func),
            None => Err(format!("No function named {name}")),
        }
    }
}

impl Environment {
    fn add_name(&mut self, name: &str, size: usize, variable_type: &str) -> &EnvironmentName {
        self.names.insert(
            name.to_string(),
            EnvironmentName::Variable(Variable {
                type_name: variable_type.to_string(),
                stack_pos: self.stack_pointer,
            }),
        );	
        
        self.names.get(&name.to_string()).unwrap()
    }

    fn get_name(&self, name: &str) -> Option<&EnvironmentName> {
        self.names.get(&name.to_string())
    }

    fn get_variable(&self, name: &str) -> Result<&Variable, String> {
        if self.get_name(name).is_none() {
            return Err(format!("{name} is not a valid name"));
        } else {
            match self.get_name(name).unwrap() {
                EnvironmentName::Variable(var) => return Ok(&var),
            }
        }
    }
}

struct Generator {
    nodes: Vec<Node>,
    current_node: usize,
}

impl Generator {
    fn next(&mut self) -> Option<&Node> {
        self.current_node += 1;
        return self.nodes.get(self.current_node);
    }

    fn get_argument_register(arg: usize) -> String {
	match arg {
	    0 => "rdi",
	    1 => "rsi",
	    2 => "rdx",
	    3 => "rcx",
	    4 => "r8",
	    5 => "r9",
	    _=> "r9"
	}.to_string()
    }

    fn get_word_from_size(size: usize) -> String {
	String::from(match size {
	    1 => "byte",
	    2 => "word",
	    4 => "dword",
	    8 => "qword",
	    _ => todo!()
	})
    }
    
    fn push(i: isize, size: usize, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        writeln!(writer, "    mov r8, {i}");
        writeln!(writer, "    push r8");
        scope.get_environment_mut().stack_pointer += size;
    }

    fn push_reg(reg: &str, size: usize, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        writeln!(writer, "    push {} {reg}", Self::get_word_from_size(size));
        scope.get_environment_mut().stack_pointer += size;
    }

    fn pop_reg(reg: &str, size: usize, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        writeln!(writer, "    pop {} {reg}", Self::get_word_from_size(size));
        scope.get_environment_mut().stack_pointer -= size;
    }

    fn mov(to: &str, size: usize, from: &str, writer: &mut BufWriter<fs::File>) {
        writeln!(writer, "    mov {to}, {} {from}", Self::get_word_from_size(size));
    }

    fn zero(reg: &str, writer: &mut BufWriter<fs::File>) {
        writeln!(writer, "    xor {reg}, {reg}");
    }

    fn generate_nanpa_expression(nanpa_expression: &NanpaExpression, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        Generator::push(nanpa_expression.value, 8, scope, writer);
    }

    fn generate_nimi_expression(nimi_expression: &NimiExpression, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        let (name, offset) = scope.get_variable(&nimi_expression.value).unwrap();
        writeln!(writer, );
        writeln!(writer, 
            "    ; Getting value of variable {} with offset {}",
            nimi_expression.value, offset
        );
        Self::push_reg(format!("[rbp - {}]", offset).as_str(), scope.get_type(&name.type_name).unwrap().size, scope, writer);
    }

    fn generate_kepeken(kepeken_statement: &KepekenStatement) {
	todo!();
    }
    
    fn generate_nimi_new(
        nimi_expression: &NimiExpression,
        variable_type: &str,
        scope: &mut Scope,
	writer: &mut BufWriter<fs::File>
    ) {
        match scope.get_environment().get_variable(&nimi_expression.value) {
            Err(_) => {
                writeln!(writer, );
                writeln!(writer, 
                    "    ; new {} {}",
                    variable_type, nimi_expression.value,
                );
                
                scope.add_variable(&nimi_expression.value, variable_type, None, writer);
            }
            Ok(_) => {
                panic!(
                    "There's already a variable named {} in this scope",
                    nimi_expression.value
                );
            }
        }
    }

    fn generate_nimi_recieve_stack(
        nimi_expression: &NimiExpression,
        variable_type: Option<&str>,
        scope: &mut Scope,
	writer: &mut BufWriter<fs::File>
    ) {
        match scope.get_variable(&nimi_expression.value) {
            Err(_) => {
                panic!("No variable named {}", nimi_expression.value);
            }
            Ok((name, offset)) => {
                writeln!(writer, );
                writeln!(writer, "    ; Setting variable {}", nimi_expression.value);
		let size = scope.get_type(&name.type_name).unwrap().size;
                Generator::pop_reg("r9", scope.get_type(&name.type_name).unwrap().size, scope, writer);
		
                Generator::mov(format!("[rbp - {offset}]").as_str(), size, "r9", writer);
            }
        };
    }

    fn generate_o_sin(osin: &OSinStatement, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        match &osin.expr {
            Some(expr) => Self::generate_expression(&expr, scope, writer),
            None => {}
        };

        Self::generate_nimi_new(&osin.name, &osin.var_type, scope, writer);
    }

    fn generate_li_kama_sama_statement(kama_sama: &LiKamaSamaStatement, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        writeln!(writer, "");

        Self::generate_expression(&kama_sama.expression, scope, writer);

        Self::generate_nimi_recieve_stack(&kama_sama.nimi, None, scope, writer);
    }

    fn generate_unary_expression(unary: &UnaryExpression, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        match unary {
            UnaryExpression::Nanpa(nanpa) => Self::generate_nanpa_expression(nanpa, scope, writer),
            UnaryExpression::Nimi(nimi) => Self::generate_nimi_expression(nimi, scope, writer),
            UnaryExpression::O(o) => Self::generate_o(o, scope, writer),
            UnaryExpression::Linja(linja) => {}
        }
    }

    fn generate_binary_expression(binary: &BinaryExpression, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        Self::generate_expression(&binary.lhs, scope, writer);
        Self::generate_expression(&binary.rhs, scope, writer);
	let sizel = scope.get_type(&binary.lhs.get_type_name(scope)).unwrap().size;
	let sizer = scope.get_type(&binary.rhs.get_type_name(scope)).unwrap().size;
        Self::pop_reg("r9", sizel, scope, writer);
        Self::pop_reg("r8", sizer, scope, writer);
	let size: usize = scope.get_type(binary.get_type_name(scope).as_str()).unwrap().size;
	
        match binary.kind {
            BinaryExpressionType::Add => {
                writeln!(writer, "    add r8, r9");
                Self::push_reg("r8", size, scope, writer);
            }
            BinaryExpressionType::Subtract => {
                writeln!(writer, "    sub r8, r9");
                Self::push_reg("r8", size, scope, writer);
            }
            BinaryExpressionType::Multiply => {
                Self::mov("rax", size, "r8", writer);
                writeln!(writer, "    mul r9");
                Self::push_reg("rax", size, scope, writer);
            }
            BinaryExpressionType::Divide => {
                Self::zero("rdx", writer);
                Self::mov("rax", size, "r8", writer);
                writeln!(writer, "    div r9");
                Self::push_reg("rax", size, scope, writer);
            }
            BinaryExpressionType::Equals => {
                Self::zero("ecx", writer);
                writeln!(writer, "    cmp r8, r9");
                writeln!(writer, "    setz cl");
                Self::push_reg("rcx", size, scope, writer);
            }
            _ => {}
        }
    }

    fn generate_expression(expression: &Expression, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        match expression {
            Expression::Unary(unary) => Self::generate_unary_expression(unary, scope, writer),
            Expression::Binary(binary) => Self::generate_binary_expression(binary, scope, writer),
        };
    }

    fn generate_otawa(otawa: &OtawaStatement, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        Self::generate_expression(&otawa.expr, scope, writer);
	let size = scope.get_type(&otawa.expr.get_type_name(scope)).unwrap().size;
        writeln!(writer, );
        writeln!(writer, "    ; Exit call:");
        Self::pop_reg("rdi", size, scope, writer);
        Self::mov("rax", size, "60", writer);
        writeln!(writer, "    syscall");
        writeln!(writer, );
    }

    fn generate_parameter(
        param: &(String, NimiExpression),
        scope: &mut Scope,
        offset: usize,
	writer: &mut BufWriter<fs::File>
    ) {
        writeln!(writer, "    ; Setting parameter {} of type {}", param.1.value, param.0);
        scope.add_variable(&param.1.value, param.0.as_str(), Some(&Self::get_argument_register(offset)), writer);
    }

    fn generate_o(o: &OExpression, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        let func = scope.get_function(&o.nimi.value).unwrap();

        let types = func.parameter_types.clone();
        let return_type = func.return_type.clone();

        writeln!(writer, );
        writeln!(writer, "    ; o {}", o.nimi.value);

        if (&o.params)
            .into_iter()
            .map(|v| return v.get_type_name(scope))
            .collect::<Vec<_>>()
            != types
        {
            panic!("caller arguments do not match function parameters");
        }

        for (index, expr) in o.params.iter().enumerate().rev() {
            Self::generate_expression(expr, scope, writer);
	    let size = scope.get_type(&expr.get_type_name(scope)).unwrap().size;
	    Self::pop_reg(&Self::get_argument_register(index), size, scope, writer);
        }

        writeln!(writer, "    call {}", o.nimi.value);

        if return_type.is_some() {
	    let size = scope.get_type(&return_type.unwrap()).unwrap().size;
            Generator::push_reg("rax", size, scope, writer);
        };
    }

    fn generate_o_weka(oweka: &OWekaStatement, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        match &oweka.expr {
            None => {}
            Some(expr) => {
		let size = scope.get_type(&expr.get_type_name(scope)).unwrap().size;
                Self::generate_expression(&expr, scope, writer);
                Self::pop_reg("rax", size, scope, writer);
            }
        }

        writeln!(writer, "    ; returning");
        Self::mov("rsp", 8, "rbp", writer);
        writeln!(writer, "    pop rbp");
        writeln!(writer, "    ret");
    }

    fn generate_tenpo(tenpo: &TenpoStatement, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        scope.label_counter += 1;
        let label_index = scope.label_counter;
        writeln!(writer, "  ; tenpo .. la");
	Self::new_scope(scope, writer);
        Self::generate_expression(&tenpo.expr, scope, writer);
	let size = scope.get_type(&tenpo.expr.get_type_name(scope)).unwrap().size;
        Self::pop_reg("rax", size, scope, writer);
        writeln!(writer, "    cmp rax, 0");
        writeln!(writer, "    je .endif_{label_index}");
	
        for node in &tenpo.nodes {
            if Self::generate_node(node, scope, writer) {
                break;
            }
        }
        writeln!(writer, 
            "    add rsp, {}",
            scope.get_environment_mut().names.len() * 8
        );

	Self::end_scope(scope, writer);
	
        writeln!(writer, "  .endif_{label_index}:").unwrap();
    }

    fn generate_pali_declaration(pali: &PaliDeclaration, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
	scope.declare_function(pali);

	writeln!(writer, "extrn {0}", pali.nimi.value).unwrap();
    }
    
    fn generate_pali(pali: &PaliStatement, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        scope.add_function(pali);

        writeln!(writer, "public {}", pali.nimi.value).unwrap();
        writeln!(writer, "{}:", pali.nimi.value).unwrap();
        Self::new_scope(scope, writer);
        Self::push_reg("rbp", 8, scope, writer);
        writeln!(writer, "    mov rbp, rsp").unwrap();

        if !pali.params.is_empty() {
            let mut offset = 0;
            for param in &pali.params {
                Self::generate_parameter(param, scope, offset, writer);
                offset += 1;
            }
        }

        for node in &pali.nodes {
            if Self::generate_node(node, scope, writer) {
                break;
            };
        }
	
	Self::end_scope(scope, writer);
    }

    fn new_scope(scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        writeln!(writer, "  ; new scope");
        scope.envs.push(Environment {
            names: HashMap::new(),
            stack_pointer: 0,
            tab_depth: 0,
        });
    }

    fn end_scope(scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        scope.envs.pop();
        writeln!(writer, "  ; end of scope");
	writeln!(writer, );
    }

    fn generate_parenthesis(paren: &Parenthesis, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        writeln!(writer, );
        Self::new_scope(scope, writer);

        for node in &paren.nodes {
            Self::generate_node(node, scope, writer);
        }

        writeln!(writer, 
            "    add rsp, {}",
            scope.get_environment_mut().names.len() * 8
        );
	Self::end_scope(scope, writer);
    }

    fn generate_node(node: &Node, scope: &mut Scope, writer: &mut BufWriter<fs::File>) -> bool {
        match node {
            Node::Otawa(otawa) => {
                Self::generate_otawa(otawa, scope, writer);
                return true;
            }
            Node::Tenpo(tenpo) => Self::generate_tenpo(tenpo, scope, writer),
            Node::Expression(expression) => Self::generate_expression(expression, scope, writer),
            Node::OSin(osin) => Self::generate_o_sin(osin, scope, writer),
            Node::LiKamaSama(kamasama) => Self::generate_li_kama_sama_statement(kamasama, scope, writer),
            Node::Pali(pali) => Self::generate_pali(pali, scope, writer),
	    Node::PaliDeclaration(pali) => Self::generate_pali_declaration(pali, scope, writer),
            Node::OWeka(oweka) => {
                Self::generate_o_weka(oweka, scope, writer);
                return true;
            }
            Node::O(o) => {
                Self::generate_o(o, scope, writer);
		let func = scope.get_function(&o.nimi.value).unwrap();
		if func.return_type.is_some()  {
		    let size = scope.get_type(&func.return_type.clone().unwrap()).unwrap().size;
		    Self::pop_reg("rax", size, scope, writer);
		}
            }
            Node::Parenthesis(paren) => Self::generate_parenthesis(paren, scope, writer),
            _ => {}
        };
        false
    }

    fn generate_prelude(&mut self, writer: &mut BufWriter<fs::File>) {
	writeln!(writer, "format ELF64");
    }
    
    fn generate(&mut self, scope: &mut Scope, writer: &mut BufWriter<fs::File>) {
        self.generate_prelude(writer);
        for node in &self.nodes {
            if Self::generate_node(node, scope, writer) {
                break;
            };
        }
    }
    
}

#[derive(Eq, PartialEq)]
enum RunMode {
    Object,
    Linked
}

fn main() {
    let debug_mode = false;

    let args: Vec<String> = env::args().collect();

    let mode = match args.get(1).unwrap().as_str() {
	"o" => RunMode::Object,
	"l" => RunMode::Linked,
	_ => panic!("not a valid mode"),
    };


    let input_file = args.get(2).unwrap();
    
    let input = fs::read_to_string(input_file).expect("no input file");

    let output_file = args.get(3).unwrap();

    let mut output = fs::File::create((*output_file).clone()+".asm").unwrap();
    
    let mut lexer = Lexer {
        current_position: 0,
        buffer: input,
        debug_mode,
    };

    let mut abstracter = Abstracter {
        words: lexer.lex(),
        current_word: 0,
        tokens: Vec::new(),
        debug_mode,
    };

    abstracter.tokenize();

    let mut parser = Parser {
        current_token: 0,
        tokens: abstracter.tokens,
        nodes: Vec::new(),
        debug_mode,
    };

    parser.parse();

    if debug_mode {
        for node in &parser.nodes {
            println!("{:#?}", node);
        }
    }

    let mut generator = Generator {
        nodes: parser.nodes,
        current_node: 0,

    };

    
    let mut scope = Scope {
        envs: Vec::new(),
        functions: HashMap::new(),
        label_counter: 0,
	types: HashMap::new()
    };
    scope.types.insert("nanpa".to_string(),
	    Rc::new(Type{
		name: "nanpa".to_string(),
		size: 8
	    }));


    scope.envs.push(Environment {
        names: HashMap::new(),
        stack_pointer: 0,
        tab_depth: 0,
    });

    generator.generate(&mut scope, &mut BufWriter::new(output));
    
    Command::new("fasm").arg((*output_file).clone()+".asm").output().unwrap();

    if mode == RunMode::Linked {
	Command::new("ld").args(&[
	    (*output_file).clone()+".o",
	    "lib/asen_asm.o".to_string(),
	    "lib/pu.o".to_string()
	]).output().unwrap();
	
	Command::new("mov").args(&[
	    "a.out".to_string(),
	    output_file.to_string()
	]);
    }
}
