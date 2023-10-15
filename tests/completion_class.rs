use testing::completion::*;
use testing::*;

#[tokio::test]
async fn should_complete_simple_class_name() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file(
            "com/test/TestClass.kt".into(),
            r#"
package com.ppro.billing

class TestClass(val i: Int)

fun x() {
    val y = T
}
    "#,
        );
    })
    .await;

    let completion_result = server
        .completion(completion_for(
            init.workspace().url_of("com/test/TestClass.kt"),
            pos(5, 13),
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
