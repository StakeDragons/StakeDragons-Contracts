use crate::msg::{DragonBirthMsg, DragonBirthWrapper};
use crate::ContractError;

pub fn generate_dragon_birth_msg(
    id: String,
    owner: String,
) -> Result<DragonBirthWrapper, ContractError> {
    let msg = DragonBirthWrapper {
        dragon_birth: DragonBirthMsg {
            id: "0000".to_string() + id.as_str(),
            owner,
        },
    };
    Ok(msg)
}
