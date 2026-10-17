//! Example tests for the native `/mcp` endpoint.

mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use common::*;
use kanso_server::basic_header;
use serde_json::{Value, json};

fn mcp_request(body: &str, extra: &[(&str, &str)]) -> Request<Body> {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri("/mcp")
        .header(header::AUTHORIZATION, basic_header(USER, PASS))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/json, text/event-stream");
    for (name, value) in extra {
        builder = builder.header(*name, *value);
    }
    builder.body(Body::from(body.to_owned())).unwrap()
}

#[tokio::test]
async fn initialize_negotiates_the_protocol_version() {
    let app = app();
    for (requested, expected) in [
        ("2025-06-18", "2025-06-18"),
        ("2024-11-05", "2024-11-05"),
        ("1999-01-01", "2025-11-25"),
    ] {
        let answer = rpc(
            &app.router,
            "initialize",
            json!({ "protocolVersion": requested, "capabilities": {}, "clientInfo": { "name": "t", "version": "0" } }),
        )
        .await;
        assert_eq!(answer["jsonrpc"], "2.0");
        assert_eq!(answer["id"], 1);
        assert_eq!(answer["result"]["protocolVersion"], expected);
        assert_eq!(answer["result"]["serverInfo"]["name"], "Kanso");
        assert!(answer["result"]["capabilities"]["tools"].is_object());
        assert!(answer["result"]["capabilities"]["resources"].is_object());
    }
}

