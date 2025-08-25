# Project Conventions & Architecture

> **Reference this file in every source file to understand project structure and workflow**

## Project Overview

N-OHLCV is a real-time cryptocurrency charting application with GPU-accelerated rendering, data aggregation, and interactive visualization capabilities.

## Core Workflow

1. **Data Fetching** → `fetch.rs` gets minute-level OHLCV from Binance API
2. **Data Processing** → `timeframe.rs` validates and stores data via `db.rs`
3. **Auto-Aggregation** → `db.rs` creates hourly data for navigation pane
4. **Rendering** → `gui.rs` + `interactivegui.rs` display charts using GPU backend
5. **User Interaction** → Mouse/keyboard events modify view state

## File Structure & Responsibilities

### Core Data Layer
- **`main.rs`** - Application entry point, initializes eframe with InteractiveGui
- **`lib.rs`** - Module exports for library usage
- **`settings.rs`** - Project constants, versions, and configuration
- **`fetch.rs`** - Binance API client, KLine struct definition, price conversion
- **`db.rs`** - Database operations, data aggregation system, OHLCV storage
- **`compress.rs`** - Data compression/decompression for storage efficiency

### Data Processing
- **`timeframe.rs`** - Data validation, consistency checks, database integration
- **`datawindow.rs`** - Memory management for chart data windows

### Visualization Core  
- **`gui.rs`** - Main GUI framework, chart layout, event handling
- **`interactivegui.rs`** - Interactive features, zoom, pan, crosshair management
- **`gpu_backend.rs`** - eframe/egui GPU configuration and setup

### Chart Components
- **`hlcbars.rs`** - Candlestick/OHLC bar rendering
- **`volbars.rs`** - Volume bar visualization
- **`axes.rs`** - Price and time axis rendering
- **`axes_util.rs`** - Axis calculation utilities
- **`crosshair.rs`** - Mouse cursor crosshair system
- **`drawing_util.rs`** - Common drawing utilities and helpers

### Technical Analysis
- **`rsi.rs`** - RSI (Relative Strength Index) indicator calculation
- **`performance.rs`** - Performance monitoring and optimization

## Data Architecture

### Database Schema
```
Raw Data:     {symbol}_{timestamp}           -> Compressed KLine data
Aggregated:   {symbol}_aggr_{timestamp}      -> Hourly OHLCV data  
Metadata:     last_{symbol}                  -> Latest timestamp
              first_{symbol}_aggr            -> First aggregated timestamp
              version_{symbol}_aggr          -> Aggregation version
```

### Data Types
```rust
KLine {
    open_time: i64,      // Unix timestamp (ms)
    open: u64,           // Price * 100 (cents precision)
    high: u64,           // Price * 100
    low: u64,            // Price * 100  
    close: u64,          // Price * 100
    volume: f64,         // Volume in base units
}
```

### Aggregation System
- **Version Control**: `AGGREGATION_VERSION` in settings.rs triggers full rebuild
- **Auto-Trigger**: Called after every data insert in `timeframe.rs::process_data_chunk()`
- **Time Alignment**: Hourly boundaries in local system timezone
- **Validation**: Displays last 5 records after aggregation

## Key Functions by Module

### fetch.rs
- `fetch_klines()` - Get OHLCV data from Binance API
- `convert_price_to_u64()` - Convert string prices to integer format

### db.rs  
- `insert_block()` - Store compressed data block
- `get_block()` - Retrieve data block by timestamp
- `aggregate_ohlcv_data()` - **Main aggregation function** (auto-called)
- `get_aggr_info()` - Get aggregated data range info

### timeframe.rs
- `process_data_chunk()` - **Data entry point** - validates, stores, triggers aggregation
- `update_loop()` - Continuous data fetching loop
- `fetch_data_chunk()` - Internal data fetching

### gui.rs / interactivegui.rs
- `new()` - Initialize GUI with symbol and timeframe
- `update()` - Main render loop, handles events
- `render_*()` - Specific component rendering

## Coding Standards

### Error Handling
- Return `Result<T, Box<dyn Error>>` for all fallible operations
- Use `?` operator for error propagation  
- Log warnings to console, don't crash on non-critical errors

### Console Output
- **Language**: English only
- **Format**: Structured logging with timestamps when relevant
- **Aggregation**: Display summary + last 5 records for validation

### Time Handling
- **Storage**: Unix milliseconds (i64)
- **Display**: Local system timezone
- **Format**: `HH:MM DD.MM.YY` for user output

### Constants & Configuration
- All project constants in `settings.rs`
- Version numbers as Unix timestamps of creation date
- Price precision: `PRICE_MULTIPLIER = 2` (cents)

## Integration Points

### Data Flow Trigger
```rust
// In timeframe.rs::process_data_chunk() - ONLY aggregation call point
db.insert_block(symbol, data[0].open_time, &compressed_data)?;
if let Err(e) = db.aggregate_ohlcv_data(symbol) {
    eprintln!("Warning: Failed to aggregate data for {}: {}", symbol, e);
}
```

### Navigation Pane Usage
```rust
// Get aggregated data range for mini-chart
let (first_time, last_time) = db.get_aggr_info("BTCUSDT")?;
// Read specific hourly data
let hourly_data = db.get_block("BTCUSDT_aggr", timestamp)?;
```

## Performance Considerations

- **GPU Rendering**: All chart rendering uses GPU via eframe/wgpu
- **Data Compression**: Raw data compressed before storage
- **Memory Management**: DataWindow limits loaded data size
- **Incremental Updates**: Only process new complete hours in aggregation

---

> **Remember**: Reference this file in source comments to maintain architectural clarity