//! Native egui WYSIWYG Markdown editor.
//!
//! Editing uses marker-free visible text plus range metadata. Markdown is
//! parsed at the document boundary and serialized canonically when saved.

use crate::markdown::{parse_target, target_uri, LinkTarget};
use eframe::egui::{self, Color32, FontId, Stroke, TextFormat, TextStyle};
use std::ops::Range;

const LINK_BLUE: Color32 = Color32::from_rgb(24, 99, 210);
pub(crate) const DEFAULT_MARKDOWN_FONT_SIZE: u16 = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
enum InlineStyle {
    Strong,
    Emphasis,
    Code,
    Strikethrough,
    Underline,
    Color { css: String, color: Option<Color32> },
    FontSize { pixels: u16 },
}

#[derive(Debug, Clone)]
struct StyleSpan {
    range: Range<usize>,
    style: InlineStyle,
}

#[derive(Debug, Clone)]
struct LinkSpan {
    range: Range<usize>,
    target: LinkTarget,
}

#[derive(Debug, Clone)]
struct HeadingSpan {
    range: Range<usize>,
    level: usize,
}

#[derive(Debug, Clone)]
struct RemovedPrefix {
    range: Range<usize>,
}

#[derive(Debug, Default)]
pub struct MarkdownEditor {
    document_key: String,
    source_markdown: String,
    source_mode: bool,
    text: String,
    links: Vec<LinkSpan>,
    styles: Vec<StyleSpan>,
    headings: Vec<HeadingSpan>,
    selection: Option<Range<usize>>,
    cursor_index: Option<usize>,
    active_font_size: Option<u16>,
    pending_selection: Option<Range<usize>>,
    text_edit_id: Option<egui::Id>,
    restore_focus: bool,
}

#[derive(Debug, Default)]
pub struct EditorOutput {
    pub changed: bool,
    pub activated_link: Option<LinkTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFormatCommand {
    Underline,
    Blue,
    Red,
    Bold,
    OrangeCrossline,
    FontSize(u16),
}

impl MarkdownEditor {
    pub fn ensure_document(&mut self, key: impl Into<String>, markdown: &str) {
        let key = key.into();
        if self.document_key != key || self.source_markdown != markdown {
            *self = Self::from_markdown(key, markdown);
        }
    }

    fn from_markdown(key: String, markdown: &str) -> Self {
        let mut editor = Self {
            document_key: key,
            source_markdown: markdown.to_owned(),
            ..Default::default()
        };
        let normalized_markdown = normalize_legacy_multiline_spans(markdown);
        parse_document(
            &normalized_markdown,
            &mut editor.text,
            &mut editor.links,
            &mut editor.styles,
            &mut editor.headings,
        );
        editor
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> EditorOutput {
        let available_size = ui.available_size();
        if self.source_mode {
            let previous = self.source_markdown.clone();
            let output = egui::TextEdit::multiline(&mut self.source_markdown)
                .code_editor()
                .desired_width(f32::INFINITY)
                .desired_rows(1)
                .min_size(available_size)
                .margin(egui::Margin::same(14))
                .show(ui);
            self.text_edit_id = Some(output.response.id);
            return EditorOutput {
                changed: self.source_markdown != previous,
                activated_link: None,
            };
        }
        let backspace_pressed = ui.input(|input| input.key_pressed(egui::Key::Backspace));
        let previous = self.text.clone();
        let links = self.links.clone();
        let styles = self.styles.clone();
        let headings = self.headings.clone();
        let mut body_font = TextStyle::Body.resolve(ui.style());
        body_font.size = f32::from(DEFAULT_MARKDOWN_FONT_SIZE);
        let text_color = ui.visuals().text_color();
        let code_background = ui.visuals().code_bg_color;

        let mut layouter = move |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, wrap_width: f32| {
            let mut job = egui::text::LayoutJob::default();
            job.wrap.max_width = wrap_width;
            for (index, character) in buffer.as_str().chars().enumerate() {
                let mut format = TextFormat {
                    font_id: body_font.clone(),
                    color: text_color,
                    ..Default::default()
                };
                if let Some(heading) = headings.iter().find(|span| span.range.contains(&index)) {
                    let size = match heading.level {
                        1 => 25.0,
                        2 => 21.0,
                        _ => 18.0,
                    };
                    format.font_id = FontId::proportional(size);
                }
                let mut strikethrough = false;
                let mut underline = false;
                for span in styles.iter().filter(|span| span.range.contains(&index)) {
                    match &span.style {
                        InlineStyle::Strong => {
                            // egui's bundled font has no distinct bold face; a subtle
                            // size increase preserves a WYSIWYG distinction.
                            format.font_id.size += 0.8;
                        }
                        InlineStyle::Emphasis => format.italics = true,
                        InlineStyle::Code => {
                            format.font_id = TextStyle::Monospace.resolve(ui.style());
                            format.background = code_background;
                        }
                        InlineStyle::Strikethrough => strikethrough = true,
                        InlineStyle::Underline => underline = true,
                        InlineStyle::Color {
                            color: Some(color), ..
                        } => format.color = *color,
                        InlineStyle::Color { color: None, .. } => {}
                        InlineStyle::FontSize { pixels } => {
                            format.font_id.size = f32::from(*pixels);
                        }
                    }
                }
                if links.iter().any(|span| span.range.contains(&index)) {
                    format.color = LINK_BLUE;
                    underline = true;
                }
                if strikethrough {
                    format.strikethrough = Stroke::new(1.0_f32, format.color);
                }
                if underline {
                    format.underline = Stroke::new(1.0_f32, format.color);
                }
                job.append(&character.to_string(), 0.0, format);
            }
            ui.fonts(|fonts| fonts.layout_job(job))
        };

        let mut output = egui::TextEdit::multiline(&mut self.text)
            .desired_width(f32::INFINITY)
            .desired_rows(1)
            .min_size(available_size)
            .margin(egui::Margin::same(14))
            .layouter(&mut layouter)
            .show(ui);
        self.text_edit_id = Some(output.response.id);
        if std::mem::take(&mut self.restore_focus) {
            output.response.request_focus();
        }

        let text_changed = self.text != previous;
        let removed_prefixes = self.finish_edit(previous, text_changed);
        let mut cursor_range = output.cursor_range;
        let mut cursor_state_changed = false;
        if !removed_prefixes.is_empty() {
            if let Some(mut cursor) = cursor_range {
                for removed in &removed_prefixes {
                    cursor.primary.index = adjust_index(cursor.primary.index, &removed.range);
                    cursor.secondary.index = adjust_index(cursor.secondary.index, &removed.range);
                }
                output.state.cursor.set_char_range(Some(cursor));
                cursor_range = Some(cursor);
                cursor_state_changed = true;
            }
        }
        if let Some(range) = self.pending_selection.take() {
            let cursor = egui::text::CCursorRange::two(
                egui::text::CCursor::new(range.start),
                egui::text::CCursor::new(range.end),
            );
            output.state.cursor.set_char_range(Some(cursor));
            output.response.request_focus();
            cursor_range = Some(cursor);
            cursor_state_changed = true;
        }
        if cursor_state_changed {
            output.state.store(ui.ctx(), output.response.id);
        }

        let remove_heading = !text_changed && output.response.has_focus() && backspace_pressed;
        let mut heading_removed = false;
        let mut activated_link = None;
        if let Some(cursor) = cursor_range {
            let range = cursor.as_sorted_char_range();
            self.cursor_index = Some(cursor.primary.index);
            self.selection = (!range.is_empty()).then_some(range.clone());
            if remove_heading && range.is_empty() {
                heading_removed = self.remove_heading_at_cursor(range.start);
            }
            if output.response.double_clicked() {
                let position = range.start.min(self.text.chars().count().saturating_sub(1));
                activated_link = self.target_at(position).cloned();
            }
        }

        EditorOutput {
            changed: text_changed || heading_removed,
            activated_link,
        }
    }

    fn remove_heading_at_cursor(&mut self, character_index: usize) -> bool {
        let Some(index) = self
            .headings
            .iter()
            .position(|heading| heading.range.start == character_index)
        else {
            return false;
        };
        self.headings.remove(index);
        self.source_markdown = self.serialize();
        true
    }

    fn finish_edit(&mut self, previous: String, changed: bool) -> Vec<RemovedPrefix> {
        if changed {
            let inserted_range = changed_character_ranges(&previous, &self.text).1;
            reconcile_ranges(
                &previous,
                &self.text,
                &mut self.links,
                &mut self.styles,
                &mut self.headings,
            );
            if let Some(pixels) = self.active_font_size {
                self.apply_font_size_to_inserted_text(inserted_range, pixels);
            }
            let mut removed_prefixes = self.apply_live_heading_shortcuts();
            removed_prefixes.extend(self.apply_live_inline_shortcuts());
            self.source_markdown = self.serialize();
            return removed_prefixes;
        }
        Vec::new()
    }

    fn apply_font_size_to_inserted_text(&mut self, range: Range<usize>, pixels: u16) {
        if range.is_empty() {
            return;
        }
        self.styles.retain(|span| {
            !(matches!(span.style, InlineStyle::FontSize { .. })
                && ranges_overlap(&span.range, &range))
        });
        let characters = self.text.chars().collect::<Vec<_>>();
        let mut segment_start = None;
        for index in range.clone() {
            if characters.get(index) == Some(&'\n') {
                if let Some(start) = segment_start.take() {
                    self.styles.push(StyleSpan {
                        range: start..index,
                        style: InlineStyle::FontSize { pixels },
                    });
                }
            } else if segment_start.is_none() {
                segment_start = Some(index);
            }
        }
        if let Some(start) = segment_start {
            self.styles.push(StyleSpan {
                range: start..range.end,
                style: InlineStyle::FontSize { pixels },
            });
        }
        self.styles
            .sort_by_key(|span| (span.range.start, span.range.end));
    }

    /// Typing `# ` through `###### ` at the beginning of a paragraph removes
    /// the source marker and turns the current visual line into a heading.
    fn apply_live_heading_shortcuts(&mut self) -> Vec<RemovedPrefix> {
        let mut shortcuts = detect_live_heading_shortcuts(&self.text, &self.headings);
        let mut removed = Vec::with_capacity(shortcuts.len());
        shortcuts.reverse();

        for (line_start, prefix_len, level) in shortcuts {
            let before = self.text.clone();
            remove_char_range(&mut self.text, line_start..line_start + prefix_len);
            reconcile_ranges(
                &before,
                &self.text,
                &mut self.links,
                &mut self.styles,
                &mut self.headings,
            );

            let line_end = self.text[line_start_byte(&self.text, line_start)..]
                .chars()
                .position(|character| character == '\n')
                .map_or_else(|| self.text.chars().count(), |length| line_start + length);
            self.headings.push(HeadingSpan {
                range: line_start..line_end,
                level,
            });
            removed.push(RemovedPrefix {
                range: line_start..line_start + prefix_len,
            });
        }

        self.headings.sort_by_key(|heading| heading.range.start);
        removed
    }

    /// Converts completed inline Markdown/HTML typed into the visible buffer
    /// into range metadata. Incomplete or unsupported syntax remains literal.
    fn apply_live_inline_shortcuts(&mut self) -> Vec<RemovedPrefix> {
        let source = self.text.clone();
        let mut visible = String::new();
        let mut parsed_links = Vec::new();
        let mut parsed_styles = Vec::new();
        let mut marker_ranges = Vec::new();
        parse_inline_tracking(
            &source,
            0,
            &mut visible,
            &mut parsed_links,
            &mut parsed_styles,
            &mut marker_ranges,
        );
        if marker_ranges.is_empty() {
            return Vec::new();
        }

        marker_ranges.sort_by_key(|range| std::cmp::Reverse(range.start));
        marker_ranges.dedup();
        for range in &marker_ranges {
            let before = self.text.clone();
            remove_char_range(&mut self.text, range.clone());
            reconcile_ranges(
                &before,
                &self.text,
                &mut self.links,
                &mut self.styles,
                &mut self.headings,
            );
        }
        debug_assert_eq!(self.text, visible);

        for link in parsed_links {
            self.links
                .retain(|existing| !ranges_overlap(&existing.range, &link.range));
            self.styles
                .retain(|existing| !ranges_cross(&existing.range, &link.range));
            self.links.push(link);
        }
        for style in parsed_styles {
            self.styles
                .retain(|existing| !ranges_cross(&existing.range, &style.range));
            if !self
                .styles
                .iter()
                .any(|existing| existing.range == style.range && existing.style == style.style)
            {
                self.styles.push(style);
            }
        }
        self.links.sort_by_key(|span| span.range.start);
        self.styles
            .sort_by_key(|span| (span.range.start, span.range.end));

        marker_ranges
            .into_iter()
            .map(|range| RemovedPrefix { range })
            .collect()
    }

    pub fn link_selection(&mut self, target: LinkTarget) -> Result<String, &'static str> {
        let Some(range) = self.selection.clone() else {
            return Err("Select text in the dictation editor first");
        };
        if range.is_empty() || range.end > self.text.chars().count() {
            return Err("Select one or more characters to link");
        }
        self.links
            .retain(|span| !ranges_overlap(&span.range, &range));
        self.styles
            .retain(|span| !ranges_cross(&span.range, &range));
        self.links.push(LinkSpan { range, target });
        self.links.sort_by_key(|span| span.range.start);
        self.source_markdown = self.serialize();
        Ok(self.source_markdown.clone())
    }

