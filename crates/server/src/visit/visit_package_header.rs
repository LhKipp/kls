use crate::scope::ScopeBuilder;
use crate::scope_error::ScopeError;
use crate::visit::*;

pub(crate) fn visit_package_header(builder: &mut ScopeBuilder, package_node: &Node) {
    let package_decl = node::PackageHeader::new(package_node.clone(), &builder.buffer.text);
    trace!(
        "Visiting package_header {:?}",
        package_decl.find_identifier()
    );

    let package_ident = if let Some(package_ident) = package_decl.find_identifier() {
        package_ident.text()
    } else {
        builder.errors.push(ScopeError::new(
            "Package declaration missing package name".into(),
        ));
        String::new()
    };

    builder.current_mut().items.insert(
        package_ident.clone(),
        SItem::new(
            package_node.range(),
            SItemKind::PackageHeader(package_ident),
        ),
    );
}
