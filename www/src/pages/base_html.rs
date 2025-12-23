use serde::Serialize;

#[derive(Serialize)]
pub struct HtmlParams<B> {
    base: BaseHtml,
    body: B,
}

// impl<B> HtmlParams<B> {
//     pub fn new(body: B) -> Self {
//         Self { base, body: () }
//     }
// }

#[derive(Serialize, Debug)]
pub struct BaseHtml {
    title: String,
    crumbs: Vec<BaseTool>,
    tools: Vec<BaseTool>,
}

impl BaseHtml {
    pub fn title(title: impl Into<String>) -> Self {
        let tools = vec![
            BaseTool {
                name: "tor".to_string(),
                href: "/browse/tor".to_string(),
            },
            BaseTool {
                name: "paths".to_string(),
                href: "/browse/paths".to_string(),
            },
        ];
        let mut crumbs = vec![
            BaseTool {
                name: "hello".to_string(),
                href: "/browse/tor".to_string(),
            },
            BaseTool {
                name: "world".to_string(),
                href: "/browse/paths".to_string(),
            },
            BaseTool {
                name: "crumbs".to_string(),
                href: "/browse/paths".to_string(),
            },
        ];
        Self {
            title: title.into(),
            crumbs,
            tools,
        }
    }

    pub fn build<B>(self, body: B) -> HtmlParams<B> {
        HtmlParams { base: self, body }
    }
}

#[derive(Serialize, Debug)]
pub struct BaseTool {
    pub name: String,
    pub href: String,
}