#[tokio::test]
async fn lists_the_seventeen_tools_and_the_note_template() {
    let app = app();
    let answer = rpc(&app.router, "tools/list", json!({})).await;
    let tools = answer["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 17);
    for tool in tools {
        assert!(tool["description"].as_str().is_some_and(|d| !d.is_empty()));
        assert_eq!(tool["inputSchema"]["type"], "object");
    }
    let answer = rpc(&app.router, "resources/templates/list", json!({})).await;
    assert_eq!(
        answer["result"],
        json!({ "resourceTemplates": [{
            "name": "note", "uriTemplate": "inkdrop://note/{noteId}",
            "description": "A note data", "mimeType": "application/json"
        }] })
    );
    let answer = rpc(&app.router, "resources/list", json!({})).await;
    assert_eq!(answer["result"], json!({ "resources": [] }));
    assert_eq!(
        rpc(&app.router, "ping", json!({})).await["result"],
        json!({})
    );
}

#[tokio::test]
async fn transport_edge_cases() {
    let app = app();
    // Notifications and client responses get 202 with no body.
    for body in [
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":7,"result":{}}"#,
        r#"[{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1}}]"#,
    ] {
        let response = send(&app.router, mcp_request(body, &[])).await;
        assert_eq!(response.status, StatusCode::ACCEPTED, "{body}");
        assert!(response.bytes.is_empty());
    }
    // Unknown methods, bad JSON, batches, invalid requests.
    let response = send(
        &app.router,
        mcp_request(r#"{"jsonrpc":"2.0","id":"a","method":"prompts/list"}"#, &[]),
    )
    .await;
    assert_eq!(
        response.json(),
        json!({ "jsonrpc": "2.0", "id": "a", "error": { "code": -32601, "message": "Method not found" } })
    );
    let response = send(&app.router, mcp_request("{nope", &[])).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.json()["error"]["code"], -32700);
    let response = send(
        &app.router,
        mcp_request(
            r#"[{"jsonrpc":"2.0","id":1,"method":"ping"},{"jsonrpc":"2.0","method":"notifications/initialized"},{"jsonrpc":"2.0","id":2,"method":"ping"}]"#,
            &[],
        ),
    )
    .await;
    assert_eq!(
        response.json(),
        json!([{ "jsonrpc": "2.0", "id": 1, "result": {} }, { "jsonrpc": "2.0", "id": 2, "result": {} }])
    );
    for body in ["[]", "3", r#"{"id":1,"method":"ping"}"#] {
        let response = send(&app.router, mcp_request(body, &[])).await;
        assert_eq!(response.json()["error"]["code"], -32600, "{body}");
    }
    let response = send(
        &app.router,
        mcp_request(
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{}}"#,
            &[],
        ),
    )
    .await;
    assert_eq!(response.json()["error"]["code"], -32602);

    // Headers: protocol version and origin.
    let ping = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
    let response = send(
        &app.router,
        mcp_request(ping, &[("mcp-protocol-version", "2025-06-18")]),
    )
    .await;
    assert_eq!(response.status, StatusCode::OK);
    let response = send(
        &app.router,
        mcp_request(ping, &[("mcp-protocol-version", "1.0")]),
    )
    .await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    let response = send(
        &app.router,
        mcp_request(ping, &[("origin", "http://localhost:6274")]),
    )
    .await;
    assert_eq!(response.status, StatusCode::OK);
    let response = send(
        &app.router,
        mcp_request(ping, &[("origin", "https://evil.example")]),
    )
    .await;
    assert_eq!(response.status, StatusCode::FORBIDDEN);

    // No event stream, no sessions.
    for method in [Method::GET, Method::DELETE] {
        let response = send(&app.router, request(method, "/mcp", None)).await;
        assert_eq!(response.status, StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(response.headers[header::ALLOW], "POST");
    }
}

#[tokio::test]
async fn tool_errors_read_like_the_node_server() {
    let app = app();
    let result = tool(&app.router, "nope", json!({})).await;
    assert_eq!(
        result,
        json!({ "content": [{ "type": "text", "text": "MCP error -32602: Tool nope not found" }], "isError": true })
    );
    let result = tool(
        &app.router,
        "create-note",
        json!({ "bookId": "x", "title": "a", "body": "b" }),
    )
    .await;
    assert_eq!(result["isError"], true);
    assert_eq!(
        text(&result, 0),
        "MCP error -32602: Input validation error: Invalid arguments for tool create-note: \
         Too small: expected string to have >=5 characters at bookId\n\
         Invalid string: must match pattern /^(book:|trash$)/ at bookId"
    );
    let result = tool(
        &app.router,
        "read-note",
        json!({ "noteId": "note:missing" }),
    )
    .await;
    assert_eq!(
        result,
        json!({ "content": [{ "type": "text", "text": "Fetch error [404] Not Found" }], "isError": true })
    );
    let result = tool(
        &app.router,
        "create-note",
        json!({ "bookId": "book:missing", "title": "a", "body": "b" }),
    )
    .await;
    assert_eq!(text(&result, 0), "Fetch error [400] Bad Request");
}

#[tokio::test]
async fn notes_round_trip_through_the_tools() {
    let app = app();
    let book_id =
        tool_json(&tool(&app.router, "create-book", json!({ "name": "Work" })).await)["id"]
            .as_str()
            .unwrap()
            .to_owned();
    let created = tool_json(&tool(&app.router, "create-tag", json!({ "name": "urgent" })).await);
    let tag_id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["ok"], true);
    let (_, tag_doc) = get(&app.router, &format!("/{tag_id}")).await;
    assert_eq!(tag_doc["color"], "default");

    let long_body = format!("{}😀tail", "x".repeat(199));
    let note = tool_json(
        &tool(
            &app.router,
            "create-note",
            json!({ "bookId": book_id, "title": "Plan", "body": long_body, "tags": [tag_id], "status": "active" }),
        )
        .await,
    );
    let note_id = note["id"].as_str().unwrap().to_owned();

    // read-note accepts the ID with or without its prefix.
    let read = tool(&app.router, "read-note", json!({ "noteId": note_id })).await;
    let (_, http_doc) = get(&app.router, &format!("/{note_id}")).await;
    assert_eq!(
        text(&read, 0),
        serde_json::to_string_pretty(&http_doc).unwrap()
    );
    assert_eq!(read["content"].as_array().unwrap().len(), 1);
    let bare = note_id.strip_prefix("note:").unwrap();
    let read_bare = tool(&app.router, "read-note", json!({ "noteId": bare })).await;
    assert_eq!(read, read_bare);

    // Summaries cut bodies at 200 UTF-16 units; the emoji straddles the cut.
    let found = tool_json(&tool(&app.router, "search-notes", json!({ "keyword": "tail" })).await);
    assert_eq!(found[0]["_id"], note_id);
    assert_eq!(found[0]["body"], "x".repeat(199));
    let listed = tool_json(
        &tool(
            &app.router,
            "list-notes",
            json!({ "bookId": book_id, "tagIds": [tag_id] }),
        )
        .await,
    );
    assert_eq!(listed.as_array().unwrap().len(), 1);
    let none = tool_json(
        &tool(
            &app.router,
            "list-notes",
            json!({ "keyword": "absent-word" }),
        )
        .await,
    );
    assert_eq!(none, json!([]));

    // update-note merges given fields into the current note.
    let rev = http_doc["_rev"].as_str().unwrap();
    let updated = tool_json(
        &tool(
            &app.router,
            "update-note",
            json!({ "_id": note_id, "_rev": rev, "title": "Plan v2" }),
        )
        .await,
    );
    let (_, doc) = get(&app.router, &format!("/{note_id}")).await;
    assert_eq!(doc["title"], "Plan v2");
    assert_eq!(doc["status"], "active");
    assert_eq!(doc["_rev"], updated["rev"]);
    // A stale rev fails like the Node server: the GET with ?rev= is a 404.
    let stale = tool(
        &app.router,
        "update-note",
        json!({ "_id": note_id, "_rev": rev, "title": "x" }),
    )
    .await;
    assert_eq!(text(&stale, 0), "Fetch error [404] Not Found");
    assert_eq!(stale["isError"], true);
}

