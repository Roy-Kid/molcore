use std::collections::HashMap;
use std::io::BufRead;
use crate::io::reader::{Reader, FrameReader};
use crate::core::array::NdArray;
use crate::core::block::Block;
use crate::core::frame::Frame;

// EXTXYZ comment line parser using winnow
use winnow::combinator::{alt, separated, opt, repeat};
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::{take_while};


// XYZ now produces a core::Frame consisting of blocks of NdArray columns


#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
	Str(String),
	Int(i64),
	Real(f64),
	Logical(bool)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtValue {
    Primitive(Primitive),
    Array1(Vec<Primitive>),
    Array2(Vec<Vec<Primitive>>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropType {
	S,
	I,
	R,
	L,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertySpec {
	pub name: String,
	pub ty: PropType,
	pub m: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct XYZComment {
	pub kv: HashMap<String, ExtValue>,
	pub properties: Option<Vec<PropertySpec>>, // parsed from key "Properties" if present
	pub comment: Option<String>, // original comment line when treated as extxyz
	pub is_plain_xyz: bool,
}

pub type ParseError = ContextError;

fn ws0<'a>() -> impl Parser<&'a str, &'a str, ParseError> {
	take_while(0.., |c: char| c.is_ascii_whitespace())
}

fn ws1<'a>() -> impl Parser<&'a str, &'a str, ParseError> {
	take_while(1.., |c: char| c.is_ascii_whitespace())
}

fn bare_key<'a>() -> impl Parser<&'a str, String, ParseError> {
	// Bare key: run of non-whitespace, not containing '='
	take_while(1.., |c: char| !c.is_ascii_whitespace() && c != '=')
		.map(|s: &str| s.to_string())
}

// JSON-like string parser that handles escapes
fn quoted_string<'a>() -> impl Parser<&'a str, String, ParseError> {
	use winnow::combinator::{preceded, terminated};
	preceded(
		'"',
		terminated(
			// does not handle escapes; acceptable for extxyz comment in common cases
			take_while(0.., |c: char| c != '"').map(|s: &str| s.to_string()),
			'"',
		),
	)
}

fn key_parser<'a>() -> impl Parser<&'a str, String, ParseError> {
	alt((quoted_string(), bare_key()))
}

fn unquoted_value_token<'a>() -> impl Parser<&'a str, String, ParseError> {
	take_while(1.., |c: char| !c.is_ascii_whitespace()).map(|s: &str| s.to_string())
}

fn parse_logical_token(tok: &str) -> Option<bool> {
	match tok.to_ascii_lowercase().as_str() {
		"t" | "true" => Some(true),
		"f" | "false" => Some(false),
		_ => None,
	}
}

fn parse_primitive_token(tok: &str) -> Primitive {
	if let Some(b) = parse_logical_token(tok) {
		Primitive::Logical(b)
	} else if tok.contains('.') || tok.contains('e') || tok.contains('E') {
		match tok.parse::<f64>() {
			Ok(v) => Primitive::Real(v),
			Err(_) => Primitive::Str(tok.to_string()),
		}
	} else {
		match tok.parse::<i64>() {
			Ok(v) => Primitive::Int(v),
			Err(_) => Primitive::Str(tok.to_string()),
		}
	}
}

fn parse_array_from_quoted(s: &str) -> ExtValue {
	// Try 2D using row separators ';' or '|' or comma between rows
	let has_row_sep = s.contains(';') || s.contains('|') || s.contains('\n');
	if has_row_sep {
		let rows: Vec<Vec<Primitive>> = s
			.split(|c| c == ';' || c == '|' || c == '\n')
			.filter(|row| !row.trim().is_empty())
			.map(|row| row.split_whitespace().map(parse_primitive_token).collect())
			.collect();
		return ExtValue::Array2(rows);
	}

	// Try comma-separated values or whitespace-separated
	let elements: Vec<&str> = if s.contains(',') {
		s.split(',').collect()
	} else {
		s.split_whitespace().collect()
	};
	if elements.len() > 1 {
		let vals = elements
			.into_iter()
			.map(|t| parse_primitive_token(t.trim()))
			.collect();
		ExtValue::Array1(vals)
	} else {
		ExtValue::Primitive(Primitive::Str(s.to_string()))
	}
}

fn value_parser<'a>() -> impl Parser<&'a str, ExtValue, ParseError> {
	alt((
		// Quoted string: could be a primitive string or an array (1D/2D). We'll post-process.
		quoted_string().map(|s| parse_array_from_quoted(&s)),
		// Unquoted token -> primitive
		unquoted_value_token().map(|t| ExtValue::Primitive(parse_primitive_token(&t))),
	))
}

fn pair_parser<'a>() -> impl Parser<&'a str, (String, ExtValue), ParseError> {
	(ws0(), key_parser(), ws0(), '=', ws0(), value_parser())
		.map(|(_, k, _, _, _, v)| (k, v))
}

fn parse_pairs<'a>() -> impl Parser<&'a str, Vec<(String, ExtValue)>, ParseError> {
	separated(1.., pair_parser(), ws1())
}

fn parse_properties(spec: &str) -> Option<Vec<PropertySpec>> {
	// Expect a colon-separated stream of triplets: name:T:m: name2:T:m: ...
	let parts: Vec<&str> = spec.split(':').filter(|s| !s.is_empty()).collect();
	if parts.len() < 3 || parts.len() % 3 != 0 {
		return None;
	}
	let mut out = Vec::new();
	let mut i = 0;
	while i + 2 < parts.len() {
		let name = parts[i].to_string();
		let ty = match parts[i + 1] {
			"S" => PropType::S,
			"I" => PropType::I,
			"R" => PropType::R,
			"L" => PropType::L,
			other => {
				// try tolerate lower-case
				match other.to_ascii_uppercase().as_str() {
					"S" => PropType::S,
					"I" => PropType::I,
					"R" => PropType::R,
					"L" => PropType::L,
					_ => return None,
				}
			}
		};
		let m = match parts[i + 2].parse::<usize>() {
			Ok(v) if v > 0 => v,
			_ => return None,
		};
		out.push(PropertySpec { name, ty, m });
		i += 3;
	}
	Some(out)
}

pub fn parse_comment_line(line: &str) -> std::result::Result<XYZComment, String> {
	let original = line.to_string();
	let mut input = line.trim();

	// Quick check: if no '=' present, treat as plain xyz comment
	if !input.contains('=') {
		let mut kv = HashMap::new();
		kv.insert("comment".to_string(), ExtValue::Primitive(Primitive::Str(original.clone())));
		return Ok(XYZComment {
			kv,
			properties: None,
			comment: None,
			is_plain_xyz: true,
		});
	}

	match parse_pairs().parse_next(&mut input) {
		Ok(pairs) => {
			let mut kv: HashMap<String, ExtValue> = HashMap::new();
			let mut properties: Option<Vec<PropertySpec>> = None;
			for (k, v) in pairs.into_iter() {
				if k.eq_ignore_ascii_case("properties") {
					// Extract the raw string form to parse triplets
					let spec_str = match &v {
						ExtValue::Primitive(Primitive::Str(s)) => s.clone(),
						ExtValue::Array1(vs) => vs
							.iter()
							.map(|p| match p { Primitive::Str(s) => s.clone(), _ => "".into() })
							.collect::<Vec<_>>()
							.join(" "),
						_ => String::new(),
					};
					properties = parse_properties(&spec_str);
				}
				kv.insert(k, v);
			}
			if properties.is_none() {
				// retroactively plain xyz: store full second line as per-config comment
				kv.insert("comment".to_string(), ExtValue::Primitive(Primitive::Str(original.clone())));
				Ok(XYZComment { kv, properties: None, comment: None, is_plain_xyz: true })
			} else {
				Ok(XYZComment { kv, properties, comment: Some(original), is_plain_xyz: false })
			}
		}
		Err(e) => Err(format!("failed to parse comment line: {e:?}")),
	}
}

fn expand_property_columns(props: &[PropertySpec]) -> Vec<(String, PropType)> {
	let mut cols = Vec::new();
	for p in props {
		if p.m == 1 {
			cols.push((p.name.clone(), p.ty));
		} else {
			// Special-case: map pos:R:3 -> x,y,z (LAMMPS naming)
			if p.name.eq_ignore_ascii_case("pos") && p.ty == PropType::R && p.m == 3 {
				cols.push(("x".to_string(), PropType::R));
				cols.push(("y".to_string(), PropType::R));
				cols.push(("z".to_string(), PropType::R));
			} else {
				for i in 0..p.m {
					cols.push((format!("{}_{}", p.name, i + 1), p.ty));
				}
			}
		}
	}
	cols
}

fn parse_bool_token(tok: &str) -> Option<bool> {
	match tok.to_ascii_lowercase().as_str() {
		"t" | "true" => Some(true),
		"f" | "false" => Some(false),
		_ => None,
	}
}

fn line_to_tokens(line: &str) -> Vec<&str> {
	line.split_whitespace().collect()
}

/// Build schema from parsed properties
fn build_complete_schema(ec: &XYZComment) -> Vec<PropertySpec> {
	// If no Properties key, return plain XYZ schema (4 columns: element, x, y, z)
	// Otherwise, return the properties as-is
	ec.properties.as_ref().map(|p| p.clone()).unwrap_or_else(|| vec![
		PropertySpec { name: "element".into(), ty: PropType::S, m: 1 },
		PropertySpec { name: "x".into(), ty: PropType::R, m: 1 },
		PropertySpec { name: "y".into(), ty: PropType::R, m: 1 },
		PropertySpec { name: "z".into(), ty: PropType::R, m: 1 },
	])
}

fn build_block_from_props(n: usize, lines: &[String], props: &[PropertySpec]) -> Result<Block, String> {
	let cols = expand_property_columns(props);
	let m_total = cols.len();
	if lines.len() != n { return Err("insufficient atom lines".into()); }

	// Prepare column buffers by type (use f32 for real values)
	enum ColBuf { S(Vec<String>), I(Vec<i64>), R(Vec<f32>), L(Vec<bool>) }
	let mut buffers: Vec<ColBuf> = cols.iter().map(|(_, t)| match t {
		PropType::S => ColBuf::S(Vec::with_capacity(n)),
		PropType::I => ColBuf::I(Vec::with_capacity(n)),
		PropType::R => ColBuf::R(Vec::with_capacity(n)),
		PropType::L => ColBuf::L(Vec::with_capacity(n)),
	}).collect();

	for (row_i, line) in lines.iter().enumerate() {
		let toks = line_to_tokens(line);
		if toks.len() < m_total { return Err(format!("line {}: expected at least {} tokens, got {}", row_i, m_total, toks.len())); }
		let mut idx = 0;
		// Iterate over props but push into flattened buffers
		for (buf_idx, (_, ty)) in cols.iter().enumerate() {
			let tok = toks[idx];
			match (&mut buffers[buf_idx], ty) {
				(ColBuf::S(v), PropType::S) => v.push(tok.to_string()),
				(ColBuf::I(v), PropType::I) => v.push(tok.parse::<i64>().map_err(|_| format!("line {} col {}: invalid int '{}" , row_i, buf_idx, tok))?),
				(ColBuf::R(v), PropType::R) => v.push(tok.parse::<f32>().map_err(|_| format!("line {} col {}: invalid float '{}" , row_i, buf_idx, tok))?),
				(ColBuf::L(v), PropType::L) => v.push(parse_bool_token(tok).ok_or_else(|| format!("line {} col {}: invalid bool '{}" , row_i, buf_idx, tok))?),
				_ => return Err(format!("type mismatch at line {} col {}", row_i, buf_idx)),
			}
			idx += 1;
		}
	}

	// Assemble core::Block: drop string columns (S) as Block stores numeric/boolean arrays only
	let mut block = Block::new();
	for ((name, ty), buf) in cols.into_iter().zip(buffers.into_iter()) {
		match (ty, buf) {
			(PropType::I, ColBuf::I(v)) => { let arr = NdArray::from_vec(vec![n, 1], v); block.insert(name, arr).map_err(|e| e.to_string())?; },
			(PropType::R, ColBuf::R(v)) => { let arr = NdArray::from_vec(vec![n, 1], v); block.insert(name, arr).map_err(|e| e.to_string())?; },
			(PropType::L, ColBuf::L(v)) => { let arr = NdArray::from_vec(vec![n, 1], v); block.insert(name, arr).map_err(|e| e.to_string())?; },
			(PropType::S, ColBuf::S(_v)) => { /* skip string columns for now */ },
			_ => { /* type mismatch shouldn't happen due to construction; skip */ }
		}
	}

	Ok(block)
}

/// Read one XYZ/EXTXYZ frame from the current position of a buffered reader
/// Returns Ok(None) on EOF before the first line
pub fn read_xyz_frame<R: BufRead>(reader: &mut R) -> std::io::Result<Option<Frame>> {

	// Read first non-empty line as atom count
	let mut line = String::new();
	let n = loop {
		line.clear();
		let bytes = reader.read_line(&mut line)?;
		if bytes == 0 {
			return Ok(None); // EOF
		}
		let trimmed = line.trim();
		if trimmed.is_empty() { continue; }
		match trimmed.parse::<usize>() {
			Ok(v) => break v,
			Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid atom count line: {}", trimmed))),
		}
	};

	// Read comment line (can be empty)
	line.clear();
	let _ = reader.read_line(&mut line)?; // if EOF after count, it's malformed but we allow empty
	let comment = line.trim_end_matches(['\r', '\n']);

	// Parse comment to metadata and properties
	let ec = parse_comment_line(comment).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
	let mut kv_meta: HashMap<String, ExtValue> = HashMap::new();
	for (k, v) in ec.kv.iter() {
		if k.eq_ignore_ascii_case("Properties") { continue; }
		kv_meta.insert(k.clone(), v.clone());
	}

	// Read N atom lines
	let mut atom_lines: Vec<String> = Vec::with_capacity(n);
	for _ in 0..n {
		line.clear();
		let bytes = reader.read_line(&mut line)?;
		if bytes == 0 { return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "unexpected EOF in atom lines")); }
		atom_lines.push(line.trim_end_matches(['\r', '\n']).to_string());
	}

	// Build complete schema (base properties + derived columns)
	let schema = build_complete_schema(&ec);
	

	// Parse columns according to schema -> atoms block
	let atoms_block = build_block_from_props(n, &atom_lines, &schema)
		.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

	let mut frame = Frame::new();
	frame.insert("atoms", atoms_block);
	// Stringify metadata into frame.meta
	for (k, v) in kv_meta.into_iter() { frame.meta.insert(k, ext_value_to_string(&v)); }

	Ok(Some(frame))
}

