//! Celestia CEL data-file grammar: tokenizer + `AssociativeArray`.
//!
//! Faithful Rust port of Celestia's `celutil` data layer
//! (`tokenizer.h`/`parser.h`, `associativearray.h`). Celestia's catalog
//! files (`.ssc`, `.stc`, `.dsc`, `.cel`) are written in a small
//! expression language:
//!
//! ```text
//! # comment to end of line
//! Body "Earth" "Sun"
//! {
//!     Eccentricity  0.0167
//!     SemiMajorAxis 1.0 : AU
//!     Period        365.25 : days
//!     Color "0.18 0.46 0.88"
//!     Class Planet
//! }
//! ```
//!
//! A top-level file is a sequence of property assignments and groups. A
//! `Group` is a `{ ... }` block, an `Array` is a `[ ... ]` block, and an
//! inline unit is written `value : unit`.
#![allow(missing_docs)]

use std::collections::BTreeMap;

// --- Tokens (celutil/tokenizer.h: TokenType) ---------------------------------

/// A single token produced by the [`Tokenizer`].
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Name(String),
    String(String),
    Number(f64),
    BeginGroup,
    EndGroup,
    BeginArray,
    EndArray,
    Equals,
    Bar,
    Units,
    End,
    Error,
}

/// A lexical analyzer for the CEL language, mirroring `celutil::Tokenizer`.
///
/// Tokens are produced lazily from an owned byte buffer; a single-token
/// pushback buffer is provided via [`Tokenizer::pushback`].
pub struct Tokenizer {
    input: Vec<u8>,
    pos: usize,
    line: u64,
    pushback: Option<Token>,
}

impl Tokenizer {
    fn new(input: &str) -> Self {
        Self {
            input: input.as_bytes().to_vec(),
            pos: 0,
            line: 1,
            pushback: None,
        }
    }

    /// Push the last token back so the next call to [`next`](Self::next)
    /// returns it again.
    fn pushback(&mut self, token: Token) {
        if !matches!(token, Token::End | Token::Error) {
            self.pushback = Some(token);
        }
    }

    fn current_line(&self) -> u64 {
        self.line
    }

    fn next(&mut self) -> Token {
        if let Some(tok) = self.pushback.take() {
            return tok;
        }

        // Skip whitespace and comments.
        loop {
            while self.pos < self.input.len() {
                match self.input[self.pos] {
                    b' ' | b'\t' | b'\r' => {
                        self.pos += 1;
                    }
                    b'\n' => {
                        self.line += 1;
                        self.pos += 1;
                    }
                    _ => break,
                }
            }
            if self.pos >= self.input.len() {
                return Token::End;
            }
            let c = self.input[self.pos];
            match c {
                b'#' => {
                    // Line comment.
                    while self.pos < self.input.len() && self.input[self.pos] != b'\n' {
                        self.pos += 1;
                    }
                    continue;
                }
                b'{' => {
                    self.pos += 1;
                    return Token::BeginGroup;
                }
                b'}' => {
                    self.pos += 1;
                    return Token::EndGroup;
                }
                b'[' => {
                    self.pos += 1;
                    return Token::BeginArray;
                }
                b']' => {
                    self.pos += 1;
                    return Token::EndArray;
                }
                b'=' => {
                    self.pos += 1;
                    return Token::Equals;
                }
                b'|' => {
                    self.pos += 1;
                    return Token::Bar;
                }
                b':' => {
                    self.pos += 1;
                    return Token::Units;
                }
                b'"' => {
                    self.pos += 1;
                    return self.read_string();
                }
                _ => {
                    // A name or number begins here.
                    return self.read_name_or_number();
                }
            }
        }
    }

    fn read_string(&mut self) -> Token {
        let mut s = String::new();
        while self.pos < self.input.len() {
            let c = self.input[self.pos];
            self.pos += 1;
            if c == b'"' {
                return Token::String(s);
            }
            if c == b'\\' && self.pos < self.input.len() {
                let esc = self.input[self.pos];
                self.pos += 1;
                match esc {
                    b'n' => s.push('\n'),
                    b't' => s.push('\t'),
                    b'r' => s.push('\r'),
                    b'\\' => s.push('\\'),
                    b'"' => s.push('"'),
                    other => {
                        s.push('\\');
                        s.push(other as char);
                    }
                }
            } else {
                s.push(c as char);
            }
        }
        Token::String(s)
    }

