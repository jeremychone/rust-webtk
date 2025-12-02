//! High-level XML utilities using quick-xml.

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::borrow::Cow;

/// Extracts an attribute value from an SVG/XML string's root element.
/// Returns None if the attribute is not found or the content is invalid.
pub fn extract_root_attribute(xml_content: &str, attr_name: &str) -> Option<String> {
	let mut reader = Reader::from_str(xml_content);

	loop {
		match reader.read_event() {
			Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
				return extract_attribute_from_element(e, attr_name);
			}
			Ok(Event::Eof) => return None,
			Err(_) => return None,
			_ => continue,
		}
	}
}

/// Extracts an attribute value from a BytesStart element.
fn extract_attribute_from_element(element: &BytesStart<'_>, attr_name: &str) -> Option<String> {
	for attr in element.attributes().flatten() {
		let key = std::str::from_utf8(attr.key.as_ref()).ok()?;
		if key == attr_name {
			let value = attr.unescape_value().ok()?;
			return Some(value.into_owned());
		}
	}
	None
}

/// Extracts the inner content of the root element as a string.
/// Returns the raw content between the opening and closing tags.
pub fn extract_root_inner_content(xml_content: &str) -> Option<String> {
	let mut reader = Reader::from_str(xml_content);
	let mut depth = 0;
	let mut content_start: Option<u64> = None;
	let mut content_end: Option<u64> = None;
	let mut root_name: Option<Vec<u8>> = None;

	loop {
		let event_start_pos = reader.buffer_position();

		match reader.read_event() {
			Ok(Event::Start(ref e)) => {
				if depth == 0 {
					root_name = Some(e.name().as_ref().to_vec());
					content_start = Some(reader.buffer_position());
				}
				depth += 1;
			}
			Ok(Event::End(ref e)) => {
				depth -= 1;
				if depth == 0
					&& let Some(ref rn) = root_name
					&& e.name().as_ref() == rn.as_slice()
				{
					content_end = Some(event_start_pos);
					break;
				}
			}
			Ok(Event::Empty(_)) => {
				if depth == 0 {
					// Self-closing root element, no inner content
					return Some(String::new());
				}
			}
			Ok(Event::Eof) => break,
			Err(_) => return None,
			_ => {}
		}
	}

	match (content_start, content_end) {
		(Some(start), Some(end)) if end > start => {
			let start_idx = start as usize;
			let end_idx = end as usize;
			if end_idx <= xml_content.len() {
				let inner = &xml_content[start_idx..end_idx];
				Some(inner.trim().to_string())
			} else {
				None
			}
		}
		_ => Some(String::new()),
	}
}

/// Transforms all `id` attribute values in XML content using the provided function.
pub fn transform_id_attributes<F>(xml_content: &str, transform_fn: F) -> String
where
	F: Fn(&str) -> String,
{
	let mut reader = Reader::from_str(xml_content);
	let mut writer = quick_xml::Writer::new(Vec::new());

	loop {
		match reader.read_event() {
			Ok(Event::Start(ref e)) => {
				let transformed = transform_element_ids(e, &transform_fn);
				writer.write_event(Event::Start(transformed)).ok();
			}
			Ok(Event::Empty(ref e)) => {
				let transformed = transform_element_ids(e, &transform_fn);
				writer.write_event(Event::Empty(transformed)).ok();
			}
			Ok(Event::Eof) => break,
			Ok(event) => {
				writer.write_event(event).ok();
			}
			Err(_) => break,
		}
	}

	String::from_utf8(writer.into_inner()).unwrap_or_else(|_| xml_content.to_string())
}

/// Transforms the id attribute of an element using the provided function.
fn transform_element_ids<'a, F>(element: &BytesStart<'a>, transform_fn: &F) -> BytesStart<'static>
where
	F: Fn(&str) -> String,
{
	let name = std::str::from_utf8(element.name().as_ref()).unwrap_or("").to_string();
	let mut new_element = BytesStart::new(name);

	for attr in element.attributes().flatten() {
		let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
		let value = attr.unescape_value().unwrap_or(Cow::Borrowed(""));

		if key == "id" {
			let transformed_value = transform_fn(&value);
			new_element.push_attribute((key, transformed_value.as_str()));
		} else {
			new_element.push_attribute((key, value.as_ref()));
		}
	}

	new_element
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
		assert_eq!(result, Some(r#"<path d="M0 0"/>"#.to_string()));

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
	fn test_support_xmls_transform_id_attributes_simple() -> Result<()> {
		// -- Setup & Fixtures
		let xml = r#"<svg><g id="ico/user/fill"><path id="path-1"/></g></svg>"#;

		// -- Exec
		let result = transform_id_attributes(xml, |id| id.replace('/', "-"));

		// -- Check
		assert!(result.contains(r#"id="ico-user-fill""#));
		assert!(result.contains(r#"id="path-1""#));

		Ok(())
	}
}

// endregion: --- Tests
