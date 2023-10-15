use testing::*;
use tower_lsp::LanguageServer;

#[tokio::test]
async fn adding_text_to_an_existing_document() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file("com/test/clock/Clock.kt".into(), r#""#);
    })
    .await;

    server
        .did_change(DidChangeTextDocumentParams {
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    // checked with nvim
                    start: pos(0, 0),
                    end: pos(0, 0),
                }),
                range_length: None,
                text: "package com.test.clock".to_string(),
            }],
            text_document: VersionedTextDocumentIdentifier {
                uri: init.workspace().url_of("com/test/clock/Clock.kt"),
                version: 1,
            },
        })
        .await;

    let actual = server.text_of(&init.workspace().url_of("com/test/clock/Clock.kt"));

    assert_eq!(actual, "package com.test.clock");

    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn removing_text_of_an_existing_document() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file(
            "com/test/clock/Clock.kt".into(),
            r#"package com.test.clock"#,
        );
    })
    .await;

    server
        .did_change(DidChangeTextDocumentParams {
            content_changes: vec![TextDocumentContentChangeEvent {
                // checked with nvim
                range: Some(Range {
                    start: pos(0, 8),
                    end: pos(0, 22),
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

    let actual = server.text_of(&init.workspace().url_of("com/test/clock/Clock.kt"));

    assert_eq!(actual, "package ");

    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn editing_text_in_an_existing_document() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file(
            "com/test/clock/Clock.kt".into(),
            r#"package com.test.clock"#,
        );
    })
    .await;

    server
        .did_change(DidChangeTextDocumentParams {
            content_changes: vec![TextDocumentContentChangeEvent {
                // nvim sends a delete and then an insert ...
                range: Some(Range {
                    start: pos(0, 8),
                    end: pos(0, 22),
                }),
                range_length: None,
                text: "com.time".to_string(),
            }],
            text_document: VersionedTextDocumentIdentifier {
                uri: init.workspace().url_of("com/test/clock/Clock.kt"),
                version: 1,
            },
        })
        .await;

    let actual = server.text_of(&init.workspace().url_of("com/test/clock/Clock.kt"));

    assert_eq!(actual, "package com.time");

    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn adding_text_to_an_existing_document_multiline() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file("com/test/clock/Clock.kt".into(), r#"package"#);
    })
    .await;

    server
        .did_change(DidChangeTextDocumentParams {
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    // Checked with nvim
                    start: pos(0, 7),
                    end: pos(0, 7),
                }),
                range_length: None,
                text: " com.time\nclass Clock()".to_string(),
            }],
            text_document: VersionedTextDocumentIdentifier {
                uri: init.workspace().url_of("com/test/clock/Clock.kt"),
                version: 1,
            },
        })
        .await;

    let actual = server.text_of(&init.workspace().url_of("com/test/clock/Clock.kt"));

    assert_eq!(actual, "package com.time\nclass Clock()");

    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn removing_text_of_an_existing_document_multiline() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file(
            "com/test/clock/Clock.kt".into(),
            "package com.time\nclass Clock()",
        );
    })
    .await;

    server
        .did_change(DidChangeTextDocumentParams {
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: pos(0, 8),
                    end: pos(1, 13),
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

    let actual = server.text_of(&init.workspace().url_of("com/test/clock/Clock.kt"));

    assert_eq!(actual, "package ");

    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn editing_text_in_an_existing_document_multiline() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file(
            "com/test/clock/Clock.kt".into(),
            "package com.time\nclass Clock()",
        );
    })
    .await;

    server
        .did_change(DidChangeTextDocumentParams {
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: pos(0, 8),
                    end: pos(1, 5),
                }),
                range_length: None,
                text: "com.test.time\npub class".to_string(),
            }],
            text_document: VersionedTextDocumentIdentifier {
                uri: init.workspace().url_of("com/test/clock/Clock.kt"),
                version: 1,
            },
        })
        .await;

    let actual = server.text_of(&init.workspace().url_of("com/test/clock/Clock.kt"));

    assert_eq!(actual, "package com.test.time\npub class Clock()");

    server.shutdown().await.unwrap();
}
