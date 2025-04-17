use std::collections::VecDeque;

// Структура для вычисления RSI в реальном времени
pub struct WilderRSI {
    period: usize,                   // Период RSI
    prices: VecDeque<f64>,           // Храним цены для инициализации и последний P_n-1, аналог deque(maxlen=period+1)
    initial_gains: Vec<f64>,         // Временное хранилище для первичного расчета приростов
    initial_losses: Vec<f64>,        // Временное хранилище для первичного расчета падений
    last_timestamp: Option<i64>,     // Последняя временная метка
    last_close: Option<f64>,         // Цена Pn (последняя известная)
    price_before_last: Option<f64>,  // Цена Pn-1 (нужна для пересчета при обновлении Pn)
    avg_gain: Option<f64>,           // Текущее сглаженное среднее AvgGain_n
    avg_loss: Option<f64>,           // Текущее сглаженное среднее AvgLoss_n
    prev_avg_gain: Option<f64>,      // AvgGain_n-1 (нужно для пересчета при обновлении Pn)
    prev_avg_loss: Option<f64>,      // AvgLoss_n-1 (нужно для пересчета при обновлении Pn)
    is_initialized: bool,            // Флаг: true после первого расчета RSI
    data_count: usize,               // Счетчик уникальных временных меток
}

impl WilderRSI {
    /// Создает новый экземпляр rsi с заданным периодом.
    /// 
    /// # Arguments
    /// * `period` - Период для расчета RSI, должен быть целым числом больше 1.
    /// 
    /// # Panics
    /// Паникует, если period <= 1.
    pub fn new(period: usize) -> Self {
        if period <= 1 {
            panic!("Период должен быть целым числом больше 1");
        }
        Self {
            period,
            prices: VecDeque::with_capacity(period + 1), // Эмулируем maxlen=period+1
            initial_gains: Vec::new(),
            initial_losses: Vec::new(),
            last_timestamp: None,
            last_close: None,
            price_before_last: None,
            avg_gain: None,
            avg_loss: None,
            prev_avg_gain: None,
            prev_avg_loss: None,
            is_initialized: false,
            data_count: 0,
        }
    }