    fn read_name_or_number(&mut self) -> Token {
        let start = self.pos;
        // A token is a run of name characters; numbers may contain digits,
        // '.', and (for exponents) 'e'/'E', plus a leading sign.
        while self.pos < self.input.len() {
            let c = self.input[self.pos];
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' || c == b'+' || c == b'-' {
                // A sign only continues a number at the start of the token.
                if (c == b'-' || c == b'+') && self.pos != start {
                    break;
                }
                self.pos += 1;
            } else {
                break;
            }
        }
        let slice = &self.input[start..self.pos];
        let text = String::from_utf8_lossy(slice);
        if text.is_empty() {
            return Token::End;
        }
        if let Ok(n) = text.parse::<f64>() {
            Token::Number(n)
        } else {
            Token::Name(text.into_owned())
        }
    }
}

// --- Values (celutil/associativearray.h: Value) ------------------------------

/// A strongly-typed value in a CEL `AssociativeArray`, mirroring
/// `celutil::Value`.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A floating-point number, optionally carrying a unit string.
    Number {
        value: f64,
        unit: Option<String>,
    },
    String(String),
    Boolean(bool),
    /// An RGB(A) color, each channel in `[0, 1]`.
    Color(Color),
    /// A 3-vector (e.g. a direction or position).
    Vector3([f64; 3]),
    /// A nested group.
    Group(AssociativeArray),
    /// An ordered list of values.
    Array(Vec<Value>),
}

