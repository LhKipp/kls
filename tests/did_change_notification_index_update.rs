use testing::*;
use tower_lsp::LanguageServer;

#[tokio::test]
async fn adding_text_to_an_existing_document() {
    init_test();

    let mut init = ServerInitOptionsBuilder::default()
        .add_kt_file("com/test/clock/Clock.kt".into(), r#"package com.test"#)
        .build()
        .unwrap();

    let (_, server) = server_init_(init.clone()).await;
    let completions_before_edit = server.indexes.completions_for("com.t");
    assert_eq!(completions_before_edit.len(), 1);
    assert_eq!(completions_before_edit[0].label, "com.test");

    server
        .did_change(DidChangeTextDocumentParams {
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: pos(0, 14),
                    end: pos(0, 16),
                }),
                range_length: None,
                text: "xt".to_string(),
            }],
            text_document: VersionedTextDocumentIdentifier {
                uri: init.workspace().url_of("com/test/clock/Clock.kt"),
                version: 1,
            },
        })
        .await;

    let completions_after_edit = server.indexes.completions_for("com.t");
    assert_eq!(completions_after_edit.len(), 1);
    assert_eq!(completions_after_edit[0].label, "com.text");

    server.shutdown().await.unwrap();
}