#[tokio::test]
async fn patch_note_needs_exactly_one_occurrence() {
    let app = app();
    let (note_id, rev) = write(
        &app.router,
        "/notes",
        json!({ "bookId": "trash", "title": "t", "body": "one two two three" }),
    )
    .await;
    let patch = |old: &str, rev: &str| json!({ "_id": note_id, "_rev": rev, "old_string": old, "new_string": "2" });
    let missing = tool(&app.router, "patch-note", patch("four", &rev)).await;
    assert_eq!(
        missing,
        json!({ "content": [{ "type": "text", "text": "Failed to patch: `old_string` was not found in the note body." }], "isError": true })
    );
    let many = tool(&app.router, "patch-note", patch("two", &rev)).await;
    assert_eq!(
        text(&many, 0),
        "Failed to patch: `old_string` matches multiple locations in the note body. Provide more surrounding context to make it unique."
    );
    let ok = tool_json(&tool(&app.router, "patch-note", patch("one", &rev)).await);
    let (_, doc) = get(&app.router, &format!("/{note_id}")).await;
    assert_eq!(doc["body"], "2 two two three");
    assert_eq!(doc["_rev"], ok["rev"]);
}

#[tokio::test]
async fn backlinks_and_templates() {
    let app = app();
    let (target, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": "trash", "title": "Target", "body": "" }),
    )
    .await;
    let bare = target.strip_prefix("note:").unwrap().to_owned();
    let result = tool(&app.router, "read-backlinks", json!({ "noteId": target })).await;
    assert_eq!(
        text(&result, 0),
        format!("No backlinks found for inkdrop://note/{bare}.")
    );

    let (book_id, _) = write(&app.router, "/books", json!({ "name": "B" })).await;
    let (linker, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "", "body": format!("see [t](inkdrop://note/{bare})") }),
    )
    .await;
    let result = tool(&app.router, "read-backlinks", json!({ "noteId": bare })).await;
    assert_eq!(
        result,
        json!({ "content": [{
            "type": "resource_link",
            "uri": format!("inkdrop://note/{}", linker.strip_prefix("note:").unwrap()),
            "name": "Untitled note",
            "mimeType": "application/json",
        }] })
    );

    // A note created from a template links to it, and embeds it on request.
    let (template, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "Weekly", "body": "Fill in the goals." }),
    )
    .await;
    let (from_template, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "Week 1", "body": "", "sourceTemplateId": template }),
    )
    .await;
    let result = tool(&app.router, "read-note", json!({ "noteId": from_template })).await;
    let blocks = result["content"].as_array().unwrap();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[1]["type"], "resource_link");
    assert_eq!(
        blocks[1]["uri"],
        format!("inkdrop://note/{}", template.strip_prefix("note:").unwrap())
    );
    assert_eq!(blocks[1]["name"], "Source template");
    let result = tool(
        &app.router,
        "read-note",
        json!({ "noteId": from_template, "includeTemplate": true }),
    )
    .await;
    assert_eq!(
        text(&result, 2),
        "# Source template: Weekly\n\nFill in the goals."
    );

    // The note resource.
    let answer = rpc(
        &app.router,
        "resources/read",
        json!({ "uri": format!("inkdrop://note/{bare}") }),
    )
    .await;
    let contents = &answer["result"]["contents"][0];
    assert_eq!(contents["uri"], format!("inkdrop://note/{bare}"));
    assert_eq!(contents["mimeType"], "application/json");
    let doc: Value = serde_json::from_str(contents["text"].as_str().unwrap()).unwrap();
    assert_eq!(doc["_id"], target);
    let answer = rpc(
        &app.router,
        "resources/read",
        json!({ "uri": "inkdrop://book/abc" }),
    )
    .await;
    assert_eq!(answer["error"]["code"], -32602);
    assert_eq!(
        answer["error"]["message"],
        "MCP error -32602: Resource inkdrop://book/abc not found"
    );
    let answer = rpc(
        &app.router,
        "resources/read",
        json!({ "uri": "inkdrop://note/missing" }),
    )
    .await;
    assert_eq!(
        answer["error"],
        json!({ "code": -32603, "message": "Fetch error [404] Not Found" })
    );
}

