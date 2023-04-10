#[macro_export]
macro_rules! constant {
    ($name:ident, $value:expr) => {
        const $name: &str = $value;
    };
}
