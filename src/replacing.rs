use crate::target::{Target, TargetType, TargetsCollection};
use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

fn matching_area_replace_deps(area: TokenStream, deps: &Vec<String>) -> Vec<TokenTree> {
    let mut res = Vec::new();
    let mut is_replaceident = false;
    for dep in deps {
        for t in area.clone() {
            if is_replaceident {
                is_replaceident = false;
                res.append(
                    &mut dep
                        .parse::<TokenStream>()
                        .unwrap()
                        .into_iter()
                        .collect::<Vec<_>>(),
                );
                continue;
            }
            if let TokenTree::Punct(ref p) = t {
                if p.as_char() == '$' {
                    is_replaceident = true;
                    continue;
                }
            }
            res.push(t);
        }
    }
    res
}

fn __matching_area_replace(
    area: TokenStream,
    code_macro: &TokenStream,
    tartype: &TargetType,
    source: &String,
    deps: &Vec<String>,
) -> Vec<TokenTree> {
    let mut res = Vec::new();
    let mut is_replaceident = false;
    for t in area {
        if is_replaceident {
            is_replaceident = false;
            if let TokenTree::Group(ref g) = t {
                if g.delimiter() == Delimiter::Parenthesis {
                    res.append(&mut matching_area_replace_deps(g.stream(), deps));
                    continue;
                }
            }
            match t.to_string().as_str() {
                "code_macro" => {
                    res.append(&mut code_macro.clone().into_iter().collect::<Vec<_>>());
                }
                "tartype" => {
                    res.append(
                        &mut tartype
                            .to_string()
                            .parse::<TokenStream>()
                            .unwrap()
                            .into_iter()
                            .collect::<Vec<_>>(),
                    );
                }
                "source" => {
                    res.append(
                        &mut source
                            .parse::<TokenStream>()
                            .unwrap()
                            .into_iter()
                            .collect::<Vec<_>>(),
                    );
                }
                _ => (),
            }
            continue;
        }
        if let TokenTree::Punct(ref p) = t {
            if p.as_char() == '$' {
                is_replaceident = true;
                continue;
            }
        }
        if let TokenTree::Group(g) = t {
            let tmp = __matching_area_replace(g.stream(), code_macro, tartype, source, deps);
            let mut tt = TokenStream::new();
            tt.extend(tmp);
            res.push(TokenTree::Group(Group::new(g.delimiter(), tt)));
        } else {
            res.push(t);
        }
    }
    res
}

fn matching_area_replace(area: TokenTree, script: &Vec<TargetsCollection>) -> Vec<TokenTree> {
    let mut res = Vec::new();
    let area = if let TokenTree::Group(ref g) = area {
        g.stream()
    } else {
        panic!("Internal panic 2.");
    };
    for TargetsCollection {
        ttype: tartype,
        targets,
        code_macro,
    } in script
    {
        for Target {
            ref source,
            ref dependencies,
        } in targets
        {
            res.append(&mut __matching_area_replace(
                area.clone(),
                code_macro,
                tartype,
                source,
                dependencies,
            ));
        }
    }
    res
}

pub fn replace(target: TokenStream, script: &Vec<TargetsCollection>) -> TokenStream {
    let mut res = Vec::new();
    let mut is_replacearea = false;
    for t in target {
        if is_replacearea {
            is_replacearea = false;
            res.append(&mut matching_area_replace(t, &script));
            continue;
        }
        if let TokenTree::Punct(ref p) = t {
            if p.as_char() == '$' {
                is_replacearea = true;
                continue;
            }
        }
        if let TokenTree::Group(g) = t {
            res.push(TokenTree::Group(Group::new(
                g.delimiter(),
                replace(g.stream(), script),
            )));
        } else {
            res.push(t);
        }
    }
    let mut res_ = TokenStream::new();
    res_.extend(res);
    res_
}
