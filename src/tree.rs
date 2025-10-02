use rustc_hash::FxHashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Tree<'a> {
    value: Option<&'a str>,
    children: FxHashMap<char, Tree<'a>>,
}

impl<'a> Tree<'a> {
    /// Returns the longest prefix from the Tree that matches the given input
    ///
    /// Returns `Some((prefix, leaf_value))` or `None` if no match was found.
    pub fn match_longest_prefix<'input>(
        &'a self,
        input: &'input str,
    ) -> Option<(&'input str, &'a str)> {
        let mut node = self;
        let mut longest_match_value = None;
        let mut current_pos = 0;

        for ch in input.chars() {
            if let Some(value) = node.value {
                longest_match_value = Some((&input[..current_pos], value));
            }

            let Some(child) = node.children.get(&ch) else {
                return longest_match_value;
            };

            node = child;
            current_pos += ch.len_utf8();
        }

        node.value
            .map(|value| (&input[..current_pos], value))
            .or(longest_match_value)
    }

    #[cfg(test)]
    pub fn new(value: Option<&'a str>, children: FxHashMap<char, Tree<'a>>) -> Tree<'a> {
        Tree { value, children }
    }

    pub fn leaf(value: &str) -> Tree<'_> {
        Tree {
            value: Some(value),
            children: FxHashMap::default(),
        }
    }
}