// =============== Winnow-based frame parser for a complete frame string ===============

fn line_ending<'a>() -> impl Parser<&'a str, &'a str, ParseError> {
	alt(("\r\n", "\n"))
}

fn line_string<'a>() -> impl Parser<&'a str, String, ParseError> {
	// Capture a line without the trailing line ending; allow optional line ending (for last line)
	(take_while(0.., |c: char| c != '\n' && c != '\r'), opt(line_ending()))
		.map(|(s, _): (&str, _)| s.to_string())
}

fn digits1<'a>() -> impl Parser<&'a str, &'a str, ParseError> {
	take_while(1.., |c: char| c.is_ascii_digit())
}

fn number_line<'a>() -> impl Parser<&'a str, usize, ParseError> {
	(ws0(), digits1(), ws0(), opt(line_ending()))
		.map(|(_, d, _, _)| d)
		.try_map(|d: &str| d.parse::<usize>())
}

/// Parse a complete XYZ/EXTXYZ frame from a &str using winnow only
pub fn parse_xyz_frame_str(s: &str) -> std::result::Result<Frame, String> {
	let mut input = s;

	let n = number_line().parse_next(&mut input)
		.map_err(|e| format!("parse atom count: {e:?}"))?;

	let comment = line_string().parse_next(&mut input)
		.map_err(|e| format!("parse comment line: {e:?}"))?;

	// Parse exactly N atom lines (last line may or may not have a trailing newline)
	let atom_lines: Vec<String> = repeat(n..=n, line_string())
		.parse_next(&mut input)
		.map_err(|e| format!("parse atom lines: {e:?}"))?;

	// Parse comment to metadata and properties
	let ec = parse_comment_line(&comment)?;
	let mut kv_meta: HashMap<String, ExtValue> = HashMap::new();
	for (k, v) in ec.kv.iter() {
		if k.eq_ignore_ascii_case("Properties") { continue; }
		kv_meta.insert(k.clone(), v.clone());
	}

	// Build complete schema (base properties + derived columns)
	let schema = build_complete_schema(&ec);
	
	// Parse columns according to schema
	let atoms_block = build_block_from_props(n, &atom_lines, &schema)?;
	let mut frame = Frame::new();
	frame.insert("atoms", atoms_block);
	for (k, v) in kv_meta.into_iter() { frame.meta.insert(k, ext_value_to_string(&v)); }
	Ok(frame)
}

