use deer_foundation_contracts::{AppendControlRequest, AppendDataRequest};

pub fn validate_append_pair(
    data: &AppendDataRequest,
    control: &AppendControlRequest,
) -> Result<(), &'static str> {
    data.validate()?;
    control.validate()?;
    Ok(())
}