pub(crate) fn generate_tree<'a>(literals: &[(&'a str, &'a str)]) -> Tree<'a> {
    let mut sorted_items: Vec<_> = literals.iter().collect();
    sorted_items.sort_by_key(|(_, literal)| std::cmp::Reverse(literal.len()));

    let mut root = Tree::default();

    for (name, literal) in sorted_items {
        let mut current = &mut root;

        // iterate over the characters in the key
        let mut chars = literal.chars().peekable();
        while let Some(c) = chars.next() {
            let entry = current.children.entry(c);
            // if there is a character after the current character
            if chars.peek().is_some() {
                // move down the tree
                current = entry.or_default();
            } else {
                // else we reached the end and insert the value at the current position
                entry
                    // if the current subtree is a node, insert the value as a subtree
                    .and_modify(|inner_tree| {
                        inner_tree.value = Some(name);
                    })
                    // if the current subtree is a node, insert the value as a leaf
                    .or_insert(Tree::leaf(name));

                break; // needed to satisfy the borrow checker
            }
        }
    }

    root
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashMap;

    use super::{generate_tree, Tree};

    macro_rules! hashmap {
        { $( $key:expr => $value:expr ),* $(,)? } => {{
            FxHashMap::from_iter([$( ($key, $value), )*])
        }};
    }

    macro_rules! map {
        { $( $key:expr => $value:expr ),* $(,)? } => {{
            [$( ($value, $key), )*]
        }};
    }

    #[test]
    fn empty_tree() {
        let tree = generate_tree(&[]);
        assert_eq!(tree, Tree::default());
    }

    #[test]
    fn flat_tree() {
        let tree = generate_tree(&map! {
            "+" => "add",
            "-" => "sub",
            "<" => "left",
            ">" => "right",
            "," => "read",
            "." => "write",
            "[" => "begin_loop",
            "]" => "end_loop",
        });

        assert!(tree.value.is_none());

        assert_eq!(
            tree.children,
            hashmap! {
                '+' => Tree::leaf("add"),
                ']' => Tree::leaf("end_loop"),
                '-' => Tree::leaf("sub"),
                '[' => Tree::leaf("begin_loop"),
                ',' => Tree::leaf("read"),
                '.' => Tree::leaf("write"),
                '>' => Tree::leaf("right"),
                '<' => Tree::leaf("left"),
            }
        );
    }

    #[test]
    fn basic_nested_tree() {
        let tree = generate_tree(&map! {
            "ABC" => "abc",
            "ACB" => "acb",
            "BAC" => "bac",
            "BCA" => "bca",
            "CAB" => "cab",
            "CBA" => "cba",
        });

        assert_eq!(
            tree,
            Tree::new(
                None,
                hashmap! {
                    'A' => Tree::new(None, hashmap! {
                        'B' => Tree::new(None, hashmap! { 'C' => Tree::leaf("abc") }),
                        'C' => Tree::new(None, hashmap! { 'B' => Tree::leaf("acb") })
                    }),
                    'B' => Tree::new(None, hashmap! {
                        'A' => Tree::new(None, hashmap! { 'C' => Tree::leaf("bac") }),
                        'C' => Tree::new(None, hashmap! { 'A' => Tree::leaf("bca") })
                    }),
                    'C' => Tree::new(None, hashmap! {
                        'A' => Tree::new(None, hashmap! { 'B' => Tree::leaf("cab") }),
                        'B' => Tree::new(None, hashmap! { 'A' => Tree::leaf("cba") })
                    }),
                }
            )
        );
    }

    #[test]
    fn break_path_nested_tree() {
        let tree = generate_tree(&map! {
            "ABC" => "x",
            "A" => "y",
            "B" => "z",
        });

        assert_eq!(
            tree,
            Tree::new(
                None,
                hashmap! {
                    'A' => Tree::new(Some("y"), hashmap! {
                        'B' => Tree::new(None, hashmap! {
                            'C' => Tree::leaf("x")
                        })
                    }),
                    'B' => Tree::leaf("z")
                }
            )
        );
    }

    #[test]
    fn same_symbol_tree() {
        let tree = generate_tree(&map! {
            "+" => "a",
            "++" => "b",
            "+++" => "c",
            "++++" => "d",
            "+++++" => "e",
            "++++++" => "f",
        });

        let expected_tree = Tree::new(
            None,
            hashmap! {
                '+' => Tree::new(Some("a"), hashmap! {
                    '+' => Tree::new(Some("b"), hashmap! {
                        '+' => Tree::new(Some("c"), hashmap! {
                            '+' => Tree::new(Some("d"), hashmap! {
                                '+' => Tree::new(Some("e"), hashmap! {
                                    '+' => Tree::leaf("f")
                                })
                            })
                        })
                    })
                })
            },
        );

        assert_eq!(tree, expected_tree);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn samarium_tree() {
        let tree = generate_tree(&map! {
            "+" => "ad",
            "&&" => "an",
            "@@@" => "ar",
            ":" => "as",
            "." => "at",
            "&" => "ba",
            "~" => "bn",
            "|" => "bo",
            "}" => "brace_c",
            "{" => "brace_o",
            "]" => "brack_c",
            "[" => "brack_o",
            "^" => "bx",
            "%" => "cas",
            "!!" => "cat",
            "@" => "cl",
            "@!" => "da",
            "<>" => "de",
            "--" => "di",
            ",," => "e",
            ";" => "end",
            "=>" => "ent",
            "#" => "enu",
            "::" => "eq",
            "=>!" => "ex",
            "&~~>" => "fi_a",
            "&%~>" => "fi_b_a",
            "<~%" => "fi_b_r",
            "<%>" => "fi_b_r_w",
            "%~>" => "fi_b_w",
            "?~>" => "fi_c",
            "&~>" => "fi_q_a",
            "&%>" => "fi_q_b_a",
            "<%" => "fi_q_b_r",
            "%>" => "fi_q_b_w",
            "<~" => "fi_q_r",
            "~>" => "fi_q_w",
            "<~>" => "fi_r_w",
            "<~~" => "fi_r",
            "~~>" => "fi_w",
            "..." => "fo",
            "<-" => "fr",
            "*" => "fu",
            ">:" => "ge",
            ">" => "gt",
            "##" => "h",
            "?" => "if",
            "<=" => "im",
            "->?" => "in",
            "'" => "ins",
            "<:" => "le",
            "<" => "lt",
            "---" => "mo",
            "++" => "mu",
            ":::" => "ne",
            "~~" => "no",
            "||" => "o",
            ")" => "p_c",
            "(" => "p_o",
            "!?" => "pa",
            "+++" => "po",
            "!" => "pr",
            "???" => "r",
            "," => "se",
            ",.," => "sle",
            ">>" => "s_c",
            "<<" => "s_o",
            "$" => "sp",
            "-" => "su",
            "}}" => "t_c",
            "{{" => "t_o",
            "!!!" => "th",
            "->" => "to",
            "??" => "tr",
            "?!" => "ty",
            "@@" => "u",
            ".." => "w",
            "^^" => "x",
            "**" => "y",
            "><" => "z",
        });

        let expected_tree = Tree::new(
            None,
            hashmap! {
                '%' => Tree::new(Some("cas"), hashmap! {
                    '>' => Tree::leaf("fi_q_b_w"),
                    '~' => Tree::new(None, hashmap! { '>' => Tree::leaf("fi_b_w") })
                }),
                '&' => Tree::new(Some("ba"), hashmap! {
                    '&' => Tree::leaf("an"),
                    '~' => Tree::new(None, hashmap! {
                        '~' => Tree::new(None, hashmap! { '>' => Tree::leaf("fi_a") }),
                        '>' => Tree::leaf("fi_q_a")
                    }),
                    '%' => Tree::new(None, hashmap! {
                        '>' => Tree::leaf("fi_q_b_a"),
                        '~' => Tree::new(None, hashmap! { '>' => Tree::leaf("fi_b_a") })
                    })
                }),
                '-' => Tree::new(Some("su"), hashmap! {
                    '>' => Tree::new(Some("to"), hashmap! {
                        '?' => Tree::leaf("in")
                    }),
                    '-' => Tree::new(Some("di"), hashmap! {
                        '-' => Tree::leaf("mo")
                    })
                }),
                '>' => Tree::new(Some("gt"), hashmap! {
                    ':' => Tree::leaf("ge"),
                    '>' => Tree::leaf("s_c"),
                    '<' => Tree::leaf("z")
                }),
                '^' => Tree::new(Some("bx"), hashmap! {
                    '^' => Tree::leaf("x")
                }),
                '$' => Tree::leaf("sp"),
                '!' => Tree::new(Some("pr"), hashmap! {
                    '?' => Tree::leaf("pa"),
                    '!' => Tree::new(Some("cat"), hashmap! {
                        '!' => Tree::leaf("th")
                    })
                }),
                '|' => Tree::new(Some("bo"), hashmap! {
                    '|' => Tree::leaf("o")
                }),
                '*' => Tree::new(Some("fu"), hashmap! {
                    '*' => Tree::leaf("y")
                }),
                '(' => Tree::leaf("p_o"),
                '<' => Tree::new(Some("lt"), hashmap! {
                    '-' => Tree::leaf("fr"),
                    '<' => Tree::leaf("s_o"),
                    ':' => Tree::leaf("le"),
                    '>' => Tree::leaf("de"),
                    '=' => Tree::leaf("im"),
                    '~' => Tree::new(Some("fi_q_r"), hashmap! {
                        '>' => Tree::leaf("fi_r_w"),
                        '%' => Tree::leaf("fi_b_r"),
                        '~' => Tree::leaf("fi_r")
                    }),
                    '%' => Tree::new(Some("fi_q_b_r"), hashmap! {
                        '>' => Tree::leaf("fi_b_r_w")
                    })
                }),
                '@' => Tree::new(Some("cl"), hashmap! {
                    '!' => Tree::leaf("da"),
                    '@' => Tree::new(Some("u"), hashmap! {
                        '@' => Tree::leaf("ar")
                    })
                }),
                '=' => Tree::new(None, hashmap! {
                    '>' => Tree::new(Some("ent"), hashmap! {
                        '!' => Tree::leaf("ex")
                    })
                }),
                '?' => Tree::new(Some("if"), hashmap! {
                    '!' => Tree::leaf("ty"),
                    '~' => Tree::new(None, hashmap! { '>' => Tree::leaf("fi_c") }),
                    '?' => Tree::new(Some("tr"), hashmap! {
                        '?' => Tree::leaf("r")
                    })
                }),
                '~' => Tree::new(Some("bn"), hashmap! {
                    '>' => Tree::leaf("fi_q_w"),
                    '~' => Tree::new(Some("no"), hashmap! {
                        '>' => Tree::leaf("fi_w")
                    })
                }),
                '{' => Tree::new(Some("brace_o"), hashmap! {
                    '{' => Tree::leaf("t_o")
                }),
                '}' => Tree::new(Some("brace_c"), hashmap! {
                    '}' => Tree::leaf("t_c")
                }),
                '+' => Tree::new(Some("ad"), hashmap! {
                    '+' => Tree::new(Some("mu"), hashmap! {
                        '+' => Tree::leaf("po")
                    })
                }),
                ')' => Tree::leaf("p_c"),
                ',' => Tree::new(Some("se"), hashmap! {
                    ',' => Tree::leaf("e"),
                    '.' => Tree::new(None, hashmap! { ',' => Tree::leaf("sle") })
                }),
                '\'' => Tree::leaf("ins"),
                ':' => Tree::new(Some("as"), hashmap! {
                    ':' => Tree::new(Some("eq"), hashmap! {
                        ':' => Tree::leaf("ne")
                    })
                }),
                '#' => Tree::new(Some("enu"), hashmap! {
                    '#' => Tree::leaf("h")
                }),
                '[' => Tree::leaf("brack_o"),
                ';' => Tree::leaf("end"),
                ']' => Tree::leaf("brack_c"),
                '.' => Tree::new(Some("at"), hashmap! {
                    '.' => Tree::new(Some("w"), hashmap! {
                        '.' => Tree::leaf("fo")
                    })
                })
            },
        );

        assert_eq!(tree, expected_tree);
    }
}
