use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::kserver::KServer;
use parser::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintScopesRequest {
    #[serde(default)]
    pub print_file_contents: bool,
    #[serde(flatten, default)]
    pub print_ast: Option<PrintAstOptions>,
    pub trim_from_file_paths: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintAstOptions {
    #[serde(default)]
    pub print_ast: bool,
}

#[derive(new)]
pub struct PrintScopesHandler<'a> {
    server: &'a KServer,
    request: &'a PrintScopesRequest,
}

impl<'a> PrintScopesHandler<'a> {
    pub fn handle(&self) -> anyhow::Result<String> {
        let r_scopes = self.server.scopes.0.read();
        let printer =
            ScopeDebugPrettyPrint::new(&r_scopes.project_nodes[0], &r_scopes.scopes, &self.request);
        Ok(format!("{:?}", printer))
    }
}

impl<'a> ScopeDebugPrettyPrint<'a> {
    /// Creates a new `DebugPrettyPrint` object for the node.
    #[inline]
    pub(crate) fn new(
        id: &'a NodeId,
        arena: &'a Arena<ARwLock<Scope>>,
        request: &'a PrintScopesRequest,
    ) -> Self {
        Self { id, arena, request }
    }
}

impl<'a> ScopeDebugPrettyPrint<'a> {
    fn print_debug(&self, scope: &ARwLock<Scope>) -> String {
        let r_scope = scope.read();
        match &r_scope.kind {
            SKind::Project(project) => format!("Project {}", project.data.name.clone()),

            SKind::SourceSet(source_set) => {
                let includes = source_set
                    .data
                    .dependencies
                    .iter()
                    .map(|d| format!("{:?} {:?} ({:?})", d.visibility, d.name, d.kind))
                    .join(", ");

                let mut result = format!("SourceSet {}", source_set.data.name.clone());
                if !includes.is_empty() {
                    result += &format!(" (includes {includes})");
                }
                result
            }

            SKind::File(s_file) => {
                let file_path = match &self.request.trim_from_file_paths {
                    Some(prefix) => s_file
                        .path
                        .strip_prefix(prefix)
                        .expect("Could not strip passed prefix from file")
                        .display()
                        .to_string(),
                    None => s_file.path.display().to_string(),
                };

                let mut result = format!("File ({})", file_path);

                if self.request.print_file_contents && !s_file.text.is_empty() {
                    result += &format!("\n{}", s_file.text);
                }

                if let Some(print_ast_options) = &self.request.print_ast {
                    if print_ast_options.print_ast {
                        let mut tree = "".to_string();
                        dfs_descend(&s_file.tree.root_node(), 0, &mut |node, depth| {
                            if node.kind_id() == *parser::SourceFileId {
                                tree += &format!("{}\n", node.kind());
                            } else {
                                tree += &format!(
                                    "{}{} {}-{} ({})\n",
                                    " ".repeat(depth * 2),
                                    node.kind(),
                                    node.start_position(),
                                    node.end_position(),
                                    parser::text_of(node, &s_file.text),
                                );
                            }
                        });
                        result += &tree;
                    }
                };

                result
            }
        }
    }
}

pub fn dfs_descend<'a, F>(node: &tree_sitter::Node<'a>, depth: usize, f: &mut F)
where
    F: FnMut(&tree_sitter::Node<'a>, usize),
{
    let mut cursor = node.walk();
    f(node, depth);
    for child in node.children(&mut cursor) {
        dfs_descend(&child, depth + 1, f);
    }
}

use crate::scope::*;
use core::fmt::{self, Write as _};
use indextree::*;
use itertools::Itertools;
use std::{fmt::Debug, path::PathBuf, vec::Vec};
use stdx::ARwLock;

//use crate::dynamic::hierarchy::traverse::{DepthFirstTraverser, DftEvent};
//use crate::dynamic::hierarchy::Hierarchy;

/// State of an indent block.
#[derive(Clone, Copy)]
struct IndentedBlockState {
    /// Whether this is the last item.
    is_last_item: bool,
    /// Whether the line is the first line.
    is_first_line: bool,
}

impl IndentedBlockState {
    /// Returns the indent string for the indent type.
    #[inline]
    fn as_str(self) -> &'static str {
        match (self.is_last_item, self.is_first_line) {
            (false, true) => "|-- ",
            (false, false) => "|   ",
            (true, true) => "`-- ",
            (true, false) => "    ",
        }
    }

    /// Returns the leading part of the indent string.
    #[inline]
    fn as_str_leading(self) -> &'static str {
        match (self.is_last_item, self.is_first_line) {
            (false, true) => "I--",
            (false, false) => "I",
            (true, true) => "L--",
            (true, false) => "",
        }
    }

    /// Returns the trailing whitespaces part of the indent string.
    #[inline]
    fn as_str_trailing_spaces(self) -> &'static str {
        match (self.is_last_item, self.is_first_line) {
            (_, true) => " ",
            (false, false) => "   ",
            (true, false) => "    ",
        }
    }

    /// Returns whether the indent string consists of only whitespaces.
    #[inline]
    #[must_use]
    fn is_all_whitespace(self) -> bool {
        self.is_last_item && !self.is_first_line
    }
}

/// State of the line writing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineState {
    /// Before any character of the indent is written to the current line.
    BeforeIndent,
    /// Indents are partially written.
    ///
    /// More precisely, trailing whitespaces are not yet written.
    PartialIndent,
    /// Writing content.
    Content,
}

/// Indent writer for the debug printer.
struct IndentWriter<'a, 'b> {
    /// Backend formatter.
    fmt: &'b mut fmt::Formatter<'a>,
    /// State of the line writing.
    line_state: LineState,
    /// Indents.
    indents: Vec<IndentedBlockState>,
    /// The number of pending whitespace-only indents.
    pending_ws_only_indent_level: usize,
}