    pub fn toggle_selection_format(
        &mut self,
        command: TextFormatCommand,
    ) -> Result<bool, &'static str> {
        if let TextFormatCommand::FontSize(pixels) = command {
            if !(8..=72).contains(&pixels) {
                return Err("Font size must be between 8 and 72 pixels");
            }
            self.active_font_size = Some(pixels);
        }
        self.restore_focus = true;
        let Some(range) = self.selection.clone() else {
            if matches!(command, TextFormatCommand::FontSize(_)) {
                return Ok(false);
            }
            return Err("Select text in the dictation editor first");
        };
        if range.is_empty() || range.end > self.text.chars().count() {
            return Err("Select one or more characters to format");
        }
        let format_ranges = non_newline_ranges(&self.text, &range);
        if format_ranges.is_empty() {
            return Err("Select one or more visible characters to format");
        }

        let desired_styles = match command {
            TextFormatCommand::Underline => vec![InlineStyle::Underline],
            TextFormatCommand::Blue => vec![InlineStyle::Color {
                css: "blue".into(),
                color: Some(Color32::BLUE),
            }],
            TextFormatCommand::Red => vec![InlineStyle::Color {
                css: "red".into(),
                color: Some(Color32::RED),
            }],
            TextFormatCommand::Bold => vec![InlineStyle::Strong],
            TextFormatCommand::OrangeCrossline => vec![
                InlineStyle::Color {
                    css: "orange".into(),
                    color: Some(Color32::from_rgb(230, 126, 34)),
                },
                InlineStyle::Strikethrough,
            ],
            TextFormatCommand::FontSize(pixels) => {
                vec![InlineStyle::FontSize { pixels }]
            }
        };
        let already_applied = desired_styles.iter().all(|style| {
            format_ranges.iter().all(|format_range| {
                self.styles
                    .iter()
                    .any(|span| span.range == *format_range && span.style == *style)
            })
        });
        self.styles.retain(|span| {
            !(desired_styles
                .iter()
                .any(|style| same_style_family(&span.style, style))
                && ranges_overlap(&span.range, &range))
        });
        if !already_applied {
            self.styles
                .extend(desired_styles.into_iter().flat_map(|style| {
                    format_ranges.iter().cloned().map(move |range| StyleSpan {
                        range,
                        style: style.clone(),
                    })
                }));
            self.styles
                .sort_by_key(|span| (span.range.start, span.range.end));
        }
        self.source_markdown = self.serialize();
        Ok(true)
    }

