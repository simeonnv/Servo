use crate::Error;

pub trait FormatValidate {
    fn validate(&self) -> Result<(), String>;
}