    /// Добавляет новую пару (timestamp, close) и возвращает RSI, если он рассчитан.
    /// 
    /// # Arguments
    /// * `timestamp` - Временная метка цены.
    /// * `close` - Цена закрытия.
    /// 
    /// # Returns
    /// Значение RSI (Option<f64>) или None, если данных недостаточно или произошла ошибка.
    pub fn add_price(&mut self, timestamp: i64, close: f64) -> Option<f64> {
        // --- Валидация ввода ---
        if close.is_nan() || close.is_infinite() {
            println!("Предупреждение: Некорректная цена закрытия ({}). Пропуск.", close);
            return self.get_rsi(); // Вернуть последнее извеTimestampстное значение
        }

        // --- Логика обработки временных меток ---
        let is_update = self.last_timestamp.map_or(false, |last| timestamp == last);
        let is_new_bar = self.last_timestamp.is_none() || self.last_timestamp.map_or(false, |last| timestamp > last);
        let is_old_data = self.last_timestamp.map_or(false, |last| timestamp < last);

        if is_old_data {
            println!("Предупреждение: Получены данные не по порядку (Timestamp {} < Последний {}). Пропуск.",
                timestamp,
                self.last_timestamp.unwrap()
            );
            return self.get_rsi(); // Вернуть последнее известное значение
        }

        // ===========================
        // === ОБНОВЛЕНИЕ ТЕКУЩЕГО БАРА ===
        // ===========================
        if is_update {
            if self.data_count < 1 || self.price_before_last.is_none() {
                // Нечего обновлять, если нет предыдущей цены Pn-1
                // Или если это обновление самой первой точки
                if !self.prices.is_empty() {
                    *self.prices.back_mut().unwrap() = close; // Обновляем последнюю цену в VecDeque
                }
                self.last_close = Some(close); // Обновляем последнюю цену в состоянии
                //println!("Info: Обновление цены, но RSI еще не инициализирован или недостаточно данных для пересчета.");
                return None;
            }

            // --- Пересчет для обновления ---
            // Вычисляем НОВЫЙ прирост/падение для обновленной цены
            let prev_close = self.price_before_last.unwrap();
            let new_gain = (close - prev_close).max(0.0);
            let new_loss = (prev_close - close).max(0.0);

            if !self.is_initialized {
                // Если мы все еще в периоде инициализации, но уже есть Pn-1
                // Обновляем последнее значение в списках initial_gains/losses
                if let Some(last_gain) = self.initial_gains.last_mut() {
                    *last_gain = new_gain;
                }
                if let Some(last_loss) = self.initial_losses.last_mut() {
                    *last_loss = new_loss;
                }
                *self.prices.back_mut().unwrap() = close; // Обновляем цену в VecDeque
                self.last_close = Some(close); // Обновляем цену в состоянии
                //println!("Info: Обновление цены в периоде инициализации.");
                return None; // RSI еще не готов
            }

            // --- Пересчет инициализированного RSI ---
            // Используем сохраненные ПРЕДЫДУЩИЕ средние (prev_avg_gain/loss)
            // и НОВЫЙ прирост/падение (new_gain/loss)
            if self.prev_avg_gain.is_none() || self.prev_avg_loss.is_none() {
                //println!("Info: Обновление цены сразу после инициализации RSI игнорируется, так как отсутствуют предыдущие средние. RSI не пересчитан.");
                *self.prices.back_mut().unwrap() = close; // Обновляем цену в VecDeque
                self.last_close = Some(close); // Обновляем цену в состоянии
                return None; // Вернуть старое значение
            }

            // Стандартный пересчет при обновлении:
            let period_f = self.period as f64;
            self.avg_gain = Some((self.prev_avg_gain.unwrap() * (period_f - 1.0) + new_gain) / period_f);

            // Обработка деления на ноль и случая отсутствия потерь
            if self.prev_avg_loss.unwrap() == 0.0 && new_loss == 0.0 {
                self.avg_loss = Some(0.0);
            } else {
                self.avg_loss = Some((self.prev_avg_loss.unwrap() * (period_f - 1.0) + new_loss) / period_f);
                if self.avg_loss.unwrap() < 0.0 {
                    self.avg_loss = Some(0.0); // Теоретически не должно быть, но для страховки
                }
            }

            // Обновляем цену в хранилище и состоянии
            *self.prices.back_mut().unwrap() = close;
            self.last_close = Some(close);
            // НЕ обновляем: last_timestamp, price_before_last, prev_avg_gain/loss

            //println!("Info: Цена обновлена для {}. Новый RSI:", timestamp);
            return self.calculate_rsi();
        }

        // ===========================
        // === НОВЫЙ БАР (НОВАЯ МЕТКА) ===
        // ===========================
        if is_new_bar {
            if self.last_timestamp.is_none() {
                // Самая первая точка данных
                self.prices.push_back(close);
                self.last_timestamp = Some(timestamp);
                self.last_close = Some(close);
                self.data_count = 1;
                //println!("Info: Получена первая точка данных.");
                return None;
            }

            // --- Обработка нового бара ---
            let prev_close = self.last_close.unwrap();
            let current_gain = (close - prev_close).max(0.0);
            let current_loss = (prev_close - close).max(0.0);

            // Сохраняем цену Pn-1 (которая была self.last_close)
            // Это нужно для случая, если СЛЕДУЮЩИЙ вызов будет обновлением этого нового бара
            self.price_before_last = self.last_close;

            // Добавляем новую цену, эмулируя maxlen=period+1
            if self.prices.len() >= self.period + 1 {
                self.prices.pop_front();
            }
            self.prices.push_back(close);
            self.data_count += 1;

            if !self.is_initialized {
                // Накапливаем данные для первоначального расчета
                self.initial_gains.push(current_gain);
                self.initial_losses.push(current_loss);

                if self.data_count == self.period + 1 {
                    // Данных достаточно для первого расчета
                    // Рассчитываем начальные средние (простое среднее)
                    let initial_avg_gain = self.initial_gains.iter().sum::<f64>() / self.period as f64;
                    let initial_avg_loss = self.initial_losses.iter().sum::<f64>() / self.period as f64;

                    self.avg_gain = Some(initial_avg_gain);
                    self.avg_loss = Some(initial_avg_loss);
                    // Обработка случая нулевых начальных потерь
                    if self.avg_loss.unwrap() < 0.0 {
                        self.avg_loss = Some(0.0); // Безопасность
                    }

                    self.is_initialized = true;
                    // Очищаем временные списки
                    self.initial_gains.clear();
                    self.initial_losses.clear();

                    // Обновляем состояние
                    self.last_timestamp = Some(timestamp);
                    self.last_close = Some(close);
                    // ВАЖНО: prev_avg_* еще не установлены, они понадобятся
                    // только перед обработкой СЛЕДУЮЩЕГО нового бара.

                    //println!("Info: RSI инициализирован на {}. RSI:", timestamp);
                    return self.calculate_rsi();
                } else {
                    // Данных все еще недостаточно
                    self.last_timestamp = Some(timestamp);
                    self.last_close = Some(close);
                    /*println!(
                        "Info: Получена точка {}/{} для инициализации.",
                        self.data_count,
                        self.period + 1
                    );*/
                    return None;
                }
            } else {
                // --- Стандартная обработка нового бара (RSI уже инициализирован) ---
                // Сохраняем ТЕКУЩИЕ средние как ПРЕДЫДУЩИЕ для возможного обновления этого бара
                self.prev_avg_gain = self.avg_gain;
                self.prev_avg_loss = self.avg_loss;

                // Обновляем средние с использованием Wilder's Smoothing
                let period_f = self.period as f64;
                self.avg_gain = Some((self.prev_avg_gain.unwrap() * (period_f - 1.0) + current_gain) / period_f);

                // Обработка деления на ноль и случая отсутствия потерь
                if self.prev_avg_loss.unwrap() == 0.0 && current_loss == 0.0 {
                    self.avg_loss = Some(0.0);
                } else {
                    self.avg_loss = Some((self.prev_avg_loss.unwrap() * (period_f - 1.0) + current_loss) / period_f);
                    if self.avg_loss.unwrap() < 0.0 {
                        self.avg_loss = Some(0.0); // Безопасность
                    }
                }

                // Обновляем состояние
                self.last_timestamp = Some(timestamp);
                self.last_close = Some(close);

                //println!("Info: Новый бар {}. RSI:", timestamp);
                return self.calculate_rsi();
            }
        }

        // Сюда не должны дойти, если логика верна
        None
    }

