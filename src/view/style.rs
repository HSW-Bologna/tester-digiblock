use iced::widget::container;
use iced_native::Color;

pub struct BorderedContainer;

impl iced::widget::container::StyleSheet for BorderedContainer {
    type Style = iced::Theme;

    fn appearance(&self, _: &<Self as container::StyleSheet>::Style) -> container::Appearance {
        container::Appearance {
            border_color: Color::WHITE.into(),
            border_radius: 2.0,
            border_width: 2.0,
            ..Default::default()
        }
    }
}

pub fn bordered_container() -> iced::theme::Container {
    iced::theme::Container::Custom(Box::from(BorderedContainer))
}
