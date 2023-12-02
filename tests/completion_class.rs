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

#[tokio::test]
async fn should_complete_simple_class_name_behind_package() {
    let (init, _, server) = init_test(|opts| {
        opts.add_kt_file(
            "com/test/TestClass.kt".into(),
            r#"
package com.ppro.billing

class TestClass(val i: Int)

fun x() {
    val y = com.pp
}
    "#,
        );
    })
    .await;

    let completion_result = server
        .completion(completion_for(
            init.workspace().url_of("com/test/TestClass.kt"),
            pos(5, 18),
        ))
        .await
        .unwrap();

    let class_completion = expect_completion_in_response(&completion_result, "com.ppro.billing.TestClass");
    assert!(class_completion.kind == Some(CompletionItemKind::CLASS));

    let package_completion = expect_completion_in_response(&completion_result, "com.ppro.billing");
    assert!(package_completion.kind == Some(CompletionItemKind::MODULE));

    server.shutdown().await.unwrap();
}