fn ext_value_to_string(v: &ExtValue) -> String {
	match v {
		ExtValue::Primitive(Primitive::Str(s)) => s.clone(),
		ExtValue::Primitive(Primitive::Int(i)) => i.to_string(),
		ExtValue::Primitive(Primitive::Real(r)) => format!("{}", r),
		ExtValue::Primitive(Primitive::Logical(b)) => b.to_string(),
		ExtValue::Array1(vals) => vals.iter().map(|p| match p {
			Primitive::Str(s) => s.clone(),
			Primitive::Int(i) => i.to_string(),
			Primitive::Real(r) => format!("{}", r),
			Primitive::Logical(b) => b.to_string(),
		}).collect::<Vec<_>>().join(" "),
		ExtValue::Array2(rows) => rows.iter().map(|row| row.iter().map(|p| match p {
			Primitive::Str(s) => s.clone(),
			Primitive::Int(i) => i.to_string(),
			Primitive::Real(r) => format!("{}", r),
			Primitive::Logical(b) => b.to_string(),
		}).collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join("; "),
	}
}

// =============== XYZFrameReader ===============

/// A reader for XYZ/EXTXYZ single-frame files
/// 
/// This reader supports both plain text and gzip-compressed files
/// through the generic `BufRead` interface.
/// 
/// # Examples
/// 
/// ```no_run
/// use molcore::io::xyz::XYZFrameReader;
/// use molcore::io::reader::{Reader, FrameReader, open_txt, open_gz};
/// 
/// // Read plain text file
/// let mut reader = XYZFrameReader::new(open_txt("file.xyz").unwrap());
/// if let Some(frame) = reader.read_frame().unwrap() {
///     if let Some(atoms) = frame.get("atoms") {
///         println!("Atoms: {}", atoms.nrows().unwrap_or(0));
///     }
/// }
/// 
/// // Read gzip-compressed file
/// let mut reader = XYZFrameReader::new(open_gz("file.xyz.gz").unwrap());
/// if let Some(frame) = reader.read_frame().unwrap() {
///     if let Some(atoms) = frame.get("atoms") {
///         println!("Atoms: {}", atoms.nrows().unwrap_or(0));
///     }
/// }
/// ```
pub struct XYZFrameReader<R: BufRead> {
	reader: R,
}

impl<R: BufRead> Reader for XYZFrameReader<R> {
	type R = R;
	type FrameLike = Frame;

	fn new(reader: Self::R) -> Self {
		Self { reader }
	}
}

impl<R: BufRead> FrameReader for XYZFrameReader<R> {
	fn read_frame(&mut self) -> std::io::Result<Option<Self::FrameLike>> {
		read_xyz_frame(&mut self.reader)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_properties_triplets() {
		let line = "Properties=species:S:1:pos:R:3:mass:R:1";
		let ec = parse_comment_line(line).expect("parse");
		assert!(ec.properties.is_some());
		let props = ec.properties.unwrap();
		assert_eq!(props.len(), 3);
		assert_eq!(props[0], PropertySpec { name: "species".into(), ty: PropType::S, m: 1 });
		assert_eq!(props[1], PropertySpec { name: "pos".into(), ty: PropType::R, m: 3 });
		assert_eq!(props[2], PropertySpec { name: "mass".into(), ty: PropType::R, m: 1 });
		assert!(!ec.is_plain_xyz);
	}

	#[test]
	fn parse_comment_with_properties() {
		let line = r#"Lattice="8.43116035 0.0 0.0 0.158219155128 14.5042431863 0.0 1.16980663624 4.4685149855 14.9100096405" Properties=species:S:1:pos:R:3:CS:R:2 ENERGY=-2069.84934116 Natoms=192 NAME=COBHUW"#;
		let ec = parse_comment_line(line).expect("parse");
		assert!(ec.properties.is_some());
		assert!(!ec.is_plain_xyz);
		// Lattice
		match ec.kv.get("Lattice").unwrap() {
			ExtValue::Array1(v) => {
				assert_eq!(v.len(), 9);
				assert!(matches!(v[0], Primitive::Real(_)) || matches!(v[0], Primitive::Int(_)));
			}
			other => panic!("unexpected Lattice value: {other:?}"),
		}
		// energy
		match ec.kv.get("ENERGY").unwrap() {
			ExtValue::Primitive(Primitive::Real(x)) => assert!((x - -2069.84934116).abs() < 1e-6),
			other => panic!("unexpected energy value: {other:?}"),
		}
	}

    #[test]
    fn parse_single_frame() {
        let frame_str = r#"3
Properties=species:S:1:pos:R:3:velo:R:3 Lattice="10 0 0 0 10 0 0 0 10"
H 0 0 1 1 0 0
O 0 1 0 0 1 0
H 1 0 0 0 0 1"#;
	let frame = parse_xyz_frame_str(frame_str).expect("parse frame");
	let atoms = frame.get("atoms").expect("atoms block");
	assert_eq!(atoms.nrows().unwrap_or(0), 3);
	assert!(atoms.len() >= 3); // pos_1,pos_2,pos_3 present (species dropped)
	// metadata should only contain Lattice (Properties is excluded)
	assert!(frame.meta.get("Lattice").is_some());
    }

	#[test]
	fn test_xyz_frame_reader() {
		use crate::io::reader::FrameReader;
		use std::io::Cursor;

		let data = b"3
Properties=species:S:1:pos:R:3
H 0.0 0.0 0.0
O 1.0 0.0 0.0
H 2.0 0.0 0.0
";
		let cursor = Cursor::new(&data[..]);
		let mut reader = XYZFrameReader::new(cursor);

	let frame = reader.read_frame().expect("read frame").expect("frame exists");
	let atoms = frame.get("atoms").expect("atoms block");
	assert_eq!(atoms.nrows().unwrap_or(0), 3);
	assert!(atoms.len() >= 3); // pos_1, pos_2, pos_3 present (species dropped)

		// Should return None on subsequent read (EOF)
		let eof = reader.read_frame().expect("read ok");
		assert!(eof.is_none());
	}

	#[test]
	fn test_xyz_frame_reader_plain_xyz() {
		use crate::io::reader::FrameReader;
		use std::io::Cursor;

		let data = b"2
Water molecule
O 0.0 0.0 0.0
H 1.0 0.0 0.0
";
		let cursor = Cursor::new(&data[..]);
		let mut reader = XYZFrameReader::new(cursor);

	let frame = reader.read_frame().expect("read frame").expect("frame exists");
	let atoms = frame.get("atoms").expect("atoms block");
	assert_eq!(atoms.nrows().unwrap_or(0), 2);
	assert!(atoms.len() >= 3); // x, y, z
	assert!(frame.meta.get("comment").is_some());
	}
}