    pub fn set_font_size(&mut self, pixels: u16) -> Result<bool, &'static str> {
        self.toggle_selection_format(TextFormatCommand::FontSize(pixels))
    }

    pub fn set_selection_heading(&mut self, level: usize) -> Result<bool, &'static str> {
        if level > 6 {
            return Err("Heading level must be between 0 and 6");
        }
        self.restore_focus = true;
        let selection = self.selection.clone().or_else(|| {
            self.cursor_index
                .map(|character_index| character_index..character_index)
        });
        let Some(selection) = selection else {
            return Err("Click in the dictation editor or select one or more lines first");
        };
        if selection.end > self.text.chars().count() {
            return Err("Select one or more lines to style");
        }

        let lines = selected_line_ranges(&self.text, &selection);
        if level > 0 {
            self.active_font_size = None;
            self.styles.retain(|span| {
                !(matches!(span.style, InlineStyle::FontSize { .. })
                    && lines.iter().any(|line| ranges_overlap(&span.range, line)))
            });
        }
        self.headings
            .retain(|heading| !lines.iter().any(|line| heading.range.start == line.start));
        if level > 0 {
            self.headings
                .extend(lines.into_iter().map(|range| HeadingSpan { range, level }));
            self.headings.sort_by_key(|heading| heading.range.start);
        }
        self.source_markdown = self.serialize();
        Ok(true)
    }

    pub fn selected_text(&self) -> Result<String, &'static str> {
        let Some(range) = self.selection.as_ref() else {
            return Err("Select text in the dictation editor first");
        };
        if range.is_empty() || range.end > self.text.chars().count() {
            return Err("Select one or more characters");
        }
        Ok(self
            .text
            .chars()
            .skip(range.start)
            .take(range.len())
            .collect())
    }

    pub fn cut_selection(&mut self) -> Result<String, &'static str> {
        let selected = self.selected_text()?;
        let range = self
            .selection
            .take()
            .expect("selected_text validated the selection");
        let previous = self.text.clone();
        remove_char_range(&mut self.text, range);
        reconcile_ranges(
            &previous,
            &self.text,
            &mut self.links,
            &mut self.styles,
            &mut self.headings,
        );
        self.source_markdown = self.serialize();
        Ok(selected)
    }

    pub fn find_next(&mut self, query: &str) -> Result<bool, &'static str> {
        if query.is_empty() {
            return Err("Enter text to find");
        }

        let character_count = self.text.chars().count();
        let start = self
            .selection
            .as_ref()
            .map_or(0, |range| range.end.min(character_count));
        let start_byte = line_start_byte(&self.text, start);
        let found_byte = self.text[start_byte..]
            .find(query)
            .map(|offset| start_byte + offset)
            .or_else(|| self.text[..start_byte].find(query));
        let Some(found_byte) = found_byte else {
            return Ok(false);
        };
        let found_start = char_offset(&self.text, found_byte);
        let range = found_start..found_start + query.chars().count();
        self.selection = Some(range.clone());
        self.pending_selection = Some(range);
        Ok(true)
    }

    pub fn replace_current(
        &mut self,
        query: &str,
        replacement: &str,
    ) -> Result<bool, &'static str> {
        if query.is_empty() {
            return Err("Enter text to find");
        }
        let selected_matches = self
            .selection
            .as_ref()
            .and_then(|range| char_slice(&self.text, range.clone()))
            .is_some_and(|selected| selected == query);
        if !selected_matches {
            let _ = self.find_next(query)?;
            return Ok(false);
        }

        let range = self.selection.take().expect("selection was validated");
        let previous = self.text.clone();
        replace_char_range(&mut self.text, range.clone(), replacement);
        reconcile_ranges(
            &previous,
            &self.text,
            &mut self.links,
            &mut self.styles,
            &mut self.headings,
        );
        let replacement_end = range.start + replacement.chars().count();
        self.selection = Some(replacement_end..replacement_end);
        self.source_markdown = self.serialize();
        let _ = self.find_next(query)?;
        Ok(true)
    }

    pub fn replace_all(&mut self, query: &str, replacement: &str) -> Result<usize, &'static str> {
        if query.is_empty() {
            return Err("Enter text to find");
        }
        let mut ranges = self
            .text
            .match_indices(query)
            .map(|(byte, _)| {
                let start = char_offset(&self.text, byte);
                start..start + query.chars().count()
            })
            .collect::<Vec<_>>();
        let count = ranges.len();
        ranges.reverse();
        for range in ranges {
            let previous = self.text.clone();
            replace_char_range(&mut self.text, range, replacement);
            reconcile_ranges(
                &previous,
                &self.text,
                &mut self.links,
                &mut self.styles,
                &mut self.headings,
            );
        }
        self.selection = None;
        self.pending_selection = None;
        self.source_markdown = self.serialize();
        Ok(count)
    }

    pub fn request_paste(&self, ctx: &egui::Context) -> Result<(), &'static str> {
        let Some(id) = self.text_edit_id else {
            return Err("Click in the dictation editor before pasting");
        };
        ctx.memory_mut(|memory| memory.request_focus(id));
        ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste);
        Ok(())
    }

    pub fn markdown(&self) -> &str {
        &self.source_markdown
    }

    pub fn source_mode(&self) -> bool {
        self.source_mode
    }

    pub fn set_source_mode(&mut self, source_mode: bool) {
        if self.source_mode == source_mode {
            return;
        }
        if source_mode {
            self.source_markdown = self.serialize();
            self.selection = None;
            self.source_mode = true;
        } else {
            let document_key = self.document_key.clone();
            let source_markdown = self.source_markdown.clone();
            let active_font_size = self.active_font_size;
            *self = Self::from_markdown(document_key, &source_markdown);
            self.active_font_size = active_font_size;
        }
    }

    fn target_at(&self, character_index: usize) -> Option<&LinkTarget> {
        self.links
            .iter()
            .find(|span| span.range.contains(&character_index))
            .map(|span| &span.target)
    }

    fn serialize(&self) -> String {
        let characters: Vec<char> = self.text.chars().collect();
        let mut result = String::new();
        for index in 0..=characters.len() {
            let mut closing: Vec<Marker> = self
                .markers()
                .into_iter()
                .filter(|marker| marker.range().end == index)
                .collect();
            closing.sort_by(|left, right| {
                right
                    .range()
                    .start
                    .cmp(&left.range().start)
                    .then_with(|| left.close_rank().cmp(&right.close_rank()))
            });
            for marker in closing {
                result.push_str(&marker.close());
            }

            if let Some(heading) = self.headings.iter().find(|span| span.range.start == index) {
                result.push_str(&"#".repeat(heading.level));
                result.push(' ');
            }

            let mut opening: Vec<Marker> = self
                .markers()
                .into_iter()
                .filter(|marker| marker.range().start == index)
                .collect();
            opening.sort_by(|left, right| {
                right
                    .range()
                    .end
                    .cmp(&left.range().end)
                    .then_with(|| left.open_rank().cmp(&right.open_rank()))
            });
            for marker in opening {
                result.push_str(&marker.open());
            }

            if let Some(character) = characters.get(index) {
                result.push(*character);
            }
        }
        result
    }

    fn markers(&self) -> Vec<Marker<'_>> {
        self.links
            .iter()
            .map(Marker::Link)
            .chain(self.styles.iter().map(Marker::Style))
            .collect()
    }
}

#[derive(Clone, Copy)]
enum Marker<'a> {
    Link(&'a LinkSpan),
    Style(&'a StyleSpan),
}

impl Marker<'_> {
    fn range(&self) -> &Range<usize> {
        match self {
            Self::Link(span) => &span.range,
            Self::Style(span) => &span.range,
        }
    }

    fn open(&self) -> String {
        match self {
            Self::Link(_) => "[".to_owned(),
            Self::Style(span) => match &span.style {
                InlineStyle::Strong => "**".to_owned(),
                InlineStyle::Emphasis => "*".to_owned(),
                InlineStyle::Code => "`".to_owned(),
                InlineStyle::Strikethrough => "~~".to_owned(),
                InlineStyle::Underline => "<u>".to_owned(),
                InlineStyle::Color { css, .. } => {
                    format!("<span style=\"color:{css}\">")
                }
                InlineStyle::FontSize { pixels } => {
                    format!("<span style=\"font-size:{pixels}px\">")
                }
            },
        }
    }

    fn close(&self) -> String {
        match self {
            Self::Link(span) => format!("]({})", target_uri(&span.target)),
            Self::Style(span) => match &span.style {
                InlineStyle::Underline => "</u>".to_owned(),
                InlineStyle::Color { .. } | InlineStyle::FontSize { .. } => "</span>".to_owned(),
                _ => self.open(),
            },
        }
    }

    fn open_rank(&self) -> u8 {
        match self {
            Self::Link(_) => 0,
            Self::Style(span) => match &span.style {
                InlineStyle::Color { .. } => 1,
                InlineStyle::FontSize { .. } => 2,
                InlineStyle::Underline => 3,
                InlineStyle::Strikethrough => 4,
                InlineStyle::Strong => 5,
                InlineStyle::Emphasis => 6,
                InlineStyle::Code => 7,
            },
        }
    }

    fn close_rank(&self) -> u8 {
        u8::MAX - self.open_rank()
    }
}

