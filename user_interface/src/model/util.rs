#[macro_export]
macro_rules! with_getters_setters {
    (
        $(#[$meta:meta])*
        $inner_vis:vis struct $Inner:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $ty:ty,
            )*
        }

        $outer_vis:vis struct $Outer:ident {
            $(
                $(#[$outer_field_meta:meta])*
                $outer_field_vis:vis $outer_field:ident : $outer_ty:ty,
            )*
        }
    ) => {
        #[warn(dead_code)]
        // ---- Inner struct definition ----
        $(#[$meta])*
        $inner_vis struct $Inner {
            changes: Vec<Field>,
            $(
                $(#[$field_meta])*
                $field_vis $field : $ty,
            )*
        }

        #[derive(Clone)]
        #[allow(non_camel_case_types)]
        $outer_vis enum Field {
            $(
                $field($ty),
            )*
        }

        impl $Inner {
            pub fn set(&mut self, field: Field) {
                match field.clone() {
                    $(Field::$field(value) => self.$field = value,)*
                }
                self.changes.push(field);
            }
        }


        // ---- Outer struct definition ----
        #[derive(Clone)]
        $outer_vis struct $Outer {
            pub(super) inner: std::sync::Arc<std::sync::Mutex<$Inner>>,
            $(
                $(#[$outer_field_meta])*
                $outer_field_vis $outer_field : $outer_ty,
            )*
        }        

        // ---- Impl for outer struct ----
        #[allow(dead_code)]
        impl $Outer {
            $(
                pub fn $field(&self) -> $ty
                where
                    $ty: Clone,
                {
                    self.inner.lock().unwrap().$field.clone()
                }
            )*
        }
    };
}
