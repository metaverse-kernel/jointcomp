use proc_macro::{Delimiter, TokenStream, TokenTree};

#[derive(Debug)]
pub enum TargetType {
    None,
    GccAsm,
    GccAsmX86,
    LinkerScript,
    LinkerMap,
}

impl From<String> for TargetType {
    fn from(value: String) -> Self {
        let value = value.as_str();
        match value {
            "GccAsm" => Self::GccAsm,
            "GccAsmX86" => Self::GccAsmX86,
            "LinkerScript" => Self::LinkerScript,
            "LinkerMap" => Self::LinkerMap,
            _ => Self::None,
        }
    }
}

impl ToString for TargetType {
    fn to_string(&self) -> String {
        format!("TargetType::{:?}", self)
    }
}

#[derive(Debug)]
pub struct Target {
    pub source: String,
    pub dependencies: Vec<String>,
}

impl Target {
    fn from_tokens(input: TokenStream) -> Option<Self> {
        if input.is_empty() {
            return None;
        }
        let mut dependencies = Vec::new();
        let mut iter = input.into_iter();
        let source = iter.next().unwrap();
        let source = {
            let mut t = TokenStream::new();
            t.extend([source]);
            t
        };
        let source = format!("{}", source);
        let split = iter.next();
        let _ = if let Some(s) = split {
            if let TokenTree::Punct(p) = s {
                if p.as_char() == ':' {
                    p
                } else {
                    panic!("Must be a : after a target.");
                }
            } else {
                panic!("Must be a : after a target.");
            }
        } else {
            return Some(Self {
                source,
                dependencies,
            });
        };
        let mut tmp = Vec::new();
        while let Some(t) = iter.next() {
            if let TokenTree::Punct(ref p) = t {
                if p.as_char() == ',' {
                    let mut t = TokenStream::new();
                    t.extend(tmp.clone());
                    dependencies.push(format!("{}", t));
                    tmp.clear();
                    continue;
                }
            }
            tmp.push(t);
        }
        if !tmp.is_empty() {
            let mut t = TokenStream::new();
            t.extend(tmp);
            dependencies.push(format!("{}", t));
        }
        Some(Self {
            source,
            dependencies,
        })
    }
}

pub fn targets_vec_from(input: TokenStream) -> Vec<Target> {
    let mut res = Vec::new();
    let mut tmp = Vec::new();
    for t in input {
        if let TokenTree::Punct(ref p) = t {
            if p.as_char() == ',' {
                let mut t = TokenStream::new();
                t.extend(tmp.clone());
                if let Some(tar) = Target::from_tokens(t) {
                    res.push(tar);
                }
                tmp.clear();
                continue;
            }
        }
        tmp.push(t);
    }
    if !tmp.is_empty() {
        let mut t = TokenStream::new();
        t.extend(tmp);
        if let Some(tar) = Target::from_tokens(t) {
            res.push(tar);
        }
    }
    res
}

#[derive(Debug)]
pub struct TargetsCollection {
    pub ttype: TargetType,
    pub targets: Vec<Target>,
    pub code_macro: TokenStream,
}

impl TargetsCollection {
    fn from_tokens(value: TokenStream) -> Option<Self> {
        if value.is_empty() {
            return None;
        }
        let mut code_macro = TokenStream::new();
        let mut iter = value.clone().into_iter();
        let mut token = iter.next();
        if let Some(TokenTree::Punct(p)) = token.clone() {
            if p.as_char() == '#' {
                code_macro.extend([token.unwrap().clone()]);
                if let Some(TokenTree::Group(g)) = iter.next() {
                    if g.delimiter() == Delimiter::Bracket {
                        code_macro.extend([TokenTree::Group(g)]);
                        token = iter.next();
                    } else {
                        panic!("After # must be a []");
                    }
                } else {
                    panic!("After # must be a []");
                }
            }
        }
        if let None = token {
            panic!("Section unexpectedly terminated.");
        }
        let token = token.unwrap();
        let ttype = token;
        let ttype_ = {
            let mut t = TokenStream::new();
            t.extend([ttype]);
            t
        };
        let ttype = TargetType::from(format!("{}", ttype_));
        if let TargetType::None = ttype {
            panic!("Unknown TargetType : {}", ttype_);
        }
        let targets = if let Some(TokenTree::Group(g)) = iter.next() {
            if g.delimiter() == Delimiter::Brace {
                g.stream()
            } else {
                panic!("Brace needed after TargetType {:?}", ttype);
            }
        } else {
            panic!("Brace needed after TargetType {:?}", ttype);
        };
        let targets = targets_vec_from(targets);
        Some(Self {
            ttype,
            targets,
            code_macro,
        })
    }
}

pub fn from_input(input: TokenStream) -> Vec<TargetsCollection> {
    let mut res = Vec::new();
    let mut tmp = Vec::new();
    for tk in input {
        if let TokenTree::Punct(ref p) = tk {
            if p.as_char() == ';' {
                let mut t = TokenStream::new();
                t.extend(tmp.clone());
                if let Some(tc) = TargetsCollection::from_tokens(t) {
                    res.push(tc);
                }
                tmp.clear();
                continue;
            }
        }
        tmp.push(tk);
    }
    res
}