    /// Вспомогательный метод для вычисления RSI по текущим avg_gain и avg_loss.
    fn calculate_rsi(&self) -> Option<f64> {
        if !self.is_initialized || self.avg_gain.is_none() || self.avg_loss.is_none() {
            return None; // Невозможно рассчитать
        }

        let avg_gain = self.avg_gain.unwrap();
        let avg_loss = self.avg_loss.unwrap();

        if avg_loss == 0.0 {
            // Если средние потери равны нулю, RSI равен 100
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - 100.0 / (1.0 + rs);
        Some(rsi)
    }

    /// Возвращает последнее рассчитанное значение RSI без добавления новых данных.
    pub fn get_rsi(&self) -> Option<f64> {
        self.calculate_rsi()
    }
}

// --- Пример использования ---
/*
fn main() {
    let mut rsi_calculator = rsi::new(3); // Пример с коротким периодом

    let data = vec![
        (1.0, 10.0),
        (2.0, 11.0), // Gain=1, Loss=0
        (3.0, 10.5), // Gain=0, Loss=0.5
        (4.0, 12.0), // Gain=1.5, Loss=0 | Инициализация RSI здесь
        (5.0, 11.5), // Gain=0, Loss=0.5
        (5.0, 11.8), // Обновление цены для timestamp=5
        (5.0, 11.6), // Еще одно обновление для timestamp=5
        (6.0, 12.5), // Новый бар, закрытие timestamp=5
        (7.0, 13.0),
        (7.0, 13.1), // Обновление timestamp=7
        (8.0, 12.8),
    ];

    println!("Period = {}\n", rsi_calculator.period);

    for (ts, price) in data {
        println!("Input: Timestamp={}, Close={}", ts, price);
        let rsi_value = rsi_calculator.add_price(ts, price);
        if let Some(rsi) = rsi_value {
            println!("Output: RSI = {:.2}", rsi);
        } else {
            println!("Output: RSI = None (недостаточно данных или обновление в инициализации)");
        }
        println!("--------------------");
    }
}*/