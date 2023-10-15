use testing::completion::*;
use testing::*;
use tower_lsp::LanguageServer;

#[tokio::test]
async fn initializing_in_source_dir_should_index_files() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file(
            "com/test/time/Time.kt".into(),
            r#"
package com.test.time
"#,
        )
        .add_kt_file(
            "com/test/clock/Clock.kt".into(),
            r#"
package com.test.clock

fun x() {
    com.test.t
}"#,
        );
    })
    .await;

    let completion_result = server
        .completion(completion_for(
            init.workspace().url_of("com/test/clock/Clock.kt"),
            pos(3, 14),
        ))
        .await
        .unwrap();

    assert!(completion_result.is_some());
    assert!(match completion_result.unwrap() {
        CompletionResponse::Array(v) => {
            assert!(v.len() == 1);
            assert!(v[0].label == "com.test.time");
            assert!(v[0].kind == Some(CompletionItemKind::MODULE));

            true
        }
        CompletionResponse::List(_) => false,
    });

    server.shutdown().await.unwrap();
}