/// An RGB(A) color, mirroring `celutil::Color`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Value {
    /// The numeric value of a [`Value::Number`], if present.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number { value, .. } => Some(*value),
            Value::Boolean(b) => Some(*b as i32 as f64),
            _ => None,
        }
    }

    /// The string value of a [`Value::String`], if present.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// The unit string on a [`Value::Number`], if any.
    pub fn as_unit(&self) -> Option<&str> {
        match self {
            Value::Number { unit, .. } => unit.as_deref(),
            _ => None,
        }
    }

    /// The nested group, if present.
    pub fn as_group(&self) -> Option<&AssociativeArray> {
        match self {
            Value::Group(g) => Some(g),
            _ => None,
        }
    }

    /// The array, if present.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Interpret this value as an RGB(A) color.
    ///
    /// Mirrors `celutil::AssociativeArray::getColor`: a 3- or 4-element array
    /// (or an already-stored [`Color`]) is read as a color. This is how
    /// Celestia disambiguates a numeric tuple between a color and a vector —
    /// the decision is made by the accessor, not the parser.
    pub fn as_color(&self) -> Option<Color> {
        match self {
            Value::Color(c) => Some(*c),
            Value::Vector3(v) => Some(Color {
                r: v[0] as f32,
                g: v[1] as f32,
                b: v[2] as f32,
                a: 1.0,
            }),
            Value::Array(arr) => {
                let nums: Vec<f64> =
                    arr.iter().filter_map(|v| v.as_number()).collect();
                match nums.len() {
                    3 => Some(Color {
                        r: nums[0] as f32,
                        g: nums[1] as f32,
                        b: nums[2] as f32,
                        a: 1.0,
                    }),
                    4 => Some(Color {
                        r: nums[0] as f32,
                        g: nums[1] as f32,
                        b: nums[2] as f32,
                        a: nums[3] as f32,
                    }),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Interpret this value as a 3-vector.
    ///
    /// Mirrors `celutil::AssociativeArray::getVector3`, which accepts a
    /// 3-element array (as well as an already-stored [`Value::Vector3`]).
    pub fn as_vector3(&self) -> Option<[f64; 3]> {
        match self {
            Value::Vector3(v) => Some(*v),
            Value::Array(arr) => {
                let nums: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();
                if nums.len() == 3 {
                    Some([nums[0], nums[1], nums[2]])
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// An ordered, keyed map of CEL values, mirroring `celutil::AssociativeArray`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AssociativeArray {
    entries: BTreeMap<String, Value>,
}

impl AssociativeArray {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: &str, value: Value) {
        self.entries.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.entries.get(key)
    }

    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.entries.get(key).and_then(|v| v.as_number())
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.entries.get(key).and_then(|v| v.as_str())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.entries.get(key) {
            Some(Value::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn get_group(&self, key: &str) -> Option<&AssociativeArray> {
        self.entries.get(key).and_then(|v| v.as_group())
    }

    pub fn get_array(&self, key: &str) -> Option<&Vec<Value>> {
        self.entries.get(key).and_then(|v| v.as_array())
    }

    /// Number with an inline unit, mirroring `getNumber` + `asUnit`.
    pub fn get_number_with_unit(&self, key: &str) -> Option<(f64, Option<String>)> {
        self.entries.get(key).map(|v| {
            let value = v.as_number().unwrap_or(0.0);
            let unit = v.as_unit().map(|s| s.to_string());
            (value, unit)
        })
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(|s| s.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// --- Parser ------------------------------------------------------------------

struct Parser<'a> {
    tok: &'a mut Tokenizer,
}

impl<'a> Parser<'a> {
    fn new(tok: &'a mut Tokenizer) -> Self {
        Self { tok }
    }

    /// Parse a top-level CEL document into a list of top-level values.
    fn parse(&mut self) -> Result<Vec<Value>, String> {
        let mut items = Vec::new();
        loop {
            let tok = self.tok.next();
            match tok {
                Token::End => break,
                Token::Error => return Err("Tokenizer error".into()),
                Token::Name(name) => {
                    items.push(self.parse_value_after_name(&name)?);
                }
                Token::BeginGroup => {
                    let group = self.parse_group()?;
                    items.push(Value::Group(group));
                }
                Token::BeginArray => {
                    self.tok.pushback(Token::BeginArray);
                    items.push(Value::Array(self.parse_array()?));
                }
                Token::Number(n) => items.push(Value::Number {
                    value: n,
                    unit: None,
                }),
                Token::String(s) => items.push(coerce_string(&s)),
                _ => return Err(format!("Unexpected token at top level: {tok:?}")),
            }
        }
        Ok(items)
    }

    /// Read a `{ ... }` group starting *at* the `BeginGroup` token, which must
    /// already have been consumed. Mirrors `celutil::Parser::readHash`.
    pub fn read_group(&mut self) -> Result<AssociativeArray, String> {
        self.parse_group()
    }

    fn parse_group(&mut self) -> Result<AssociativeArray, String> {
        let mut arr = AssociativeArray::new();
        loop {
            let tok = self.tok.next();
            match tok {
                Token::EndGroup | Token::End => break,
                Token::Name(name) => {
                    let value = self.parse_value_after_name(&name)?;
                    arr.insert(&name, value);
                }
                _ => return Err(format!("Unexpected token in group: {tok:?}")),
            }
        }
        Ok(arr)
    }

    /// Parse a value that immediately follows a name token.
    fn parse_value_after_name(&mut self, _name: &str) -> Result<Value, String> {
        let tok = self.tok.next();
        let value = match tok {
            Token::BeginGroup => Value::Group(self.parse_group()?),
            Token::BeginArray => Value::Array(self.parse_array()?),
            Token::Number(n) => {
                let mut v = Value::Number {
                    value: n,
                    unit: None,
                };
                // Optional inline unit: `1.0 : AU`.
                let peek = self.tok.next();
                if matches!(peek, Token::Units) {
                    let unit = self.read_unit_name()?;
                    v = Value::Number {
                        value: n,
                        unit: Some(unit),
                    };
                } else {
                    self.tok.pushback(peek);
                }
                v
            }
            Token::String(s) => coerce_string(&s),
            Token::Name(n2) => Value::String(n2),
            Token::Units => {
                let unit = self.read_unit_name()?;
                Value::Number {
                    value: 0.0,
                    unit: Some(unit),
                }
            }
            Token::End => return Err("Unexpected end of input in group".into()),
            _ => return Err(format!("Unexpected token after name: {tok:?}")),
        };
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        let tok = self.tok.next();
        match tok {
            Token::BeginGroup => Ok(Value::Group(self.parse_group()?)),
            Token::BeginArray => Ok(Value::Array(self.parse_array()?)),
            Token::Number(n) => Ok(Value::Number {
                value: n,
                unit: None,
            }),
            Token::String(s) => Ok(coerce_string(&s)),
            Token::Name(n) => Ok(Value::String(n)),
            Token::End => Err("Unexpected end of input in value".into()),
            _ => Err(format!("Unexpected token in value: {tok:?}")),
        }
    }

    fn parse_array(&mut self) -> Result<Vec<Value>, String> {
        let mut items = Vec::new();
        loop {
            let tok = self.tok.next();
            match tok {
                Token::EndArray | Token::End => break,
                _ => {
                    self.tok.pushback(tok);
                    items.push(self.parse_value()?);
                }
            }
        }
        Ok(items)
    }

    fn read_unit_name(&mut self) -> Result<String, String> {
        match self.tok.next() {
            Token::Name(n) => Ok(n),
            Token::String(s) => Ok(s),
            _ => Err("Expected unit name after ':'".into()),
        }
    }
}

/// Coerce a string to a [`Value`]: a 3/4-tuple of numbers becomes a
/// [`Value::Color`], a 3-tuple otherwise becomes a [`Value::Vector3`],
/// otherwise a plain string.
fn coerce_string(s: &str) -> Value {
    let trimmed = s.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if (3..=4).contains(&parts.len()) {
        if let Ok(nums) = parts.iter().map(|p| p.parse::<f64>()).collect::<Result<Vec<_>, _>>() {
            if parts.len() == 4 {
                return Value::Color(Color {
                    r: nums[0] as f32,
                    g: nums[1] as f32,
                    b: nums[2] as f32,
                    a: nums[3] as f32,
                });
            }
            return Value::Vector3([nums[0], nums[1], nums[2]]);
        }
    }
    Value::String(trimmed.to_string())
}

/// Parse a `{ ... }` hash starting *at* the already-consumed `BeginGroup`
/// token, mirroring `celutil::Parser::readHash`. The parser borrows the
/// tokenizer for the duration of the call only.
fn parse_hash(tz: &mut Tokenizer) -> Result<AssociativeArray, String> {
    let mut parser = Parser::new(tz);
    parser.read_group()
}

/// Parse a top-level CEL document into a list of top-level values.
pub fn parse(input: &str) -> Result<Vec<Value>, String> {
    let mut tokenizer = Tokenizer::new(input);
    let mut parser = Parser::new(&mut tokenizer);
    parser.parse()
}

/// Parse a CEL group (`{ ... }`) into an [`AssociativeArray`].
///
/// A top-level file is a sequence of groups; their entries are concatenated
/// into a single array, matching how Celestia reads a `.ssc` body table.
pub fn parse_group(input: &str) -> Result<AssociativeArray, String> {
    let items = parse(input)?;
    let mut arr = AssociativeArray::new();
    for item in items {
        match item {
            Value::Group(g) => {
                for k in g.keys() {
                    if let Some(v) = g.get(k) {
                        arr.insert(k, v.clone());
                    }
                }
            }
            Value::String(s) => {
                arr.insert(&s.clone(), Value::String(s));
            }
            Value::Number { value, unit } => {
                arr.insert("__number", Value::Number { value, unit });
            }
            other => arr.insert("__value", other),
        }
    }
    Ok(arr)
}

/// A single parsed `.ssc` entry: the primary name, the parent name, and the
/// body's property group.
#[derive(Debug, Clone)]
pub struct SscEntry {
    /// The primary name (the first of a `:`-delimited name list).
    pub name: String,
    /// The parent body's name (empty for the central star).
    pub parent: String,
    /// The body's property group (`{ ... }`).
    pub data: AssociativeArray,
}

/// Parse a full `.ssc` catalog into a list of [`SscEntry`]s, mirroring the
/// outer loop of `celengine::SolarSystemsBuilder::parseSsc`.
///
/// Each entry follows the grammar:
///
/// ```text
/// [ Add | Replace | Modify ] [ Body | Location | AltSurface ]
///     "nameList" "parentName" { <properties> }
/// ```
///
/// The walk consumes tokens directly (like Celestia, which reads the item type
/// and the name/parent as raw tokens *before* the property hash), because the
/// generic CEL parser would otherwise merge `Body "Mercury"` into a single
/// value and discard the `Body` keyword.
///
/// `Location` and `AltSurface` entries are currently skipped. The name list may
/// hold several `:`-delimited names; the first is the primary name.
pub fn parse_ssc_entries(input: &str) -> Result<Vec<SscEntry>, String> {
    let mut tokenizer = Tokenizer::new(input);
    let mut entries = Vec::new();

    loop {
        // The next token begins a new entry, or ends the file.
        let cur = match tokenizer.next() {
            Token::End => break,
            other => other,
        };

        // Optional disposition keyword (Add / Replace / Modify).
        let cur = if let Token::Name(n) = &cur {
            if matches!(n.as_str(), "Add" | "Replace" | "Modify") {
                tokenizer.next()
            } else {
                cur
            }
        } else {
            cur
        };

        // Optional item type; defaults to "Body".
        let mut item_type = "Body".to_string();
        let cur = if let Token::Name(n) = &cur {
            item_type = n.clone();
            tokenizer.next()
        } else {
            cur
        };
        // Location / AltSurface are not modelled yet; skip their entry.
        if item_type != "Body" {
            continue;
        }

        // The name list: "a:b:c".
        let (name_list, cur) = match &cur {
            Token::String(s) => (s.clone(), tokenizer.next()),
            _ => return Err("object name expected".into()),
        };
        let primary = name_list.split(':').next().unwrap_or("").to_string();
        if primary.is_empty() {
            continue;
        }

        // The parent name.
        let (parent, cur) = match &cur {
            Token::String(s) => (s.clone(), tokenizer.next()),
            _ => return Err(format!("bad parent object name for '{primary}'")),
        };

        // The property group `{ ... }`. The `BeginGroup` token has already
        // been consumed, so the hash is parsed from the current position.
        let data = match &cur {
            Token::BeginGroup => parse_hash(&mut tokenizer)?,
            // No group; treat the body as fixed at its parent.
            _ => AssociativeArray::new(),
        };

        entries.push(SscEntry {
            name: primary,
            parent,
            data,
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_group() {
        let input = r#"
            {
                Eccentricity 0.0167
                SemiMajorAxis 1.0 : AU
                Period 365.25 : days
                Class Planet
            }
        "#;
        let arr = parse_group(input).unwrap();
        assert!((arr.get_number("Eccentricity").unwrap() - 0.0167).abs() < 1e-9);
        let (a, unit) = arr.get_number_with_unit("SemiMajorAxis").unwrap();
        assert!((a - 1.0).abs() < 1e-9);
        assert_eq!(unit.as_deref(), Some("AU"));
        assert_eq!(arr.get_string("Class"), Some("Planet"));
    }

    #[test]
    fn parse_color_and_vector() {
        // Celestia has no `Color` value type: a color is stored the same way as
        // any other numeric tuple and only interpreted as a color by the
        // `as_color` accessor. A `Color` field written as an inline string
        // "r g b" is stored as a 3-element array and read back via `as_color`.
        let input = r#"{ Color "0.18 0.46 0.88" }"#;
        let arr = parse_group(input).unwrap();
        let c = arr.get("Color").and_then(|v| v.as_color()).expect("color");
        assert!((c.r - 0.18).abs() < 1e-6);
        assert!((c.g - 0.46).abs() < 1e-6);
        assert!((c.b - 0.88).abs() < 1e-6);
    }

    #[test]
    fn parse_vector3() {
        // A vector field written as an inline string "x y z" is stored as a
        // 3-element array and read back via `as_vector3`.
        let input = r#"{ SpinAxis "0.0 0.0 1.0" }"#;
        let arr = parse_group(input).unwrap();
        let v = arr.get("SpinAxis").and_then(|v| v.as_vector3()).expect("vector3");
        assert!((v[0] - 0.0).abs() < 1e-9);
        assert!((v[1] - 0.0).abs() < 1e-9);
        assert!((v[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn comments_ignored() {
        let input = r#"
            # a comment
            { Value 42 }
        "#;
        let arr = parse_group(input).unwrap();
        assert!((arr.get_number("Value").unwrap() - 42.0).abs() < 1e-9);
    }

    #[test]
    fn parse_nested_group() {
        let input = r#"{ Orbit { SemiMajorAxis 1.0 : AU Eccentricity 0.1 } }"#;
        let arr = parse_group(input).unwrap();
        let orbit = arr.get_group("Orbit").expect("orbit group");
        assert!((orbit.get_number("SemiMajorAxis").unwrap() - 1.0).abs() < 1e-9);
        assert!((orbit.get_number("Eccentricity").unwrap() - 0.1).abs() < 1e-9);
    }

    #[test]
    fn parse_array() {
        let input = r#"{ List [ 1.0 2.0 3.0 ] }"#;
        let arr = parse_group(input).unwrap();
        let list = arr.get_array("List").expect("array");
        assert_eq!(list.len(), 3);
        assert!((list[0].as_number().unwrap() - 1.0).abs() < 1e-9);
    }
}
