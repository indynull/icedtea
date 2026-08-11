//! Gallery page fixtures. Not part of the icedtea library.

/// Full markdown document: every construct icedtea's markdown view must show.
pub const MARKDOWN: &str = r#"# Markdown

A document control, not a one-line stub. Headings, emphasis, lists,
quotes, tables, rules, tasks, links, and fenced code.

## Headings

### Level 3
#### Level 4
##### Level 5
###### Level 6

## Inline

Paragraph with **bold**, *italic*, ***both***, `inline code`, and a
[link](https://example.com).

~~Strikethrough if the parser keeps it.~~

## Lists

- Unordered one
- Unordered two
  - Nested
- Unordered three

1. Ordered one
2. Ordered two
3. Ordered three

- [ ] Unchecked task
- [x] Checked task

## Quote

> Block quote with **emphasis** and `code`.
>
> Second paragraph in the quote.

## Table

| Name | Role | Ready |
| --- | --- | --- |
| List | collection | yes |
| Table | collection | yes |
| Tree | collection | yes |

## Image

![Logo](pixel.png)

## Rule

---

## Fenced code

```rust
fn main() {
    println!("hello from a fence");
}
```

Closing paragraph after the fence.
"#;

/// One highlighted sample for the code page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeLang {
    pub name: &'static str,
    pub syntax: &'static str,
    pub source: &'static str,
}

impl std::fmt::Display for CodeLang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

/// Languages the code page must offer.
pub const CODE_LANGS: &[CodeLang] = &[
    CodeLang {
        name: "Rust",
        syntax: "rs",
        source: "fn greet(name: &str) -> String {\n    format!(\"hello, {name}\")\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn greets() {\n        assert_eq!(greet(\"icedtea\"), \"hello, icedtea\");\n    }\n}\n",
    },
    CodeLang {
        name: "Python",
        syntax: "py",
        source: "from dataclasses import dataclass\n\n@dataclass\nclass Point:\n    x: float\n    y: float\n\n    def length(self) -> float:\n        return (self.x ** 2 + self.y ** 2) ** 0.5\n\nif __name__ == \"__main__\":\n    print(Point(3, 4).length())\n",
    },
    CodeLang {
        name: "JavaScript",
        syntax: "js",
        source: "export function greet(name) {\n  return `hello, ${name}`;\n}\n\nconst points = [{ x: 1, y: 2 }, { x: 3, y: 4 }];\nconsole.log(points.map((p) => p.x + p.y));\n",
    },
    CodeLang {
        name: "TypeScript",
        syntax: "ts",
        source: "type Point = { x: number; y: number };\n\nexport function length(p: Point): number {\n  return Math.hypot(p.x, p.y);\n}\n",
    },
    CodeLang {
        name: "Go",
        syntax: "go",
        source: "package main\n\nimport \"fmt\"\n\nfunc greet(name string) string {\n\treturn fmt.Sprintf(\"hello, %s\", name)\n}\n\nfunc main() {\n\tfmt.Println(greet(\"icedtea\"))\n}\n",
    },
    CodeLang {
        name: "C",
        syntax: "c",
        source: "#include <stdio.h>\n\nint add(int a, int b) {\n    return a + b;\n}\n\nint main(void) {\n    printf(\"%d\\n\", add(2, 40));\n    return 0;\n}\n",
    },
    CodeLang {
        name: "Java",
        syntax: "java",
        source: "public final class Main {\n    public static String greet(String name) {\n        return \"hello, \" + name;\n    }\n\n    public static void main(String[] args) {\n        System.out.println(greet(\"icedtea\"));\n    }\n}\n",
    },
    CodeLang {
        name: "JSON",
        syntax: "json",
        source: "{\n  \"name\": \"icedtea\",\n  \"version\": \"0.1.0\",\n  \"keywords\": [\"gui\", \"iced\", \"desktop\"]\n}\n",
    },
    CodeLang {
        name: "TOML",
        syntax: "toml",
        source: "[package]\nname = \"icedtea\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\niced = \"0.14\"\n",
    },
    CodeLang {
        name: "YAML",
        syntax: "yaml",
        source: "name: check\non: [push, pull_request]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: just check\n",
    },
    CodeLang {
        name: "SQL",
        syntax: "sql",
        source: "SELECT id, title, enabled\nFROM actions\nWHERE enabled = TRUE\nORDER BY title;\n",
    },
    CodeLang {
        name: "HTML",
        syntax: "html",
        source: "<!DOCTYPE html>\n<html lang=\"en\">\n  <head><title>icedtea</title></head>\n  <body>\n    <h1>Hello</h1>\n    <p>Desktop toolkit on iced.</p>\n  </body>\n</html>\n",
    },
    CodeLang {
        name: "CSS",
        syntax: "css",
        source: ":root {\n  --canvas: #0f1115;\n  --text: #e8eaed;\n}\n\nbody {\n  margin: 0;\n  background: var(--canvas);\n  color: var(--text);\n  font: 15px/1.45 system-ui, sans-serif;\n}\n",
    },
    CodeLang {
        name: "Bash",
        syntax: "bash",
        source: "#!/usr/bin/env bash\nset -euo pipefail\njust check\ncargo run -p icedtea-gallery\n",
    },
    CodeLang {
        name: "Markdown",
        syntax: "md",
        source: "# Title\n\nA **code** sample that is itself markdown.\n\n- item\n- item\n\n```rust\nfn main() {}\n```\n",
    },
];

