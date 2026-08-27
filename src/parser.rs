use comrak::nodes::NodeValue;
use comrak::options::Options;
use comrak::{Arena, Node, parse_document};

const FRONTMATTER_DELIMITER: &str = "---";

pub struct Parser {
    pub options: Options<'static>,
}

impl Parser {
    pub fn new() -> Self {
        let mut options = Options::default();
        options.extension.alerts = true;
        options.extension.autolink = true;
        options.extension.footnotes = true;
        options.extension.front_matter_delimiter = Some(FRONTMATTER_DELIMITER.to_owned());
        options.extension.strikethrough = true;
        options.extension.table = true;
        options.extension.tasklist = true;
        options.render.github_pre_lang = true;
        options.render.r#unsafe = true;
        options.render.tasklist_classes = true;

        Self { options }
    }

    pub fn parse<'a>(&self, arena: &'a Arena<'a>, markdown: &str) -> Node<'a> {
        parse_document(arena, markdown, &self.options)
    }

    pub fn frontmatter(&self, markdown: &str) -> Option<String> {
        let arena = Arena::new();
        let root = self.parse(&arena, markdown);
        root.first_child().and_then(|node| {
            let data = node.data();
            let NodeValue::FrontMatter(source) = &data.value else {
                return None;
            };

            Some(
                source
                    .trim()
                    .strip_prefix(FRONTMATTER_DELIMITER)
                    .unwrap()
                    .strip_suffix(FRONTMATTER_DELIMITER)
                    .unwrap()
                    .trim()
                    .to_owned(),
            )
        })
    }

    pub fn title_heading<'a>(root: Node<'a>) -> Option<Node<'a>> {
        root.children().find(|node| {
            matches!(
                &node.data().value,
                NodeValue::Heading(heading) if heading.level == 1
            )
        })
    }

    pub fn description_nodes<'a>(title: Node<'a>) -> Vec<Node<'a>> {
        let mut nodes = Vec::new();
        let mut node = title.next_sibling();

        while let Some(current) = node {
            if matches!(&current.data().value, NodeValue::Heading(_)) {
                break;
            }

            nodes.push(current);
            node = current.next_sibling();
        }

        nodes
    }

    pub fn plain_text<'a>(nodes: impl IntoIterator<Item = Node<'a>>) -> String {
        let mut text = String::new();
        for root in nodes {
            for node in root.descendants() {
                match &node.data().value {
                    NodeValue::Text(value) => text.push_str(value),
                    NodeValue::Code(code) => text.push_str(&code.literal),
                    NodeValue::CodeBlock(code)
                        if matches!(
                            code.info.split_whitespace().next(),
                            Some("friends" | "devices")
                        ) => {}
                    NodeValue::CodeBlock(code) => {
                        text.push(' ');
                        text.push_str(&code.literal);
                        text.push(' ');
                    }
                    NodeValue::Math(math) => text.push_str(&math.literal),
                    NodeValue::Paragraph
                    | NodeValue::Heading(_)
                    | NodeValue::LineBreak
                    | NodeValue::SoftBreak => text.push(' '),
                    _ => {}
                }
            }
        }

        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}
