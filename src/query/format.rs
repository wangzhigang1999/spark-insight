use arrow::array::{Array, ArrayRef};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use serde_json::{Map, Value};

/// Format a RecordBatch as a list of JSON objects
pub fn batch_to_json(batch: &RecordBatch) -> Vec<Value> {
    let schema = batch.schema();
    let mut rows = Vec::with_capacity(batch.num_rows());

    for row in 0..batch.num_rows() {
        let mut obj = Map::new();
        for (col_idx, field) in schema.fields().iter().enumerate() {
            let col = batch.column(col_idx);
            let val = array_value_to_json(col, row);
            obj.insert(field.name().clone(), val);
        }
        rows.push(Value::Object(obj));
    }
    rows
}

fn array_value_to_json(col: &ArrayRef, row: usize) -> Value {
    if col.is_null(row) {
        return Value::Null;
    }

    match col.data_type() {
        DataType::Int32 => {
            let arr = col
                .as_any()
                .downcast_ref::<arrow::array::Int32Array>()
                .unwrap();
            Value::Number(arr.value(row).into())
        }
        DataType::Int64 => {
            let arr = col
                .as_any()
                .downcast_ref::<arrow::array::Int64Array>()
                .unwrap();
            Value::Number(arr.value(row).into())
        }
        DataType::Float32 => {
            let arr = col
                .as_any()
                .downcast_ref::<arrow::array::Float32Array>()
                .unwrap();
            serde_json::Number::from_f64(arr.value(row) as f64)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
        DataType::Float64 => {
            let arr = col
                .as_any()
                .downcast_ref::<arrow::array::Float64Array>()
                .unwrap();
            serde_json::Number::from_f64(arr.value(row))
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
        DataType::Boolean => {
            let arr = col
                .as_any()
                .downcast_ref::<arrow::array::BooleanArray>()
                .unwrap();
            Value::Bool(arr.value(row))
        }
        DataType::Utf8 => {
            let arr = col
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .unwrap();
            Value::String(arr.value(row).to_string())
        }
        DataType::Utf8View => {
            let arr = col
                .as_any()
                .downcast_ref::<arrow::array::StringViewArray>()
                .unwrap();
            Value::String(arr.value(row).to_string())
        }
        DataType::LargeUtf8 => {
            let arr = col
                .as_any()
                .downcast_ref::<arrow::array::LargeStringArray>()
                .unwrap();
            Value::String(arr.value(row).to_string())
        }
        DataType::UInt64 => {
            let arr = col
                .as_any()
                .downcast_ref::<arrow::array::UInt64Array>()
                .unwrap();
            Value::Number(arr.value(row).into())
        }
        _ => Value::String(format!("{:?}", col.data_type())),
    }
}

/// Format a RecordBatch as a comfy-table string for terminal display
pub fn batch_to_table_string(batch: &RecordBatch) -> String {
    use comfy_table::{Cell, Table};

    let schema = batch.schema();
    let mut table = Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL_CONDENSED);

    // Header
    let headers: Vec<Cell> = schema.fields().iter().map(|f| Cell::new(f.name())).collect();
    table.set_header(headers);

    // Rows
    for row in 0..batch.num_rows() {
        let cells: Vec<Cell> = (0..batch.num_columns())
            .map(|col_idx| {
                let col = batch.column(col_idx);
                let val = array_value_to_json(col, row);
                Cell::new(match &val {
                    Value::Null => "NULL".to_string(),
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
            })
            .collect();
        table.add_row(cells);
    }

    table.to_string()
}

/// Simple struct for TUI table display
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub fn batch_to_table_data(batch: &RecordBatch) -> TableData {
    let schema = batch.schema();
    let headers = schema
        .fields()
        .iter()
        .map(|f| f.name().clone())
        .collect::<Vec<_>>();

    let mut rows = Vec::with_capacity(batch.num_rows());
    for row in 0..batch.num_rows() {
        let cells: Vec<String> = (0..batch.num_columns())
            .map(|col_idx| {
                let col = batch.column(col_idx);
                let val = array_value_to_json(col, row);
                match val {
                    Value::Null => "NULL".to_string(),
                    Value::String(s) => s,
                    other => other.to_string(),
                }
            })
            .collect();
        rows.push(cells);
    }

    TableData { headers, rows }
}
