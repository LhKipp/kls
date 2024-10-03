# KLS

## Logging

This project uses the `tracing` crate. Logs can be enabled by setting `RUST_LOG`.

## Testing

- Test log severity can be set with `KLS_TEST_LOG` (trace, debug, info, warn, error). 
- Golden tests can be updated by running tests with GOLDEN_TEST_UPDATE=1


# Scratch
?SFunction(?) <=> TreeElement

SFunction(text_range) <=> TreeElement
- on tree update, update all text_ranges

SFunction(id) <=> TreeElement
- requires id <=> node_id <=> TreeElement
- can update all node_ids at once

# Comments
CLONE -> Clones which are possibly optimisable
CONTINUE_HERE -> where I left off working on
