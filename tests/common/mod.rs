pub fn extract_links(body: &str) -> Vec<String> {
    linkify::LinkFinder::new()
        .links(body)
        .filter_map(|link| match link.kind() {
            linkify::LinkKind::Url => Some(link.as_str().to_string()),
            _ => None,
        })
        .collect()
}
