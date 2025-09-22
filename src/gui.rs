use std::sync::LazyLock;

use iced::advanced::graphics::core::Element;
use iced::widget::container;
use iced::window::{icon, Position};
use iced::{window, Color, Font, Length, Size, Subscription, Task, Theme};
use iced_term::TerminalView;

static ICON: LazyLock<icon::Icon> = LazyLock::new(|| {
	use iced::advanced::graphics::image::image_rs::ImageFormat;

	icon::from_file_data(
		include_bytes!("../GModPatchToolLogo.png"),
		Some(ImageFormat::Png),
	)
	.expect("failed to load icon data")
});

#[derive(Debug, Clone)]
pub enum Event {
	Terminal(iced_term::Event),
}

struct App {
	title: String,
	term: iced_term::Terminal,
}

impl App {
	fn new() -> (Self, Task<Event>) {
		let program = if let Ok(current_exe) = std::env::current_exe() {
			current_exe.display().to_string()
		} else if cfg!(target_os = "windows") {
			"gmodpatchtool.exe".to_owned()
		} else {
			"gmodpatchtool".to_owned()
		};
		let args = std::env::args().skip(1).collect::<Vec<String>>();

		let term_id = 0;
		let term_settings = iced_term::settings::Settings {
			font: iced_term::settings::FontSettings {
				size: 14.0,
				font_type: Font::MONOSPACE,
				..Default::default()
			},
			theme: iced_term::settings::ThemeSettings::new(Box::new(iced_term::ColorPalette {
				background: "#0c0c0c".to_owned(),
				foreground: "#b4b4b4".to_owned(),
				black: "#000000".to_owned(),
				red: "#c23621".to_owned(),
				green: "#25bc24".to_owned(),
				yellow: "#999900".to_owned(),
				blue: "#0000b2".to_owned(),
				magenta: "#b200b2".to_owned(),
				cyan: "#00a6b2".to_owned(),
				white: "#bfbfbf".to_owned(),
				bright_black: "#666666".to_owned(),
				bright_red: "#e60000".to_owned(),
				bright_green: "#00d900".to_owned(),
				bright_yellow: "#e6e600".to_owned(),
				bright_blue: "#0000ff".to_owned(),
				bright_magenta: "#e600e6".to_owned(),
				bright_cyan: "#00e6e6".to_owned(),
				bright_white: "#e6e6e6".to_owned(),
				..Default::default()
			})),
			backend: iced_term::settings::BackendSettings { program, args },
		};

		(
			Self {
				title: String::from("GModPatchTool"),
				term: iced_term::Terminal::new(term_id, term_settings).expect("Failed to create the new terminal instance"),
			},
			Task::none(),
		)
	}

	fn title(&self) -> String {
		self.title.clone()
	}

	fn subscription(&self) -> Subscription<Event> {
		Subscription::run_with_id(self.term.id, self.term.subscription()).map(Event::Terminal)
	}

	fn update(&mut self, event: Event) -> Task<Event> {
		match event {
			Event::Terminal(iced_term::Event::BackendCall(_, cmd)) => {
				match self.term.handle(iced_term::Command::ProxyToBackend(cmd)) {
					iced_term::actions::Action::Shutdown => {
						return window::get_latest().and_then(window::close)
					}
					iced_term::actions::Action::ChangeTitle(title) => {
						self.title = title;
					}
					_ => {},
				};

				Task::none()
			}
		}
	}

	fn view(&self) -> Element<Event, Theme, iced::Renderer> {
		// TODO: Scrollbar for Terminal
		container(TerminalView::show(&self.term).map(Event::Terminal))
			.width(Length::Fill)
			.height(Length::Fill)
			.padding(4)
			.style(|_| container::background(Color::from_rgb(12.0 / 255.0, 12.0 / 255.0, 12.0 / 255.0)))
			.into()
	}
}

pub fn main() -> iced::Result {
	iced::application(App::title, App::update, App::view)
		.antialiasing(true)
		.window(iced::window::Settings {
			size: Size {
				width: 960.0,
				height: 540.0,
			},
			position: Position::Centered,
			resizable: false,
			icon: Some(ICON.clone()),
			..Default::default()
		})
		.subscription(App::subscription)
		.run_with(App::new)
}
