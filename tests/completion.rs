use testing::completion::*;
use testing::*;
use tower_lsp::{lsp_types::*, LanguageServer};

#[tokio::test]
async fn initializing_in_source_dir_should_index_files() {
    init_test();

    let mut init_opts = ServerInitOptionsBuilder::default().build().unwrap();
    let kt_file_url = init_opts.workspace().add_kt_file(
        "com/test/TestClass.kt".into(),
        r#"
package com.ppro.billing

class TestClass(val i: Int)

fun x() {
    val y = T
}
    "#
        .trim()
        .into(),
    );

    let (_, server) = server_init_(init_opts).await;

    let completion_result = server
        .completion(completion_for(
            kt_file_url,
            Position {
                line: 5,
                character: 13,
            },
        ))
        .await
        .unwrap();

    assert!(completion_result.is_some());
    assert!(match completion_result.unwrap() {
        CompletionResponse::Array(v) => {
            assert!(v.len() == 1);
            assert!(v[0].label == "TestClass");
            assert!(v[0].kind == Some(CompletionItemKind::CLASS));

            true
        }
        CompletionResponse::List(_) => false,
    });

    server.shutdown().await.unwrap();
}
