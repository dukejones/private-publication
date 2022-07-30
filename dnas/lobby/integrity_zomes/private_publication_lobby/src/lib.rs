use hdi::prelude::*;
use hdk::prelude::*;
pub use membrane_proof::PrivatePublicationMembraneProof;

#[hdk_entry_defs]
#[unit_enum(UnitEntryType)]
pub enum EntryTypes {
    PrivatePublicationMembraneProof(PrivatePublicationMembraneProof),
}

#[hdk_link_types]
pub enum LinkTypes {
    AgentToMembraneProof,
}

#[hdk_extern]
fn create_membrane_proof_for(pubkey: AgentPubKey) -> ExternResult<()> {
    call();
    Ok(())
}