impl CodeLang {
    pub fn named(name: &str) -> Option<&'static CodeLang> {
        CODE_LANGS.iter().find(|l| l.name == name)
    }

    pub fn names() -> Vec<String> {
        CODE_LANGS.iter().map(|l| l.name.to_string()).collect()
    }
}

/// Decoded 48×48 checker. `Handle::from_bytes` paints on a worker and
/// the first frames of a tour beat are empty.
pub fn sample_handle() -> icedtea::iced::widget::image::Handle {
    const W: u32 = 48;
    const H: u32 = 48;
    let mut px = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let on = ((x / 8) + (y / 8)) % 2 == 1;
            let i = ((y * W + x) * 4) as usize;
            if on {
                px[i] = 0x1a;
                px[i + 1] = 0x73;
                px[i + 2] = 0xe8;
            } else {
                px[i] = 0xf4;
                px[i + 1] = 0xc4;
                px[i + 2] = 0x30;
            }
            px[i + 3] = 255;
        }
    }
    icedtea::iced::widget::image::Handle::from_rgba(W, H, px)
}

/// Which markdown item kinds `MARKDOWN` must produce after parse.
#[cfg(test)]
fn markdown_kinds(source: &str) -> Vec<&'static str> {
    use icedtea::iced::widget::markdown::{self, Bullet, Item};
    let items: Vec<Item> = markdown::parse(source).collect();
    let mut out = Vec::new();
    fn walk(items: &[Item], out: &mut Vec<&'static str>) {
        for item in items {
            match item {
                Item::Heading(_, _) => out.push("heading"),
                Item::Paragraph(_) => out.push("paragraph"),
                Item::CodeBlock { .. } => out.push("code"),
                Item::List { bullets, .. } => {
                    out.push("list");
                    if bullets.iter().any(|b| matches!(b, Bullet::Task { .. })) {
                        out.push("task");
                    }
                    for b in bullets {
                        match b {
                            Bullet::Point { items } | Bullet::Task { items, .. } => {
                                walk(items, out);
                            }
                        }
                    }
                }
                Item::Image { .. } => out.push("image"),
                Item::Quote(inner) => {
                    out.push("quote");
                    walk(inner, out);
                }
                Item::Rule => out.push("rule"),
                Item::Table { .. } => out.push("table"),
            }
        }
    }
    walk(&items, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_document_covers_constructs() {
        let kinds = markdown_kinds(MARKDOWN);
        for need in [
            "heading",
            "paragraph",
            "code",
            "list",
            "task",
            "quote",
            "rule",
            "table",
            "image",
        ] {
            assert!(
                kinds.contains(&need),
                "markdown sample missing {need}: {kinds:?}"
            );
        }
        assert!(MARKDOWN.contains("# Markdown"));
        assert!(MARKDOWN.contains("- [x]"));
        assert!(MARKDOWN.contains("```rust"));
    }

    #[test]
    fn code_langs_are_unique_and_nonempty() {
        let mut names = CodeLang::names();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), CODE_LANGS.len());
        assert!(CODE_LANGS.len() >= 8);
        for lang in CODE_LANGS {
            assert!(!lang.source.trim().is_empty(), "{}", lang.name);
            assert!(CodeLang::named(lang.name).is_some());
        }
        assert_eq!(format!("{}", CODE_LANGS[0]), "Rust");
        assert!(CodeLang::named("nope").is_none());
    }
}
