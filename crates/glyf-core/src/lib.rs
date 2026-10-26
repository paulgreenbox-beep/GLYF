use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct GlyfDocument {
    pub version: u32,
    pub images: Vec<GlyfImage>,
    pub groups: Vec<GlyfGroup>,
    pub annotations: Vec<GlyfAnnotation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyfImage {
    pub name: String,
    pub source: String,
    pub comment: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlyfGroup {
    pub name: String,
    pub parts: Vec<GlyfPart>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlyfPart {
    pub name: String,
    pub source: String,
    pub clip: Option<GlyfClip>,
    pub rel1: GlyfPoint,
    pub rel2: GlyfPoint,
    pub comment: String,
    pub parent: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyfClip {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyfPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlyfAnnotation {
    pub name: String,
    pub role: String,
    pub geometry: GlyfAnnotationGeometry,
    pub comment: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GlyfAnnotationGeometry {
    Point { rel: GlyfPoint },
    Region { rel1: GlyfPoint, rel2: GlyfPoint },
}

pub const DEFAULT_ANNOTATION_ROLE: &str = "canvas_annotation";

impl GlyfAnnotation {
    pub fn point(name: impl Into<String>, rel: GlyfPoint, comment: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            role: DEFAULT_ANNOTATION_ROLE.into(),
            geometry: GlyfAnnotationGeometry::Point { rel },
            comment: comment.into(),
        }
    }

    pub fn region(
        name: impl Into<String>,
        rel1: GlyfPoint,
        rel2: GlyfPoint,
        comment: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            role: DEFAULT_ANNOTATION_ROLE.into(),
            geometry: GlyfAnnotationGeometry::Region { rel1, rel2 },
            comment: comment.into(),
        }
    }
}

#[derive(Debug)]
pub enum GlyfError {
    Io(std::io::Error),
    InvalidDocument(String),
    Parse { position: usize, message: String },
}

impl fmt::Display for GlyfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::InvalidDocument(message) => write!(f, "invalid GLYF document: {message}"),
            Self::Parse { position, message } => {
                write!(f, "parse error at character {position}: {message}")
            }
        }
    }
}

impl std::error::Error for GlyfError {}

impl From<std::io::Error> for GlyfError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn parse(input: &str) -> Result<GlyfDocument, GlyfError> {
    let mut parser = Parser::new(input);
    parser.expect_keyword("glyf")?;
    parser.expect('{')?;

    let mut version = None;
    let mut images = Vec::new();
    let mut groups = Vec::new();
    let mut annotations = Vec::new();
    while !parser.consume('}') {
        let field = parser.identifier()?;
        match field.as_str() {
            "version" => {
                if version.is_some() {
                    return parser.error("duplicate version field");
                }
                parser.expect(':')?;
                version = Some(parser.number()?);
                parser.expect(';')?;
            }
            "image" => images.push(parser.image()?),
            "group" => groups.push(parser.group()?),
            "annotation" => annotations.push(parser.annotation()?),
            _ => return parser.error(format!("unknown field '{field}'")),
        }
    }
    parser.skip_space();
    if parser.position < parser.chars.len() {
        return parser.error("unexpected content after the document");
    }

    let document = GlyfDocument {
        version: version.ok_or_else(|| GlyfError::InvalidDocument("missing version".into()))?,
        images,
        groups,
        annotations,
    };
    validate(&document)?;
    Ok(document)
}

pub fn parse_file(path: impl AsRef<std::path::Path>) -> Result<GlyfDocument, GlyfError> {
    let text = std::fs::read_to_string(path)?;
    parse(&text)
}

pub fn serialize(document: &GlyfDocument) -> Result<String, GlyfError> {
    validate(document)?;
    let mut output = String::from("glyf {\n");
    output.push_str(&format!("    version: {};\n", document.version));
    for image in &document.images {
        output.push_str("\n    image {\n");
        output.push_str(&format!("        name: {};\n", quote(&image.name)));
        output.push_str(&format!("        source: {};\n", quote(&image.source)));
        output.push_str(&format!("        comment: {};\n", quote(&image.comment)));
        output.push_str("    }\n");
    }
    for annotation in &document.annotations {
        output.push_str("\n    annotation {\n");
        output.push_str(&format!("        name: {};\n", quote(&annotation.name)));
        output.push_str(&format!("        role: {};\n", quote(&annotation.role)));
        match annotation.geometry {
            GlyfAnnotationGeometry::Point { rel } => output.push_str(&format!(
                "        rel: {} {};\n",
                display_float(rel.x),
                display_float(rel.y)
            )),
            GlyfAnnotationGeometry::Region { rel1, rel2 } => {
                output.push_str(&format!(
                    "        rel1: {} {};\n",
                    display_float(rel1.x),
                    display_float(rel1.y)
                ));
                output.push_str(&format!(
                    "        rel2: {} {};\n",
                    display_float(rel2.x),
                    display_float(rel2.y)
                ));
            }
        }
        if !annotation.comment.is_empty() {
            output.push_str(&format!(
                "        comment: {};\n",
                quote(&annotation.comment)
            ));
        }
        output.push_str("    }\n");
    }
    for group in &document.groups {
        output.push_str("\n    group {\n");
        output.push_str(&format!("        name: {};\n", quote(&group.name)));
        for part in &group.parts {
            output.push_str("\n        part {\n");
            output.push_str(&format!("            name: {};\n", quote(&part.name)));
            output.push_str(&format!("            source: {};\n", quote(&part.source)));
            if let Some(clip) = part.clip {
                output.push_str(&format!(
                    "            clip: {} {} {} {};\n",
                    display_float(clip.x),
                    display_float(clip.y),
                    display_float(clip.w),
                    display_float(clip.h)
                ));
            }
            output.push_str(&format!(
                "            rel1: {} {};\n",
                display_float(part.rel1.x),
                display_float(part.rel1.y)
            ));
            output.push_str(&format!(
                "            rel2: {} {};\n",
                display_float(part.rel2.x),
                display_float(part.rel2.y)
            ));
            if let Some(parent) = &part.parent {
                output.push_str(&format!("            parent: {};\n", quote(parent)));
            }
            if !part.comment.is_empty() {
                output.push_str(&format!("            comment: {};\n", quote(&part.comment)));
            }
            output.push_str("        }\n");
        }
        output.push_str("    }\n");
    }
    output.push_str("}\n");
    Ok(output)
}

pub fn serialize_file(
    document: &GlyfDocument,
    path: impl AsRef<std::path::Path>,
) -> Result<(), GlyfError> {
    std::fs::write(path, serialize(document)?)?;
    Ok(())
}

pub fn validate(document: &GlyfDocument) -> Result<(), GlyfError> {
    if document.version == 0 {
        return Err(GlyfError::InvalidDocument(
            "version must be greater than zero".into(),
        ));
    }
    let mut names = std::collections::BTreeSet::new();
    for (index, image) in document.images.iter().enumerate() {
        if image.name.trim().is_empty() {
            return Err(GlyfError::InvalidDocument(format!(
                "image {index} has an empty name"
            )));
        }
        if image.source.trim().is_empty() {
            return Err(GlyfError::InvalidDocument(format!(
                "image '{}' has an empty source",
                image.name
            )));
        }
        if !names.insert(image.name.clone()) {
            return Err(GlyfError::InvalidDocument(format!(
                "duplicate image name '{}'",
                image.name
            )));
        }
    }
    let mut annotation_names = std::collections::BTreeSet::new();
    for annotation in &document.annotations {
        if annotation.name.trim().is_empty() {
            return Err(GlyfError::InvalidDocument(
                "annotation name cannot be empty".into(),
            ));
        }
        if annotation.role.trim().is_empty() {
            return Err(GlyfError::InvalidDocument(format!(
                "annotation '{}' role cannot be empty",
                annotation.name
            )));
        }
        if !annotation_names.insert(annotation.name.clone()) {
            return Err(GlyfError::InvalidDocument(format!(
                "duplicate annotation name '{}'",
                annotation.name
            )));
        }
        match annotation.geometry {
            GlyfAnnotationGeometry::Point { rel } => {
                validate_point(rel, &format!("annotation '{}' rel", annotation.name))?;
            }
            GlyfAnnotationGeometry::Region { rel1, rel2 } => {
                validate_point(rel1, &format!("annotation '{}' rel1", annotation.name))?;
                validate_point(rel2, &format!("annotation '{}' rel2", annotation.name))?;
                if rel1.x >= rel2.x || rel1.y >= rel2.y {
                    return Err(GlyfError::InvalidDocument(format!(
                        "annotation '{}' region must have positive size",
                        annotation.name
                    )));
                }
            }
        }
    }
    for group in &document.groups {
        if group.name.trim().is_empty() {
            return Err(GlyfError::InvalidDocument(
                "group name cannot be empty".into(),
            ));
        }
        let mut part_names = std::collections::BTreeSet::new();
        for part in &group.parts {
            if part.name.trim().is_empty() {
                return Err(GlyfError::InvalidDocument(
                    "part name cannot be empty".into(),
                ));
            }
            if !part_names.insert(part.name.clone()) {
                return Err(GlyfError::InvalidDocument(format!(
                    "duplicate part name '{}' in group '{}'",
                    part.name, group.name
                )));
            }
            if !names.contains(&part.source) {
                return Err(GlyfError::InvalidDocument(format!(
                    "part '{}' references missing image '{}'",
                    part.name, part.source
                )));
            }
            validate_point(part.rel1, &format!("part '{}' rel1", part.name))?;
            validate_point(part.rel2, &format!("part '{}' rel2", part.name))?;
            if part.rel1.x > part.rel2.x || part.rel1.y > part.rel2.y {
                return Err(GlyfError::InvalidDocument(format!(
                    "part '{}' rel1 must not be greater than rel2",
                    part.name
                )));
            }
            if let Some(clip) = part.clip {
                if !clip.x.is_finite()
                    || !clip.y.is_finite()
                    || !clip.w.is_finite()
                    || !clip.h.is_finite()
                    || clip.x < 0.0
                    || clip.y < 0.0
                    || clip.w <= 0.0
                    || clip.h <= 0.0
                {
                    return Err(GlyfError::InvalidDocument(format!(
                        "part '{}' has an invalid clip rectangle",
                        part.name
                    )));
                }
            }
        }
        for part in &group.parts {
            if let Some(parent) = &part.parent {
                if parent == &part.name {
                    return Err(GlyfError::InvalidDocument(format!(
                        "part '{}' cannot parent itself",
                        part.name
                    )));
                }
                if !part_names.contains(parent) {
                    return Err(GlyfError::InvalidDocument(format!(
                        "part '{}' references missing parent '{}'",
                        part.name, parent
                    )));
                }
                let mut cursor = Some(parent.as_str());
                let mut seen = std::collections::BTreeSet::new();
                while let Some(name) = cursor {
                    if !seen.insert(name) {
                        return Err(GlyfError::InvalidDocument(format!(
                            "parent cycle involving part '{}'",
                            part.name
                        )));
                    }
                    cursor = group
                        .parts
                        .iter()
                        .find(|candidate| candidate.name == name)
                        .and_then(|candidate| candidate.parent.as_deref());
                }
            }
        }
    }
    Ok(())
}

fn validate_point(point: GlyfPoint, label: &str) -> Result<(), GlyfError> {
    if !point.x.is_finite()
        || !point.y.is_finite()
        || !(0.0..=1.0).contains(&point.x)
        || !(0.0..=1.0).contains(&point.y)
    {
        return Err(GlyfError::InvalidDocument(format!(
            "{label} must be within 0..1"
        )));
    }
    Ok(())
}

fn display_float(value: f32) -> String {
    let mut value = format!("{value:.6}");
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.push('0');
    }
    value
}