fn parse_document(
    markdown: &str,
    text: &mut String,
    links: &mut Vec<LinkSpan>,
    styles: &mut Vec<StyleSpan>,
    headings: &mut Vec<HeadingSpan>,
) {
    for raw_line in markdown.split_inclusive('\n') {
        let has_newline = raw_line.ends_with('\n');
        let line = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let (level, content) = heading_content(line);
        let start = text.chars().count();
        parse_inline(content, text, links, styles);
        let end = text.chars().count();
        if level > 0 && start < end {
            headings.push(HeadingSpan {
                range: start..end,
                level,
            });
        }
        if has_newline {
            text.push('\n');
        }
    }
}

fn normalize_legacy_multiline_spans(markdown: &str) -> String {
    let mut normalized = String::with_capacity(markdown.len());
    let mut remaining = markdown;

    while let Some(open_start) = remaining.find("<span") {
        normalized.push_str(&remaining[..open_start]);
        let candidate = &remaining[open_start..];
        let Some(open_end) = candidate.find('>').map(|index| index + 1) else {
            normalized.push_str(candidate);
            return normalized;
        };
        let opening = &candidate[..open_end];
        let supported = font_size_span(opening).is_some() || color_span(opening).is_some();
        let Some((close_start, close_end)) = matching_span_close(candidate, open_end) else {
            normalized.push_str(candidate);
            return normalized;
        };
        let inner = &candidate[open_end..close_start];
        let normalized_inner = normalize_legacy_multiline_spans(inner);

        if supported && normalized_inner.contains('\n') {
            for line in normalized_inner.split_inclusive('\n') {
                let has_newline = line.ends_with('\n');
                let content = line.strip_suffix('\n').unwrap_or(line);
                if !content.is_empty() {
                    normalized.push_str(opening);
                    normalized.push_str(content);
                    normalized.push_str("</span>");
                }
                if has_newline {
                    normalized.push('\n');
                }
            }
        } else {
            normalized.push_str(opening);
            normalized.push_str(&normalized_inner);
            normalized.push_str("</span>");
        }
        remaining = &candidate[close_end..];
    }
    normalized.push_str(remaining);
    normalized
}

fn matching_span_close(source: &str, open_end: usize) -> Option<(usize, usize)> {
    let mut depth = 1_usize;
    let mut cursor = open_end;
    loop {
        let next_open = source[cursor..].find("<span").map(|offset| cursor + offset);
        let next_close = source[cursor..]
            .find("</span>")
            .map(|offset| cursor + offset)?;
        if next_open.is_some_and(|open| open < next_close) {
            depth += 1;
            cursor = next_open.expect("checked nested opening") + "<span".len();
            continue;
        }
        depth -= 1;
        let close_end = next_close + "</span>".len();
        if depth == 0 {
            return Some((next_close, close_end));
        }
        cursor = close_end;
    }
}

fn heading_content(line: &str) -> (usize, &str) {
    let count = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if (1..=6).contains(&count) && line.as_bytes().get(count) == Some(&b' ') {
        (count, &line[count + 1..])
    } else {
        (0, line)
    }
}

fn detect_live_heading_shortcuts(
    text: &str,
    headings: &[HeadingSpan],
) -> Vec<(usize, usize, usize)> {
    let characters: Vec<char> = text.chars().collect();
    let mut shortcuts = Vec::new();
    let mut line_start = 0;

    while line_start <= characters.len() {
        let line_end = characters[line_start..]
            .iter()
            .position(|character| *character == '\n')
            .map_or(characters.len(), |length| line_start + length);
        let already_a_heading = headings
            .iter()
            .any(|heading| heading.range.start == line_start);

        if !already_a_heading {
            let level = characters[line_start..line_end]
                .iter()
                .take_while(|character| **character == '#')
                .count();
            if (1..=6).contains(&level) && characters.get(line_start + level) == Some(&' ') {
                shortcuts.push((line_start, level + 1, level));
            }
        }

        if line_end == characters.len() {
            break;
        }
        line_start = line_end + 1;
    }

    shortcuts
}

fn remove_char_range(text: &mut String, range: Range<usize>) {
    let byte_start = line_start_byte(text, range.start);
    let byte_end = line_start_byte(text, range.end);
    text.replace_range(byte_start..byte_end, "");
}

fn replace_char_range(text: &mut String, range: Range<usize>, replacement: &str) {
    let byte_start = line_start_byte(text, range.start);
    let byte_end = line_start_byte(text, range.end);
    text.replace_range(byte_start..byte_end, replacement);
}

fn char_slice(text: &str, range: Range<usize>) -> Option<&str> {
    if range.start > range.end || range.end > text.chars().count() {
        return None;
    }
    let byte_start = line_start_byte(text, range.start);
    let byte_end = line_start_byte(text, range.end);
    text.get(byte_start..byte_end)
}

fn non_newline_ranges(text: &str, range: &Range<usize>) -> Vec<Range<usize>> {
    let characters = text.chars().collect::<Vec<_>>();
    let mut ranges = Vec::new();
    let mut segment_start = None;
    for index in range.clone() {
        if characters.get(index) == Some(&'\n') {
            if let Some(start) = segment_start.take() {
                ranges.push(start..index);
            }
        } else if segment_start.is_none() {
            segment_start = Some(index);
        }
    }
    if let Some(start) = segment_start {
        ranges.push(start..range.end);
    }
    ranges
}

fn line_start_byte(text: &str, character_index: usize) -> usize {
    text.char_indices()
        .nth(character_index)
        .map_or(text.len(), |(byte_index, _)| byte_index)
}

fn adjust_index(index: usize, removed: &Range<usize>) -> usize {
    if index <= removed.start {
        index
    } else if index < removed.end {
        removed.start
    } else {
        index - removed.len()
    }
}

fn selected_line_ranges(text: &str, selection: &Range<usize>) -> Vec<Range<usize>> {
    let characters: Vec<char> = text.chars().collect();
    if selection.is_empty() {
        let line_start = characters[..selection.start]
            .iter()
            .rposition(|character| *character == '\n')
            .map_or(0, |position| position + 1);
        let line_end = characters[selection.start..]
            .iter()
            .position(|character| *character == '\n')
            .map_or(characters.len(), |length| selection.start + length);
        return std::iter::once(line_start..line_end).collect();
    }
    let mut line_start = characters[..selection.start]
        .iter()
        .rposition(|character| *character == '\n')
        .map_or(0, |position| position + 1);
    let last_selected = selection.end.saturating_sub(1);
    let mut lines = Vec::new();

    while line_start <= last_selected && line_start <= characters.len() {
        let line_end = characters[line_start..]
            .iter()
            .position(|character| *character == '\n')
            .map_or(characters.len(), |length| line_start + length);
        lines.push(line_start..line_end);
        if line_end == characters.len() {
            break;
        }
        line_start = line_end + 1;
    }
    lines
}

fn parse_inline(
    source: &str,
    text: &mut String,
    links: &mut Vec<LinkSpan>,
    styles: &mut Vec<StyleSpan>,
) {
    parse_inline_tracking(source, 0, text, links, styles, &mut Vec::new());
}

