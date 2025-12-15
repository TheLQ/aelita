use std::sync::LazyLock;

struct TemplateConst {
    inner: LazyLock<String>,
}
