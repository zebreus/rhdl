use std::ops::Range;

use anyhow::bail;
use anyhow::Result;

use crate::ast::ast_impl::Member;
use crate::rhif::spec::Slot;
use crate::DiscriminantAlignment;
use crate::Kind;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathElement {
    Index(usize),
    Field(String),
    EnumDiscriminant,
    EnumPayload(String),
    EnumPayloadByValue(i64),
    DynamicIndex(Slot),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Path {
    pub elements: Vec<PathElement>,
}

impl FromIterator<PathElement> for Path {
    fn from_iter<T: IntoIterator<Item = PathElement>>(iter: T) -> Self {
        Path {
            elements: iter.into_iter().collect(),
        }
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.elements {
            match e {
                PathElement::Index(i) => write!(f, "[{}]", i)?,
                PathElement::Field(s) => write!(f, ".{}", s)?,
                PathElement::EnumDiscriminant => write!(f, "#")?,
                PathElement::EnumPayload(s) => write!(f, "#{}", s)?,
                PathElement::EnumPayloadByValue(v) => write!(f, "#{}", v)?,
                PathElement::DynamicIndex(slot) => write!(f, "[[{}]]", slot)?,
            }
        }
        Ok(())
    }
}

impl Path {
    pub fn iter(&self) -> impl Iterator<Item = &PathElement> {
        self.elements.iter()
    }
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    pub fn dynamic_slots(&self) -> impl Iterator<Item = &Slot> {
        self.elements.iter().filter_map(|e| match e {
            PathElement::DynamicIndex(slot) => Some(slot),
            _ => None,
        })
    }
    pub fn index(mut self, index: usize) -> Self {
        self.elements.push(PathElement::Index(index));
        self
    }
    pub fn field(mut self, field: &str) -> Self {
        self.elements.push(PathElement::Field(field.to_string()));
        self
    }
    pub fn member(mut self, member: Member) -> Self {
        match member {
            Member::Named(name) => self.elements.push(PathElement::Field(name)),
            Member::Unnamed(ndx) => self.elements.push(PathElement::Index(ndx as usize)),
        }
        self
    }
    pub fn discriminant(mut self) -> Self {
        self.elements.push(PathElement::EnumDiscriminant);
        self
    }
    pub fn dynamic(mut self, slot: Slot) -> Self {
        self.elements.push(PathElement::DynamicIndex(slot));
        self
    }
    pub fn payload(mut self, name: &str) -> Self {
        self.elements
            .push(PathElement::EnumPayload(name.to_string()));
        self
    }
    pub fn join(mut self, other: &Path) -> Self {
        self.elements.extend(other.elements.clone());
        self
    }
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
    pub fn payload_by_value(mut self, discriminant: i64) -> Self {
        self.elements
            .push(PathElement::EnumPayloadByValue(discriminant));
        self
    }
    pub fn any_dynamic(&self) -> bool {
        self.elements
            .iter()
            .any(|e| matches!(e, PathElement::DynamicIndex(_)))
    }
    pub fn remap_slots<F: FnMut(Slot) -> Slot>(self, mut f: F) -> Path {
        Path {
            elements: self
                .elements
                .into_iter()
                .map(|e| match e {
                    PathElement::DynamicIndex(slot) => PathElement::DynamicIndex(f(slot)),
                    _ => e,
                })
                .collect(),
        }
    }
    pub fn is_prefix_of(&self, other: &Path) -> bool {
        self.elements.len() <= other.elements.len()
            && self
                .elements
                .iter()
                .zip(other.elements.iter())
                .all(|(a, b)| a == b)
    }
    pub fn strip_prefix(&self, prefix: &Path) -> Result<Path> {
        if !prefix.is_prefix_of(self) {
            bail!("Path is not a prefix of self")
        }
        Ok(Path {
            elements: self.elements[prefix.elements.len()..].to_vec(),
        })
    }
}

impl From<Member> for Path {
    fn from(member: Member) -> Self {
        match member {
            Member::Named(name) => Path {
                elements: vec![PathElement::Field(name)],
            },
            Member::Unnamed(ndx) => Path {
                elements: vec![PathElement::Index(ndx as usize)],
            },
        }
    }
}

// Given a path and a kind, computes all possible paths that can be
// generated from the base path using legal values for the dynamic
// indices.
pub fn path_star(kind: &Kind, path: &Path) -> Result<Vec<Path>> {
    eprintln!("path star called with kind {} and path {}", kind, path);
    if !path.any_dynamic() {
        return Ok(vec![path.clone()]);
    }
    if let Some(element) = path.elements.first() {
        match element {
            PathElement::DynamicIndex(_) => {
                let Kind::Array(array) = kind else {
                    bail!("Dynamic index on non-array type")
                };
                let mut paths = Vec::new();
                for i in 0..array.size {
                    let mut path = path.clone();
                    path.elements[0] = PathElement::Index(i);
                    paths.extend(path_star(kind, &path)?);
                }
                return Ok(paths);
            }
            p => {
                let prefix_path = Path {
                    elements: vec![p.clone()],
                };
                let prefix_kind = sub_kind(kind.clone(), &prefix_path)?;
                let suffix_path = path.strip_prefix(&prefix_path)?;
                let suffix_star = path_star(&prefix_kind, &suffix_path)?;
                return Ok(suffix_star
                    .into_iter()
                    .map(|suffix| prefix_path.clone().join(&suffix))
                    .collect());
            }
        }
    }
    Ok(vec![path.clone()])
}

pub fn sub_kind(kind: Kind, path: &Path) -> Result<Kind> {
    bit_range(kind, path).map(|(_, kind)| kind)
}

// Given a Kind and a Vec<Path>, compute the bit offsets of
// the endpoint of the path within the original data structure.
pub fn bit_range(kind: Kind, path: &Path) -> Result<(Range<usize>, Kind)> {
    let mut range = 0..kind.bits();
    let mut kind = kind;
    for p in &path.elements {
        match p {
            PathElement::Index(i) => match &kind {
                Kind::Array(array) => {
                    let element_size = array.base.bits();
                    if i >= &array.size {
                        bail!("Array index out of bounds")
                    }
                    range = range.start + i * element_size..range.start + (i + 1) * element_size;
                    kind = *array.base.clone();
                }
                Kind::Tuple(tuple) => {
                    if i >= &tuple.elements.len() {
                        bail!("Tuple index out of bounds")
                    }
                    let offset = tuple.elements[0..*i]
                        .iter()
                        .map(|e| e.bits())
                        .sum::<usize>();
                    let size = tuple.elements[*i].bits();
                    range = range.start + offset..range.start + offset + size;
                    kind = tuple.elements[*i].clone();
                }
                Kind::Struct(structure) => {
                    if i >= &structure.fields.len() {
                        bail!("Struct index out of bounds")
                    }
                    let offset = structure
                        .fields
                        .iter()
                        .take(*i)
                        .map(|f| f.kind.bits())
                        .sum::<usize>();
                    let size = structure.fields[*i].kind.bits();
                    range = range.start + offset..range.start + offset + size;
                    kind = structure.fields[*i].kind.clone();
                }
                _ => bail!("Indexing non-indexable type {kind}"),
            },
            PathElement::Field(field) => match &kind {
                Kind::Struct(structure) => {
                    if !structure.fields.iter().any(|f| &f.name == field) {
                        bail!("Field not found")
                    }
                    let offset = structure
                        .fields
                        .iter()
                        .take_while(|f| &f.name != field)
                        .map(|f| f.kind.bits())
                        .sum::<usize>();
                    let field = &structure
                        .fields
                        .iter()
                        .find(|f| &f.name == field)
                        .unwrap()
                        .kind;
                    let size = field.bits();
                    range = range.start + offset..range.start + offset + size;
                    kind = field.clone();
                }
                _ => bail!("Field indexing not allowed on this type {kind}"),
            },
            PathElement::EnumDiscriminant => match &kind {
                Kind::Enum(enumerate) => {
                    range = match enumerate.discriminant_layout.alignment {
                        DiscriminantAlignment::Lsb => {
                            range.start..range.start + enumerate.discriminant_layout.width
                        }
                        DiscriminantAlignment::Msb => {
                            range.end - enumerate.discriminant_layout.width..range.end
                        }
                    };
                    kind = if enumerate.discriminant_layout.ty == crate::DiscriminantType::Signed {
                        Kind::make_signed(enumerate.discriminant_layout.width)
                    } else {
                        Kind::make_bits(enumerate.discriminant_layout.width)
                    };
                }
                _ => bail!("Enum discriminant not valid for non-enum types"),
            },
            PathElement::EnumPayload(name) => match &kind {
                Kind::Enum(enumerate) => {
                    let field = enumerate
                        .variants
                        .iter()
                        .find(|f| &f.name == name)
                        .ok_or_else(|| anyhow::anyhow!("Enum payload not found"))?
                        .kind
                        .clone();
                    range = match enumerate.discriminant_layout.alignment {
                        DiscriminantAlignment::Lsb => {
                            range.start + enumerate.discriminant_layout.width
                                ..range.start + enumerate.discriminant_layout.width + field.bits()
                        }
                        DiscriminantAlignment::Msb => range.start..range.start + field.bits(),
                    };
                    kind = field;
                }
                _ => bail!("Enum payload not valid for non-enum types"),
            },
            PathElement::EnumPayloadByValue(disc) => match &kind {
                Kind::Enum(enumerate) => {
                    let field = enumerate
                        .variants
                        .iter()
                        .find(|f| f.discriminant == *disc)
                        .ok_or_else(|| anyhow::anyhow!("Enum payload not found"))?
                        .kind
                        .clone();
                    range = match enumerate.discriminant_layout.alignment {
                        DiscriminantAlignment::Lsb => {
                            range.start + enumerate.discriminant_layout.width
                                ..range.start + enumerate.discriminant_layout.width + field.bits()
                        }
                        DiscriminantAlignment::Msb => range.start..range.start + field.bits(),
                    };
                    kind = field;
                }
                _ => bail!("Enum payload not valid for non-enum types"),
            },
            PathElement::DynamicIndex(_slot) => {
                bail!("Dynamic indices must be resolved before calling bit_range")
            }
        }
    }
    Ok((range, kind))
}

#[cfg(test)]
mod tests {
    use crate::{path::path_star, rhif::spec::Slot, Kind};

    use super::Path;

    #[test]
    fn test_path_star() {
        let base_struct = Kind::make_struct(
            "base",
            vec![
                Kind::make_field("a", Kind::make_bits(8)),
                Kind::make_field("b", Kind::make_array(Kind::make_bits(8), 3)),
            ],
        );
        // Create a path with a struct, containing and array of structs
        let kind = Kind::make_struct(
            "foo",
            vec![
                Kind::make_field("c", base_struct.clone()),
                Kind::make_field("d", Kind::make_array(base_struct.clone(), 4)),
            ],
        );
        let path1 = Path::default().field("c").field("a");
        assert_eq!(path_star(&kind, &path1).unwrap().len(), 1);
        let path1 = Path::default().field("c").field("b");
        assert_eq!(path_star(&kind, &path1).unwrap().len(), 1);
        let path1 = Path::default().field("c").field("b").index(0);
        assert_eq!(path_star(&kind, &path1).unwrap().len(), 1);
        let path1 = Path::default()
            .field("c")
            .field("b")
            .dynamic(Slot::Register(0));
        let path1_star = path_star(&kind, &path1).unwrap();
        assert_eq!(path1_star.len(), 3);
        for path in path1_star {
            assert_eq!(path.elements.len(), 3);
            assert!(!path.any_dynamic());
            eprintln!("{}", path);
        }
        let path2 = Path::default()
            .field("d")
            .dynamic(Slot::Register(0))
            .field("b");
        let path2_star = path_star(&kind, &path2).unwrap();
        assert_eq!(path2_star.len(), 4);
        for path in path2_star {
            assert_eq!(path.elements.len(), 3);
            assert!(!path.any_dynamic());
            eprintln!("{}", path);
        }
        let path3 = Path::default()
            .field("d")
            .dynamic(Slot::Register(0))
            .field("b")
            .dynamic(Slot::Register(1));
        let path3_star = path_star(&kind, &path3).unwrap();
        assert_eq!(path3_star.len(), 12);
        for path in path3_star {
            assert_eq!(path.elements.len(), 4);
            assert!(!path.any_dynamic());
            eprintln!("{}", path);
        }
    }
}
