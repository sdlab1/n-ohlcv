// settings.rs
pub const ZOOM_SENSITIVITY: f64 = 0.05;
pub const DRAG_SENSITIVITY: f64 = 1.0;
pub const CHART_MARGIN: f32 = 0.0;
pub const CHART_BOTTOM_MARGIN: f32 = 5.0;
pub const PRICE_FRACTION_THRESHOLD: f64 = 0.01; // 1% порог для отображения дробной части
pub const BAR_SPACING: f32 = 1.0; // расстояние между барами
pub const INITIAL_LOAD_DAYS: i64 = 15; // Количество дней для начальной загрузки данных
pub const AVERAGE_FRAME_HISTORY_SIZE: usize = 60; // Количество кадров на значение (avg)
pub const STATUS_MESSAGE_MAX_COUNT: usize = 8; // Максимальное количество сообщений в списке статуса
pub const STATUS_MESSAGE_LIFETIME_SECONDS: u64 = 5; // Время жизни сообщения статуса в секундах