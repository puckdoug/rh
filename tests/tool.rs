use rh::tool::Tool;

#[test]
fn tool_exposes_its_fields() {
    let tool = Tool {
        name: String::from("search"),
        description: String::from("Search files"),
        parameters: vec![String::from("query")],
        command: String::from("grep"),
    };

    assert_eq!(tool.name, "search");
    assert_eq!(tool.description, "Search files");
    assert_eq!(tool.parameters, vec!["query"]);
    assert_eq!(tool.command, "grep");
}
