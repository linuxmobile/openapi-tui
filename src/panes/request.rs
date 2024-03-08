use std::sync::{Arc, RwLock};

use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use oas3::{spec::RequestBody, Schema};
use ratatui::{
  prelude::*,
  widgets::{block::*, *},
};
use syntect::{
  easy::HighlightLines,
  highlighting::ThemeSet,
  parsing::{SyntaxReference, SyntaxSet},
  util::LinesWithEndings,
};

use crate::{
  action::Action,
  pages::home::State,
  panes::Pane,
  tui::{EventResponse, Frame},
};

#[derive(Default)]
pub struct RequestPane {
  focused: bool,
  focused_border_style: Style,
  state: Arc<RwLock<State>>,
  request_body: Option<RequestBody>,
  request_schema: Option<Schema>,
  request_schema_line_offset: usize,
  request_schema_styles: Vec<Vec<(Style, String)>>,
  highlighter_syntax_set: SyntaxSet,
  highlighter_theme_set: ThemeSet,
}

impl RequestPane {
  pub fn new(state: Arc<RwLock<State>>, focused: bool, focused_border_style: Style) -> Self {
    Self {
      state,
      focused,
      focused_border_style,
      request_body: None,
      request_schema: None,
      request_schema_line_offset: 0,
      request_schema_styles: Vec::default(),
      highlighter_syntax_set: SyntaxSet::load_defaults_newlines(),
      highlighter_theme_set: ThemeSet::load_defaults(),
    }
  }

  fn yaml_syntax(&self) -> &SyntaxReference {
    return self.highlighter_syntax_set.find_syntax_by_extension("yaml").unwrap();
  }

  fn border_style(&self) -> Style {
    match self.focused {
      true => self.focused_border_style,
      false => Style::default(),
    }
  }

  fn border_type(&self) -> BorderType {
    match self.focused {
      true => BorderType::Thick,
      false => BorderType::Plain,
    }
  }

  fn set_request_schema_styles(&mut self) -> Result<()> {
    if let Some(request_schema) = &self.request_schema {
      let yaml_schema = serde_yaml::to_string(&request_schema)?;
      let mut highlighter =
        HighlightLines::new(self.yaml_syntax(), &self.highlighter_theme_set.themes["Solarized (dark)"]);
      for (line_num, line) in LinesWithEndings::from(yaml_schema.as_str()).enumerate() {
        let mut line_styles: Vec<(Style, String)> = highlighter
          .highlight_line(line, &self.highlighter_syntax_set)?
          .into_iter()
          .map(|segment| {
            (
              syntect_tui::translate_style(segment.0)
                .ok()
                .unwrap_or_default()
                .underline_color(Color::Reset)
                .bg(Color::Reset),
              segment.1.to_string(),
            )
          })
          .collect();
        line_styles.insert(0, (Style::default().dim(), format!(" {:<3} ", line_num + 1)));
        self.request_schema_styles.push(line_styles);
      }
    }
    Ok(())
  }

  fn init_request_schema(&mut self) -> Result<()> {
    {
      let state = self.state.read().unwrap();
      self.request_body = None;
      self.request_schema_styles = vec![];
      self.request_schema_line_offset = 0;
      if let Some((_path, _method, operation)) = state.active_operation() {
        if let Some(oor) = &operation.request_body {
          let resolved_oor = oor.resolve(&state.openapi_spec)?;

          if let Some(req_content) = resolved_oor.content.get("application/json") {
            let request_schema = req_content.schema(&state.openapi_spec)?;

            self.request_schema = Some(request_schema);
          }
          self.request_body = Some(resolved_oor);
        }
      }
    }
    self.set_request_schema_styles()?;
    Ok(())
  }
}
impl Pane for RequestPane {
  fn init(&mut self) -> Result<()> {
    Ok(())
  }

  fn focus(&mut self) -> Result<()> {
    self.focused = true;
    Ok(())
  }

  fn unfocus(&mut self) -> Result<()> {
    self.focused = false;
    Ok(())
  }

  fn handle_key_events(&mut self, _key: KeyEvent) -> Result<Option<EventResponse<Action>>> {
    Ok(None)
  }

  #[allow(unused_variables)]
  fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<EventResponse<Action>>> {
    Ok(None)
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Update => {
        self.init_request_schema()?;
      },
      Action::Down => {
        self.request_schema_line_offset =
          self.request_schema_line_offset.saturating_add(1).min(self.request_schema_styles.len() - 1);
      },
      Action::Up => {
        self.request_schema_line_offset = self.request_schema_line_offset.saturating_sub(1);
      },
      Action::Submit => {},
      _ => {},
    }

    Ok(None)
  }

  fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()> {
    if let Some(request_body) = &self.request_body {
      let inner_margin: Margin = Margin { horizontal: 1, vertical: 1 };

      let inner = area.inner(&inner_margin);
      let media_types: Vec<String> = request_body.content.keys().map(|item| item.to_string()).collect();

      frame.render_widget(
        Tabs::new(media_types)
          .style(Style::default().dark_gray())
          .highlight_style(Style::default().white().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
          .select(0)
          .divider(symbols::DOT)
          .padding(" ", " "),
        inner,
      );

      let inner_margin: Margin = Margin { horizontal: 1, vertical: 1 };
      let mut inner = inner.inner(&inner_margin);
      inner.height = inner.height.saturating_add(1);
      let lines = self.request_schema_styles.iter().map(|items| {
        return Line::from(
          items
            .iter()
            .map(|item| {
              return Span::styled(&item.1, item.0.bg(Color::Reset));
            })
            .collect::<Vec<_>>(),
        );
      });
      let mut list_state = ListState::default().with_selected(Some(self.request_schema_line_offset));

      frame.render_stateful_widget(
        List::new(lines)
          .highlight_symbol(symbols::scrollbar::HORIZONTAL.end)
          .highlight_spacing(HighlightSpacing::Always),
        inner,
        &mut list_state,
      );
    }
    frame.render_widget(
      Block::default()
        .title("Request")
        .borders(Borders::ALL)
        .border_style(self.border_style())
        .border_type(self.border_type()),
      area,
    );

    Ok(())
  }
}