fn parse_inline_tracking(
    source: &str,
    source_char_base: usize,
    text: &mut String,
    links: &mut Vec<LinkSpan>,
    styles: &mut Vec<StyleSpan>,
    marker_ranges: &mut Vec<Range<usize>>,
) {
    let mut cursor = 0;
    while cursor < source.len() {
        if let Some((opening_len, pixels)) = font_size_span(&source[cursor..]) {
            let content_start = cursor + opening_len;
            if let Some((close_start, close_end)) =
                matching_span_close(&source[cursor..], opening_len)
            {
                let content_end = cursor + close_start;
                let start = text.chars().count();
                parse_inline_tracking(
                    &source[content_start..content_end],
                    source_char_base + char_offset(source, content_start),
                    text,
                    links,
                    styles,
                    marker_ranges,
                );
                let end = text.chars().count();
                styles.push(StyleSpan {
                    range: start..end,
                    style: InlineStyle::FontSize { pixels },
                });
                let closing_end = cursor + close_end;
                marker_ranges.push(source_char_range(
                    source,
                    source_char_base,
                    cursor..content_start,
                ));
                marker_ranges.push(source_char_range(
                    source,
                    source_char_base,
                    content_end..closing_end,
                ));
                cursor = closing_end;
                continue;
            }
        }

        if let Some((opening_len, css, color)) = color_span(&source[cursor..]) {
            let content_start = cursor + opening_len;
            if let Some((close_start, close_end)) =
                matching_span_close(&source[cursor..], opening_len)
            {
                let content_end = cursor + close_start;
                let start = text.chars().count();
                parse_inline_tracking(
                    &source[content_start..content_end],
                    source_char_base + char_offset(source, content_start),
                    text,
                    links,
                    styles,
                    marker_ranges,
                );
                let end = text.chars().count();
                styles.push(StyleSpan {
                    range: start..end,
                    style: InlineStyle::Color { css, color },
                });
                let closing_end = cursor + close_end;
                marker_ranges.push(source_char_range(
                    source,
                    source_char_base,
                    cursor..content_start,
                ));
                marker_ranges.push(source_char_range(
                    source,
                    source_char_base,
                    content_end..closing_end,
                ));
                cursor = closing_end;
                continue;
            }
        }

        if source[cursor..].starts_with("<u>") {
            let content_start = cursor + "<u>".len();
            if let Some(end_rel) = source[content_start..].find("</u>") {
                let content_end = content_start + end_rel;
                let start = text.chars().count();
                parse_inline_tracking(
                    &source[content_start..content_end],
                    source_char_base + char_offset(source, content_start),
                    text,
                    links,
                    styles,
                    marker_ranges,
                );
                let end = text.chars().count();
                styles.push(StyleSpan {
                    range: start..end,
                    style: InlineStyle::Underline,
                });
                let closing_end = content_end + "</u>".len();
                marker_ranges.push(source_char_range(
                    source,
                    source_char_base,
                    cursor..content_start,
                ));
                marker_ranges.push(source_char_range(
                    source,
                    source_char_base,
                    content_end..closing_end,
                ));
                cursor = closing_end;
                continue;
            }
        }

        if source[cursor..].starts_with('[') {
            if let Some(label_end_rel) = source[cursor + 1..].find("](") {
                let label_end = cursor + 1 + label_end_rel;
                let uri_start = label_end + 2;
                if let Some(uri_end_rel) = source[uri_start..].find(')') {
                    let uri_end = uri_start + uri_end_rel;
                    if let Some(target) = parse_target(&source[uri_start..uri_end]) {
                        let start = text.chars().count();
                        parse_inline_tracking(
                            &source[cursor + 1..label_end],
                            source_char_base + char_offset(source, cursor + 1),
                            text,
                            links,
                            styles,
                            marker_ranges,
                        );
                        let end = text.chars().count();
                        links.push(LinkSpan {
                            range: start..end,
                            target,
                        });
                        let closing_end = uri_end + 1;
                        marker_ranges.push(source_char_range(
                            source,
                            source_char_base,
                            cursor..cursor + 1,
                        ));
                        marker_ranges.push(source_char_range(
                            source,
                            source_char_base,
                            label_end..closing_end,
                        ));
                        cursor = closing_end;
                        continue;
                    }
                }
            }
        }

        let style_marker = if source[cursor..].starts_with("**") {
            Some(("**", InlineStyle::Strong))
        } else if source[cursor..].starts_with("~~") {
            Some(("~~", InlineStyle::Strikethrough))
        } else if source[cursor..].starts_with('*') {
            Some(("*", InlineStyle::Emphasis))
        } else if source[cursor..].starts_with('`') {
            Some(("`", InlineStyle::Code))
        } else {
            None
        };
        if let Some((marker, style)) = style_marker {
            let content_start = cursor + marker.len();
            if let Some(end_rel) = source[content_start..].find(marker) {
                let content_end = content_start + end_rel;
                let start = text.chars().count();
                parse_inline_tracking(
                    &source[content_start..content_end],
                    source_char_base + char_offset(source, content_start),
                    text,
                    links,
                    styles,
                    marker_ranges,
                );
                let end = text.chars().count();
                styles.push(StyleSpan {
                    range: start..end,
                    style,
                });
                let closing_end = content_end + marker.len();
                marker_ranges.push(source_char_range(
                    source,
                    source_char_base,
                    cursor..content_start,
                ));
                marker_ranges.push(source_char_range(
                    source,
                    source_char_base,
                    content_end..closing_end,
                ));
                cursor = closing_end;
                continue;
            }
        }

        let character = source[cursor..].chars().next().expect("valid character");
        text.push(character);
        cursor += character.len_utf8();
    }
}

fn char_offset(source: &str, byte_offset: usize) -> usize {
    source[..byte_offset].chars().count()
}

fn source_char_range(
    source: &str,
    source_char_base: usize,
    byte_range: Range<usize>,
) -> Range<usize> {
    source_char_base + char_offset(source, byte_range.start)
        ..source_char_base + char_offset(source, byte_range.end)
}

fn color_span(source: &str) -> Option<(usize, String, Option<Color32>)> {
    if !source.starts_with("<span") {
        return None;
    }
    let tag_end = source.find('>')?;
    let opening = &source[..=tag_end];
    let lower = opening.to_ascii_lowercase();
    let style_start = lower.find("style=")? + "style=".len();
    let quote = opening[style_start..].chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let value_start = style_start + quote.len_utf8();
    let value_end = opening[value_start..].find(quote)? + value_start;
    let style = &opening[value_start..value_end];
    let css = style.split(';').find_map(|declaration| {
        let (property, value) = declaration.split_once(':')?;
        property
            .trim()
            .eq_ignore_ascii_case("color")
            .then(|| value.trim().to_owned())
    })?;
    let color = parse_css_color(&css);
    Some((tag_end + 1, css, color))
}

fn font_size_span(source: &str) -> Option<(usize, u16)> {
    if !source.starts_with("<span") {
        return None;
    }
    let tag_end = source.find('>')?;
    let opening = &source[..=tag_end];
    let lower = opening.to_ascii_lowercase();
    let style_start = lower.find("style=")? + "style=".len();
    let quote = opening[style_start..].chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let value_start = style_start + quote.len_utf8();
    let value_end = opening[value_start..].find(quote)? + value_start;
    let style = &opening[value_start..value_end];
    let pixels = style.split(';').find_map(|declaration| {
        let (property, value) = declaration.split_once(':')?;
        if !property.trim().eq_ignore_ascii_case("font-size") {
            return None;
        }
        value
            .trim()
            .to_ascii_lowercase()
            .strip_suffix("px")?
            .trim()
            .parse::<u16>()
            .ok()
    })?;
    (8..=72).contains(&pixels).then_some((tag_end + 1, pixels))
}

fn parse_css_color(css: &str) -> Option<Color32> {
    let normalized = css.trim().to_ascii_lowercase();
    let named = match normalized.as_str() {
        "black" => Some(Color32::BLACK),
        "white" => Some(Color32::WHITE),
        "red" => Some(Color32::RED),
        "orange" => Some(Color32::from_rgb(230, 126, 34)),
        "green" => Some(Color32::GREEN),
        "blue" => Some(Color32::BLUE),
        "yellow" => Some(Color32::YELLOW),
        "gray" | "grey" => Some(Color32::GRAY),
        "lightgray" | "lightgrey" => Some(Color32::LIGHT_GRAY),
        "darkgray" | "darkgrey" => Some(Color32::DARK_GRAY),
        "transparent" => Some(Color32::TRANSPARENT),
        _ => None,
    };
    named.or_else(|| parse_hex_color(&normalized))
}

fn parse_hex_color(css: &str) -> Option<Color32> {
    let hex = css.strip_prefix('#')?;
    let expanded;
    let hex = if hex.len() == 3 || hex.len() == 4 {
        expanded = hex
            .chars()
            .flat_map(|character| [character, character])
            .collect::<String>();
        expanded.as_str()
    } else {
        hex
    };
    match hex.len() {
        6 => Some(Color32::from_rgb(
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
        )),
        8 => Some(Color32::from_rgba_unmultiplied(
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            u8::from_str_radix(&hex[6..8], 16).ok()?,
        )),
        _ => None,
    }
}

