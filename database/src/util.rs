#[allow(dead_code)]
pub trait ChangeTracking {
    fn update_from_json(&mut self, value: serde_json::Value) -> Vec<String>;
}

/// Enriches passed ActiveModel with a function to update the model from a JSON value.
/// It will only update fields that actually have different value and will return the model
/// and the names of changed fields.
/// The macro requires sea_orm::Iterable to be imported
macro_rules! with_change_tracking {
    ($AM: tt) => {
        impl ChangeTracking for $AM {
            fn update_from_json(&mut self, value: serde_json::Value) -> Vec<String> {
                let model = $AM::from_json(value.clone()).unwrap();

                Column::iter()
                    .filter_map(|column| {
                        // Skip primary keys
                        if PrimaryKey::from_column(column).is_some() {
                            return None;
                        }
                        let new_value = model.get(column).into_value();
                        let old_value = self.get(column).into_value();

                        if new_value != old_value {
                            println!("changed? {}", column.to_string());
                            match new_value {
                                Some(value) => self.set(column, value),
                                None => self.not_set(column)
                            }
                            Some(column.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
            }
        }
    };
}

pub(crate) use with_change_tracking;