#[tokio::test]
async fn tags_and_notebooks_through_the_tools() {
    let app = app();
    let parent =
        tool_json(&tool(&app.router, "create-book", json!({ "name": "Parent" })).await)["id"]
            .as_str()
            .unwrap()
            .to_owned();
    let child = tool_json(
        &tool(
            &app.router,
            "create-book",
            json!({ "name": "Child", "parentBookId": parent }),
        )
        .await,
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let books = tool_json(&tool(&app.router, "list-notebooks", json!({})).await);
    let (_, http_books) = get(&app.router, "/books").await;
    assert_eq!(books, http_books);
    let book = tool_json(
        &tool(
            &app.router,
            "read-book",
            json!({ "bookId": child.strip_prefix("book:").unwrap() }),
        )
        .await,
    );
    assert_eq!(book["parentBookId"], parent);
    tool_json(
        &tool(
            &app.router,
            "update-book",
            json!({ "_id": child, "parentBookId": null, "name": "Top" }),
        )
        .await,
    );
    let (_, doc) = get(&app.router, &format!("/{child}")).await;
    assert_eq!(
        (doc["name"].as_str(), &doc["parentBookId"]),
        (Some("Top"), &Value::Null)
    );

    let tag_id = tool_json(
        &tool(
            &app.router,
            "create-tag",
            json!({ "name": "x", "color": "teal" }),
        )
        .await,
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, tag_doc) = get(&app.router, &format!("/{tag_id}")).await;
    let stale_rev = tag_doc["_rev"].as_str().unwrap().to_owned();
    tool_json(
        &tool(
            &app.router,
            "update-tag",
            json!({ "_id": tag_id, "color": "pink" }),
        )
        .await,
    );
    let tag = tool_json(&tool(&app.router, "read-tag", json!({ "tagId": tag_id })).await);
    assert_eq!(
        (tag["name"].as_str(), tag["color"].as_str()),
        (Some("x"), Some("pink"))
    );
    let conflict = tool(
        &app.router,
        "update-tag",
        json!({ "_id": tag_id, "_rev": stale_rev, "name": "y" }),
    )
    .await;
    assert_eq!(text(&conflict, 0), "Fetch error [409] Conflict");
    let tags = tool_json(&tool(&app.router, "list-tags", json!({})).await);
    assert_eq!(tags.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn files_through_the_tools() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let png_path = dir.path().join("pixel.png");
    std::fs::write(&png_path, PNG).unwrap();

    let neither = tool(&app.router, "create-file", json!({})).await;
    assert_eq!(
        text(&neither, 0),
        "Failed to create the attachment: provide exactly one of `filePath` or `data`."
    );
    let missing = tool(
        &app.router,
        "create-file",
        json!({ "filePath": "/no/such/file.png" }),
    )
    .await;
    assert!(text(&missing, 0).starts_with("Failed to read the file at /no/such/file.png: "));
    let untyped = tool(
        &app.router,
        "create-file",
        json!({ "data": STANDARD.encode(PNG) }),
    )
    .await;
    assert!(text(&untyped, 0).starts_with(
        "Failed to create the attachment: could not infer the content type. Pass `contentType` explicitly."
    ));

    let created = tool(&app.router, "create-file", json!({ "filePath": png_path })).await;
    let res = tool_json(&created);
    let file_id = res["id"].as_str().unwrap().to_owned();
    assert_eq!(
        text(&created, 1),
        format!(
            "Markdown to embed this attachment in a note body:\n\n![pixel.png](inkdrop://{file_id})"
        )
    );
    let (_, doc) = get(&app.router, &format!("/{file_id}?attachments=true")).await;
    assert_eq!(doc["name"], "pixel.png");
    assert_eq!(doc["contentType"], "image/png");
    assert_eq!(doc["_attachments"]["index"]["data"], STANDARD.encode(PNG));

    let read = tool(&app.router, "read-file", json!({ "fileId": file_id })).await;
    let metadata: Value = serde_json::from_str(text(&read, 0)).unwrap();
    assert!(metadata.get("_attachments").is_none());
    assert_eq!(metadata["_id"], file_id);
    assert_eq!(
        read["content"][1],
        json!({ "type": "image", "data": STANDARD.encode(PNG), "mimeType": "image/png" })
    );
    let meta_only = tool(
        &app.router,
        "read-file",
        json!({ "fileId": file_id, "includeImage": false }),
    )
    .await;
    assert_eq!(meta_only["content"].as_array().unwrap().len(), 1);
    let out = dir.path().join("out.png");
    let written = tool(
        &app.router,
        "read-file",
        json!({ "fileId": file_id, "outputPath": out }),
    )
    .await;
    assert_eq!(
        text(&written, 1),
        format!("Attachment written to {}", out.display())
    );
    assert_eq!(std::fs::read(&out).unwrap(), PNG);

    // Inline data; SVG cannot be shown inline.
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\"/>";
    let created = tool(
        &app.router,
        "create-file",
        json!({ "data": STANDARD.encode(svg), "contentType": "image/svg+xml" }),
    )
    .await;
    let svg_id = tool_json(&created)["id"].as_str().unwrap().to_owned();
    assert!(text(&created, 1).contains("![image.svg+xml](inkdrop://"));
    let read = tool(&app.router, "read-file", json!({ "fileId": svg_id })).await;
    assert_eq!(
        text(&read, 1),
        "The attachment is a image/svg+xml file, which cannot be displayed inline. Pass `outputPath` to write it to disk."
    );
}

#[tokio::test]
async fn local_files_are_off_for_non_loopback_binds() {
    let dir = tempfile::tempdir().unwrap();
    let store = std::sync::Arc::new(kanso_store::Store::open(dir.path().join("k.db")).unwrap());
    let config = kanso_server::ServerConfig {
        bind: "0.0.0.0:0".parse().unwrap(),
        ..config()
    };
    let router = kanso_server::router(store, &config);
    let png_path = dir.path().join("pixel.png");
    std::fs::write(&png_path, PNG).unwrap();
    let result = tool(&router, "create-file", json!({ "filePath": png_path })).await;
    assert_eq!(result["isError"], true);
    assert!(
        text(&result, 0).contains("only available when the server is bound to a loopback address")
    );
}
