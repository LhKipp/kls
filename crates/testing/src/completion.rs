use tower_lsp::lsp_types::*;

pub fn completion_for(file: Url, pos: Position) -> CompletionParams {
    CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: file },
            position: pos,
        },
        work_done_progress_params: WorkDoneProgressParams {
            work_done_token: None,
        },
        partial_result_params: PartialResultParams {
            partial_result_token: None,
        },
        context: None,
    }
}

pub fn expect_completion_in_response<'a>(
    result: &'a Option<CompletionResponse>,
    item_wanted: &str,
) -> &'a CompletionItem {
    assert!(result.is_some());
    match result.as_ref().unwrap() {
        CompletionResponse::Array(v) => expect_completion_in_vec(v, item_wanted),
        _ => unreachable!(),
    }
}

pub fn expect_completion_in_vec<'a>(
    v: &'a Vec<CompletionItem>,
    item_wanted: &str,
) -> &'a CompletionItem {
    let x = v.iter().filter(|c| c.label == item_wanted).next();
    return x.expect(&format!("No item found with label {}", item_wanted));
}
