use serde::{Deserialize, Serialize};

/// Represents a single section in a meeting template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSection {
    /// Section title (e.g., "Summary", "Action Items")
    pub title: String,

    /// Instruction for the LLM on what to extract/include
    pub instruction: String,

    /// Format type: "paragraph", "list", or "string"
    pub format: String,

    /// Optional markdown formatting hint for list items (e.g., table structure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_format: Option<String>,

    /// Alternative formatting hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example_item_format: Option<String>,
}

/// Represents a complete meeting template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template display name
    pub name: String,

    /// Brief description of the template's purpose
    pub description: String,

    /// List of sections in the template
    pub sections: Vec<TemplateSection>,
}

impl Template {
    /// Validates the template structure
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Template name cannot be empty".to_string());
        }

        if self.description.is_empty() {
            return Err("Template description cannot be empty".to_string());
        }

        if self.sections.is_empty() {
            return Err("Template must have at least one section".to_string());
        }

        for (i, section) in self.sections.iter().enumerate() {
            if section.title.is_empty() {
                return Err(format!("Section {} has empty title", i));
            }

            if section.instruction.is_empty() {
                return Err(format!("Section '{}' has empty instruction", section.title));
            }

            match section.format.as_str() {
                "paragraph" | "list" | "string" => {},
                other => return Err(format!(
                    "Section '{}' has invalid format '{}'. Must be 'paragraph', 'list', or 'string'",
                    section.title, other
                )),
            }
        }

        Ok(())
    }

    /// Generates a clean markdown template structure
    pub fn to_markdown_structure(&self) -> String {
        let mut markdown = String::from("# <Add Title here>\n\n");

        for section in &self.sections {
            match section.format.as_str() {
                "list" => markdown.push_str(&format!("**{}**\n\n- [item]\n\n", section.title)),
                "string" => markdown.push_str(&format!("**{}:** [value]\n\n", section.title)),
                _ => markdown.push_str(&format!("**{}**\n\n[paragraph]\n\n", section.title)),
            }
        }

        markdown
    }

    /// Generates section-specific instructions for the LLM
    pub fn to_section_instructions(&self) -> String {
        let mut instructions = String::from(
            "- **For the main title (`# [AI-Generated Title]`):** Analyze the entire transcript and create a concise, descriptive title for the meeting.\n"
        );

        for section in &self.sections {
            instructions.push_str(&format!(
                "- **For the '{}' section:** {}.\n",
                section.title, section.instruction
            ));

            let format_instruction = match section.format.as_str() {
                "list" => "write one Markdown bullet per item",
                "string" => "write one concise value on the heading line after the colon",
                _ => "write one or more prose paragraphs under the heading",
            };
            instructions.push_str(&format!(
                "  - **Output format:** {}.\n",
                format_instruction
            ));

            // Add item format instructions if present
            let item_format = section.item_format.as_ref()
                .or(section.example_item_format.as_ref());

            if let Some(format) = item_format {
                instructions.push_str(&format!(
                    "  - Items in this section should follow the format: `{}`.\n",
                    format
                ));
            }
        }

        instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_template() {
        let template = Template {
            name: "Test Template".to_string(),
            description: "A test template".to_string(),
            sections: vec![
                TemplateSection {
                    title: "Summary".to_string(),
                    instruction: "Provide a summary".to_string(),
                    format: "paragraph".to_string(),
                    item_format: None,
                    example_item_format: None,
                },
            ],
        };

        assert!(template.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let template = Template {
            name: "".to_string(),
            description: "A test template".to_string(),
            sections: vec![],
        };

        assert!(template.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_format() {
        let template = Template {
            name: "Test".to_string(),
            description: "Test".to_string(),
            sections: vec![
                TemplateSection {
                    title: "Test".to_string(),
                    instruction: "Test".to_string(),
                    format: "invalid".to_string(),
                    item_format: None,
                    example_item_format: None,
                },
            ],
        };

        assert!(template.validate().is_err());
    }

    #[test]
    fn markdown_structure_and_instructions_honor_section_formats() {
        let template = Template {
            name: "Formats".to_string(),
            description: "Format coverage".to_string(),
            sections: vec![
                TemplateSection {
                    title: "Summary".to_string(),
                    instruction: "Capture the main point".to_string(),
                    format: "paragraph".to_string(),
                    item_format: None,
                    example_item_format: None,
                },
                TemplateSection {
                    title: "Actions".to_string(),
                    instruction: "Capture tasks".to_string(),
                    format: "list".to_string(),
                    item_format: None,
                    example_item_format: None,
                },
                TemplateSection {
                    title: "Date".to_string(),
                    instruction: "Use YYYY-MM-DD".to_string(),
                    format: "string".to_string(),
                    item_format: None,
                    example_item_format: None,
                },
            ],
        };

        let markdown = template.to_markdown_structure();
        assert!(markdown.contains("**Summary**\n\n[paragraph]"));
        assert!(markdown.contains("**Actions**\n\n- [item]"));
        assert!(markdown.contains("**Date:** [value]"));

        let instructions = template.to_section_instructions();
        assert!(instructions.contains("one or more prose paragraphs"));
        assert!(instructions.contains("one Markdown bullet per item"));
        assert!(instructions.contains("one concise value on the heading line"));
    }
}