fn quote(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            other => output.push(other),
        }
    }
    output.push('"');
    output
}

struct Parser {
    chars: Vec<char>,
    position: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            position: 0,
        }
    }

    fn skip_space(&mut self) {
        while self.position < self.chars.len() {
            if self.chars[self.position].is_whitespace() {
                self.position += 1;
                continue;
            }
            if self.chars[self.position] == '/' && self.chars.get(self.position + 1) == Some(&'/') {
                self.position += 2;
                while self.position < self.chars.len() && self.chars[self.position] != '\n' {
                    self.position += 1;
                }
                continue;
            }
            break;
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        self.skip_space();
        if self.chars.get(self.position) == Some(&expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), GlyfError> {
        if self.consume(expected) {
            Ok(())
        } else {
            self.error(format!("expected '{expected}'"))
        }
    }

    fn expect_keyword(&mut self, expected: &str) -> Result<(), GlyfError> {
        let actual = self.identifier()?;
        if actual == expected {
            Ok(())
        } else {
            self.error(format!("expected '{expected}', found '{actual}'"))
        }
    }

    fn identifier(&mut self) -> Result<String, GlyfError> {
        self.skip_space();
        let start = self.position;
        while self.position < self.chars.len()
            && (self.chars[self.position].is_ascii_alphanumeric()
                || self.chars[self.position] == '_'
                || self.chars[self.position] == '-')
        {
            self.position += 1;
        }
        if start == self.position {
            return self.error("expected an identifier");
        }
        Ok(self.chars[start..self.position].iter().collect())
    }

    fn number(&mut self) -> Result<u32, GlyfError> {
        self.skip_space();
        let start = self.position;
        while self.position < self.chars.len() && self.chars[self.position].is_ascii_digit() {
            self.position += 1;
        }
        if start == self.position {
            return self.error("expected a positive integer");
        }
        let value: String = self.chars[start..self.position].iter().collect();
        value.parse::<u32>().map_err(|_| GlyfError::Parse {
            position: start,
            message: "integer is out of range".into(),
        })
    }

    fn float(&mut self) -> Result<f32, GlyfError> {
        self.skip_space();
        let start = self.position;
        while self.position < self.chars.len()
            && (self.chars[self.position].is_ascii_digit()
                || matches!(self.chars[self.position], '-' | '+' | '.' | 'e' | 'E'))
        {
            self.position += 1;
        }
        if start == self.position {
            return self.error("expected a number");
        }
        let value: String = self.chars[start..self.position].iter().collect();
        let value = value.parse::<f32>().map_err(|_| GlyfError::Parse {
            position: start,
            message: "invalid number".into(),
        })?;
        if !value.is_finite() {
            return self.error("number must be finite");
        }
        Ok(value)
    }

    fn string(&mut self) -> Result<String, GlyfError> {
        self.expect('"')?;
        let mut output = String::new();
        while self.position < self.chars.len() {
            let character = self.chars[self.position];
            self.position += 1;
            match character {
                '"' => return Ok(output),
                '\\' => {
                    let escaped =
                        self.chars
                            .get(self.position)
                            .copied()
                            .ok_or_else(|| GlyfError::Parse {
                                position: self.position,
                                message: "unfinished escape sequence".into(),
                            })?;
                    self.position += 1;
                    output.push(match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '"' => '"',
                        '\\' => '\\',
                        other => {
                            return self.error(format!("unsupported escape '\\{other}'"));
                        }
                    });
                }
                other => output.push(other),
            }
        }
        self.error("unterminated string")
    }

    fn annotation(&mut self) -> Result<GlyfAnnotation, GlyfError> {
        self.expect('{')?;
        let mut name = None;
        let mut role = None;
        let mut rel = None;
        let mut rel1 = None;
        let mut rel2 = None;
        let mut comment = None;
        while !self.consume('}') {
            let field = self.identifier()?;
            self.expect(':')?;
            match field.as_str() {
                "name" | "role" | "comment" => {
                    let value = self.string()?;
                    self.expect(';')?;
                    let slot = match field.as_str() {
                        "name" => &mut name,
                        "role" => &mut role,
                        _ => &mut comment,
                    };
                    if slot.is_some() {
                        return self.error(format!("duplicate annotation field '{field}'"));
                    }
                    *slot = Some(value);
                }
                "rel" => {
                    let value = GlyfPoint {
                        x: self.float()?,
                        y: self.float()?,
                    };
                    self.expect(';')?;
                    if rel.replace(value).is_some() {
                        return self.error("duplicate annotation field 'rel'");
                    }
                }
                "rel1" | "rel2" => {
                    let value = GlyfPoint {
                        x: self.float()?,
                        y: self.float()?,
                    };
                    self.expect(';')?;
                    let slot = if field == "rel1" { &mut rel1 } else { &mut rel2 };
                    if slot.replace(value).is_some() {
                        return self.error(format!("duplicate annotation field '{field}'"));
                    }
                }
                _ => return self.error(format!("unknown annotation field '{field}'")),
            }
        }
        let geometry = match (rel, rel1, rel2) {
            (Some(rel), None, None) => GlyfAnnotationGeometry::Point { rel },
            (None, Some(rel1), Some(rel2)) => GlyfAnnotationGeometry::Region { rel1, rel2 },
            (None, None, None) => return self.error("annotation is missing geometry"),
            _ => return self.error("annotation must use either rel or rel1 plus rel2"),
        };
        Ok(GlyfAnnotation {
            name: name.ok_or_else(|| self.parse_error("annotation is missing name"))?,
            role: role.unwrap_or_else(|| DEFAULT_ANNOTATION_ROLE.into()),
            geometry,
            comment: comment.unwrap_or_default(),
        })
    }

    fn image(&mut self) -> Result<GlyfImage, GlyfError> {
        self.expect('{')?;
        let mut name = None;
        let mut source = None;
        let mut comment = None;
        while !self.consume('}') {
            let field = self.identifier()?;
            self.expect(':')?;
            let value = self.string()?;
            self.expect(';')?;
            match field.as_str() {
                "name" if name.is_none() => name = Some(value),
                "source" if source.is_none() => source = Some(value),
                "comment" if comment.is_none() => comment = Some(value),
                "name" | "source" | "comment" => {
                    return self.error(format!("duplicate image field '{field}'"));
                }
                _ => return self.error(format!("unknown image field '{field}'")),
            }
        }
        let image = GlyfImage {
            name: name.ok_or_else(|| self.parse_error("image is missing name"))?,
            source: source.ok_or_else(|| self.parse_error("image is missing source"))?,
            comment: comment.unwrap_or_default(),
        };
        if image.name.trim().is_empty() {
            return self.error("image name cannot be empty");
        }
        Ok(image)
    }

    fn group(&mut self) -> Result<GlyfGroup, GlyfError> {
        self.expect('{')?;
        let mut name = None;
        let mut parts = Vec::new();
        while !self.consume('}') {
            let field = self.identifier()?;
            match field.as_str() {
                "name" => {
                    if name.is_some() {
                        return self.error("duplicate group field 'name'");
                    }
                    self.expect(':')?;
                    name = Some(self.string()?);
                    self.expect(';')?;
                }
                "part" => parts.push(self.part()?),
                _ => return self.error(format!("unknown group field '{field}'")),
            }
        }
        Ok(GlyfGroup {
            name: name.ok_or_else(|| self.parse_error("group is missing name"))?,
            parts,
        })
    }

    fn part(&mut self) -> Result<GlyfPart, GlyfError> {
        self.expect('{')?;
        let mut name = None;
        let mut source = None;
        let mut clip = None;
        let mut rel1 = None;
        let mut rel2 = None;
        let mut comment = None;
        let mut parent = None;
        while !self.consume('}') {
            let field = self.identifier()?;
            self.expect(':')?;
            match field.as_str() {
                "name" | "source" | "comment" | "parent" => {
                    let value = self.string()?;
                    self.expect(';')?;
                    let slot = match field.as_str() {
                        "name" => &mut name,
                        "source" => &mut source,
                        "comment" => &mut comment,
                        _ => &mut parent,
                    };
                    if slot.is_some() {
                        return self.error(format!("duplicate part field '{field}'"));
                    }
                    *slot = Some(value);
                }
                "clip" => {
                    let value = GlyfClip {
                        x: self.float()?,
                        y: self.float()?,
                        w: self.float()?,
                        h: self.float()?,
                    };
                    self.expect(';')?;
                    if clip.replace(value).is_some() {
                        return self.error("duplicate part field 'clip'");
                    }
                }
                "rel1" | "rel2" => {
                    let value = GlyfPoint {
                        x: self.float()?,
                        y: self.float()?,
                    };
                    self.expect(';')?;
                    let slot = if field == "rel1" {
                        &mut rel1
                    } else {
                        &mut rel2
                    };
                    if slot.replace(value).is_some() {
                        return self.error(format!("duplicate part field '{field}'"));
                    }
                }
                _ => return self.error(format!("unknown part field '{field}'")),
            }
        }
        Ok(GlyfPart {
            name: name.ok_or_else(|| self.parse_error("part is missing name"))?,
            source: source.ok_or_else(|| self.parse_error("part is missing source"))?,
            clip,
            rel1: rel1.ok_or_else(|| self.parse_error("part is missing rel1"))?,
            rel2: rel2.ok_or_else(|| self.parse_error("part is missing rel2"))?,
            comment: comment.unwrap_or_default(),
            parent,
        })
    }

    fn parse_error(&self, message: impl Into<String>) -> GlyfError {
        GlyfError::Parse {
            position: self.position,
            message: message.into(),
        }
    }

    fn error<T>(&self, message: impl Into<String>) -> Result<T, GlyfError> {
        Err(self.parse_error(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> GlyfDocument {
        GlyfDocument {
            version: 1,
            images: vec![GlyfImage {
                name: "main_menu".into(),
                source: "main_menu.jpg".into(),
                comment: "Главное меню игры.\nТёмный sci-fi интерфейс.".into(),
            }],
            groups: vec![],
            annotations: vec![],
        }
    }

    #[test]
    fn parser_reads_multiline_utf8_comment() {
        let text = serialize(&sample()).expect("serialize sample");
        let parsed = parse(&text).expect("parse sample");
        assert_eq!(parsed, sample());
    }

    #[test]
    fn serializer_round_trips_escaped_values() {
        let document = GlyfDocument {
            version: 1,
            images: vec![GlyfImage {
                name: "quote\\name".into(),
                source: "a\"b.png".into(),
                comment: "line 1\nline 2\tok".into(),
            }],
            groups: vec![],
            annotations: vec![],
        };
        let encoded = serialize(&document).expect("serialize escaped sample");
        assert_eq!(parse(&encoded).expect("parse escaped sample"), document);
    }

    #[test]
    fn point_annotation_round_trips_and_keeps_comment_first_shape() {
        let document = GlyfDocument {
            version: 1,
            images: vec![],
            groups: vec![],
            annotations: vec![GlyfAnnotation::point("save-button", GlyfPoint { x: 0.82, y: 0.12 }, "Кнопка Save")],
        };
        let encoded = serialize(&document).expect("serialize annotation document");
        assert!(encoded.contains("annotation"));
        assert_eq!(
            parse(&encoded).expect("parse annotation document"),
            document
        );
    }

    #[test]
    fn region_annotation_round_trips_with_role_and_geometry() {
        let document = GlyfDocument {
            version: 1,
            images: vec![],
            groups: vec![],
            annotations: vec![GlyfAnnotation {
                name: "images-list".into(),
                role: "region".into(),
                geometry: GlyfAnnotationGeometry::Region {
                    rel1: GlyfPoint { x: 0.1, y: 0.2 },
                    rel2: GlyfPoint { x: 0.8, y: 0.9 },
                },
                comment: "Images list region".into(),
            }],
        };
        let encoded = serialize(&document).expect("serialize region annotation");
        assert!(encoded.contains("role: \"region\";"));
        assert!(encoded.contains("rel1: 0.1 0.2;"));
        assert!(encoded.contains("rel2: 0.8 0.9;"));
        assert_eq!(parse(&encoded).expect("parse region annotation"), document);
    }

    #[test]
    fn parser_keeps_legacy_point_annotation_without_role() {
        let text = r#"glyf { version: 1; annotation { name: "legacy"; rel: 0.25 0.5; comment: "Legacy point"; } }"#;
        let parsed = parse(text).expect("parse legacy annotation");
        assert_eq!(parsed.annotations.len(), 1);
        assert_eq!(parsed.annotations[0].role, DEFAULT_ANNOTATION_ROLE);
        assert!(matches!(parsed.annotations[0].geometry, GlyfAnnotationGeometry::Point { rel } if rel == (GlyfPoint { x: 0.25, y: 0.5 })));
    }

    #[test]
    fn parser_rejects_mixed_or_missing_annotation_geometry() {
        for text in [
            r#"glyf { version: 1; annotation { name: "bad"; comment: "missing"; } }"#,
            r#"glyf { version: 1; annotation { name: "bad"; rel: 0.1 0.1; rel1: 0.1 0.1; rel2: 0.2 0.2; } }"#,
            r#"glyf { version: 1; annotation { name: "bad"; rel1: 0.1 0.1; } }"#,
        ] {
            assert!(parse(text).is_err(), "expected invalid annotation: {text}");
        }
    }

    #[test]
    fn region_validation_rejects_zero_size() {
        let document = GlyfDocument {
            version: 1,
            images: vec![],
            groups: vec![],
            annotations: vec![GlyfAnnotation::region(
                "zero",
                GlyfPoint { x: 0.4, y: 0.4 },
                GlyfPoint { x: 0.4, y: 0.8 },
                "zero width",
            )],
        };
        assert!(validate(&document).is_err());
    }
    #[test]
    fn annotation_validation_rejects_bad_position_and_duplicate_name() {
        let base = || GlyfDocument {
            version: 1,
            images: vec![],
            groups: vec![],
            annotations: vec![GlyfAnnotation::point("note", GlyfPoint { x: 0.5, y: 0.5 }, "note")],
        };
        let mut invalid = base();
        if let GlyfAnnotationGeometry::Point { rel } = &mut invalid.annotations[0].geometry {
            rel.x = 1.1;
        }
        assert!(validate(&invalid).is_err());
        let mut invalid = base();
        invalid.annotations.push(invalid.annotations[0].clone());
        assert!(validate(&invalid).is_err());
    }

    #[test]
    fn spatial_part_round_trips_and_keeps_layer_order() {
        let document = GlyfDocument {
            version: 1,
            images: vec![GlyfImage {
                name: "atlas".into(),
                source: "atlas.png".into(),
                comment: String::new(),
            }],
            groups: vec![GlyfGroup {
                name: "main".into(),
                parts: vec![
                    GlyfPart {
                        name: "window".into(),
                        source: "atlas".into(),
                        clip: Some(GlyfClip {
                            x: 0.0,
                            y: 0.0,
                            w: 800.0,
                            h: 600.0,
                        }),
                        rel1: GlyfPoint { x: 0.1, y: 0.1 },
                        rel2: GlyfPoint { x: 0.9, y: 0.9 },
                        comment: "Window".into(),
                        parent: None,
                    },
                    GlyfPart {
                        name: "button".into(),
                        source: "atlas".into(),
                        clip: Some(GlyfClip {
                            x: 120.0,
                            y: 640.0,
                            w: 240.0,
                            h: 64.0,
                        }),
                        rel1: GlyfPoint { x: 0.4, y: 0.4 },
                        rel2: GlyfPoint { x: 0.6, y: 0.47 },
                        comment: "Button".into(),
                        parent: Some("window".into()),
                    },
                ],
            }],
            annotations: vec![],
        };
        let encoded = serialize(&document).expect("serialize spatial document");
        assert_eq!(parse(&encoded).expect("parse spatial document"), document);
    }

    #[test]
    fn spatial_validation_rejects_invalid_references_and_ranges() {
        let part = |name: &str| GlyfPart {
            name: name.into(),
            source: "atlas".into(),
            clip: Some(GlyfClip {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            }),
            rel1: GlyfPoint { x: 0.0, y: 0.0 },
            rel2: GlyfPoint { x: 1.0, y: 1.0 },
            comment: String::new(),
            parent: None,
        };
        let base = || GlyfDocument {
            version: 1,
            images: vec![GlyfImage {
                name: "atlas".into(),
                source: "atlas.png".into(),
                comment: String::new(),
            }],
            groups: vec![GlyfGroup {
                name: "main".into(),
                parts: vec![part("a")],
            }],
            annotations: vec![],
        };
        let mut invalid = base();
        invalid.groups[0].parts[0].source = "missing".into();
        assert!(validate(&invalid).is_err());
        let mut invalid = base();
        invalid.groups[0].parts[0].rel1.x = 1.1;
        assert!(validate(&invalid).is_err());
        let mut invalid = base();
        invalid.groups[0].parts[0].parent = Some("a".into());
        assert!(validate(&invalid).is_err());
        let mut invalid = base();
        invalid.groups[0].parts.push(part("a"));
        assert!(validate(&invalid).is_err());
    }

    #[test]
    fn validation_rejects_empty_and_duplicate_names() {
        let empty = GlyfDocument {
            version: 1,
            images: vec![GlyfImage {
                name: " ".into(),
                source: "a.png".into(),
                comment: String::new(),
            }],
            groups: vec![],
            annotations: vec![],
        };
        assert!(validate(&empty).is_err());

        let duplicate = GlyfDocument {
            version: 1,
            images: vec![
                GlyfImage {
                    name: "a".into(),
                    source: "a.png".into(),
                    comment: String::new(),
                },
                GlyfImage {
                    name: "a".into(),
                    source: "b.png".into(),
                    comment: String::new(),
                },
            ],
            groups: vec![],
            annotations: vec![],
        };
        assert!(validate(&duplicate).is_err());
    }
}
