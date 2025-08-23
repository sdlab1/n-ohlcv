// drawing_util.rs
use eframe::egui::{Pos2, Rect};

/// Рассчитывает X-координаты и ширину бара.
///
/// # Arguments
/// * `visible_index` - Индекс бара в текущем видимом диапазоне (0-based).
/// * `visible_count` - Общее количество видимых баров.
/// * `chart_rect` - Прямоугольник, описывающий область для отрисовки баров.
/// * `pixel_offset` - Смещение графика в пикселях (для панорамирования).
///
/// # Returns
/// Возвращает кортеж `(x_left, x_right)` - левая и правая X-координаты бара.
pub fn calculate_bar_x_position(
    visible_index: usize,
    visible_count: usize,
    chart_rect: Rect,
    pixel_offset: f32,
) -> (f32, f32) {
    let count_f = visible_count as f32;
    // Общая ширина, выделенная под один бар (включая промежуток)
    // Это как "ширина слота" для каждого бара
    let total_bar_slot_width = chart_rect.width() / count_f;

    // Ширина самого бара. Можно сделать ее чуть меньше, чтобы были промежутки.
    // 90% от ширины слота, но не более 5.0 пикселей для максимальной ширины.
    let bar_width = (total_bar_slot_width * 0.9).min(5.0);

    // X-координата центра слота для текущего бара
    let x_center_of_slot = chart_rect.left() + (visible_index as f32 + 0.5) * total_bar_slot_width;

    // Смещение X-координат с учетом панорамирования и выравнивания бара по центру слота
    let x_left = x_center_of_slot - bar_width / 2.0 + pixel_offset;
    let x_right = x_center_of_slot + bar_width / 2.0 + pixel_offset;

    (x_left, x_right)
}