fn reconcile_ranges(
    old: &str,
    new: &str,
    links: &mut Vec<LinkSpan>,
    styles: &mut Vec<StyleSpan>,
    headings: &mut Vec<HeadingSpan>,
) {
    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();
    let prefix = old_chars
        .iter()
        .zip(&new_chars)
        .take_while(|(left, right)| left == right)
        .count();
    let suffix = old_chars[prefix..]
        .iter()
        .rev()
        .zip(new_chars[prefix..].iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    let old_change = prefix..old_chars.len() - suffix;
    let new_end = new_chars.len() - suffix;

    update_ranges(links, &old_change, new_end, |span| &mut span.range);
    update_ranges(styles, &old_change, new_end, |span| &mut span.range);
    update_heading_ranges(headings, &old_change, new_end);
    normalize_heading_ranges(new, headings);
}

fn changed_character_ranges(old: &str, new: &str) -> (Range<usize>, Range<usize>) {
    let old_chars = old.chars().collect::<Vec<_>>();
    let new_chars = new.chars().collect::<Vec<_>>();
    let prefix = old_chars
        .iter()
        .zip(&new_chars)
        .take_while(|(left, right)| left == right)
        .count();
    let suffix = old_chars[prefix..]
        .iter()
        .rev()
        .zip(new_chars[prefix..].iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    (
        prefix..old_chars.len() - suffix,
        prefix..new_chars.len() - suffix,
    )
}

fn update_heading_ranges(
    headings: &mut Vec<HeadingSpan>,
    old_change: &Range<usize>,
    new_end: usize,
) {
    let delta = new_end as isize - old_change.end as isize;
    headings.retain_mut(|heading| {
        let range = &mut heading.range;
        let deletion_touches_heading = !old_change.is_empty()
            && if range.start == range.end {
                old_change.start <= range.start && range.start < old_change.end
            } else {
                ranges_overlap(range, old_change)
            };
        if old_change.is_empty() {
            if old_change.start < range.start {
                range.start = shift(range.start, delta);
                range.end = shift(range.end, delta);
            } else if old_change.start <= range.end {
                range.end = shift(range.end, delta);
            }
        } else if range.end <= old_change.start {
            // The edit is after this heading.
        } else if range.start >= old_change.end {
            range.start = shift(range.start, delta);
            range.end = shift(range.end, delta);
        } else {
            if old_change.start < range.start {
                range.start = new_end;
            }
            range.end = if old_change.end < range.end {
                shift(range.end, delta)
            } else {
                new_end
            };
            if range.start > range.end {
                range.start = range.end;
            }
        }
        !(deletion_touches_heading && range.start == range.end)
    });
}

fn normalize_heading_ranges(text: &str, headings: &mut Vec<HeadingSpan>) {
    let characters: Vec<char> = text.chars().collect();
    for heading in headings.iter_mut() {
        let anchor = heading.range.start.min(characters.len());
        let line_start = characters[..anchor]
            .iter()
            .rposition(|character| *character == '\n')
            .map_or(0, |position| position + 1);
        let line_end = characters[anchor..]
            .iter()
            .position(|character| *character == '\n')
            .map_or(characters.len(), |length| anchor + length);
        heading.range = line_start..line_end;
    }
    headings.sort_by_key(|heading| heading.range.start);
    headings.dedup_by_key(|heading| heading.range.start);
}

fn update_ranges<T>(
    spans: &mut Vec<T>,
    old_change: &Range<usize>,
    new_end: usize,
    mut range_of: impl FnMut(&mut T) -> &mut Range<usize>,
) {
    let delta = new_end as isize - old_change.end as isize;
    spans.retain_mut(|span| {
        let range = range_of(span);
        if old_change.is_empty() {
            if old_change.start <= range.start {
                range.start = shift(range.start, delta);
                range.end = shift(range.end, delta);
            } else if old_change.start < range.end {
                range.end = shift(range.end, delta);
            }
            return range.start < range.end;
        }
        if range.end <= old_change.start {
            true
        } else if range.start >= old_change.end {
            range.start = shift(range.start, delta);
            range.end = shift(range.end, delta);
            true
        } else if range.start <= old_change.start && old_change.end <= range.end {
            range.end = shift(range.end, delta);
            range.start < range.end
        } else {
            false
        }
    });
}

fn shift(value: usize, delta: isize) -> usize {
    value.saturating_add_signed(delta)
}

fn ranges_overlap(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

fn ranges_cross(left: &Range<usize>, right: &Range<usize>) -> bool {
    ranges_overlap(left, right)
        && !(left.start <= right.start && right.end <= left.end)
        && !(right.start <= left.start && left.end <= right.end)
}

fn same_style_family(left: &InlineStyle, right: &InlineStyle) -> bool {
    matches!(
        (left, right),
        (InlineStyle::Color { .. }, InlineStyle::Color { .. })
            | (InlineStyle::FontSize { .. }, InlineStyle::FontSize { .. })
    ) || left == right
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{kittest::Queryable as _, Harness};

    #[test]
    fn markdown_is_loaded_as_visible_wysiwyg_text() {
        let editor = MarkdownEditor::from_markdown(
            "lesson".into(),
            "# Dictation\n\nHear [snow](bibi://word/s1/w2) **again**.",
        );
        assert_eq!(editor.text, "Dictation\n\nHear snow again.");
        assert_eq!(editor.links.len(), 1);
        assert_eq!(editor.styles.len(), 1);
        assert_eq!(
            editor.serialize(),
            "# Dictation\n\nHear [snow](bibi://word/s1/w2) **again**."
        );
    }

    #[test]
    fn source_mode_edits_raw_markdown_and_reparses_wysiwyg_text() {
        let mut editor =
            MarkdownEditor::from_markdown("lesson".into(), "# Heading\n<u>underlined</u>");
        editor.set_source_mode(true);
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            editor,
        );

        let field = harness.get_by_role(egui::accesskit::Role::MultilineTextInput);
        assert_eq!(
            field.value().as_deref(),
            Some("# Heading\n<u>underlined</u>")
        );
        field.focus();
        harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        field.type_text("## New heading\n~~changed~~");
        harness.run();
        assert_eq!(harness.state().markdown(), "## New heading\n~~changed~~");

        harness.state_mut().set_source_mode(false);
        harness.run();
        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::MultilineTextInput)
                .value()
                .as_deref(),
            Some("New heading\nchanged")
        );
        assert_eq!(harness.state().markdown(), "## New heading\n~~changed~~");
    }

    #[test]
    fn selection_becomes_a_portable_bibi_link() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "Listen to snow");
        editor.selection = Some(10..14);
        let markdown = editor
            .link_selection(LinkTarget::Sentence {
                sentence_id: "s1".into(),
            })
            .unwrap();
        assert_eq!(markdown, "Listen to [snow](bibi://sentence/s1)");
        assert_eq!(
            editor.target_at(11),
            Some(&LinkTarget::Sentence {
                sentence_id: "s1".into()
            })
        );
    }

    #[test]
    fn typing_before_a_link_keeps_its_metadata_attached() {
        let mut editor =
            MarkdownEditor::from_markdown("lesson".into(), "Hear [snow](bibi://sentence/s1)");
        let old = editor.text.clone();
        editor.text.insert_str(0, "Please ");
        reconcile_ranges(
            &old,
            &editor.text,
            &mut editor.links,
            &mut editor.styles,
            &mut editor.headings,
        );
        assert_eq!(editor.links[0].range, 12..16);
        assert_eq!(editor.serialize(), "Please Hear [snow](bibi://sentence/s1)");
    }

    #[test]
    fn unicode_selection_uses_character_offsets() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "初雪 falls");
        editor.selection = Some(0..2);
        let markdown = editor
            .link_selection(LinkTarget::Sentence {
                sentence_id: "s2".into(),
            })
            .unwrap();
        assert_eq!(markdown, "[初雪](bibi://sentence/s2) falls");
    }

    #[test]
    fn styled_link_serializes_with_valid_nesting() {
        let editor =
            MarkdownEditor::from_markdown("lesson".into(), "[**snow**](bibi://sentence/s1)");
        assert_eq!(editor.text, "snow");
        assert_eq!(editor.serialize(), "[**snow**](bibi://sentence/s1)");
    }

    #[test]
    fn strikethrough_is_loaded_as_visible_text_and_round_trips() {
        let editor =
            MarkdownEditor::from_markdown("lesson".into(), "~~this text has a line through it~~");

        assert_eq!(editor.text, "this text has a line through it");
        assert_eq!(editor.serialize(), "~~this text has a line through it~~");
    }

    #[test]
    fn underline_html_is_loaded_as_visible_text_and_round_trips() {
        let editor =
            MarkdownEditor::from_markdown("lesson".into(), "<u>this text is underlined</u>");

        assert_eq!(editor.text, "this text is underlined");
        assert_eq!(editor.serialize(), "<u>this text is underlined</u>");
    }

    #[test]
    fn underline_toolbar_format_toggles_selected_text() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "snow falls");
        editor.selection = Some(0..4);

        assert!(editor
            .toggle_selection_format(TextFormatCommand::Underline)
            .unwrap());
        assert_eq!(editor.markdown(), "<u>snow</u> falls");

        assert!(editor
            .toggle_selection_format(TextFormatCommand::Underline)
            .unwrap());
        assert_eq!(editor.markdown(), "snow falls");
    }

    #[test]
    fn blue_toolbar_format_replaces_the_selected_text_color() {
        let mut editor = MarkdownEditor::from_markdown(
            "lesson".into(),
            "<span style=\"color:red\">snow</span> falls",
        );
        editor.selection = Some(0..4);

        editor
            .toggle_selection_format(TextFormatCommand::Blue)
            .unwrap();

        assert_eq!(
            editor.markdown(),
            "<span style=\"color:blue\">snow</span> falls"
        );
    }

    #[test]
    fn red_toolbar_format_replaces_the_selected_text_color() {
        let mut editor = MarkdownEditor::from_markdown(
            "lesson".into(),
            "<span style=\"color:blue\">snow</span> falls",
        );
        editor.selection = Some(0..4);

        editor
            .toggle_selection_format(TextFormatCommand::Red)
            .unwrap();

        assert_eq!(
            editor.markdown(),
            "<span style=\"color:red\">snow</span> falls"
        );
    }

    #[test]
    fn bold_toolbar_format_toggles_selected_text() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "snow falls");
        editor.selection = Some(0..4);

        editor
            .toggle_selection_format(TextFormatCommand::Bold)
            .unwrap();

        assert_eq!(editor.markdown(), "**snow** falls");
    }

    #[test]
    fn orange_crossline_toolbar_format_is_atomic() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "snow falls");
        editor.selection = Some(0..4);

        editor
            .toggle_selection_format(TextFormatCommand::OrangeCrossline)
            .unwrap();
        assert_eq!(
            editor.markdown(),
            "<span style=\"color:orange\">~~snow~~</span> falls"
        );

        editor
            .toggle_selection_format(TextFormatCommand::OrangeCrossline)
            .unwrap();
        assert_eq!(editor.markdown(), "snow falls");
    }

    #[test]
    fn font_size_toolbar_round_trips_portable_span_markup() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "snow falls");
        editor.selection = Some(0..4);

        editor
            .toggle_selection_format(TextFormatCommand::FontSize(18))
            .unwrap();
        assert_eq!(
            editor.markdown(),
            "<span style=\"font-size:18px\">snow</span> falls"
        );

        let reparsed = MarkdownEditor::from_markdown("lesson".into(), editor.markdown());
        assert_eq!(reparsed.text, "snow falls");
        assert_eq!(reparsed.serialize(), editor.markdown());
    }

    #[test]
    fn multiline_font_size_does_not_expose_html_tags_as_visible_text() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "first\nsecond");
        editor.selection = Some(0..12);

        editor.set_font_size(18).unwrap();
        assert_eq!(
            editor.markdown(),
            "<span style=\"font-size:18px\">first</span>\n\
             <span style=\"font-size:18px\">second</span>"
        );

        let reparsed = MarkdownEditor::from_markdown("lesson".into(), editor.markdown());
        assert_eq!(reparsed.text, "first\nsecond");
    }

    #[test]
    fn legacy_multiline_font_span_is_repaired_when_loaded() {
        let editor = MarkdownEditor::from_markdown(
            "lesson".into(),
            "<span style=\"font-size:18px\">first\nsecond</span>",
        );

        assert_eq!(editor.text, "first\nsecond");
        assert_eq!(
            editor.serialize(),
            "<span style=\"font-size:18px\">first</span>\n\
             <span style=\"font-size:18px\">second</span>"
        );
    }

    #[test]
    fn nested_legacy_multiline_spans_are_repaired_when_loaded() {
        let editor = MarkdownEditor::from_markdown(
            "lesson".into(),
            "<span style=\"font-size:18px\">first\n\
             <span style=\"color:red\">second\nthird</span>\n\
             fourth</span>",
        );

        assert_eq!(editor.text, "first\nsecond\nthird\nfourth");
        assert!(!editor.text.contains("<span"));
        assert!(!editor.text.contains("</span>"));
        let reparsed = MarkdownEditor::from_markdown("reparsed".into(), &editor.serialize());
        assert_eq!(reparsed.text, editor.text);
    }

    #[test]
    fn chosen_font_size_becomes_the_default_for_new_text() {
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), ""),
        );

        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .focus();
        harness.run();
        assert!(!harness.state_mut().set_font_size(24).unwrap());
        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .type_text("snow");
        harness.run();

        assert_eq!(
            harness.state().markdown(),
            "<span style=\"font-size:24px\">snow</span>"
        );
    }

    #[test]
    fn paragraph_style_toolbar_applies_heading_to_each_selected_line() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "first\nsecond");
        editor.selection = Some(0..12);

        editor.set_selection_heading(2).unwrap();
        assert_eq!(editor.markdown(), "## first\n## second");

        editor.set_selection_heading(0).unwrap();
        assert_eq!(editor.markdown(), "first\nsecond");
    }

    #[test]
    fn heading_style_replaces_conflicting_inline_font_size() {
        let mut editor = MarkdownEditor::from_markdown(
            "lesson".into(),
            "<span style=\"font-size:18px\">Title</span>",
        );
        editor.selection = Some(0..5);

        editor.set_selection_heading(1).unwrap();

        assert_eq!(editor.markdown(), "# Title");
    }

    #[test]
    fn heading_is_rendered_physically_larger_than_body_text() {
        let mut editor = MarkdownEditor::from_markdown(
            "lesson".into(),
            "<span style=\"font-size:18px\">Title</span>\nBody",
        );
        editor.selection = Some(0..5);
        editor.set_selection_heading(1).unwrap();
        let harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            editor,
        );

        let text_runs = harness
            .get_all_by_role(egui::accesskit::Role::TextRun)
            .collect::<Vec<_>>();
        let heading = text_runs
            .iter()
            .find(|node| node.value().is_some_and(|value| value.starts_with("Title")))
            .expect("heading text run");
        let body = text_runs
            .iter()
            .find(|node| node.value().as_deref() == Some("Body"))
            .expect("body text run");
        assert!(heading.rect().height() > body.rect().height());
    }

    #[test]
    fn chosen_heading_style_applies_to_text_typed_on_an_empty_line() {
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), ""),
        );

        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .focus();
        harness.run();
        assert!(harness.state_mut().set_selection_heading(2).unwrap());
        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .type_text("snow");
        harness.run();

        assert_eq!(harness.state().markdown(), "## snow");
    }

    #[test]
    fn heading_style_clears_the_previous_default_pixel_size() {
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), ""),
        );

        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .focus();
        harness.run();
        harness.state_mut().set_font_size(18).unwrap();
        harness.state_mut().set_selection_heading(1).unwrap();
        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .type_text("Title");
        harness.run();

        assert_eq!(harness.state().markdown(), "# Title");
    }

    #[test]
    fn heading_style_can_be_chosen_on_a_new_trailing_line() {
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), "first\n"),
        );

        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .focus();
        harness.run();
        harness.key_press(egui::Key::End);
        harness.run();
        assert!(harness.state_mut().set_selection_heading(3).unwrap());
        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .type_text("snow");
        harness.run();

        assert_eq!(harness.state().markdown(), "first\n### snow");
    }

    #[test]
    fn copy_toolbar_reads_the_unicode_selection() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "初雪 falls");
        editor.selection = Some(0..2);

        assert_eq!(editor.selected_text().unwrap(), "初雪");
    }

    #[test]
    fn cut_toolbar_removes_only_the_selected_text() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "Hear **snow** again");
        editor.selection = Some(5..9);

        assert_eq!(editor.cut_selection().unwrap(), "snow");
        assert_eq!(editor.markdown(), "Hear  again");
    }

    #[test]
    fn search_toolbar_finds_next_and_wraps() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "snow then snow");

        assert!(editor.find_next("snow").unwrap());
        assert_eq!(editor.selection, Some(0..4));
        assert!(editor.find_next("snow").unwrap());
        assert_eq!(editor.selection, Some(10..14));
        assert!(editor.find_next("snow").unwrap());
        assert_eq!(editor.selection, Some(0..4));
    }

    #[test]
    fn replace_toolbar_replaces_current_match_and_selects_next() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "snow then snow");
        editor.find_next("snow").unwrap();

        assert!(editor.replace_current("snow", "rain").unwrap());
        assert_eq!(editor.markdown(), "rain then snow");
        assert_eq!(editor.selection, Some(10..14));
    }

    #[test]
    fn replace_all_toolbar_is_unicode_safe() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "初雪 and 初雪");

        assert_eq!(editor.replace_all("初雪", "snow").unwrap(), 2);
        assert_eq!(editor.markdown(), "snow and snow");
    }

    #[test]
    fn color_html_is_loaded_as_visible_text_and_round_trips() {
        let editor = MarkdownEditor::from_markdown(
            "lesson".into(),
            "<span style=\"color:red\">this text is red</span>",
        );

        assert_eq!(editor.text, "this text is red");
        assert_eq!(
            editor.serialize(),
            "<span style=\"color:red\">this text is red</span>"
        );
    }

    #[test]
    fn new_inline_styles_remain_valid_when_nested() {
        let markdown = "<span style=\"color:red\"><u>~~alert~~</u></span>";
        let editor = MarkdownEditor::from_markdown("lesson".into(), markdown);

        assert_eq!(editor.text, "alert");
        assert_eq!(editor.serialize(), markdown);
    }

    #[test]
    fn backspace_at_heading_start_removes_heading_style() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "## Heading");

        assert!(editor.remove_heading_at_cursor(0));
        assert_eq!(editor.text, "Heading");
        assert_eq!(editor.markdown(), "Heading");
    }

    #[test]
    fn focused_editor_backspace_removes_heading_style() {
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), "## Heading"),
        );

        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .focus();
        harness.run();
        assert!(harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .is_focused());
        harness.key_press(egui::Key::Home);
        harness.run();
        harness.key_press(egui::Key::Backspace);
        harness.run();

        assert_eq!(harness.state().markdown(), "Heading");
    }

    #[test]
    fn backspace_clears_an_empty_heading_created_by_live_shortcut() {
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), ""),
        );

        let field = harness.get_by_role(egui::accesskit::Role::MultilineTextInput);
        field.focus();
        field.type_text("## ");
        harness.run();
        assert_eq!(harness.state().markdown(), "## ");

        harness.key_press(egui::Key::Backspace);
        harness.run();

        assert_eq!(harness.state().markdown(), "");
    }

    #[test]
    fn deleting_an_empty_heading_line_does_not_promote_the_following_line() {
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), "## X\nFollowing line"),
        );

        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .focus();
        harness.run();
        harness.key_press(egui::Key::Home);
        harness.run();
        harness.key_press(egui::Key::ArrowUp);
        harness.run();
        harness.key_press(egui::Key::Home);
        harness.run();
        harness.key_press(egui::Key::Delete);
        harness.run();
        assert_eq!(harness.state().markdown(), "\nFollowing line");

        harness.key_press(egui::Key::Delete);
        harness.run();
        assert_eq!(harness.state().markdown(), "Following line");
    }

    #[test]
    fn backspacing_an_empty_heading_line_does_not_promote_the_following_line() {
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), "## X\nFollowing line"),
        );

        harness
            .get_by_role(egui::accesskit::Role::MultilineTextInput)
            .focus();
        harness.run();
        harness.key_press(egui::Key::Home);
        harness.run();
        harness.key_press(egui::Key::ArrowUp);
        harness.run();
        harness.key_press(egui::Key::End);
        harness.run();
        harness.key_press(egui::Key::Backspace);
        harness.run();
        assert_eq!(harness.state().markdown(), "\nFollowing line");

        harness.key_press(egui::Key::ArrowDown);
        harness.run();
        harness.key_press(egui::Key::Home);
        harness.run();
        harness.key_press(egui::Key::Backspace);
        harness.run();
        assert_eq!(harness.state().markdown(), "Following line");
    }

    #[test]
    fn completed_inline_markdown_becomes_rendered_text_while_typing() {
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), ""),
        );

        let field = harness.get_by_role(egui::accesskit::Role::MultilineTextInput);
        field.focus();
        field.type_text("~~strike~~");
        harness.run();

        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::MultilineTextInput)
                .value()
                .as_deref(),
            Some("strike")
        );
        assert_eq!(harness.state().markdown(), "~~strike~~");
    }

    #[test]
    fn incomplete_inline_markdown_stays_visible_and_lossless() {
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), ""),
        );

        let field = harness.get_by_role(egui::accesskit::Role::MultilineTextInput);
        field.focus();
        field.type_text("~~strike~");
        harness.run();

        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::MultilineTextInput)
                .value()
                .as_deref(),
            Some("~~strike~")
        );
        assert_eq!(harness.state().markdown(), "~~strike~");
    }

    #[test]
    fn completed_inline_html_becomes_rendered_text_while_typing() {
        let markdown = "<u>under</u> <span style=\"color:red\">red</span>";
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), ""),
        );

        let field = harness.get_by_role(egui::accesskit::Role::MultilineTextInput);
        field.focus();
        field.type_text(markdown);
        harness.run();

        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::MultilineTextInput)
                .value()
                .as_deref(),
            Some("under red")
        );
        assert_eq!(harness.state().markdown(), markdown);
    }

    #[test]
    fn live_normalization_preserves_existing_format_metadata() {
        let mut harness = Harness::new_ui_state(
            |ui, editor: &mut MarkdownEditor| {
                editor.show(ui);
            },
            MarkdownEditor::from_markdown("lesson".into(), "**bold** plain"),
        );

        let field = harness.get_by_role(egui::accesskit::Role::MultilineTextInput);
        field.focus();
        field.type_text(" ~~strike~~");
        harness.run();

        assert_eq!(
            harness
                .get_by_role(egui::accesskit::Role::MultilineTextInput)
                .value()
                .as_deref(),
            Some("bold plain strike")
        );
        assert_eq!(harness.state().markdown(), "**bold** plain ~~strike~~");
    }

    #[test]
    fn typed_atx_prefix_becomes_a_wysiwyg_heading_immediately() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "");
        let previous = editor.text.clone();
        editor.text.push_str("## ");

        let removed = editor.finish_edit(previous, true);

        assert_eq!(editor.text, "");
        assert_eq!(editor.headings[0].level, 2);
        assert_eq!(editor.headings[0].range, 0..0);
        assert_eq!(editor.markdown(), "## ");
        assert_eq!(removed[0].range, 0..3);
    }

    #[test]
    fn typing_continues_inside_a_live_heading() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "");
        let previous = editor.text.clone();
        editor.text.push_str("## ");
        editor.finish_edit(previous, true);

        let previous = editor.text.clone();
        editor.text.push_str("Word");
        editor.finish_edit(previous, true);

        assert_eq!(editor.text, "Word");
        assert_eq!(editor.headings[0].range, 0..4);
        assert_eq!(editor.markdown(), "## Word");
    }

    #[test]
    fn pasted_heading_shortcuts_convert_each_line() {
        let mut editor = MarkdownEditor::from_markdown("lesson".into(), "");
        let previous = editor.text.clone();
        editor.text.push_str("## Word\n### Phrase");
        editor.finish_edit(previous, true);

        assert_eq!(editor.text, "Word\nPhrase");
        assert_eq!(editor.headings.len(), 2);
        assert_eq!(editor.headings[0].range, 0..4);
        assert_eq!(editor.headings[1].range, 5..11);
        assert_eq!(editor.markdown(), "## Word\n### Phrase");
    }
}
