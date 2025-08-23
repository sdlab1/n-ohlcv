// crosshair.rs
use crate::datawindow::DataWindow;
use crate::drawing_util; // Добавлен импорт для drawing_util
use chrono::{DateTime, Utc};
use eframe::egui::Rect;

#[derive(Default)]
pub struct Crosshair {
    rect: Option<egui::Rect>, // Private field for chart area
    cached_bar_index: Option<usize>,
    cached_bar_info: Option<String>,
}

impl Crosshair {
    // Вспомогательная функция для получения бара под курсором мыши
    fn get_bar_under_cursor_data<'a>(
        &self,
        mouse_pos: egui::Pos2,
        data_window: &'a DataWindow,
        chart_rect: egui::Rect, // Передаем релевантный rect графика (price_rect)
    ) -> Option<(usize, &'a crate::timeframe::Bar)> {
        let (start, end) = data_window.visible_range;
        let visible_slice = &data_window.bars.get(start as usize..end as usize)?;
        if visible_slice.is_empty() {
            return None;
        }

        let chart_left = chart_rect.left();
        let chart_width = chart_rect.width();

        let adjusted_x = mouse_pos.x - data_window.pixel_offset;
        let normalized_x = (adjusted_x - chart_left) / chart_width;
        if normalized_x < 0.0 || normalized_x >= 1.0 {
            return None;
        }
        let index_float = normalized_x * visible_slice.len() as f32;
        let index = index_float.floor() as usize;
        if index >= visible_slice.len() {
            return None;
        }
        let actual_index = start as usize + index;
        Some((actual_index, &visible_slice[index]))
    }

    pub fn get_bar_info(
        &mut self,
        mouse_pos: egui::Pos2,
        data_window: &DataWindow,
    ) -> Option<String> {
        let chart_area_rect = match self.rect {
            Some(rect) => rect,
            None => return None, // Область графика не определена
        };

        // Определяем price_rect для информации о баре (исключая область объема)
        let volume_height = chart_area_rect.height() * data_window.volume_height_ratio;
        let price_rect = egui::Rect::from_min_max(
            chart_area_rect.min,
            egui::pos2(chart_area_rect.max.x, chart_area_rect.max.y - volume_height),
        );

        // Используем новую вспомогательную функцию
        let (actual_index, bar) =
            self.get_bar_under_cursor_data(mouse_pos, data_window, price_rect)?;

        // Проверяем, есть ли уже информация об этом баре в кеше
        if let Some(cached_index) = self.cached_bar_index {
            if cached_index == actual_index {
                return self.cached_bar_info.clone();
            }
        }

        let dt = DateTime::<Utc>::from_timestamp_millis(bar.time).unwrap_or(Utc::now());
        let volume_str = {
            let volume = bar.volume;
            let (base, unit) = if volume < 1000.0 {
                (1.0, "")
            } else if volume < 1_000_000.0 {
                (1000.0, "k")
            } else {
                (1_000_000.0, "m")
            };
            let value = volume / base;
            let decimals = if value < 10.0 {
                2
            } else if value < 100.0 {
                1
            } else {
                0
            };
            format!("{:.*}{}", decimals, value, unit)
        };
        let bar_info = format!(
            "{} | o {:.2} h {:.2} l {:.2} c {:.2} v {}",
            dt.format("%H:%M"),
            bar.open,
            bar.high,
            bar.low,
            bar.close,
            volume_str
        );

        // Кешируем результат
        self.cached_bar_index = Some(actual_index);
        self.cached_bar_info = Some(bar_info.clone());

        Some(bar_info)
    }

    pub fn highlight_bar(
        &self,
        ui: &mut egui::Ui,
        rect: Rect, // Это общий прямоугольник области графика
        data_window: &DataWindow,
        mouse_pos: egui::Pos2,
        scale_price: &impl Fn(f64) -> f32,
    ) {
        let painter = ui.painter();
        let highlight_color = egui::Color32::from_rgb(100, 100, 100);

        let volume_height = rect.height() * data_window.volume_height_ratio;
        let price_rect =
            egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.max.y - volume_height));

        let (actual_index, bar) =
            match self.get_bar_under_cursor_data(mouse_pos, data_window, price_rect) {
                Some(data) => data,
                None => return,
            };

        let (start, end) = data_window.visible_range;
        let visible_count = (end - start) as usize;
        let visible_index = actual_index - start as usize; // Индекс бара относительно видимого слайса

        let (x_left, x_right) = drawing_util::calculate_bar_x_position(
            visible_index,
            visible_count,
            price_rect,
            data_window.pixel_offset,
        );

        let high_y = scale_price(bar.high);
        let low_y = scale_price(bar.low);

        let expanded_rect = egui::Rect::from_min_max(
            egui::pos2(x_left - 0.5, high_y - 0.5), // Сдвиг влево-вверх на 0.5px
            egui::pos2(x_right + 0.5, low_y + 0.5), // Сдвиг вправо-вниз на 0.5px
        );
        // Отрисовка закрашенного прямоугольника
        painter.rect_filled(
            expanded_rect,
            1.0, // Скругление углов
            highlight_color,
        );
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        rect: Rect,
        _data_window: &DataWindow,
        mouse_pos: egui::Pos2,
    ) {
        self.rect = Some(rect);
        let painter = ui.painter();
        let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100);

        painter.line_segment(
            [
                egui::pos2(mouse_pos.x, rect.top()),
                egui::pos2(mouse_pos.x, rect.bottom()),
            ],
            (1.0, color),
        );
        painter.line_segment(
            [
                egui::pos2(rect.left(), mouse_pos.y),
                egui::pos2(rect.right(), mouse_pos.y),
            ],
            (1.0, color),
        );
    }
}