impl<'a, 'b> IndentWriter<'a, 'b> {
    /// Creates a new `PadAdapter`.
    #[inline]
    fn new(fmt: &'b mut fmt::Formatter<'a>) -> Self {
        Self {
            fmt,
            line_state: LineState::BeforeIndent,
            indents: Vec::new(),
            pending_ws_only_indent_level: 0,
        }
    }

    /// Opens the next item.
    ///
    /// Writes a newline if necessary and prepares to write the next item.
    ///
    /// This should **not** be called for the root item.
    fn open_item(&mut self, is_last_item: bool) -> fmt::Result {
        if self.line_state != LineState::BeforeIndent {
            self.fmt.write_char('\n')?;
            self.line_state = LineState::BeforeIndent;
            self.pending_ws_only_indent_level = 0;
        }
        if let Some(indent) = self.indents.last_mut() {
            indent.is_first_line = false;
        }
        self.indents.push(IndentedBlockState {
            is_last_item,
            is_first_line: true,
        });

        Ok(())
    }

    /// Closes the current item.
    ///
    /// Returns `Ok(())` if an item is successfully closed.
    /// Returns `Err(())` if there are no items that can be closed, i.e. the
    /// current item is the root.
    #[inline]
    fn close_item(&mut self) -> Result<(), ()> {
        match self.indents.pop() {
            Some(_) => Ok(()),
            None => Err(()),
        }
    }

    /// Writes the indent except for the trailing whitespaces.
    fn write_indent_partial(&mut self) -> fmt::Result {
        self.pending_ws_only_indent_level = self
            .indents
            .iter()
            .rev()
            .take_while(|i| i.is_all_whitespace())
            .count();

        let ws_indent_first_level = self.indents.len() - self.pending_ws_only_indent_level;
        let indents_to_print = &self.indents[..ws_indent_first_level];
        if let Some(last) = indents_to_print.last() {
            for indent in &indents_to_print[..(indents_to_print.len() - 1)] {
                self.fmt.write_str(indent.as_str())?;
            }
            self.fmt.write_str(last.as_str_leading())?;
        }

        Ok(())
    }

    /// Writes the rest of the indents which are partially written.
    fn complete_partial_indent(&mut self) -> fmt::Result {
        debug_assert_eq!(self.line_state, LineState::PartialIndent);
        if let Some(last_non_ws_indent_level) =
            (self.indents.len() - self.pending_ws_only_indent_level).checked_sub(1)
        {
            self.fmt
                .write_str(self.indents[last_non_ws_indent_level].as_str_trailing_spaces())?;
        }
        for _ in 0..self.pending_ws_only_indent_level {
            self.fmt.write_str("    ")?;
        }
        self.pending_ws_only_indent_level = 0;

        Ok(())
    }
}

impl fmt::Write for IndentWriter<'_, '_> {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while !s.is_empty() {
            // There remains something to print.

            if self.line_state == LineState::BeforeIndent {
                self.write_indent_partial()?;
                self.line_state = LineState::PartialIndent;
            }

            let (line_end, ends_with_newline) = match s.find('\n') {
                Some(pos) => (pos + 1, true),
                None => (s.len(), false),
            };
            let content = &s[..line_end];
            if !content.is_empty() {
                debug_assert_ne!(
                    self.line_state,
                    LineState::BeforeIndent,
                    "[consistency] indent must be written since there are something to write"
                );
                if self.line_state == LineState::PartialIndent {
                    self.complete_partial_indent()?;
                }
                if let Some(level) = self.indents.last_mut() {
                    level.is_first_line = level.is_first_line && !ends_with_newline;
                }
                self.fmt.write_str(content)?;

                self.line_state = if ends_with_newline {
                    LineState::BeforeIndent
                } else {
                    LineState::Content
                };
            }
            s = &s[line_end..];
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
struct ScopeDebugPrettyPrint<'a> {
    /// Root node ID of the (sub)tree to print.
    id: &'a NodeId,
    /// Arena the node belongs to.
    arena: &'a Arena<ARwLock<Scope>>,
    /// params
    request: &'a PrintScopesRequest,
}
impl<'a> fmt::Debug for ScopeDebugPrettyPrint<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut writer = IndentWriter::new(f);
        let mut traverser = self.id.traverse(self.arena);

        // Print the first (root) node.
        traverser.next();
        {
            let data = self.arena[*self.id].get();
            writer.write_str(&self.print_debug(data))?;
        }

        // Print the descendants.
        while let Some(id) = prepare_next_node_printing(&mut writer, &mut traverser, &self.arena)? {
            let data = self.arena[id].get();
            writer.write_str(&self.print_debug(data))?;
        }

        Ok(())
    }
}

/// Prepares printing of next node.
///
/// Internally, this searches next node open and adjust indent level and prefix.
fn prepare_next_node_printing<'a, T>(
    writer: &mut IndentWriter<'_, '_>,
    traverser: &mut Traverse<'_, T>,
    arena: &Arena<ARwLock<Scope>>,
) -> Result<Option<NodeId>, fmt::Error> {
    // Not using `for ev in traverser` in order to access to `traverser`
    // directly in the loop.
    while let Some(ev) = traverser.next() {
        let id = match ev {
            NodeEdge::Start(id) => id,
            NodeEdge::End(_) => {
                if writer.close_item().is_ok() {
                    // Closed a non-root node.
                    continue;
                } else {
                    // Closed the root node.
                    break;
                }
            }
        };
        let is_last_sibling = arena[id].next_sibling().is_none();
        writer.open_item(is_last_sibling)?;

        return Ok(Some(id));
    }

    Ok(None)
}
