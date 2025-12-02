//! High-level XML utilities using xmltree.

use xmltree::{Element, EmitterConfig, XMLNode};

/// Extracts an attribute value from an XML string's root element.
/// Returns None if the attribute is not found or the content is invalid.
pub fn extract_root_attribute(xml_content: &str, attr_name: &str) -> Option<String> {
	let root = Element::parse(xml_content.as_bytes()).ok()?;
	root.attributes.get(attr_name).cloned()
}

/// Extracts the inner content of the root element as a list of XMLNode.
/// Returns the children of the root element.
pub fn extract_root_inner_nodes(xml_content: &str) -> Option<Vec<XMLNode>> {
	let root = Element::parse(xml_content.as_bytes()).ok()?;
	Some(root.children)
}

/// Extracts the inner content of the root element as a string.
/// Returns the serialized children of the root element.
#[allow(unused)]
pub fn extract_root_inner_content(xml_content: &str) -> Option<String> {
	let nodes = extract_root_inner_nodes(xml_content)?;
	Some(nodes_to_string(&nodes))
}

/// Transforms all `id` attribute values in a list of XMLNodes using the provided function.
/// Returns the transformed nodes.
pub fn transform_nodes_id_attributes<F>(nodes: Vec<XMLNode>, transform_fn: F) -> Vec<XMLNode>
where
	F: Fn(&str) -> String,
{
	let mut nodes = nodes;
	for node in &mut nodes {
		if let Some(elem) = node.as_mut_element() {
			transform_element_ids_recursive(elem, &transform_fn);
		}
	}
	nodes
}

/// Converts a list of XMLNodes to a string.
pub fn nodes_to_string(nodes: &[XMLNode]) -> String {
	if nodes.is_empty() {
		return String::new();
	}

	let mut result = String::new();
	for node in nodes {
		if let Some(node_str) = node_to_string(node) {
			if !result.is_empty() && !node_str.trim().is_empty() {
				result.push('\n');
			}
			result.push_str(&node_str);
		}
	}

	result.trim().to_string()
}

/// Recursively transforms id attributes in an element and its children.
fn transform_element_ids_recursive<F>(element: &mut Element, transform_fn: &F)
where
	F: Fn(&str) -> String,
{
	// Transform id attribute if present
	if let Some(id_value) = element.attributes.get("id").cloned() {
		let transformed = transform_fn(&id_value);
		element.attributes.insert("id".to_string(), transformed);
	}

	// Recurse into children
	for child in &mut element.children {
		if let Some(child_elem) = child.as_mut_element() {
			transform_element_ids_recursive(child_elem, transform_fn);
		}
	}
}

/// Converts an XMLNode to a string.
fn node_to_string(node: &XMLNode) -> Option<String> {
	match node {
		XMLNode::Element(el) => element_to_string(el),
		XMLNode::Text(text) => Some(text.clone()),
		XMLNode::CData(cdata) => Some(format!("<![CDATA[{cdata}]]>")),
		XMLNode::Comment(comment) => Some(format!("<!--{comment}-->")),
		XMLNode::ProcessingInstruction(target, data) => {
			if let Some(d) = data {
				Some(format!("<?{target} {d}?>"))
			} else {
				Some(format!("<?{target}?>"))
			}
		}
	}
}

/// Converts an Element to a string with proper formatting.
fn element_to_string(element: &Element) -> Option<String> {
	let config = EmitterConfig::new()
		.perform_indent(true)
		.indent_string("  ")
		.write_document_declaration(false);

	let mut output = Vec::new();
	element.write_with_config(&mut output, config).ok()?;
	String::from_utf8(output).ok()
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_support_xmls_extract_root_attribute_simple() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r#"<svg viewBox="0 0 24 24" width="24" height="24"></svg>"#;

		// -- Exec
		let result = extract_root_attribute(xml, "viewBox");

		// -- Check
		assert_eq!(result, Some("0 0 24 24".to_string()));

		Ok(())
	}

	#[test]
	fn test_support_xmls_extract_root_attribute_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r#"<svg width="24" height="24"></svg>"#;

		// -- Exec
		let result = extract_root_attribute(xml, "viewBox");

		// -- Check
		assert_eq!(result, None);

		Ok(())
	}

	#[test]
	fn test_support_xmls_extract_root_inner_content_simple() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r#"<svg viewBox="0 0 24 24"><path d="M0 0"/></svg>"#;

		// -- Exec
		let result = extract_root_inner_content(xml);

		// -- Check
		let content = result.ok_or("Should have content")?;
		assert!(content.contains("path"));
		assert!(content.contains(r#"d="M0 0""#));

		Ok(())
	}

	#[test]
	fn test_support_xmls_extract_root_inner_content_with_xml_decl() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg width="16px" height="16px" viewBox="0 0 16 16" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <title>ico/chevron-down</title>
    <g id="ico/chevron-down" stroke="none" fill="none" fill-rule="evenodd">
        <polygon id="Shape" fill="#CECECE" points="3.41 4.58 8 9.17 12.59 4.58 14 6 8 12 2 6"></polygon>
    </g>
</svg>"##;

		// -- Exec
		let result = extract_root_inner_content(xml);

		// -- Check
		let content = result.ok_or("Should have content")?;
		assert!(content.contains("<title>"));
		assert!(content.contains("<g id="));
		assert!(content.contains("<polygon"));
		assert!(content.contains("points="));

		Ok(())
	}

	#[test]
	fn test_support_xmls_extract_root_inner_content_empty() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r#"<svg viewBox="0 0 24 24"/>"#;

		// -- Exec
		let result = extract_root_inner_content(xml);

		// -- Check
		assert_eq!(result, Some(String::new()));

		Ok(())
	}

	#[test]
	fn test_support_xmls_transform_nodes_id_attributes_siblings() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r##"<svg>
    <title>ico/chevron-down</title>
    <g id="ico/chevron-down">
        <polygon id="Shape"/>
    </g>
</svg>"##;

		// -- Exec
		let nodes = extract_root_inner_nodes(xml).ok_or("Should have nodes")?;
		let transformed = transform_nodes_id_attributes(nodes, |id| id.replace('/', "-"));
		let result = nodes_to_string(&transformed);

		// -- Check
		assert!(result.contains("<title>"));
		assert!(result.contains(r#"id="ico-chevron-down""#));
		assert!(result.contains(r#"id="Shape""#));

		Ok(())
	}
}

// endregion: --- Tests
