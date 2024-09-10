use testing::{completion::expect_completion_in_vec, *};
use tower_lsp::LanguageServer;

fn x() -> CompletionParams {
    return CompletionParams {
        text_document_position: TextDocumentPositionParams{
            text_document: TextDocumentIdentifier {
                uri: Url::parse("com/test/clock/Clock.kt")
            }
            position: todo!(),
        },
        ..
    };
}

#[tokio::test]
async fn adding_text_to_an_existing_document() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file("com/test/clock/Clock.kt".into(), r#"package com.test"#);
    })
    .await;

    let completions_before_edit = server.completion("com.t");
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

#[tokio::test]
async fn removing_text_of_an_existing_document() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file("com/test/clock/Clock.kt".into(), r#"package com.test"#);
    })
    .await;

    let completions_before_edit = server.indexes.completions_for("com.t");
    assert_eq!(completions_before_edit.len(), 1);
    assert_eq!(completions_before_edit[0].label, "com.test");

    server
        .did_change(DidChangeTextDocumentParams {
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: pos(0, 0),
                    end: pos(0, 16),
                }),
                range_length: None,
                text: "".to_string(),
            }],
            text_document: VersionedTextDocumentIdentifier {
                uri: init.workspace().url_of("com/test/clock/Clock.kt"),
                version: 1,
            },
        })
        .await;

    let completions_after_edit = server.indexes.completions_for("com.t");
    assert_eq!(completions_after_edit.len(), 0);

    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn removing_class_in_an_existing_document_removes_class_indexes() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file(
            "com/test/clock/Clock.kt".into(),
            "package com.test\nclass TestClass",
        );
    })
    .await;

    let completions_before_edit = server.indexes.completions_for("com.t");
    assert_eq!(completions_before_edit.len(), 2);
    expect_completion_in_vec(&completions_before_edit, "com.test");
    expect_completion_in_vec(&completions_before_edit, "com.test.TestClass");

    server
        .did_change(DidChangeTextDocumentParams {
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: pos(1, 0),
                    end: pos(1, 15),
                }),
                range_length: None,
                text: "".to_string(),
            }],
            text_document: VersionedTextDocumentIdentifier {
                uri: init.workspace().url_of("com/test/clock/Clock.kt"),
                version: 1,
            },
        })
        .await;

    let completions_after_edit = server.indexes.completions_for("com.t");
    assert_eq!(completions_after_edit.len(), 1);
    expect_completion_in_vec(&completions_after_edit, "com.test");

    server.shutdown().await.unwrap();
}
