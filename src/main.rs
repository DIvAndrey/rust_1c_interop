use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::{fs::File, io::BufReader};
use paste::paste;

macro_rules! type_to_jsx_type {
    (i64) => {
        "jxs:decimal"
    };
    (i32) => {
        "jxs:decimal"
    };
    (String) => {
        "jxs:string"
    };
    (bool) => {
        "jxs:boolean"
    };
}

macro_rules! json_value_to_strongly_typed {
    ($value:expr, i64) => {
        $value.as_i64().unwrap()
    };
    ($value:expr, i32) => {
        $value.as_i64().unwrap() as i32
    };
    ($value:expr, String) => {
        $value.as_str().unwrap().to_string()
    };
    ($value:expr, bool) => {
        $value.as_bool().unwrap()
    };
}

macro_rules! strongly_typed_to_json_value {
    ($value:expr, i64) => {
        Value::Number($value.into())
    };
    ($value:expr, i32) => {
        Value::Number($value.into())
    };
    ($value:expr, String) => {
        Value::String($value.into())
    };
    ($value:expr, bool) => {
        Value::Bool($value.into())
    };
}

macro_rules! make_serializable_object {
    ($struct_name:ident, $($field_name:ident: $field_type:ty),+ $(,)?) => {
        #[derive(Clone)]
        struct $struct_name {
            $(
                $field_name: $field_type,
            )+
        }

        impl $struct_name {
            fn from_value(value: Value) -> Self {
                let value = value.as_array().expect(&format!("Expected an array, found {:?}", &value));
                let mut i = 0;
                Self {
                    $(
                        $field_name: {
                            let val = &value[i];
                            i += 1;
                            paste! {
                                assert_eq!(
                                    val["#type"].as_str().unwrap(), 
                                    type_to_jsx_type!([<$field_type>])
                                );
                                json_value_to_strongly_typed!(
                                    &val["#value"], 
                                    [<$field_type>]
                                )
                            }
                        },
                    )+
                }
            }

            fn into_value(self) -> Value {
                Value::Array(vec![
                    $(
                        Value::Object(serde_json::Map::from_iter(paste! {[
                            ("#type".to_string(), Value::String(type_to_jsx_type!([<$field_type>]).to_string())),
                            ("#value".to_string(), strongly_typed_to_json_value!(self.$field_name, [<$field_type>]))
                        ]})),
                    )+
                ])
            }
        }

        impl Serialize for $struct_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let value = self.clone().into_value();
                value.serialize(serializer)
            }
        }
        
        impl<'de> Deserialize<'de> for $struct_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = Value::deserialize(deserializer)?;
                Ok($struct_name::from_value(value))
            }
        }
    };
}
make_serializable_object!(
    TableRow,
    code: String,
    specification_code: String,
    p_number: i64,
    op_number: String,
    operation_type_code: String,
    working_center_code: String,
    num_plan: i64,
    num1: i64,
    bool1: bool,
);

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct Table<T> {
    rows: Vec<T>,
}

fn read_table_and_columns_info() -> (Table<TableRow>, Value) {
    let file = File::open("data/ТекстJSN.json").unwrap();
    let reader = BufReader::new(file);
    let value: Value = serde_json::from_reader(reader).unwrap();
    let rows_data = value["#value"]["row"].clone();
    (serde_json::from_value(rows_data).unwrap(), value["#value"]["column"].clone())
}

fn write_table(table: &Table<TableRow>, columns_data: Value) {
    let file = File::create("data/ТекстJSN_изм.json").unwrap();
    let value = serde_json::to_value(table).unwrap();
    let value = serde_json::json!({
        "#value": {
            "column": columns_data,
            "row": value
        }
    });
    serde_json::to_writer_pretty(file, &value).unwrap();
}

fn main() {
    let (mut table, columns_data) = read_table_and_columns_info();
    for row in &mut table.rows {
        row.num1 += 1;
        row.bool1 = !row.bool1;
    }
    write_table(&table, columns_data);
}